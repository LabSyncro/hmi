use chrono;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rng, Rng};
use serde_json::json;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::task;
use uuid::Uuid;

use hmi_lib::commands::db_commands::QueryParams;

mod common;
use common::{ensure_bench_env, AppState};

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

async fn fetch_devices(
    app_state: &AppState,
    params: &QueryParams,
) -> Result<Vec<serde_json::Value>, StdError> {
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
    device_id: &str,
) -> Result<serde_json::Value, StdError> {
    let client = app_state.db.get_client().await?;

    if device_id.is_empty() {
        return Err("Device ID is required".into());
    }

    let device_uuid = match Uuid::parse_str(device_id) {
        Ok(uuid) => uuid,
        Err(_) => return Err("Invalid device ID format".into()),
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
        Ok(None) => return Err("Device not found".into()),
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

async fn get_device_borrow_history(
    app_state: &AppState,
    device_id: &str,
) -> Result<Vec<serde_json::Value>, StdError> {
    let client = app_state.db.get_client().await?;

    if device_id.is_empty() {
        return Ok(Vec::new());
    }

    let device_uuid = match Uuid::parse_str(device_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(Vec::new()),
    };

    let sql = "
        SELECT
            d.id,
            d.full_id AS \"fullId\",
            CASE
                WHEN rd.returned_receipt_id IS NULL THEN
                    CASE
                        WHEN rd.expected_returned_at < NOW() THEN 'OVERDUE'
                        WHEN rd.expected_returned_at < NOW() + INTERVAL '3 days' THEN 'NEAR_DUE'
                        ELSE 'ON_TIME'
                    END
                ELSE 'RETURNED'
            END AS status,
            actor.id AS \"borrower_id\",
            actor.name AS \"borrower_name\",
            actor.email AS \"borrower_email\",
            actor.image AS \"borrower_avatar\",
            a_borrow.created_at AS \"borrow_date\",
            rd.expected_returned_at,
            (rd.return_id IS NOT NULL) AS has_been_returned,
            a_return.created_at AS \"returned_at\",
            a_return.note AS \"returned_note\",
            bl.name AS \"borrowed_lab\",
            COALESCE(rl.name, 'N/A') AS \"expected_return_lab\"
        FROM
            bench_devices d
            JOIN bench_receipts_devices rd ON d.id = rd.device_id
            JOIN bench_receipts r_borrow ON rd.borrowed_receipt_id = r_borrow.id
            JOIN bench_users actor ON r_borrow.actor_id = actor.id
            JOIN bench_labs bl ON r_borrow.lab_id = bl.id
            LEFT JOIN bench_labs rl ON rd.expected_returned_lab_id = rl.id
            LEFT JOIN bench_activities a_borrow ON rd.borrow_id = a_borrow.id
            LEFT JOIN bench_receipts r_return ON rd.returned_receipt_id = r_return.id
            LEFT JOIN bench_activities a_return ON rd.return_id = a_return.id
        WHERE
            d.id = $1
            AND d.deleted_at IS NULL
        ORDER BY
            a_borrow.created_at DESC
    ";

    let rows = client.query(sql, &[&device_uuid]).await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let results = rows
        .iter()
        .map(|row| {
            let borrow_date: chrono::DateTime<chrono::Utc> = row.get("borrow_date");
            let expected_returned_at: chrono::DateTime<chrono::Utc> =
                row.get("expected_returned_at");
            let returned_at: Option<chrono::DateTime<chrono::Utc>> =
                row.try_get("returned_at").ok();

            json!({
                "id": row.get::<_, Uuid>("id").to_string(),
                "fullId": row.get::<_, String>("fullId"),
                "status": row.get::<_, String>("status"),
                "borrower": {
                    "id": row.get::<_, Uuid>("borrower_id").to_string(),
                    "name": row.get::<_, String>("borrower_name"),
                    "email": row.get::<_, String>("borrower_email"),
                    "avatar": row.get::<_, serde_json::Value>("borrower_avatar")
                },
                "borrowDate": borrow_date.to_rfc3339(),
                "expectedReturnedAt": expected_returned_at.to_rfc3339(),
                "hasBeenReturned": row.get::<_, bool>("has_been_returned"),
                "returnedAt": returned_at.map(|dt| dt.to_rfc3339()),
                "returnedNote": row.try_get::<_, String>("returned_note").ok(),
                "borrowedLab": row.get::<_, String>("borrowed_lab"),
                "expectedReturnLab": row.get::<_, String>("expected_return_lab")
            })
        })
        .collect();

    Ok(results)
}

async fn get_device_inventory_by_kind(
    app_state: &AppState,
    kind_id: &str,
) -> Result<Vec<serde_json::Value>, StdError> {
    let client = app_state.db.get_client().await?;

    if kind_id.is_empty() {
        return Ok(Vec::new());
    }

    let kind_uuid = match Uuid::parse_str(kind_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(Vec::new()),
    };

    let sql = "
        SELECT
            l.name || ', ' || l.branch AS location,
            COUNT(d.id) FILTER (WHERE d.status = 'healthy') AS healthy,
            COUNT(d.id) FILTER (WHERE d.status = 'broken') AS broken,
            COUNT(d.id) FILTER (WHERE d.status = 'discarded') AS discarded,
            COUNT(d.id) FILTER (WHERE d.status = 'lost') AS lost
        FROM
            bench_devices d
            JOIN bench_labs l ON d.lab_id = l.id
        WHERE
            d.kind = $1
            AND d.deleted_at IS NULL
        GROUP BY
            l.name, l.branch
        ORDER BY
            l.branch, l.name
    ";

    let rows = client.query(sql, &[&kind_uuid]).await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let results = rows
        .iter()
        .map(|row| {
            json!({
                "location": row.get::<_, String>("location"),
                "healthy": row.get::<_, i64>("healthy"),
                "broken": row.get::<_, i64>("broken"),
                "discarded": row.get::<_, i64>("discarded"),
                "lost": row.get::<_, i64>("lost")
            })
        })
        .collect();

    Ok(results)
}

async fn get_random_device_ids(
    app_state: &AppState,
    count: usize,
) -> Result<Vec<String>, StdError> {
    let client = app_state.db.get_client().await?;

    let sql = format!(
        "SELECT id::text FROM bench_devices WHERE deleted_at IS NULL ORDER BY RANDOM() LIMIT {}",
        count
    );

    let rows = client.query(&sql, &[]).await?;
    let device_ids: Vec<String> = rows.iter().map(|row| row.get(0)).collect();

    Ok(device_ids)
}

async fn get_random_device_kind_ids(
    app_state: &AppState,
    count: usize,
) -> Result<Vec<String>, StdError> {
    let client = app_state.db.get_client().await?;

    let sql = format!(
        "SELECT id::text FROM bench_device_kinds WHERE deleted_at IS NULL ORDER BY RANDOM() LIMIT {}",
        count
    );

    let rows = client.query(&sql, &[]).await?;
    let kind_ids: Vec<String> = rows.iter().map(|row| row.get(0)).collect();

    Ok(kind_ids)
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

    // Pre-fetch some random IDs to use in the tests
    let random_device_ids = get_random_device_ids(&app_state, 100).await?;
    let random_kind_ids = get_random_device_kind_ids(&app_state, 20).await?;

    for _ in 0..concurrency {
        let app_state_clone = app_state.clone();
        let counter_clone = counter.clone();
        let errors_clone = errors.clone();
        let latencies_clone = latencies.clone();
        let operation_clone = operation.to_string();
        let device_ids_clone = random_device_ids.clone();
        let kind_ids_clone = random_kind_ids.clone();

        let handle = task::spawn(async move {
            while Instant::now() < end_time {
                let request_start = Instant::now();
                let result = match operation_clone.as_str() {
                    "fetch_devices" => {
                        let params = QueryParams {
                            table: "bench_devices".to_string(),
                            columns: None,
                            conditions: None,
                            order_by: Some(vec![("bench_devices.full_id".to_string(), true)]),
                            limit: Some(10),
                            offset: Some(0),
                            joins: None,
                        };
                        fetch_devices(&app_state_clone, &params).await.map(|_| ())
                    }
                    "fetch_device_details" => {
                        if device_ids_clone.is_empty() {
                            Err("No device IDs available".into())
                        } else {
                            let idx = rng().random_range(0..device_ids_clone.len());
                            fetch_device_details(&app_state_clone, &device_ids_clone[idx])
                                .await
                                .map(|_| ())
                        }
                    }
                    "get_device_borrow_history" => {
                        if device_ids_clone.is_empty() {
                            Err("No device IDs available".into())
                        } else {
                            let idx = rng().random_range(0..device_ids_clone.len());
                            get_device_borrow_history(&app_state_clone, &device_ids_clone[idx])
                                .await
                                .map(|_| ())
                        }
                    }
                    "get_device_inventory_by_kind" => {
                        if kind_ids_clone.is_empty() {
                            Err("No kind IDs available".into())
                        } else {
                            let idx = rng().random_range(0..kind_ids_clone.len());
                            get_device_inventory_by_kind(&app_state_clone, &kind_ids_clone[idx])
                                .await
                                .map(|_| ())
                        }
                    }
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
    println!("\n=== STARTING DEVICE PERFORMANCE STRESS TEST ===\n");

    let (
        _,
        _,
        _,
        _,
        concurrent_requests,
        test_duration_secs,
        _,
    ) = get_config();

    // We use ensure_bench_env to set up the database
    println!("Using existing database state...");

    println!("\n=== RUNNING DEVICE LOAD TESTS ===\n");

    println!("Testing fetch_devices operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "fetch_devices",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    println!("\nTesting fetch_device_details operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "fetch_device_details",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    println!("\nTesting get_device_borrow_history operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "get_device_borrow_history",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    println!("\nTesting get_device_inventory_by_kind operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "get_device_inventory_by_kind",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    // Database state is preserved for future runs
    println!("Database state preserved for future runs.");

    println!("\n=== DEVICE STRESS TEST COMPLETED ===\n");
    Ok(())
}

fn benchmark_stress(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for stress test");
    let app_state = rt.block_on(ensure_bench_env());
    let app_state_arc = Arc::new(app_state);

    let mut group = c.benchmark_group("Device-Stress-Test");

    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(60));

    group.bench_function("Full Device Stress Test", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = run_stress_test(app_state_arc.clone()).await;
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_stress);
criterion_main!(benches);
