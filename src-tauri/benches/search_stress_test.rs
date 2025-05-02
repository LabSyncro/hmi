use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rng, Rng};
use serde_json::json;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::task;

mod common;
use common::{cleanup_test_tables, populate_large_test_data, setup_bench_env, AppState};

const DEFAULT_NUM_USERS: usize = 1_000;
const DEFAULT_NUM_DEVICE_KINDS: usize = 2_000;
const DEFAULT_NUM_DEVICES: usize = 50_000;
const DEFAULT_NUM_LABS: usize = 10;

const DEFAULT_CONCURRENT_REQUESTS: &[usize] = &[1, 5, 10, 50, 100];
const DEFAULT_TEST_DURATION_SECS: u64 = 10;

const FULL_CONCURRENT_REQUESTS: &[usize] = &[1, 5, 10, 50, 100, 250, 500];
const FULL_TEST_DURATION_SECS: u64 = 30;

type StdError = Box<dyn std::error::Error + Send + Sync>;

fn get_config() -> (usize, usize, usize, usize, &'static [usize], u64, bool) {
    let num_users = env::var("STRESS_NUM_USERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_NUM_USERS);

    let num_device_kinds = env::var("STRESS_NUM_DEVICE_KINDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_NUM_DEVICE_KINDS);

    let num_devices = env::var("STRESS_NUM_DEVICES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_NUM_DEVICES);

    let num_labs = env::var("STRESS_NUM_LABS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_NUM_LABS);

    let full_test = env::var("STRESS_FULL_TEST")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(false);

    let concurrent_requests = if full_test {
        FULL_CONCURRENT_REQUESTS
    } else {
        DEFAULT_CONCURRENT_REQUESTS
    };

    let test_duration = if full_test {
        FULL_TEST_DURATION_SECS
    } else {
        DEFAULT_TEST_DURATION_SECS
    };

    println!("\n=== STRESS TEST CONFIGURATION ===");
    println!("Users: {}", num_users);
    println!("Device Kinds: {}", num_device_kinds);
    println!("Devices: {}", num_devices);
    println!("Labs: {}", num_labs);
    println!("Concurrency Levels: {:?}", concurrent_requests);
    println!("Test Duration: {}s per concurrency level", test_duration);
    println!("Full Test: {}", if full_test { "Yes" } else { "No" });
    println!("================================\n");

    (
        num_users,
        num_device_kinds,
        num_devices,
        num_labs,
        concurrent_requests,
        test_duration,
        full_test,
    )
}

async fn search_devices(
    app_state: &AppState,
    query: &str,
) -> Result<Vec<serde_json::Value>, StdError> {
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
) -> Result<Vec<serde_json::Value>, StdError> {
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
) -> Result<Vec<serde_json::Value>, StdError> {
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

async fn search_all(app_state: &AppState, query: &str) -> Result<Vec<serde_json::Value>, StdError> {
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
        let a_type = a
            .get("resultType")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let b_type = b
            .get("resultType")
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
    query: &str,
) -> Result<Vec<serde_json::Value>, StdError> {
    let client = app_state.db.get_client().await?;

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

async fn get_random_search_terms() -> Vec<String> {
    // Common search terms that should match some data in the test database
    vec![
        "lab".to_string(),
        "test".to_string(),
        "a".to_string(),
        "device".to_string(),
        "laptop".to_string(),
        "user".to_string(),
        "room".to_string(),
        "branch".to_string(),
        "healthy".to_string(),
        "broken".to_string(),
    ]
}

async fn measure_throughput(
    app_state: Arc<AppState>,
    operation: &str,
    concurrency: usize,
    test_duration_secs: u64,
) -> Result<(f64, f64, f64), StdError> {
    let counter = Arc::new(Mutex::new(0));
    let errors = Arc::new(Mutex::new(0));
    let latencies = Arc::new(Mutex::new(Vec::new()));

    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(test_duration_secs);

    let mut handles = Vec::new();

    // Pre-fetch some random search terms to use in the tests
    let search_terms = get_random_search_terms().await;

    for _ in 0..concurrency {
        let app_state_clone = app_state.clone();
        let counter_clone = counter.clone();
        let errors_clone = errors.clone();
        let latencies_clone = latencies.clone();
        let operation_clone = operation.to_string();
        let search_terms_clone = search_terms.clone();

        let handle = task::spawn(async move {
            while Instant::now() < end_time {
                let request_start = Instant::now();

                // Get a random search term
                let idx = rng().random_range(0..search_terms_clone.len());
                let search_term = &search_terms_clone[idx];

                let result = match operation_clone.as_str() {
                    "search_devices" => search_devices(&app_state_clone, search_term)
                        .await
                        .map(|_| ()),
                    "search_users" => search_users(&app_state_clone, search_term)
                        .await
                        .map(|_| ()),
                    "search_labs" => search_labs(&app_state_clone, search_term).await.map(|_| ()),
                    "search_all" => search_all(&app_state_clone, search_term).await.map(|_| ()),
                    "search_device_kind" => search_device_kind(&app_state_clone, search_term)
                        .await
                        .map(|_| ()),
                    _ => Err("Unknown operation".into()) as Result<(), StdError>,
                };

                let request_duration = request_start.elapsed();

                match result {
                    Ok(_) => {
                        let mut count = counter_clone.lock().unwrap();
                        *count += 1;

                        let mut lat = latencies_clone.lock().unwrap();
                        lat.push(request_duration.as_secs_f64() * 1000.0);
                    }
                    Err(e) => {
                        let mut err_count = errors_clone.lock().unwrap();
                        *err_count += 1;
                        // Print the first few errors to help diagnose issues
                        if *err_count <= 3 {
                            println!("Error in operation {}: {}", operation_clone, e);
                        }
                    }
                }

                // Small delay to prevent burning CPU
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let total = *counter.lock().unwrap();
    let error_count = *errors.lock().unwrap();

    let throughput = total as f64 / elapsed;

    let latencies_vec = latencies.lock().unwrap();
    let avg_latency = if !latencies_vec.is_empty() {
        latencies_vec.iter().sum::<f64>() / latencies_vec.len() as f64
    } else {
        0.0
    };

    let p95_latency = if !latencies_vec.is_empty() {
        let mut sorted_latencies = latencies_vec.clone();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_idx = (sorted_latencies.len() as f64 * 0.95) as usize;
        sorted_latencies[p95_idx]
    } else {
        0.0
    };

    println!(
        "Operation: {}, Concurrency: {}, Throughput: {:.2} req/s, Avg Latency: {:.2} ms, P95 Latency: {:.2} ms, Errors: {}",
        operation, concurrency, throughput, avg_latency, p95_latency, error_count
    );

    Ok((throughput, avg_latency, p95_latency))
}

async fn run_stress_test(app_state: Arc<AppState>) -> Result<(), StdError> {
    println!("\n=== STARTING SEARCH PERFORMANCE STRESS TEST ===\n");

    let (
        num_users,
        num_device_kinds,
        num_devices,
        num_labs,
        concurrent_requests,
        test_duration_secs,
        _,
    ) = get_config();

    // Clean up test data
    // Clean up existing data and populate with large dataset
    println!("Cleaning up existing data...");
    match cleanup_test_tables(&app_state.db).await {
        Ok(_) => println!("Data cleanup complete"),
        Err(e) => return Err(format!("Error cleaning up test tables: {}", e).into()),
    }

    // Use the populate_large_test_data function from common.rs
    populate_large_test_data(
        &app_state.db,
        num_users,
        num_device_kinds,
        num_devices,
        num_labs,
    )
    .await?;

    println!("\n=== RUNNING SEARCH LOAD TESTS ===\n");

    println!("Testing search_devices operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "search_devices",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    println!("\nTesting search_users operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "search_users",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    println!("\nTesting search_labs operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "search_labs",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    println!("\nTesting search_device_kind operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "search_device_kind",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    println!("\nTesting search_all operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "search_all",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    // Clean up test tables after benchmarks
    println!("Cleaning up test data...");
    match cleanup_test_tables(&app_state.db).await {
        Ok(_) => println!("Test data cleanup complete"),
        Err(e) => return Err(format!("Error cleaning up test tables: {}", e).into()),
    }

    println!("\n=== SEARCH STRESS TEST COMPLETED ===\n");
    Ok(())
}

fn benchmark_stress(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for stress test");
    let app_state = rt.block_on(setup_bench_env());
    let app_state_arc = Arc::new(app_state);

    let mut group = c.benchmark_group("Search-Stress-Test");

    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(60));

    group.bench_function("Full Search Stress Test", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = run_stress_test(app_state_arc.clone()).await;
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_stress);
criterion_main!(benches);
