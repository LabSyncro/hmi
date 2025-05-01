use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use hmi_lib::commands::db_commands::{InsertParams, QueryParams};

mod common;
use common::{cleanup_test_tables, setup_bench_env, AppState};

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
        _ => "ORDER BY bench_device_kinds.name ASC".to_string(),
    };

    let sql = format!(
        "SELECT 
            bench_device_kinds.id as kind,
            bench_device_kinds.name,
            bench_device_kinds.image,
            COUNT(*) as quantity,
            bench_labs.name as place,
            COUNT(*) OVER() as total_count
        FROM 
            bench_devices
            JOIN bench_device_kinds ON bench_devices.kind = bench_device_kinds.id
            LEFT JOIN bench_labs ON bench_devices.lab_id = bench_labs.id
        WHERE
            bench_devices.status::text = 'healthy'
            AND bench_devices.deleted_at IS NULL
        GROUP BY 
            bench_device_kinds.id,
            bench_device_kinds.name,
            bench_device_kinds.image,
            bench_labs.name
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
        _ => "ORDER BY bench_activities.created_at DESC".to_string(),
    };

    let sql = format!(
        "SELECT 
            bench_receipts.id as receipt_code,
            bench_users.name as borrower_name,
            bench_users.image as borrower_image,
            COUNT(*) as total_qty,
            COUNT(CASE WHEN bench_receipts_devices.return_id IS NOT NULL THEN 1 END) as returned_qty,
            bench_labs.name as borrowed_place,
            bench_activities.created_at as borrowed_at,
            bench_receipts_devices.expected_returned_at,
            CASE 
                WHEN bench_receipts_devices.expected_returned_at < CURRENT_TIMESTAMP THEN 'late'
                ELSE 'on_time'
            END as status,
            CASE 
                WHEN bench_receipts_devices.return_id IS NULL THEN 'borrowing'
                ELSE 'returned'
            END as borrow_state,
            COUNT(*) OVER() as total_count
        FROM 
            bench_receipts_devices
            JOIN bench_receipts ON bench_receipts_devices.borrowed_receipt_id = bench_receipts.id
            JOIN bench_users ON bench_receipts.actor_id = bench_users.id
            JOIN bench_labs ON bench_receipts.lab_id = bench_labs.id
            JOIN bench_activities ON bench_receipts_devices.borrow_id = bench_activities.id
        WHERE 
            bench_users.deleted_at IS NULL
            AND bench_receipts_devices.return_id IS NULL
        GROUP BY 
            bench_receipts.id,
            bench_users.name,
            bench_users.image,
            bench_labs.name,
            bench_activities.created_at,
            bench_receipts_devices.expected_returned_at,
            bench_receipts_devices.return_id
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
        _ => "ORDER BY bench_activities.created_at DESC".to_string(),
    };

    let sql = format!(
        "SELECT 
            bench_receipts.id as receipt_code,
            bench_users.name as returned_name,
            bench_users.image as returned_image,
            COUNT(*) as quantity,
            bench_labs.name as returned_place,
            bench_activities.created_at as returned_at,
            CASE 
                WHEN bench_receipts_devices.expected_returned_at < bench_activities.created_at THEN 'late'
                ELSE 'on_time'
            END as status,
            bench_receipts_devices.note,
            COUNT(*) OVER() as total_count
        FROM 
            bench_receipts_devices
            JOIN bench_receipts ON bench_receipts_devices.returned_receipt_id = bench_receipts.id
            JOIN bench_users ON bench_receipts_devices.return_id IS NOT NULL
            JOIN bench_labs ON bench_receipts.lab_id = bench_labs.id
            JOIN bench_activities ON bench_receipts_devices.return_id = bench_activities.id
        WHERE 
            bench_users.deleted_at IS NULL
            AND bench_receipts_devices.return_id IS NOT NULL
        GROUP BY 
            bench_receipts.id,
            bench_users.name,
            bench_users.image,
            bench_labs.name,
            bench_activities.created_at,
            bench_receipts_devices.expected_returned_at,
            bench_receipts_devices.note
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
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let receipt_uuid = Uuid::new_v4();
    let receipt_id = receipt_uuid.to_string();

    let borrower_id = params
        .value
        .get("borrowerId")
        .and_then(|v| v.as_str())
        .unwrap_or("11111111-1111-1111-1111-111111111111");
    let borrow_checker_id = params
        .value
        .get("borrowCheckerId")
        .and_then(|v| v.as_str())
        .unwrap_or("22222222-2222-2222-2222-222222222222");
    let borrowed_lab_id = params
        .value
        .get("borrowedLabId")
        .and_then(|v| v.as_str())
        .unwrap_or("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");

    let devices_vec = params
        .value
        .get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;

    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for borrowing" }));
    }

    let receipt_query = format!(
        "INSERT INTO bench_receipts (id, actor_id, checker_id, lab_id)
        VALUES ('{}', '{}', '{}', '{}')",
        receipt_id, borrower_id, borrow_checker_id, borrowed_lab_id
    );
    transaction.execute(&receipt_query, &[]).await?;

    let activity_query = "INSERT INTO bench_activities (id, type) VALUES (gen_random_uuid(), 'borrow'::bench_activity_type) RETURNING id::text";
    let activity_row = transaction.query_one(activity_query, &[]).await?;
    let activity_id = activity_row.get::<_, String>(0);

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

        let expected_returned_lab_id = device
            .get("expectedReturnedLabId")
            .and_then(|v| v.as_str())
            .unwrap_or(borrowed_lab_id);
        let prev_quality = device
            .get("prevQuality")
            .and_then(|v| v.as_str())
            .unwrap_or("healthy");

        let device_query = format!(
            "INSERT INTO bench_receipts_devices (
                borrowed_receipt_id, device_id, borrow_id, 
                expected_returned_at, expected_returned_lab_id, prev_quality
            ) 
            VALUES (
                '{}', '{}', '{}', 
                {}, '{}', '{}'::bench_device_status
            )",
            receipt_id,
            device_id,
            activity_id,
            expected_returned_at,
            expected_returned_lab_id,
            prev_quality
        );

        transaction.execute(&device_query, &[]).await?;

        let update_query = format!(
            "UPDATE bench_devices SET status = 'borrowing'::bench_device_status WHERE id = '{}'",
            device_id
        );
        transaction.execute(&update_query, &[]).await?;
    }

    transaction.commit().await?;

    Ok(json!({
        "success": true,
        "id": receipt_id
    }))
}

async fn return_receipt(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let receipt_uuid = Uuid::new_v4();
    let receipt_id = receipt_uuid.to_string();

    let returner_id = params
        .value
        .get("returnerId")
        .and_then(|v| v.as_str())
        .unwrap_or("11111111-1111-1111-1111-111111111111");
    let return_checker_id = params
        .value
        .get("returnedCheckerId")
        .and_then(|v| v.as_str())
        .unwrap_or("22222222-2222-2222-2222-222222222222");
    let returned_lab_id = params
        .value
        .get("returnedLabId")
        .and_then(|v| v.as_str())
        .unwrap_or("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
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

    let receipt_query = format!(
        "INSERT INTO bench_receipts (id, actor_id, checker_id, lab_id)
        VALUES ('{}', '{}', '{}', '{}')",
        receipt_id, returner_id, return_checker_id, returned_lab_id
    );
    transaction.execute(&receipt_query, &[]).await?;

    let note_value = note.map_or("NULL".to_string(), |n| format!("'{}'", n));
    let activity_query = format!(
        "INSERT INTO bench_activities (id, type, note) 
        VALUES (gen_random_uuid(), 'return'::bench_activity_type, {}) 
        RETURNING id::text",
        note_value
    );
    let activity_row = transaction.query_one(&activity_query, &[]).await?;
    let activity_id = activity_row.get::<_, String>(0);

    for device in devices {
        let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if device_id.is_empty() {
            continue;
        }

        let after_quality = device
            .get("afterQuality")
            .and_then(|v| v.as_str())
            .unwrap_or("healthy");

        let update_receipt_query = format!(
            "UPDATE bench_receipts_devices 
            SET returned_receipt_id = '{}', 
                return_id = '{}',
                after_quality = '{}'::bench_device_status
            WHERE device_id = '{}' 
                AND returned_receipt_id IS NULL",
            receipt_id, activity_id, after_quality, device_id
        );
        transaction.execute(&update_receipt_query, &[]).await?;

        let update_device_query = format!(
            "UPDATE bench_devices 
            SET status = '{}'::bench_device_status 
            WHERE id = '{}'",
            after_quality, device_id
        );
        transaction.execute(&update_device_query, &[]).await?;
    }

    transaction.commit().await?;

    Ok(json!({
        "success": true,
        "id": receipt_id
    }))
}

fn benchmark_borrow_return(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for borrow-return benchmarks");
    let app_state = rt.block_on(setup_bench_env());

    let real_device_ids = rt.block_on(async {
        let client = app_state
            .db
            .get_client()
            .await
            .expect("Failed to get client");
        let query = "SELECT id::text FROM bench_devices LIMIT 2";
        let rows = client
            .query(query, &[])
            .await
            .expect("Failed to fetch device IDs");

        if rows.len() < 2 {
            vec![
                "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_string(),
                "aaaaaaaa-bbbb-cccc-dddd-ffffffffffff".to_string(),
            ]
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
        order_by: Some(vec![("bench_device_kinds.name".to_string(), true)]),
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
        order_by: Some(vec![("bench_activities.created_at".to_string(), false)]),
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
        order_by: Some(vec![("bench_activities.created_at".to_string(), false)]),
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

    // Benchmark 4: Create borrow receipt
    let borrow_params = InsertParams {
        table: "bench_receipts".to_string(),
        value: json!({
            "borrowerId": "11111111-1111-1111-1111-111111111111",
            "borrowCheckerId": "22222222-2222-2222-2222-222222222222",
            "borrowedLabId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "devices": [
                {
                    "id": real_device_ids.get(0).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_string()),
                    "expectedReturnedAt": "NOW() + INTERVAL '7 days'",
                    "prevQuality": "healthy"
                },
                {
                    "id": real_device_ids.get(1).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-ffffffffffff".to_string()),
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

    // Benchmark 5: Return receipt
    let return_params = InsertParams {
        table: "bench_receipts".to_string(),
        value: json!({
            "returnerId": "11111111-1111-1111-1111-111111111111",
            "returnedCheckerId": "22222222-2222-2222-2222-222222222222",
            "returnedLabId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "devices": [
                {
                    "id": real_device_ids.get(0).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_string()),
                    "afterQuality": "healthy"
                },
                {
                    "id": real_device_ids.get(1).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-ffffffffffff".to_string()),
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

    group.finish();

    rt.block_on(cleanup_test_tables(&app_state.db))
        .expect("Failed to clean up test tables");
}

criterion_group!(benches, benchmark_borrow_return);
criterion_main!(benches);
