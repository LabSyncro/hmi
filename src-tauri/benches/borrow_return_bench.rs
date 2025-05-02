use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use hmi_lib::commands::db_commands::{InsertParams, QueryParams};

mod common;
use common::{ensure_bench_env, AppState};

async fn fetch_ready_borrow_devices(
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
        _ => "ORDER BY di.name ASC".to_string(),
    };

    let sql = format!(
        "WITH healthy_devices AS (
            SELECT
                d.kind,
                d.lab_id,
                COUNT(*) as quantity
            FROM
                bench_devices d
            WHERE
                d.status::text = 'healthy'
                AND d.deleted_at IS NULL
            GROUP BY
                d.kind, d.lab_id
        ),
        device_info AS (
            SELECT
                dk.id as kind,
                dk.name,
                dk.image,
                hd.quantity,
                l.name as place
            FROM
                healthy_devices hd
                JOIN bench_device_kinds dk ON hd.kind = dk.id
                LEFT JOIN bench_labs l ON hd.lab_id = l.id
        )
        SELECT
            di.kind,
            di.name,
            di.image,
            di.quantity,
            di.place,
            COUNT(*) OVER() as total_count
        FROM
            device_info di
        {} LIMIT {} OFFSET {}",
        order_clause, limit, offset
    );

    let rows = client.query(&sql, &[]).await?;

    let mut results = Vec::with_capacity(rows.len());
    let mut _total_count = 0;

    for row in rows {
        let quantity: i64 = row.get("quantity");
        _total_count = row.get::<_, i64>("total_count");

        results.push(json!({
            "kind": row.get::<_, Uuid>("kind").to_string(),
            "name": row.get::<_, String>("name"),
            "image": row.get::<_, serde_json::Value>("image"),
            "quantity": quantity,
            "place": row.get::<_, String>("place")
        }));
    }

    Ok(results)
}

async fn fetch_borrowing_devices(
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
        _ => "ORDER BY borrowed_at DESC".to_string(),
    };

    let sql = format!(
        "WITH active_borrowings AS (
            SELECT
                rd.borrowed_receipt_id,
                rd.return_id,
                rd.expected_returned_at,
                a.created_at as borrowed_at,
                COUNT(*) as device_count,
                COUNT(CASE WHEN rd.return_id IS NOT NULL THEN 1 END) as returned_count
            FROM
                bench_receipts_devices rd
                JOIN bench_activities a ON rd.borrow_id = a.id
            WHERE
                rd.return_id IS NULL
            GROUP BY
                rd.borrowed_receipt_id,
                rd.return_id,
                rd.expected_returned_at,
                a.created_at
        ),
        borrower_info AS (
            SELECT
                r.id as receipt_code,
                u.name as borrower_name,
                u.image as borrower_image,
                l.name as borrowed_place,
                ab.borrowed_at,
                ab.expected_returned_at,
                ab.device_count as total_qty,
                ab.returned_count as returned_qty,
                CASE
                    WHEN ab.expected_returned_at < CURRENT_TIMESTAMP THEN 'late'
                    ELSE 'on_time'
                END as status,
                CASE
                    WHEN ab.return_id IS NULL THEN 'borrowing'
                    ELSE 'returned'
                END as borrow_state
            FROM
                active_borrowings ab
                JOIN bench_receipts r ON ab.borrowed_receipt_id = r.id
                JOIN bench_users u ON r.actor_id = u.id
                JOIN bench_labs l ON r.lab_id = l.id
            WHERE
                u.deleted_at IS NULL
        )
        SELECT
            bi.*,
            COUNT(*) OVER() as total_count
        FROM
            borrower_info bi
        {} LIMIT {} OFFSET {}",
        order_clause, limit, offset
    );

    let rows = client.query(&sql, &[]).await?;

    let mut results = Vec::with_capacity(rows.len());
    let mut _total_count = 0;

    for row in rows {
        _total_count = row.get::<_, i64>("total_count");
        let borrowed_at: chrono::DateTime<chrono::Utc> = row.get("borrowed_at");
        let expected_returned_at: chrono::DateTime<chrono::Utc> = row.get("expected_returned_at");

        results.push(json!({
            "receipt_code": row.get::<_, Uuid>("receipt_code").to_string(),
            "borrower_name": row.get::<_, String>("borrower_name"),
            "borrower_image": row.get::<_, serde_json::Value>("borrower_image"),
            "total_qty": row.get::<_, i64>("total_qty"),
            "returned_qty": row.get::<_, i64>("returned_qty"),
            "borrowed_place": row.get::<_, String>("borrowed_place"),
            "borrowed_at": borrowed_at.to_rfc3339(),
            "expected_returned_at": expected_returned_at.to_rfc3339(),
            "status": row.get::<_, String>("status"),
            "borrow_state": row.get::<_, String>("borrow_state")
        }));
    }

    Ok(results)
}

async fn fetch_returned_devices(
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
        _ => "ORDER BY returned_at DESC".to_string(),
    };

    let sql = format!(
        "WITH returned_items AS (
            SELECT
                rd.returned_receipt_id,
                rd.expected_returned_at,
                rd.note,
                a.created_at as returned_at,
                COUNT(*) as device_count
            FROM
                bench_receipts_devices rd
                JOIN bench_activities a ON rd.return_id = a.id
            WHERE
                rd.return_id IS NOT NULL
            GROUP BY
                rd.returned_receipt_id,
                rd.expected_returned_at,
                rd.note,
                a.created_at
        ),
        return_info AS (
            SELECT
                r.id as receipt_code,
                u.name as returned_name,
                u.image as returned_image,
                l.name as returned_place,
                ri.returned_at,
                ri.device_count as quantity,
                ri.note,
                CASE
                    WHEN ri.expected_returned_at < ri.returned_at THEN 'late'
                    ELSE 'on_time'
                END as status
            FROM
                returned_items ri
                JOIN bench_receipts r ON ri.returned_receipt_id = r.id
                JOIN bench_users u ON r.actor_id = u.id
                JOIN bench_labs l ON r.lab_id = l.id
            WHERE
                u.deleted_at IS NULL
        )
        SELECT
            ri.*,
            COUNT(*) OVER() as total_count
        FROM
            return_info ri
        {} LIMIT {} OFFSET {}",
        order_clause, limit, offset
    );

    let rows = client.query(&sql, &[]).await?;

    let mut results = Vec::with_capacity(rows.len());
    let mut _total_count = 0;

    for row in rows {
        _total_count = row.get::<_, i64>("total_count");
        let returned_at: chrono::DateTime<chrono::Utc> = row.get("returned_at");

        results.push(json!({
            "receipt_code": row.get::<_, Uuid>("receipt_code").to_string(),
            "returned_name": row.get::<_, String>("returned_name"),
            "returned_image": row.get::<_, serde_json::Value>("returned_image"),
            "quantity": row.get::<_, i64>("quantity"),
            "returned_place": row.get::<_, String>("returned_place"),
            "returned_at": returned_at.to_rfc3339(),
            "status": row.get::<_, String>("status"),
            "note": row.get::<_, Option<String>>("note")
        }));
    }

    Ok(results)
}

async fn create_receipt(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let receipt_uuid = Uuid::new_v4();
    let receipt_id = receipt_uuid.to_string();

    let borrower_id = match params.value.get("borrowerId").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(json!({ "error": "Borrower ID is required" })),
    };
    let borrow_checker_id = match params.value.get("borrowCheckerId").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(json!({ "error": "Borrow checker ID is required" })),
    };
    let borrowed_lab_id = match params.value.get("borrowedLabId").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(json!({ "error": "Borrowed lab ID is required" })),
    };

    let devices_vec = params
        .value
        .get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;

    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for borrowing" }));
    }

    let mut device_values = Vec::new();
    for device in devices {
        let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if device_id.is_empty() {
            continue;
        }

        let expected_returned_at = match device.get("expectedReturnedAt").and_then(|v| v.as_str()) {
            Some(date_str) if date_str.starts_with("NOW()") => date_str.to_string(),
            Some(date_str) => format!("'{}'", date_str),
            None => "NOW() + INTERVAL '7 days'".to_string(),
        };

        let expected_returned_lab_id =
            match device.get("expectedReturnedLabId").and_then(|v| v.as_str()) {
                Some(id) if !id.is_empty() => id,
                _ => borrowed_lab_id,
            };
        let prev_quality = match device.get("prevQuality").and_then(|v| v.as_str()) {
            Some(quality) if !quality.is_empty() => quality,
            _ => continue,
        };

        device_values.push(format!(
            "('{}', {}, '{}', '{}'::bench_device_status)",
            device_id, expected_returned_at, expected_returned_lab_id, prev_quality
        ));
    }

    if device_values.is_empty() {
        return Ok(json!({ "error": "No valid devices specified for borrowing" }));
    }

    let device_values_str = device_values.join(", ");

    let query = format!(
        "WITH new_receipt AS (
            INSERT INTO bench_receipts (id, actor_id, checker_id, lab_id)
            VALUES ('{}', '{}', '{}', '{}')
            RETURNING id
        ),
        new_activity AS (
            INSERT INTO bench_activities (id, type)
            VALUES (gen_random_uuid(), 'borrow'::bench_activity_type)
            RETURNING id
        ),
        device_data(device_id, expected_returned_at, expected_returned_lab_id, prev_quality) AS (
            VALUES {}
        ),
        insert_devices AS (
            INSERT INTO bench_receipts_devices (
                borrowed_receipt_id, device_id, borrow_id,
                expected_returned_at, expected_returned_lab_id, prev_quality
            )
            SELECT
                '{}',
                dd.device_id::uuid,
                (SELECT id FROM new_activity),
                dd.expected_returned_at::timestamptz,
                dd.expected_returned_lab_id::uuid,
                dd.prev_quality
            FROM device_data dd
            RETURNING device_id
        ),
        update_devices AS (
            UPDATE bench_devices
            SET status = 'borrowing'::bench_device_status
            WHERE id IN (SELECT device_id FROM insert_devices)
            RETURNING id
        )
        SELECT '{}' as id",
        receipt_id,
        borrower_id,
        borrow_checker_id,
        borrowed_lab_id,
        device_values_str,
        receipt_id,
        receipt_id
    );

    client.query_one(&query, &[]).await?;

    Ok(json!({
        "success": true,
        "id": receipt_id
    }))
}

async fn return_receipt(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let receipt_uuid = Uuid::new_v4();
    let receipt_id = receipt_uuid.to_string();

    let returner_id = match params.value.get("returnerId").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(json!({ "error": "Returner ID is required" })),
    };
    let return_checker_id = match params
        .value
        .get("returnedCheckerId")
        .and_then(|v| v.as_str())
    {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(json!({ "error": "Return checker ID is required" })),
    };
    let returned_lab_id = match params.value.get("returnedLabId").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(json!({ "error": "Returned lab ID is required" })),
    };
    let note = params.value.get("note").and_then(|v| v.as_str());

    let devices_vec = params
        .value
        .get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;

    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for return" }));
    }

    let mut device_values = Vec::new();
    for device in devices {
        let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if device_id.is_empty() {
            continue;
        }

        let after_quality = match device.get("afterQuality").and_then(|v| v.as_str()) {
            Some(quality) if !quality.is_empty() => quality,
            _ => continue,
        };

        device_values.push(format!(
            "('{}', '{}'::bench_device_status)",
            device_id, after_quality
        ));
    }

    if device_values.is_empty() {
        return Ok(json!({ "error": "No valid devices specified for return" }));
    }

    let device_values_str = device_values.join(", ");
    let note_value = note.map_or("NULL".to_string(), |n| {
        format!("'{}'", n.replace("'", "''"))
    });

    let query = format!(
        "WITH new_receipt AS (
            INSERT INTO bench_receipts (id, actor_id, checker_id, lab_id)
            VALUES ('{}', '{}', '{}', '{}')
            RETURNING id
        ),
        new_activity AS (
            INSERT INTO bench_activities (id, type, note)
            VALUES (gen_random_uuid(), 'return'::bench_activity_type, {})
            RETURNING id
        ),
        device_data(device_id, after_quality) AS (
            VALUES {}
        ),
        update_receipts_devices AS (
            UPDATE bench_receipts_devices
            SET
                returned_receipt_id = '{}',
                return_id = (SELECT id FROM new_activity),
                after_quality = dd.after_quality
            FROM device_data dd
            WHERE bench_receipts_devices.device_id = dd.device_id::uuid
                AND bench_receipts_devices.returned_receipt_id IS NULL
            RETURNING bench_receipts_devices.device_id
        ),
        update_devices AS (
            UPDATE bench_devices
            SET status = dd.after_quality
            FROM device_data dd
            WHERE bench_devices.id = dd.device_id::uuid
            RETURNING id
        )
        SELECT '{}' as id",
        receipt_id,
        returner_id,
        return_checker_id,
        returned_lab_id,
        note_value,
        device_values_str,
        receipt_id,
        receipt_id
    );

    client.query_one(&query, &[]).await?;

    Ok(json!({
        "success": true,
        "id": receipt_id
    }))
}

fn benchmark_borrow_return(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for borrow-return benchmarks");

    let app_state = rt.block_on(async { ensure_bench_env().await });

    let real_device_ids = rt.block_on(async {
        let client = app_state
            .db
            .get_client()
            .await
            .expect("Failed to get client");
        let query = "SELECT id::text FROM bench_devices WHERE status = 'healthy' LIMIT 2";
        let rows = client
            .query(query, &[])
            .await
            .expect("Failed to fetch device IDs");

        if rows.len() < 2 {
            println!("Warning: Not enough healthy devices found for benchmarking");
            Vec::new()
        } else {
            rows.iter()
                .map(|row| row.get::<_, String>(0))
                .collect::<Vec<_>>()
        }
    });

    let mut group = c.benchmark_group("Borrow-Return Operations");

    // Benchmark 1: Fetch ready to borrow devices
    let ready_borrow_params = QueryParams {
        table: "bench_devices".to_string(),
        columns: None,
        conditions: None,
        order_by: Some(vec![("name".to_string(), true)]),
        limit: Some(10),
        offset: Some(0),
        joins: None,
    };

    group.bench_with_input(
        BenchmarkId::new("Fetch Ready Borrow Devices", 10),
        &ready_borrow_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                match fetch_ready_borrow_devices(&app_state, p).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Ready Borrow Devices benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        },
    );

    // Benchmark 2: Fetch borrowing devices
    let borrowing_params = QueryParams {
        table: "bench_receipts_devices".to_string(),
        columns: None,
        conditions: None,
        order_by: Some(vec![("borrowed_at".to_string(), false)]),
        limit: Some(10),
        offset: Some(0),
        joins: None,
    };

    group.bench_with_input(
        BenchmarkId::new("Fetch Borrowing Devices", 10),
        &borrowing_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                match fetch_borrowing_devices(&app_state, p).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Borrowing Devices benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        },
    );

    // Benchmark 3: Fetch returned devices
    let returned_params = QueryParams {
        table: "bench_receipts_devices".to_string(),
        columns: None,
        conditions: None,
        order_by: Some(vec![("returned_at".to_string(), false)]),
        limit: Some(10),
        offset: Some(0),
        joins: None,
    };

    group.bench_with_input(
        BenchmarkId::new("Fetch Returned Devices", 10),
        &returned_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                match fetch_returned_devices(&app_state, p).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Returned Devices benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        },
    );

    // Benchmark 4: Create borrow receipt - only run if we have real device IDs
    let _create_borrow_benchmark = if real_device_ids.len() >= 2 {
        let (user_id, checker_id, lab_id) = rt.block_on(async {
            let client = app_state
                .db
                .get_client()
                .await
                .expect("Failed to get client");

            let users = client
                .query("SELECT id::text FROM bench_users LIMIT 2", &[])
                .await
                .expect("Failed to fetch user IDs");

            let user_id = if !users.is_empty() {
                users[0].get::<_, String>(0)
            } else {
                println!("Warning: No users found for benchmarking");
                return ("".to_string(), "".to_string(), "".to_string());
            };

            let checker_id = if users.len() > 1 {
                users[1].get::<_, String>(0)
            } else {
                user_id.clone()
            };

            let lab_id = client
                .query_one("SELECT id::text FROM bench_labs LIMIT 1", &[])
                .await
                .map(|row| row.get::<_, String>(0))
                .unwrap_or_else(|_| "".to_string());

            (user_id, checker_id, lab_id)
        });

        if user_id.is_empty() || lab_id.is_empty() {
            println!("Warning: Could not find users or lab for borrow benchmark");
            false
        } else {
            let borrow_params = InsertParams {
                table: "bench_receipts".to_string(),
                value: json!({
                    "borrowerId": user_id,
                    "borrowCheckerId": checker_id,
                    "borrowedLabId": lab_id,
                    "devices": [
                        {
                            "id": real_device_ids[0],
                            "expectedReturnedAt": "NOW() + INTERVAL '7 days'",
                            "prevQuality": "healthy"
                        },
                        {
                            "id": real_device_ids[1],
                            "expectedReturnedAt": "NOW() + INTERVAL '14 days'",
                            "prevQuality": "healthy"
                        }
                    ]
                }),
            };

            group.bench_with_input(
                BenchmarkId::new("Create Borrow Receipt", 2),
                &borrow_params,
                |b, p| {
                    b.to_async(&rt).iter(|| async {
                        match create_receipt(&app_state, p).await {
                            Ok(result) => {
                                black_box(result);
                            }
                            Err(err) => {
                                eprintln!("Error in Create Borrow Receipt benchmark: {}", err);
                                black_box(json!({"error": err.to_string()}));
                            }
                        }
                    });
                },
            );

            true
        }
    } else {
        println!("Warning: Not enough device IDs for borrow receipt benchmark");
        false
    };

    // Benchmark 5: Return receipt - only run if we have real device IDs
    let _return_benchmark = if real_device_ids.len() >= 2 {
        let (user_id, checker_id, lab_id) = rt.block_on(async {
            let client = app_state
                .db
                .get_client()
                .await
                .expect("Failed to get client");

            let users = client
                .query("SELECT id::text FROM bench_users LIMIT 2", &[])
                .await
                .expect("Failed to fetch user IDs");

            let user_id = if !users.is_empty() {
                users[0].get::<_, String>(0)
            } else {
                println!("Warning: No users found for benchmarking");
                return ("".to_string(), "".to_string(), "".to_string());
            };

            let checker_id = if users.len() > 1 {
                users[1].get::<_, String>(0)
            } else {
                user_id.clone()
            };

            let lab_id = client
                .query_one("SELECT id::text FROM bench_labs LIMIT 1", &[])
                .await
                .map(|row| row.get::<_, String>(0))
                .unwrap_or_else(|_| "".to_string());

            (user_id, checker_id, lab_id)
        });

        if user_id.is_empty() || lab_id.is_empty() {
            println!("Warning: Could not find users or lab for return benchmark");
            false
        } else {
            let return_params = InsertParams {
                table: "bench_receipts".to_string(),
                value: json!({
                    "returnerId": user_id,
                    "returnedCheckerId": checker_id,
                    "returnedLabId": lab_id,
                    "devices": [
                        {
                            "id": real_device_ids[0],
                            "afterQuality": "healthy"
                        },
                        {
                            "id": real_device_ids[1],
                            "afterQuality": "broken"
                        }
                    ],
                    "note": "Returned after lab session"
                }),
            };

            group.bench_with_input(
                BenchmarkId::new("Return Receipt", 2),
                &return_params,
                |b, p| {
                    b.to_async(&rt).iter(|| async {
                        match return_receipt(&app_state, p).await {
                            Ok(result) => {
                                black_box(result);
                            }
                            Err(err) => {
                                eprintln!("Error in Return Receipt benchmark: {}", err);
                                black_box(json!({"error": err.to_string()}));
                            }
                        }
                    });
                },
            );

            true
        }
    } else {
        println!("Warning: Not enough device IDs for return receipt benchmark");
        false
    };

    group.finish();

    println!("Benchmark completed. Database state preserved for future runs.");
}

criterion_group!(benches, benchmark_borrow_return);
criterion_main!(benches);
