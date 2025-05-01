use criterion::{black_box, BenchmarkId, criterion_group, criterion_main, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use hmi_lib::commands::db_commands::{InsertParams, QueryParams};

mod common;
use common::{setup_bench_env, cleanup_test_tables, AppState};

async fn fetch_maintenance_records(
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
            m.id::text, 
            m.status,
            u.name as technician_name,
            u.id::text as technician_id,
            l.name as lab_name,
            l.id::text as lab_id,
            a.created_at as created_at,
            m.finished_at,
            COUNT(md.id) as device_count,
            a.id::text as activity_id,
            a.note
        FROM 
            bench_maintenance m
            JOIN bench_activities a ON m.id = a.id
            JOIN bench_users u ON m.technician_id = u.id
            JOIN bench_labs l ON m.lab_id = l.id
            LEFT JOIN bench_maintenance_devices md ON m.id = md.maintenance_id
        GROUP BY
            m.id, m.status, u.name, u.id, l.name, l.id, a.created_at, m.finished_at, a.id, a.note
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
            "technicianName": row.get::<_, String>(2),
            "technicianId": row.get::<_, String>(3),
            "labName": row.get::<_, String>(4),
            "labId": row.get::<_, String>(5),
            "createdAt": created_at.to_rfc3339(),
            "finishedAt": finished_at.map(|dt| dt.to_rfc3339()),
            "deviceCount": row.get::<_, i64>(8),
            "activityId": row.get::<_, String>(9),
            "note": row.get::<_, Option<String>>(10)
        }));
        
        _total_count += 1;
    }

    Ok(results)
}

async fn fetch_maintenance_details(
    app_state: &AppState,
    params: &QueryParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let maintenance_id = params
        .conditions
        .as_ref()
        .and_then(|conds| conds.first())
        .and_then(|(_, val)| val.as_str())
        .unwrap_or("");

    if maintenance_id.is_empty() {
        return Ok(json!({ "error": "Maintenance ID is required" }));
    }

    let maintenance_uuid = match Uuid::parse_str(maintenance_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid maintenance ID format" })),
    };

    // First, get the maintenance details
    let maintenance_query = "
        SELECT 
            m.id::text, 
            m.status,
            u.name as technician_name,
            u.id::text as technician_id,
            l.name as lab_name,
            l.id::text as lab_id,
            a.created_at as created_at,
            m.finished_at,
            a.note,
            a.id::text as activity_id
        FROM 
            bench_maintenance m
            JOIN bench_activities a ON m.id = a.id
            JOIN bench_users u ON m.technician_id = u.id
            JOIN bench_labs l ON m.lab_id = l.id
        WHERE m.id = $1";

    let maintenance_row = match client.query_opt(maintenance_query, &[&maintenance_uuid]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(json!({ "error": "Maintenance record not found" })),
        Err(err) => return Err(Box::new(err)),
    };

    let created_at: chrono::DateTime<chrono::Utc> = maintenance_row.get("created_at");
    let finished_at: Option<chrono::DateTime<chrono::Utc>> = maintenance_row.get("finished_at");

    // Then, get the devices
    let devices_query = "
        SELECT 
            md.id::text,
            d.id::text as device_id,
            d.full_id as device_full_id,
            dk.name as device_kind_name,
            md.prev_status::text,
            md.after_status::text
        FROM 
            bench_maintenance_devices md
            JOIN bench_devices d ON md.device_id = d.id
            JOIN bench_device_kinds dk ON d.kind = dk.id
        WHERE md.maintenance_id = $1";

    let device_rows = client.query(devices_query, &[&maintenance_uuid]).await?;

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
        "id": maintenance_row.get::<_, String>(0),
        "status": maintenance_row.get::<_, String>(1),
        "technicianName": maintenance_row.get::<_, String>(2),
        "technicianId": maintenance_row.get::<_, String>(3),
        "labName": maintenance_row.get::<_, String>(4),
        "labId": maintenance_row.get::<_, String>(5),
        "createdAt": created_at.to_rfc3339(),
        "finishedAt": finished_at.map(|dt| dt.to_rfc3339()),
        "note": maintenance_row.get::<_, Option<String>>(8),
        "activityId": maintenance_row.get::<_, String>(9),
        "devices": devices
    }))
}

async fn create_maintenance(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let lab_id = params.value.get("labId").and_then(|v| v.as_str()).unwrap_or("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
    let technician_id = params.value.get("technicianId").and_then(|v| v.as_str()).unwrap_or("11111111-1111-1111-1111-111111111111");
    let note = params.value.get("note").and_then(|v| v.as_str());
    
    // Create stable vector to avoid temporary value issue
    let devices_vec = params.value.get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;
    
    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for maintenance" }));
    }

    // Create maintenance activity
    let note_part = match note {
        Some(n) => format!("'{}'", n),
        None => "NULL".to_string()
    };
    
    let query = format!(
        "WITH new_activity AS (
            INSERT INTO bench_activities (id, type, note) 
            VALUES (gen_random_uuid(), 'maintenance'::bench_activity_type, {})
            RETURNING id
        ),
        new_maintenance AS (
            INSERT INTO bench_maintenance (id, lab_id, technician_id, status) 
            SELECT 
                id, 
                '{}'::uuid, 
                '{}'::uuid, 
                'in_progress'
            FROM new_activity
            RETURNING id
        )
        SELECT id::text FROM new_maintenance",
        note_part, lab_id, technician_id
    );

    let row = transaction.query_one(&query, &[]).await?;
    let maintenance_id = row.get::<_, String>(0);

    // Insert devices
    for device in devices {
        let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if device_id.is_empty() {
            continue;
        }

        let prev_status = device.get("prevStatus").and_then(|v| v.as_str()).unwrap_or("healthy");
        let after_status = device.get("afterStatus").and_then(|v| v.as_str()).unwrap_or("healthy");

        let device_query = format!(
            "INSERT INTO bench_maintenance_devices 
             (maintenance_id, device_id, prev_status, after_status) 
             VALUES 
             ('{}', '{}', '{}'::bench_device_status, '{}'::bench_device_status)",
            maintenance_id, device_id, prev_status, after_status
        );

        transaction.execute(&device_query, &[]).await?;

        // Update device status
        let update_query = format!(
            "UPDATE bench_devices 
             SET status = 'maintaining'::bench_device_status 
             WHERE id = '{}'::uuid",
            device_id
        );

        transaction.execute(&update_query, &[]).await?;
    }

    transaction.commit().await?;

    Ok(json!({
        "success": true,
        "id": maintenance_id
    }))
}

async fn finish_maintenance(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let maintenance_id = params.value.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if maintenance_id.is_empty() {
        return Ok(json!({ "error": "Maintenance ID is required" }));
    }

    // Create stable vector to avoid temporary value issue
    let devices_vec = params.value.get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;
    
    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for maintenance completion" }));
    }

    // Update maintenance status
    let update_query = format!(
        "UPDATE bench_maintenance 
         SET status = 'completed', finished_at = CURRENT_TIMESTAMP 
         WHERE id = '{}'",
        maintenance_id
    );
    transaction.execute(&update_query, &[]).await?;

    // Update devices
    for device in devices {
        let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if device_id.is_empty() {
            continue;
        }

        let after_status = device.get("afterStatus").and_then(|v| v.as_str()).unwrap_or("healthy");

        // Update maintenance device
        let update_maintenance_device_query = format!(
            "UPDATE bench_maintenance_devices 
             SET after_status = '{}'::bench_device_status 
             WHERE device_id = '{}' 
                AND maintenance_id = '{}'",
            after_status, device_id, maintenance_id
        );
        transaction.execute(&update_maintenance_device_query, &[]).await?;

        // Update device status
        let update_device_query = format!(
            "UPDATE bench_devices 
             SET status = '{}'::bench_device_status 
             WHERE id = '{}'",
            after_status, device_id
        );
        transaction.execute(&update_device_query, &[]).await?;
    }

    transaction.commit().await?;

    Ok(json!({
        "success": true,
        "id": maintenance_id
    }))
}

fn benchmark_maintenance(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for maintenance benchmarks");
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
    
    // Create a test maintenance record to use for the benchmark
    let maintenance_id = rt.block_on(async {
        let create_params = InsertParams {
            table: "bench_maintenance".to_string(),
            value: json!({
                "labId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "technicianId": "11111111-1111-1111-1111-111111111111",
                "note": "Test maintenance for benchmarking",
                "devices": [
                    { 
                        "id": real_device_ids.get(0).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_string()), 
                        "prevStatus": "broken",
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
        
        match create_maintenance(&app_state, &create_params).await {
            Ok(result) => result.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            Err(_) => "".to_string()
        }
    });
    
    let mut group = c.benchmark_group("Maintenance Operations");
    
    // Benchmark 1: Fetch maintenance records
    let fetch_params = QueryParams {
        table: "bench_maintenance".to_string(),
        columns: None,
        conditions: None,
        order_by: Some(vec![("a.created_at".to_string(), false)]),
        limit: Some(10),
        offset: Some(0),
        joins: None,
    };
    
    group.bench_with_input(
        BenchmarkId::new("Fetch Maintenance Records", 10),
        &fetch_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                match fetch_maintenance_records(&app_state, p).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Maintenance Records benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        },
    );
    
    // Benchmark 2: Fetch maintenance details (if test maintenance was created successfully)
    if !maintenance_id.is_empty() {
        let details_params = QueryParams {
            table: "bench_maintenance".to_string(),
            columns: None,
            conditions: Some(vec![("id".to_string(), json!(maintenance_id))]),
            order_by: None,
            limit: None,
            offset: None,
            joins: None,
        };
        
        group.bench_with_input(
            BenchmarkId::new("Fetch Maintenance Details", 1),
            &details_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match fetch_maintenance_details(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Fetch Maintenance Details benchmark: {}", err);
                            black_box(json!({"error": err.to_string()}));
                        }
                    }
                });
            },
        );
    }
    
    // Benchmark 3: Create maintenance record
    let create_params = InsertParams {
        table: "bench_maintenance".to_string(),
        value: json!({
            "labId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "technicianId": "11111111-1111-1111-1111-111111111111",
            "note": "Benchmark test maintenance",
            "devices": [
                { 
                    "id": real_device_ids.get(0).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".to_string()), 
                    "prevStatus": "broken",
                    "afterStatus": "healthy"
                },
                { 
                    "id": real_device_ids.get(1).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-ffffffffffff".to_string()),
                    "prevStatus": "healthy",
                    "afterStatus": "healthy"
                },
                { 
                    "id": real_device_ids.get(2).unwrap_or(&"aaaaaaaa-bbbb-cccc-dddd-111111111111".to_string()),
                    "prevStatus": "broken",
                    "afterStatus": "healthy"
                }
            ]
        }),
    };
    
    group.bench_with_input(
        BenchmarkId::new("Create Maintenance Record", 3),
        &create_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                match create_maintenance(&app_state, p).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Create Maintenance Record benchmark: {}", err);
                        black_box(json!({"error": err.to_string()}));
                    }
                }
            });
        },
    );
    
    // Benchmark 4: Finish maintenance (if test maintenance was created successfully)
    if !maintenance_id.is_empty() {
        let finish_params = InsertParams {
            table: "bench_maintenance".to_string(),
            value: json!({
                "id": maintenance_id,
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
            BenchmarkId::new("Finish Maintenance", 2),
            &finish_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match finish_maintenance(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Finish Maintenance benchmark: {}", err);
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

criterion_group!(benches, benchmark_maintenance);
criterion_main!(benches); 