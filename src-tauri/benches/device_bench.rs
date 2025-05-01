use criterion::{black_box, BenchmarkId, criterion_group, criterion_main, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use hmi_lib::commands::db_commands::{InsertParams, QueryParams};

mod common;
use common::{setup_bench_env, cleanup_test_tables, AppState};

async fn fetch_devices(
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
        _ => "ORDER BY bench_devices.full_id ASC".to_string(),
    };

    let sql = format!(
        "SELECT 
            d.id::text, 
            d.full_id,
            dk.name as kind_name,
            dk.id::text as kind_id,
            l.name as lab_name,
            l.id::text as lab_id,
            d.status::text,
            COUNT(*) OVER() as total_count
        FROM 
            bench_devices d
            JOIN bench_device_kinds dk ON d.kind = dk.id
            JOIN bench_labs l ON d.lab_id = l.id
        WHERE 
            d.deleted_at IS NULL
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
            "fullId": row.get::<_, String>(1),
            "kindName": row.get::<_, String>(2),
            "kindId": row.get::<_, String>(3),
            "labName": row.get::<_, String>(4),
            "labId": row.get::<_, String>(5),
            "status": row.get::<_, String>(6)
        }));
    }

    Ok(results)
}

async fn fetch_device_details(
    app_state: &AppState,
    params: &QueryParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let device_id = params
        .conditions
        .as_ref()
        .and_then(|conds| conds.first())
        .and_then(|(_, val)| val.as_str())
        .unwrap_or("");

    if device_id.is_empty() {
        return Ok(json!({ "error": "Device ID is required" }));
    }

    let device_uuid = match Uuid::parse_str(device_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid device ID format" })),
    };

    let sql = "
        SELECT 
            d.id::text, 
            d.full_id,
            dk.name as kind_name,
            dk.id::text as kind_id,
            dk.brand,
            dk.manufacturer,
            dk.description,
            l.name as lab_name,
            l.id::text as lab_id,
            d.status::text,
            c.name as category_name,
            c.id::text as category_id
        FROM 
            bench_devices d
            JOIN bench_device_kinds dk ON d.kind = dk.id
            JOIN bench_labs l ON d.lab_id = l.id
            LEFT JOIN bench_categories c ON dk.category_id = c.id
        WHERE 
            d.id = $1 AND
            d.deleted_at IS NULL";

    let row = match client.query_opt(sql, &[&device_uuid]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(json!({ "error": "Device not found" })),
        Err(err) => return Err(Box::new(err)),
    };

    Ok(json!({
        "id": row.get::<_, String>(0),
        "fullId": row.get::<_, String>(1),
        "kindName": row.get::<_, String>(2),
        "kindId": row.get::<_, String>(3),
        "brand": row.get::<_, String>(4),
        "manufacturer": row.get::<_, String>(5),
        "description": row.get::<_, String>(6),
        "labName": row.get::<_, String>(7),
        "labId": row.get::<_, String>(8),
        "status": row.get::<_, String>(9),
        "categoryName": row.get::<_, String>(10),
        "categoryId": row.get::<_, String>(11)
    }))
}

async fn create_device(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let full_id = params.value.get("fullId").and_then(|v| v.as_str()).unwrap_or("");
    let kind_id = params.value.get("kindId").and_then(|v| v.as_str()).unwrap_or("");
    let lab_id = params.value.get("labId").and_then(|v| v.as_str()).unwrap_or("");
    let status = params.value.get("status").and_then(|v| v.as_str()).unwrap_or("healthy");

    if full_id.is_empty() || kind_id.is_empty() || lab_id.is_empty() {
        return Ok(json!({ "error": "Missing required fields (fullId, kindId, labId)" }));
    }

    // Check if full_id already exists
    let check_query = "SELECT COUNT(*) FROM bench_devices WHERE full_id = $1 AND deleted_at IS NULL";
    let count: i64 = transaction.query_one(check_query, &[&full_id]).await?.get(0);
    
    if count > 0 {
        return Ok(json!({ "error": "A device with this ID already exists" }));
    }

    // Insert the device
    let query = format!(
        "INSERT INTO bench_devices (id, full_id, kind, lab_id, status)
        VALUES (gen_random_uuid(), '{}', '{}'::uuid, '{}'::uuid, '{}'::bench_device_status)
        RETURNING id::text",
        full_id, kind_id, lab_id, status
    );

    let row = transaction.query_one(&query, &[]).await?;
    let device_id = row.get::<_, String>(0);

    transaction.commit().await?;

    Ok(json!({
        "success": true,
        "id": device_id
    }))
}

async fn update_device(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let device_id = params.value.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if device_id.is_empty() {
        return Ok(json!({ "error": "Device ID is required" }));
    }

    let full_id = params.value.get("fullId").and_then(|v| v.as_str());
    let kind_id = params.value.get("kindId").and_then(|v| v.as_str());
    let lab_id = params.value.get("labId").and_then(|v| v.as_str());
    let status = params.value.get("status").and_then(|v| v.as_str());

    // Build the SET part of the query
    let mut updates = Vec::new();
    
    if let Some(full_id_val) = full_id {
        updates.push(format!("full_id = '{}'", full_id_val));
    }
    
    if let Some(kind_val) = kind_id {
        updates.push(format!("kind = '{}'::uuid", kind_val));
    }
    
    if let Some(lab_val) = lab_id {
        updates.push(format!("lab_id = '{}'::uuid", lab_val));
    }
    
    if let Some(status_val) = status {
        updates.push(format!("status = '{}'::bench_device_status", status_val));
    }

    if updates.is_empty() {
        return Ok(json!({ "error": "No fields provided for update" }));
    }

    let update_clause = updates.join(", ");
    let query = format!(
        "UPDATE bench_devices
        SET {}
        WHERE id = '{}'::uuid
        RETURNING id::text",
        update_clause, device_id
    );

    let result = transaction.query_opt(&query, &[]).await?;
    
    match result {
        Some(row) => {
            let updated_id = row.get::<_, String>(0);
            transaction.commit().await?;
            
            Ok(json!({
                "success": true,
                "id": updated_id
            }))
        },
        None => {
            transaction.rollback().await?;
            
            Ok(json!({
                "success": false,
                "error": "Device not found"
            }))
        }
    }
}

async fn delete_device(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let device_id = params.value.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if device_id.is_empty() {
        return Ok(json!({ "error": "Device ID is required" }));
    }

    // Soft delete by setting deleted_at
    let query = format!(
        "UPDATE bench_devices
        SET deleted_at = CURRENT_TIMESTAMP
        WHERE id = '{}'::uuid
        RETURNING id::text",
        device_id
    );

    let result = transaction.query_opt(&query, &[]).await?;
    
    match result {
        Some(row) => {
            let deleted_id = row.get::<_, String>(0);
            transaction.commit().await?;
            
            Ok(json!({
                "success": true,
                "id": deleted_id
            }))
        },
        None => {
            transaction.rollback().await?;
            
            Ok(json!({
                "success": false,
                "error": "Device not found"
            }))
        }
    }
}

fn benchmark_device(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for device benchmarks");
    let app_state = rt.block_on(setup_bench_env());
    
    // Create test device for detailed operations
    let test_device_id = rt.block_on(async {
        let create_params = InsertParams {
            table: "bench_devices".to_string(),
            value: json!({
                "fullId": "DEV-BENCH-TEST",
                "kindId": "aaaaaaaa-0000-4000-a000-000000000001",
                "labId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "status": "healthy"
            }),
        };
        
        match create_device(&app_state, &create_params).await {
            Ok(result) => result.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            Err(_) => "".to_string()
        }
    });
    
    let mut group = c.benchmark_group("Device Operations");
    
    // Benchmark 1: Fetch devices
    let fetch_params = QueryParams {
        table: "bench_devices".to_string(),
        columns: None,
        conditions: None,
        order_by: Some(vec![("bench_devices.full_id".to_string(), true)]),
        limit: Some(10),
        offset: Some(0),
        joins: None,
    };
    
    group.bench_with_input(
        BenchmarkId::new("Fetch Devices", 10),
        &fetch_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                match fetch_devices(&app_state, p).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Devices benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        },
    );
    
    // Benchmark 2: Fetch device details (if test device was created successfully)
    if !test_device_id.is_empty() {
        let details_params = QueryParams {
            table: "bench_devices".to_string(),
            columns: None,
            conditions: Some(vec![("id".to_string(), json!(test_device_id))]),
            order_by: None,
            limit: None,
            offset: None,
            joins: None,
        };
        
        group.bench_with_input(
            BenchmarkId::new("Fetch Device Details", 1),
            &details_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match fetch_device_details(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Fetch Device Details benchmark: {}", err);
                            black_box(json!({"error": err.to_string()}));
                        }
                    }
                });
            },
        );
    }
    
    // Benchmark 3: Create device
    let create_params = InsertParams {
        table: "bench_devices".to_string(),
        value: json!({
            "fullId": format!("DEV-BENCH-{}", Uuid::new_v4().to_string().split('-').next().unwrap()),
            "kindId": "aaaaaaaa-0000-4000-a000-000000000001",
            "labId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "status": "healthy"
        }),
    };
    
    group.bench_with_input(
        BenchmarkId::new("Create Device", 1),
        &create_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                let params = InsertParams {
                    table: p.table.clone(),
                    value: json!({
                        "fullId": format!("DEV-BENCH-{}", Uuid::new_v4().to_string().split('-').next().unwrap()),
                        "kindId": "aaaaaaaa-0000-4000-a000-000000000001",
                        "labId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "status": "healthy"
                    }),
                };
                
                match create_device(&app_state, &params).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Create Device benchmark: {}", err);
                        black_box(json!({"error": err.to_string()}));
                    }
                }
            });
        },
    );
    
    // Benchmark 4: Update device (if test device was created successfully)
    if !test_device_id.is_empty() {
        let update_params = InsertParams {
            table: "bench_devices".to_string(),
            value: json!({
                "id": test_device_id,
                "status": "assessing",
                "labId": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"
            }),
        };
        
        group.bench_with_input(
            BenchmarkId::new("Update Device", 1),
            &update_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match update_device(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Update Device benchmark: {}", err);
                            black_box(json!({"error": err.to_string()}));
                        }
                    }
                });
            },
        );
    }
    
    // Benchmark 5: Delete device (if test device was created successfully)
    if !test_device_id.is_empty() {
        let delete_params = InsertParams {
            table: "bench_devices".to_string(),
            value: json!({
                "id": test_device_id
            }),
        };
        
        group.bench_with_input(
            BenchmarkId::new("Delete Device", 1),
            &delete_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match delete_device(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Delete Device benchmark: {}", err);
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

criterion_group!(benches, benchmark_device);
criterion_main!(benches); 