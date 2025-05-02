use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use hmi_lib::commands::db_commands::QueryParams;

mod common;
use common::{ensure_bench_env, AppState};

async fn fetch_users(
    app_state: &AppState,
    params: &QueryParams,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    let order_clause = match params.order_by {
        Some(ref order) if !order.is_empty() => {
            let (field, is_asc) = &order[0];
            format!(
                "ORDER BY {} {}",
                field,
                if *is_asc { "ASC" } else { "DESC" }
            )
        }
        _ => "ORDER BY bench_users.name ASC".to_string(),
    };

    let sql = format!(
        "SELECT
            id::text,
            name,
            email,
            image,
            COUNT(*) OVER() as total_count
        FROM
            bench_users
        WHERE
            deleted_at IS NULL
        {} LIMIT {} OFFSET {}",
        order_clause, limit, offset
    );

    let rows = client.query(&sql, &[]).await?;

    let mut results = Vec::with_capacity(rows.len());
    let mut _total_count = 0;

    for row in rows {
        _total_count = row.get::<_, i64>("total_count");

        results.push(json!({
            "id": row.get::<_, String>(0),
            "name": row.get::<_, String>(1),
            "email": row.get::<_, String>(2),
            "image": row.get::<_, serde_json::Value>(3)
        }));
    }

    Ok(results)
}

async fn fetch_user_details(
    app_state: &AppState,
    params: &QueryParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let user_id = params
        .conditions
        .as_ref()
        .and_then(|conds| conds.first())
        .and_then(|(_, val)| val.as_str())
        .unwrap_or("");

    if user_id.is_empty() {
        return Ok(json!({ "error": "User ID is required" }));
    }

    let user_uuid = match Uuid::parse_str(user_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid user ID format" })),
    };

    let sql = "
        SELECT
            id::text,
            name,
            email,
            image
        FROM
            bench_users
        WHERE
            id = $1 AND
            deleted_at IS NULL";

    let row = match client.query_opt(sql, &[&user_uuid]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(json!({ "error": "User not found" })),
        Err(err) => return Err(Box::new(err)),
    };

    // Get user's recent activities
    let activities_sql = "
        WITH borrow_activities AS (
            SELECT
                a.id::text,
                'borrow' as activity_type,
                a.created_at,
                r.id::text as receipt_id,
                COUNT(rd.id) as device_count,
                l.name as lab_name
            FROM
                bench_activities a
                JOIN bench_receipts r ON r.id IN (
                    SELECT borrowed_receipt_id FROM bench_receipts_devices WHERE borrow_id = a.id
                )
                JOIN bench_receipts_devices rd ON rd.borrow_id = a.id
                JOIN bench_labs l ON r.lab_id = l.id
            WHERE
                a.type = 'borrow' AND
                r.actor_id = $1
            GROUP BY
                a.id, a.created_at, r.id, l.name
        ),
        return_activities AS (
            SELECT
                a.id::text,
                'return' as activity_type,
                a.created_at,
                r.id::text as receipt_id,
                COUNT(rd.id) as device_count,
                l.name as lab_name
            FROM
                bench_activities a
                JOIN bench_receipts r ON r.id IN (
                    SELECT returned_receipt_id FROM bench_receipts_devices WHERE return_id = a.id
                )
                JOIN bench_receipts_devices rd ON rd.return_id = a.id
                JOIN bench_labs l ON r.lab_id = l.id
            WHERE
                a.type = 'return' AND
                r.actor_id = $1
            GROUP BY
                a.id, a.created_at, r.id, l.name
        )
        SELECT * FROM (
            SELECT * FROM borrow_activities
            UNION ALL
            SELECT * FROM return_activities
        ) activities
        ORDER BY created_at DESC
        LIMIT 5";

    let activity_rows = client.query(activities_sql, &[&user_uuid]).await?;

    let activities = activity_rows
        .iter()
        .map(|row| {
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

            json!({
                "id": row.get::<_, String>(0),
                "activityType": row.get::<_, String>(1),
                "createdAt": created_at.to_rfc3339(),
                "receiptId": row.get::<_, String>(3),
                "deviceCount": row.get::<_, i64>(4),
                "labName": row.get::<_, String>(5)
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "id": row.get::<_, String>(0),
        "name": row.get::<_, String>(1),
        "email": row.get::<_, String>(2),
        "image": row.get::<_, serde_json::Value>(3),
        "recentActivities": activities
    }))
}

async fn get_borrowed_history_by_user(
    app_state: &AppState,
    user_id: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    if user_id.is_empty() {
        return Ok(Vec::new());
    }

    let user_uuid = match Uuid::parse_str(user_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(Vec::new()),
    };

    let sql = "
        SELECT
          r.id AS receipt_id,
          d.id AS device_id,
          d.kind AS device_kind_id,
          dk.name AS device_name,
          dk.image AS device_image,
          dk.is_borrowable_lab_only,
          bl.id AS lab_id,
          bl.room AS lab_room,
          bl.branch AS lab_branch,
          a_borrow.created_at AS borrow_date,
          rd.expected_returned_at,
          CASE
            WHEN rd.expected_returned_at < NOW() THEN 'OVERDUE'
            WHEN rd.expected_returned_at < NOW() + INTERVAL '3 days' THEN 'NEAR_DUE'
            ELSE 'ON_TIME'
          END AS status
        FROM
          bench_receipts r
          JOIN bench_receipts_devices rd ON r.id = rd.borrowed_receipt_id
          JOIN bench_devices d ON rd.device_id = d.id
          JOIN bench_device_kinds dk ON d.kind = dk.id
          JOIN bench_labs bl ON r.lab_id = bl.id
          LEFT JOIN bench_activities a_borrow ON rd.borrow_id = a_borrow.id
        WHERE
          r.actor_id = $1
          AND rd.returned_receipt_id IS NULL
          AND d.deleted_at IS NULL
        ORDER BY
          a_borrow.created_at DESC
    ";

    let rows = client.query(sql, &[&user_uuid]).await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let results = rows.iter().map(|row| {
        let device_image: serde_json::Value = row.get("device_image");

        json!({
            "receiptId": row.get::<_, String>("receipt_id"),
            "deviceId": row.get::<_, String>("device_id"),
            "deviceKindId": row.get::<_, String>("device_kind_id"),
            "deviceName": row.get::<_, String>("device_name"),
            "deviceImage": device_image,
            "deviceBorrowableLabOnly": row.get::<_, bool>("is_borrowable_lab_only"),
            "labId": row.get::<_, String>("lab_id"),
            "labRoom": row.get::<_, String>("lab_room"),
            "labBranch": row.get::<_, String>("lab_branch"),
            "borrowDate": row.get::<_, chrono::DateTime<chrono::Utc>>("borrow_date").to_rfc3339(),
            "expectedReturnedAt": row.get::<_, chrono::DateTime<chrono::Utc>>("expected_returned_at").to_rfc3339(),
            "status": row.get::<_, String>("status")
        })
    }).collect();

    Ok(results)
}

async fn get_user_activities_history(
    app_state: &AppState,
    user_id: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    if user_id.is_empty() {
        return Ok(Vec::new());
    }

    let user_uuid = match Uuid::parse_str(user_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(Vec::new()),
    };

    let sql = "
        WITH audit_activities AS (
          SELECT
            ia.id::text,
            d.id AS device_id,
            d.kind AS device_kind_id,
            dk.name AS device_name,
            dk.image AS device_image,
            l.room || ', ' || l.branch AS location,
            a.created_at AS activity_date,
            ia.status::text,
            a.note,
            'AUDIT' AS activity_type,
            NULL as prev_quality,
            NULL as after_quality
          FROM
            bench_inventory_assessments ia
            JOIN bench_inventory_assessments_devices iad ON ia.id = iad.assessing_id
            JOIN bench_devices d ON iad.device_id = d.id
            JOIN bench_device_kinds dk ON d.kind = dk.id
            JOIN bench_labs l ON ia.lab_id = l.id
            JOIN bench_activities a ON ia.id = a.id
          WHERE
            ia.accountant_id = $1
            AND d.deleted_at IS NULL
        ),
        maintenance_activities AS (
          SELECT
            m.id::text,
            d.id AS device_id,
            d.kind AS device_kind_id,
            dk.name AS device_name,
            dk.image AS device_image,
            l.room || ', ' || l.branch AS location,
            a.created_at AS activity_date,
            m.status::text,
            a.note,
            'MAINTENANCE' AS activity_type,
            NULL as prev_quality,
            NULL as after_quality
          FROM
            bench_maintenances m
            JOIN bench_maintenances_devices md ON m.id = md.maintaining_id
            JOIN bench_devices d ON md.device_id = d.id
            JOIN bench_device_kinds dk ON d.kind = dk.id
            JOIN bench_labs l ON d.lab_id = l.id
            JOIN bench_activities a ON m.id = a.id
          WHERE
            m.maintainer_id = $1
            AND d.deleted_at IS NULL
        ),
        transport_activities AS (
          SELECT
            s.id::text,
            d.id AS device_id,
            d.kind AS device_kind_id,
            dk.name AS device_name,
            dk.image AS device_image,
            start_lab.room || ', ' || start_lab.branch || ' → ' || arrive_lab.room || ', ' || arrive_lab.branch AS location,
            a.created_at AS activity_date,
            s.status::text,
            a.note,
            'TRANSPORT' AS activity_type,
            NULL as prev_quality,
            NULL as after_quality
          FROM
            bench_shipments s
            JOIN bench_shipments_devices sd ON s.id = sd.shipment_id
            JOIN bench_devices d ON sd.device_id = d.id
            JOIN bench_device_kinds dk ON d.kind = dk.id
            JOIN bench_labs start_lab ON s.start_lab_id = start_lab.id
            JOIN bench_labs arrive_lab ON s.arrive_lab_id = arrive_lab.id
            JOIN bench_activities a ON s.from_at = a.id
          WHERE
            (s.sender_id = $1 OR s.receiver_id = $1)
            AND d.deleted_at IS NULL
        ),
        returned_devices_activities AS (
          SELECT
            rd.returned_receipt_id::text AS id,
            d.id AS device_id,
            d.kind AS device_kind_id,
            dk.name AS device_name,
            dk.image AS device_image,
            l.room || ', ' || l.branch AS location,
            a.created_at AS activity_date,
            'returned' AS status,
            a.note,
            'RETURNED' AS activity_type,
            rd.prev_quality::text as prev_quality,
            rd.after_quality::text as after_quality
          FROM
            bench_receipts_devices rd
            JOIN bench_devices d ON rd.device_id = d.id
            JOIN bench_device_kinds dk ON d.kind = dk.id
            JOIN bench_receipts r ON rd.returned_receipt_id = r.id
            JOIN bench_labs l ON r.lab_id = l.id
            JOIN bench_activities a ON rd.return_id = a.id
          WHERE
            rd.returned_receipt_id IS NOT NULL
            AND (r.actor_id = $1 OR r.checker_id = $1)
            AND d.deleted_at IS NULL
        )

        SELECT * FROM audit_activities
        UNION ALL
        SELECT * FROM maintenance_activities
        UNION ALL
        SELECT * FROM transport_activities
        UNION ALL
        SELECT * FROM returned_devices_activities
        ORDER BY activity_date DESC
    ";

    let rows = client.query(sql, &[&user_uuid]).await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let results = rows
        .iter()
        .map(|row| {
            let device_image: serde_json::Value = row.get("device_image");

            json!({
                "id": row.get::<_, String>("id"),
                "type": row.get::<_, String>("activity_type"),
                "deviceId": row.get::<_, String>("device_id"),
                "deviceKindId": row.get::<_, String>("device_kind_id"),
                "deviceName": row.get::<_, String>("device_name"),
                "deviceImage": device_image,
                "location": row.get::<_, String>("location"),
                "date": row.get::<_, chrono::DateTime<chrono::Utc>>("activity_date").to_rfc3339(),
                "status": row.get::<_, String>("status"),
                "note": row.try_get::<_, String>("note").ok(),
                "prevQuality": row.try_get::<_, String>("prev_quality").ok(),
                "afterQuality": row.try_get::<_, String>("after_quality").ok()
            })
        })
        .collect();

    Ok(results)
}

fn benchmark_user(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for user benchmarks");

    // Use ensure_bench_env which will check if we have the correct number of records
    // and only recreate the data if needed
    let app_state = rt.block_on(ensure_bench_env());

    // Create a test user ID for benchmarks
    let test_user_id = rt.block_on(async {
        let client = app_state
            .db
            .get_client()
            .await
            .expect("Failed to get client");

        // Insert a test user directly
        let query = "INSERT INTO bench_users (id, name, email, image)
            VALUES (gen_random_uuid(), $1, $2, $3)
            RETURNING id::text";

        let name = "Benchmark Test User";
        let email = format!(
            "benchmark.test.{}@example.com",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );
        let image = json!({"url": "https://example.com/avatar_test.jpg"});

        match client.query_one(query, &[&name, &email, &image]).await {
            Ok(row) => row.get::<_, String>(0),
            Err(err) => {
                eprintln!("Error creating test user: {}", err);
                "".to_string()
            }
        }
    });

    let mut group = c.benchmark_group("User Operations");

    // Benchmark 1: Fetch users
    let fetch_params = QueryParams {
        table: "bench_users".to_string(),
        columns: None,
        conditions: None,
        order_by: Some(vec![("bench_users.name".to_string(), true)]),
        limit: Some(10),
        offset: Some(0),
        joins: None,
    };

    group.bench_with_input(
        BenchmarkId::new("Fetch Users", 10),
        &fetch_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                match fetch_users(&app_state, p).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Users benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        },
    );

    // Benchmark 2: Fetch user details (if test user was created successfully)
    if !test_user_id.is_empty() {
        let details_params = QueryParams {
            table: "bench_users".to_string(),
            columns: None,
            conditions: Some(vec![("id".to_string(), json!(test_user_id))]),
            order_by: None,
            limit: None,
            offset: None,
            joins: None,
        };

        group.bench_with_input(
            BenchmarkId::new("Fetch User Details", 1),
            &details_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match fetch_user_details(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Fetch User Details benchmark: {}", err);
                            black_box(json!({"error": err.to_string()}));
                        }
                    }
                });
            },
        );
    }

    // Benchmark 3: Get Borrowed History By User
    if !test_user_id.is_empty() {
        group.bench_with_input(
            BenchmarkId::new("Get Borrowed History By User", 1),
            &test_user_id,
            |b, user_id| {
                b.to_async(&rt).iter(|| async {
                    match get_borrowed_history_by_user(&app_state, user_id).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Get Borrowed History benchmark: {}", err);
                            black_box(Vec::<serde_json::Value>::new());
                        }
                    }
                });
            },
        );
    }

    // Benchmark 4: Get User Activities History
    if !test_user_id.is_empty() {
        group.bench_with_input(
            BenchmarkId::new("Get User Activities History", 1),
            &test_user_id,
            |b, user_id| {
                b.to_async(&rt).iter(|| async {
                    match get_user_activities_history(&app_state, user_id).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Get User Activities History benchmark: {}", err);
                            black_box(Vec::<serde_json::Value>::new());
                        }
                    }
                });
            },
        );
    }

    group.finish();

    // We no longer clean up tables to preserve the database state
    // and avoid recreating data for each benchmark run
    println!("Benchmark completed. Database state preserved for future runs.");
}

criterion_group!(benches, benchmark_user);
criterion_main!(benches);
