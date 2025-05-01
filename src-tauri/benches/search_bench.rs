use criterion::{black_box, BenchmarkId, criterion_group, criterion_main, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;

use hmi_lib::commands::db_commands::QueryParams;

mod common;
use common::{setup_bench_env, cleanup_test_tables, AppState};

async fn search_devices(
    app_state: &AppState,
    query: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let search_term = format!("%{}%", query.to_lowercase());
    
    let sql = "
        SELECT 
            d.id::text, 
            d.full_id,
            dk.name as kind_name,
            dk.id::text as kind_id,
            l.name as lab_name,
            l.id::text as lab_id,
            d.status::text,
            'device'::text as result_type
        FROM 
            bench_devices d
            JOIN bench_device_kinds dk ON d.kind = dk.id
            JOIN bench_labs l ON d.lab_id = l.id
        WHERE 
            (LOWER(d.full_id) LIKE $1 OR
             LOWER(dk.name) LIKE $1 OR
             LOWER(l.name) LIKE $1)
            AND d.deleted_at IS NULL
        ORDER BY 
            CASE 
                WHEN LOWER(d.full_id) LIKE $1 THEN 0
                WHEN LOWER(dk.name) LIKE $1 THEN 1
                ELSE 2
            END
        LIMIT 20";

    let rows = client.query(sql, &[&search_term]).await?;

    let results = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.get::<_, String>(0),
                "fullId": row.get::<_, String>(1),
                "kindName": row.get::<_, String>(2),
                "kindId": row.get::<_, String>(3),
                "labName": row.get::<_, String>(4),
                "labId": row.get::<_, String>(5),
                "status": row.get::<_, String>(6),
                "resultType": row.get::<_, String>(7)
            })
        })
        .collect();

    Ok(results)
}

async fn search_users(
    app_state: &AppState,
    query: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let search_term = format!("%{}%", query.to_lowercase());
    
    let sql = "
        SELECT 
            u.id::text,
            u.name,
            u.email,
            u.image,
            'user'::text as result_type
        FROM 
            bench_users u
        WHERE 
            (LOWER(u.name) LIKE $1 OR
             LOWER(u.email) LIKE $1)
            AND u.deleted_at IS NULL
        ORDER BY 
            CASE 
                WHEN LOWER(u.name) LIKE $1 THEN 0
                ELSE 1
            END
        LIMIT 20";

    let rows = client.query(sql, &[&search_term]).await?;

    let results = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.get::<_, String>(0),
                "name": row.get::<_, String>(1),
                "email": row.get::<_, String>(2),
                "image": row.get::<_, serde_json::Value>(3),
                "resultType": row.get::<_, String>(4)
            })
        })
        .collect();

    Ok(results)
}

async fn search_labs(
    app_state: &AppState,
    query: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let search_term = format!("%{}%", query.to_lowercase());
    
    let sql = "
        SELECT 
            l.id::text,
            l.name,
            l.room,
            l.branch,
            'lab'::text as result_type
        FROM 
            bench_labs l
        WHERE 
            (LOWER(l.name) LIKE $1 OR
             LOWER(l.room) LIKE $1 OR
             LOWER(l.branch) LIKE $1)
            AND l.deleted_at IS NULL
        ORDER BY 
            CASE 
                WHEN LOWER(l.name) LIKE $1 THEN 0
                WHEN LOWER(l.room) LIKE $1 THEN 1
                ELSE 2
            END
        LIMIT 20";

    let rows = client.query(sql, &[&search_term]).await?;

    let results = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.get::<_, String>(0),
                "name": row.get::<_, String>(1),
                "room": row.get::<_, String>(2),
                "branch": row.get::<_, String>(3),
                "resultType": row.get::<_, String>(4)
            })
        })
        .collect();

    Ok(results)
}

async fn search_all(
    app_state: &AppState,
    query: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    
    // Run all searches in parallel
    let (devices, users, labs) = tokio::join!(
        search_devices(app_state, query),
        search_users(app_state, query),
        search_labs(app_state, query)
    );
    
    // Combine results
    if let Ok(device_results) = devices {
        results.extend(device_results);
    }
    
    if let Ok(user_results) = users {
        results.extend(user_results);
    }
    
    if let Ok(lab_results) = labs {
        results.extend(lab_results);
    }
    
    // Sort results by result type for consistency
    results.sort_by(|a, b| {
        let a_type = a.get("resultType")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let b_type = b.get("resultType")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        a_type.cmp(&b_type)
    });
    
    // Limit total results
    if results.len() > 30 {
        results.truncate(30);
    }
    
    Ok(results)
}

async fn search_device_kind(
    app_state: &AppState,
    params: &QueryParams,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let query = params
        .conditions
        .as_ref()
        .and_then(|conds| conds.first())
        .and_then(|(_, val)| val.as_str())
        .unwrap_or("");

    let search_term = format!("%{}%", query.to_lowercase());
    
    let sql = "
        SELECT 
            dk.id::text,
            dk.name,
            dk.brand,
            dk.manufacturer,
            dk.description,
            dk.image,
            c.name as category_name,
            c.id::text as category_id
        FROM 
            bench_device_kinds dk
            LEFT JOIN bench_categories c ON dk.category_id = c.id
        WHERE 
            (LOWER(dk.name) LIKE $1 OR
             LOWER(dk.brand) LIKE $1 OR
             LOWER(dk.manufacturer) LIKE $1 OR
             LOWER(dk.description) LIKE $1 OR
             LOWER(c.name) LIKE $1)
            AND dk.deleted_at IS NULL
        ORDER BY 
            CASE 
                WHEN LOWER(dk.name) LIKE $1 THEN 0
                WHEN LOWER(dk.brand) LIKE $1 THEN 1
                WHEN LOWER(dk.manufacturer) LIKE $1 THEN 2
                ELSE 3
            END
        LIMIT 20";

    let rows = client.query(sql, &[&search_term]).await?;

    let results = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.get::<_, String>(0),
                "name": row.get::<_, String>(1),
                "brand": row.get::<_, String>(2),
                "manufacturer": row.get::<_, String>(3),
                "description": row.get::<_, String>(4),
                "image": row.get::<_, serde_json::Value>(5),
                "categoryName": row.get::<_, String>(6),
                "categoryId": row.get::<_, String>(7)
            })
        })
        .collect();

    Ok(results)
}

fn benchmark_search(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for search benchmarks");
    let app_state = rt.block_on(setup_bench_env());
    
    let mut group = c.benchmark_group("Search Operations");
    
    // Benchmark 1: Search devices
    group.bench_function("Search Devices", |b| {
        b.to_async(&rt).iter(|| async {
            match search_devices(&app_state, "lap").await {
                Ok(result) => {
                    black_box(result);
                }
                Err(err) => {
                    eprintln!("Error in Search Devices benchmark: {}", err);
                    black_box(Vec::<serde_json::Value>::new());
                }
            }
        });
    });
    
    // Benchmark 2: Search users
    group.bench_function("Search Users", |b| {
        b.to_async(&rt).iter(|| async {
            match search_users(&app_state, "test").await {
                Ok(result) => {
                    black_box(result);
                }
                Err(err) => {
                    eprintln!("Error in Search Users benchmark: {}", err);
                    black_box(Vec::<serde_json::Value>::new());
                }
            }
        });
    });
    
    // Benchmark 3: Search labs
    group.bench_function("Search Labs", |b| {
        b.to_async(&rt).iter(|| async {
            match search_labs(&app_state, "lab").await {
                Ok(result) => {
                    black_box(result);
                }
                Err(err) => {
                    eprintln!("Error in Search Labs benchmark: {}", err);
                    black_box(Vec::<serde_json::Value>::new());
                }
            }
        });
    });
    
    // Benchmark 4: Search all
    group.bench_function("Search All", |b| {
        b.to_async(&rt).iter(|| async {
            match search_all(&app_state, "a").await {
                Ok(result) => {
                    black_box(result);
                }
                Err(err) => {
                    eprintln!("Error in Search All benchmark: {}", err);
                    black_box(Vec::<serde_json::Value>::new());
                }
            }
        });
    });
    
    // Benchmark 5: Search device kinds
    let device_kind_params = QueryParams {
        table: "bench_device_kinds".to_string(),
        columns: None,
        conditions: Some(vec![("search".to_string(), json!("laptop"))]),
        order_by: None,
        limit: Some(20),
        offset: Some(0),
        joins: None,
    };
    
    group.bench_with_input(
        BenchmarkId::new("Search Device Kinds", 20),
        &device_kind_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                match search_device_kind(&app_state, p).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Search Device Kinds benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        },
    );
    
    group.finish();
    
    // Clean up test tables after benchmarks
    rt.block_on(cleanup_test_tables(&app_state.db))
        .expect("Failed to clean up test tables");
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches); 