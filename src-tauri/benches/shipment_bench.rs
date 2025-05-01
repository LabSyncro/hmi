use criterion::{black_box, BenchmarkId, criterion_group, criterion_main, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use hmi_lib::commands::db_commands::{InsertParams, QueryParams};

mod common;
use common::{setup_bench_env, cleanup_test_tables, AppState};

async fn fetch_shipments(
    app_state: &AppState,
    params: &QueryParams,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    let order_clause = match params.order_by {
        Some(ref order) if !order.is_empty() => {
            let (field, is_asc) = &order[0];
            format!("ORDER BY {} {}", field, if *is_asc { "ASC" } else { "DESC" })
        }
        _ => "ORDER BY a.created_at DESC".to_string(),
    };

    let sql = format!(
        "SELECT 
            s.id::text, 
            s.status,
            u.name as shipper_name,
            u.id::text as shipper_id,
            l_from.name as from_lab_name,
            l_from.id::text as from_lab_id,
            l_to.name as to_lab_name,
            l_to.id::text as to_lab_id,
            a.created_at as created_at,
            s.finished_at,
            COUNT(sd.id) as device_count,
            a.id::text as activity_id,
            a.note
        FROM 
            bench_shipments s
            JOIN bench_activities a ON s.id = a.id
            JOIN bench_users u ON s.shipper_id = u.id
            JOIN bench_labs l_from ON s.from_lab_id = l_from.id
            JOIN bench_labs l_to ON s.to_lab_id = l_to.id
            LEFT JOIN bench_shipments_devices sd ON s.id = sd.shipment_id
        GROUP BY
            s.id, s.status, u.name, u.id, l_from.name, l_from.id, l_to.name, l_to.id, 
            a.created_at, s.finished_at, a.id, a.note
        {} LIMIT {} OFFSET {}",
        order_clause, limit, offset
    );

    let rows = client.query(&sql, &[]).await?;

    let mut results = Vec::with_capacity(rows.len());
    let mut _total_count = 0;

    for row in rows {
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let finished_at: Option<chrono::DateTime<chrono::Utc>> = row.get("finished_at");
        
        results.push(json!({
            "id": row.get::<_, String>(0),
            "status": row.get::<_, String>(1),
            "shipperName": row.get::<_, String>(2),
            "shipperId": row.get::<_, String>(3),
            "fromLabName": row.get::<_, String>(4),
            "fromLabId": row.get::<_, String>(5),
            "toLabName": row.get::<_, String>(6),
            "toLabId": row.get::<_, String>(7),
            "createdAt": created_at.to_rfc3339(),
            "finishedAt": finished_at.map(|dt| dt.to_rfc3339()),
            "deviceCount": row.get::<_, i64>(10),
            "activityId": row.get::<_, String>(11),
            "note": row.get::<_, Option<String>>(12)
        }));
        
        _total_count += 1;
    }

    Ok(results)
}

async fn fetch_shipment_details(
    app_state: &AppState,
    params: &QueryParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let shipment_id = params
        .conditions
        .as_ref()
        .and_then(|conds| conds.first())
        .and_then(|(_, val)| val.as_str())
        .unwrap_or("");

    if shipment_id.is_empty() {
        return Ok(json!({ "error": "Shipment ID is required" }));
    }

    let shipment_uuid = match Uuid::parse_str(shipment_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid shipment ID format" })),
    };

    // First, get the shipment details
    let shipment_query = "
        SELECT 
            s.id::text, 
            s.status,
            u.name as shipper_name,
            u.id::text as shipper_id,
            l_from.name as from_lab_name,
            l_from.id::text as from_lab_id,
            l_to.name as to_lab_name,
            l_to.id::text as to_lab_id,
            a.created_at as created_at,
            s.finished_at,
            a.note,
            a.id::text as activity_id
        FROM 
            bench_shipments s
            JOIN bench_activities a ON s.id = a.id
            JOIN bench_users u ON s.shipper_id = u.id
            JOIN bench_labs l_from ON s.from_lab_id = l_from.id
            JOIN bench_labs l_to ON s.to_lab_id = l_to.id
        WHERE s.id = $1";

    let shipment_row = match client.query_opt(shipment_query, &[&shipment_uuid]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(json!({ "error": "Shipment not found" })),
        Err(err) => return Err(Box::new(err)),
    };

    let created_at: chrono::DateTime<chrono::Utc> = shipment_row.get("created_at");
    let finished_at: Option<chrono::DateTime<chrono::Utc>> = shipment_row.get("finished_at");

    // Then, get the devices
    let devices_query = "
        SELECT 
            sd.id::text,
            d.id::text as device_id,
            d.full_id as device_full_id,
            dk.name as device_kind_name,
            sd.prev_status::text,
            sd.after_status::text
        FROM 
            bench_shipments_devices sd
            JOIN bench_devices d ON sd.device_id = d.id
            JOIN bench_device_kinds dk ON d.kind = dk.id
        WHERE sd.shipment_id = $1";

    let device_rows = client.query(devices_query, &[&shipment_uuid]).await?;

    let devices = device_rows
        .iter()
        .map(|row| {
            json!({
                "id": row.get::<_, String>(0),
                "deviceId": row.get::<_, String>(1),
                "deviceFullId": row.get::<_, String>(2),
                "deviceKindName": row.get::<_, String>(3),
                "prevStatus": row.get::<_, String>(4),
                "afterStatus": row.get::<_, String>(5)
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "id": shipment_row.get::<_, String>(0),
        "status": shipment_row.get::<_, String>(1),
        "shipperName": shipment_row.get::<_, String>(2),
        "shipperId": shipment_row.get::<_, String>(3),
        "fromLabName": shipment_row.get::<_, String>(4),
        "fromLabId": shipment_row.get::<_, String>(5),
        "toLabName": shipment_row.get::<_, String>(6),
        "toLabId": shipment_row.get::<_, String>(7),
        "createdAt": created_at.to_rfc3339(),
        "finishedAt": finished_at.map(|dt| dt.to_rfc3339()),
        "note": shipment_row.get::<_, Option<String>>(10),
        "activityId": shipment_row.get::<_, String>(11),
        "devices": devices
    }))
}

async fn create_shipment(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let from_lab_id = params.value.get("fromLabId").and_then(|v| v.as_str()).unwrap_or("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
    let to_lab_id = params.value.get("toLabId").and_then(|v| v.as_str()).unwrap_or("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb");
    let shipper_id = params.value.get("shipperId").and_then(|v| v.as_str()).unwrap_or("11111111-1111-1111-1111-111111111111");
    let note = params.value.get("note").and_then(|v| v.as_str());
    
    // Create stable vector to avoid temporary value issue
    let devices_vec = params.value.get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;
    
    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for shipment" }));
    }

    // Validate that from_lab_id and to_lab_id are different
    if from_lab_id == to_lab_id {
        return Ok(json!({ "error": "Source and destination labs must be different" }));
    }

    // Create shipment activity
    let note_part = match note {
        Some(n) => format!("'{}'", n),
        None => "NULL".to_string()
    };
    
    let query = format!(
        "WITH new_activity AS (
            INSERT INTO bench_activities (id, type, note) 
            VALUES (gen_random_uuid(), 'shipment'::bench_activity_type, {})
            RETURNING id
        ),
        new_shipment AS (
            INSERT INTO bench_shipments (id, from_lab_id, to_lab_id, shipper_id, status) 
            SELECT 
                id, 
                '{}'::uuid, 
                '{}'::uuid,
                '{}'::uuid,
                'preparing'
            FROM new_activity
            RETURNING id
        )
        SELECT id::text FROM new_shipment",
        note_part, from_lab_id, to_lab_id, shipper_id
    );

    let row = transaction.query_one(&query, &[]).await?;
    let shipment_id = row.get::<_, String>(0);

    // Insert devices
    for device in devices {
        let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if device_id.is_empty() {
            continue;
        }

        let prev_status = device.get("prevStatus").and_then(|v| v.as_str()).unwrap_or("healthy");
        let after_status = device.get("afterStatus").and_then(|v| v.as_str()).unwrap_or("healthy");

        let device_query = format!(
            "INSERT INTO bench_shipments_devices 
             (shipment_id, device_id, prev_status, after_status) 
             VALUES 
             ('{}', '{}', '{}'::bench_device_status, '{}'::bench_device_status)",
            shipment_id, device_id, prev_status, after_status
        );

        transaction.execute(&device_query, &[]).await?;

        // Update device status
        let update_query = format!(
            "UPDATE bench_devices 
             SET status = 'shipping'::bench_device_status 
             WHERE id = '{}'::uuid",
            device_id
        );

        transaction.execute(&update_query, &[]).await?;
    }

    transaction.commit().await?;

    Ok(json!({
        "success": true,
        "id": shipment_id
    }))
}

async fn complete_shipment(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let shipment_id = params.value.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if shipment_id.is_empty() {
        return Ok(json!({ "error": "Shipment ID is required" }));
    }

    // Get the shipment details
    let shipment_query = format!(
        "SELECT to_lab_id::text FROM bench_shipments WHERE id = '{}'::uuid",
        shipment_id
    );
    
    let shipment_row = match transaction.query_opt(&shipment_query, &[]).await? {
        Some(row) => row,
        None => return Ok(json!({ "error": "Shipment not found" })),
    };
    
    let to_lab_id = shipment_row.get::<_, String>(0);

    // Create stable vector to avoid temporary value issue
    let devices_vec = params.value.get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;
    
    if devices.is_empty() {
        // If no devices specified, get all devices in the shipment
        let get_devices_query = format!(
            "SELECT device_id::text FROM bench_shipments_devices WHERE shipment_id = '{}'::uuid",
            shipment_id
        );
        
        let device_rows = transaction.query(&get_devices_query, &[]).await?;
        
        for row in device_rows {
            let device_id = row.get::<_, String>(0);
            
            // Update device lab and status
            let update_device_query = format!(
                "UPDATE bench_devices 
                 SET status = 'healthy'::bench_device_status, lab_id = '{}'::uuid
                 WHERE id = '{}'::uuid",
                to_lab_id, device_id
            );
            
            transaction.execute(&update_device_query, &[]).await?;
        }
    } else {
        // Update specified devices
        for device in devices {
            let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if device_id.is_empty() {
                continue;
            }

            let after_status = device.get("afterStatus").and_then(|v| v.as_str()).unwrap_or("healthy");

            // Update shipment device
            let update_shipment_device_query = format!(
                "UPDATE bench_shipments_devices 
                SET after_status = '{}'::bench_device_status 
                WHERE device_id = '{}' 
                    AND shipment_id = '{}'",
                after_status, device_id, shipment_id
            );
            transaction.execute(&update_shipment_device_query, &[]).await?;

            // Update device lab and status
            let update_device_query = format!(
                "UPDATE bench_devices 
                SET status = '{}'::bench_device_status, lab_id = '{}'::uuid
                WHERE id = '{}'",
                after_status, to_lab_id, device_id
            );
            transaction.execute(&update_device_query, &[]).await?;
        }
    }

    // Update shipment status
    let update_shipment_query = format!(
        "UPDATE bench_shipments 
         SET status = 'completed', finished_at = CURRENT_TIMESTAMP 
         WHERE id = '{}'",
        shipment_id
    );
    transaction.execute(&update_shipment_query, &[]).await?;

    transaction.commit().await?;

    Ok(json!({
        "success": true,
        "id": shipment_id
    }))
}

fn benchmark_shipment(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for shipment benchmarks");
    let app_state = rt.block_on(setup_bench_env());
    
    // Get real device IDs for testing
    let real_device_ids = rt.block_on(async {
        let client = app_state.db.get_client().await.expect("Failed to get client");
        let query = "SELECT id::text FROM bench_devices LIMIT 3";
        let rows = client.query(query, &[]).await.expect("Failed to fetch device IDs");
        
        if rows.len() < 3 {
            vec![
                "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_string(),
                "aaaaaaaa-bbbb-cccc-dddd-ffffffffffff".to_string(),
                "aaaaaaaa-bbbb-cccc-dddd-111111111111".to_string()
            ]
        } else {
            rows.iter().map(|row| row.get::<_, String>(0)).collect::<Vec<_>>()
        }
    });
    
    // Create a test shipment to use for the benchmark
    let shipment_id = rt.block_on(async {
        let create_params = InsertParams {
            table: "bench_shipments".to_string(),
            value: json!({
                "fromLabId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "toLabId": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                "shipperId": "11111111-1111-1111-1111-111111111111",
                "note": "Test shipment for benchmarking",
                "devices": [
                    { 
                        "id": real_device_ids.get(0).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_string()), 
                        "prevStatus": "healthy",
                        "afterStatus": "healthy"
                    },
                    { 
                        "id": real_device_ids.get(1).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-ffffffffffff".to_string()),
                        "prevStatus": "healthy",
                        "afterStatus": "healthy"
                    }
                ]
            }),
        };
        
        match create_shipment(&app_state, &create_params).await {
            Ok(result) => result.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            Err(_) => "".to_string()
        }
    });
    
    let mut group = c.benchmark_group("Shipment Operations");
    
    // Benchmark 1: Fetch shipments
    let fetch_params = QueryParams {
        table: "bench_shipments".to_string(),
        columns: None,
        conditions: None,
        order_by: Some(vec![("a.created_at".to_string(), false)]),
        limit: Some(10),
        offset: Some(0),
        joins: None,
    };
    
    group.bench_with_input(
        BenchmarkId::new("Fetch Shipments", 10),
        &fetch_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                match fetch_shipments(&app_state, p).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Shipments benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        },
    );
    
    // Benchmark 2: Fetch shipment details (if test shipment was created successfully)
    if !shipment_id.is_empty() {
        let details_params = QueryParams {
            table: "bench_shipments".to_string(),
            columns: None,
            conditions: Some(vec![("id".to_string(), json!(shipment_id))]),
            order_by: None,
            limit: None,
            offset: None,
            joins: None,
        };
        
        group.bench_with_input(
            BenchmarkId::new("Fetch Shipment Details", 1),
            &details_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match fetch_shipment_details(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Fetch Shipment Details benchmark: {}", err);
                            black_box(json!({"error": err.to_string()}));
                        }
                    }
                });
            },
        );
    }
    
    // Benchmark 3: Create shipment
    let create_params = InsertParams {
        table: "bench_shipments".to_string(),
        value: json!({
            "fromLabId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "toLabId": "cccccccc-cccc-cccc-cccc-cccccccccccc",
            "shipperId": "11111111-1111-1111-1111-111111111111",
            "note": "Benchmark test shipment",
            "devices": [
                { 
                    "id": real_device_ids.get(0).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_string()), 
                    "prevStatus": "healthy",
                    "afterStatus": "healthy"
                },
                { 
                    "id": real_device_ids.get(1).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-ffffffffffff".to_string()),
                    "prevStatus": "healthy",
                    "afterStatus": "healthy"
                },
                { 
                    "id": real_device_ids.get(2).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-111111111111".to_string()),
                    "prevStatus": "healthy",
                    "afterStatus": "healthy"
                }
            ]
        }),
    };
    
    group.bench_with_input(
        BenchmarkId::new("Create Shipment", 3),
        &create_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                match create_shipment(&app_state, p).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Create Shipment benchmark: {}", err);
                        black_box(json!({"error": err.to_string()}));
                    }
                }
            });
        },
    );
    
    // Benchmark 4: Complete shipment (if test shipment was created successfully)
    if !shipment_id.is_empty() {
        let complete_params = InsertParams {
            table: "bench_shipments".to_string(),
            value: json!({
                "id": shipment_id,
                "devices": [
                    { 
                        "id": real_device_ids.get(0).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_string()), 
                        "afterStatus": "healthy"
                    },
                    { 
                        "id": real_device_ids.get(1).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-ffffffffffff".to_string()),
                        "afterStatus": "healthy"
                    }
                ]
            }),
        };
        
        group.bench_with_input(
            BenchmarkId::new("Complete Shipment", 2),
            &complete_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match complete_shipment(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Complete Shipment benchmark: {}", err);
                            black_box(json!({"error": err.to_string()}));
                        }
                    }
                });
            },
        );
    }
    
    group.finish();
    
    // Clean up test tables after benchmarks
    rt.block_on(cleanup_test_tables(&app_state.db))
        .expect("Failed to clean up test tables");
}

criterion_group!(benches, benchmark_shipment);
criterion_main!(benches); 