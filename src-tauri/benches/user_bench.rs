use criterion::{black_box, BenchmarkId, criterion_group, criterion_main, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use hmi_lib::commands::db_commands::{InsertParams, QueryParams};

mod common;
use common::{setup_bench_env, cleanup_test_tables, AppState};

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
            format!("ORDER BY {} {}", field, if *is_asc { "ASC" } else { "DESC" })
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

async fn create_user(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let name = params.value.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let email = params.value.get("email").and_then(|v| v.as_str()).unwrap_or("");
    let image = params.value.get("image").cloned().unwrap_or_else(|| json!({"url": ""}));

    if name.is_empty() || email.is_empty() {
        return Ok(json!({ "error": "Name and email are required" }));
    }

    // Check if email already exists
    let check_query = "SELECT COUNT(*) FROM bench_users WHERE email = $1 AND deleted_at IS NULL";
    let count: i64 = transaction.query_one(check_query, &[&email]).await?.get(0);
    
    if count > 0 {
        return Ok(json!({ "error": "A user with this email already exists" }));
    }

    // Insert the user with parameterized query
    let query = "INSERT INTO bench_users (id, name, email, image)
        VALUES (gen_random_uuid(), $1, $2, $3)
        RETURNING id::text";

    let row = transaction.query_one(query, &[&name, &email, &image]).await?;
    let user_id = row.get::<_, String>(0);

    transaction.commit().await?;

    Ok(json!({
        "success": true,
        "id": user_id
    }))
}

async fn update_user(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let user_id = params.value.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if user_id.is_empty() {
        return Ok(json!({ "error": "User ID is required" }));
    }
    
    let user_uuid = match Uuid::parse_str(user_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid user ID format" })),
    };

    // Store the values to be updated
    let name_opt = params.value.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let email_opt = params.value.get("email").and_then(|v| v.as_str()).map(|s| s.to_string());
    let image_opt = params.value.get("image").cloned();

    // Build the update query based on provided fields
    if name_opt.is_none() && email_opt.is_none() && image_opt.is_none() {
        return Ok(json!({ "error": "No fields provided for update" }));
    }
    
    // Build the query, collecting parts and values
    let mut query_parts = Vec::new();
    let mut param_values = Vec::new();
    let mut param_index = 1;
    
    if let Some(ref name_val) = name_opt {
        query_parts.push(format!("name = ${}", param_index));
        param_values.push(name_val as &(dyn tokio_postgres::types::ToSql + Sync));
        param_index += 1;
    }
    
    if let Some(ref email_val) = email_opt {
        query_parts.push(format!("email = ${}", param_index));
        param_values.push(email_val as &(dyn tokio_postgres::types::ToSql + Sync));
        param_index += 1;
    }
    
    if let Some(ref image_val) = image_opt {
        query_parts.push(format!("image = ${}", param_index));
        param_values.push(image_val as &(dyn tokio_postgres::types::ToSql + Sync));
        param_index += 1;
    }
    
    let set_clause = query_parts.join(", ");
    let query = format!(
        "UPDATE bench_users
        SET {}
        WHERE id = ${}
        RETURNING id::text",
        set_clause, param_index
    );
    
    param_values.push(&user_uuid);
    
    let result = transaction.query_opt(&query, &param_values[..]).await?;
    
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
                "error": "User not found"
            }))
        }
    }
}

async fn delete_user(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut client = app_state.db.get_client().await?;
    let transaction = client.transaction().await?;

    let user_id = params.value.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if user_id.is_empty() {
        return Ok(json!({ "error": "User ID is required" }));
    }
    
    let user_uuid = match Uuid::parse_str(user_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid user ID format" })),
    };

    // Soft delete by setting deleted_at using parameterized query
    let query = 
        "UPDATE bench_users
        SET deleted_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING id::text";

    let result = transaction.query_opt(query, &[&user_uuid]).await?;
    
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
                "error": "User not found"
            }))
        }
    }
}

fn benchmark_user(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for user benchmarks");
    let app_state = rt.block_on(setup_bench_env());
    
    // Create test user for detailed operations
    let test_user_id = rt.block_on(async {
        let create_params = InsertParams {
            table: "bench_users".to_string(),
            value: json!({
                "name": "Benchmark Test User",
                "email": format!("benchmark.test.{}@example.com", Uuid::new_v4().to_string().split('-').next().unwrap()),
                "image": json!({"url": "https://example.com/avatar_test.jpg"})
            }),
        };
        
        match create_user(&app_state, &create_params).await {
            Ok(result) => result.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            Err(_) => "".to_string()
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
    
    // Benchmark 3: Create user
    let unique_email = format!("benchmark.test.{}@example.com", Uuid::new_v4().to_string().split('-').next().unwrap());
    let create_params = InsertParams {
        table: "bench_users".to_string(),
        value: json!({
            "name": "Benchmark New User",
            "email": unique_email,
            "image": json!({"url": "https://example.com/avatar_bench.jpg"})
        }),
    };
    
    group.bench_with_input(
        BenchmarkId::new("Create User", 1),
        &create_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                let unique_email = format!("benchmark.test.{}@example.com", Uuid::new_v4().to_string().split('-').next().unwrap());
                let params = InsertParams {
                    table: p.table.clone(),
                    value: json!({
                        "name": "Benchmark New User",
                        "email": unique_email,
                        "image": json!({"url": "https://example.com/avatar_bench.jpg"})
                    }),
                };
                
                match create_user(&app_state, &params).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Create User benchmark: {}", err);
                        black_box(json!({"error": err.to_string()}));
                    }
                }
            });
        },
    );
    
    // Benchmark 4: Update user (if test user was created successfully)
    if !test_user_id.is_empty() {
        let update_params = InsertParams {
            table: "bench_users".to_string(),
            value: json!({
                "id": test_user_id,
                "name": "Benchmark Updated User",
                "image": json!({"url": "https://example.com/avatar_updated.jpg"})
            }),
        };
        
        group.bench_with_input(
            BenchmarkId::new("Update User", 1),
            &update_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match update_user(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Update User benchmark: {}", err);
                            black_box(json!({"error": err.to_string()}));
                        }
                    }
                });
            },
        );
    }
    
    // Benchmark 5: Delete user (if test user was created successfully)
    if !test_user_id.is_empty() {
        let delete_params = InsertParams {
            table: "bench_users".to_string(),
            value: json!({
                "id": test_user_id
            }),
        };
        
        group.bench_with_input(
            BenchmarkId::new("Delete User", 1),
            &delete_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match delete_user(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Delete User benchmark: {}", err);
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

criterion_group!(benches, benchmark_user);
criterion_main!(benches); 