use chrono;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rng, Rng};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::task;
use uuid::Uuid;

use hmi_lib::commands::db_commands::{InsertParams, QueryParams};

mod common;
use common::{setup_bench_env, AppState};

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

async fn print_table_counts(client: &tokio_postgres::Client) -> Result<(), StdError> {
    println!("\n--- Current Database State ---");
    let tables = [
        "bench_labs",
        "bench_users",
        "bench_device_kinds",
        "bench_devices",
        "bench_receipts",
        "bench_activities",
        "bench_receipts_devices",
    ];

    for table in tables {
        match client
            .query_one(&format!("SELECT COUNT(*) FROM {}", table), &[])
            .await
        {
            Ok(row) => {
                let count: i64 = row.get(0);
                println!("{}: {} rows", table, count);
            }
            Err(e) => println!("Error querying {}: {}", table, e),
        }
    }
    println!("-----------------------------\n");
    Ok(())
}

async fn generate_test_data(
    app_state: &AppState,
    num_users: usize,
    num_device_kinds: usize,
    num_devices: usize,
    num_labs: usize,
) -> Result<(), StdError> {
    println!("Generating test data...");
    let start_time = Instant::now();

    let mut client = app_state.db.get_client().await?;

    println!("Cleaning up existing data...");

    // Drop indexes first to speed up deletion
    client
        .batch_execute(
            "
        DROP INDEX IF EXISTS idx_bench_devices_status_id_kind_lab;
        DROP INDEX IF EXISTS idx_bench_devices_status_healthy;
        DROP INDEX IF EXISTS idx_bench_devices_status_borrowing;
        DROP INDEX IF EXISTS idx_bench_activities_created_at;
        DROP INDEX IF EXISTS idx_bench_receipts_devices_borrowed_receipt;
        DROP INDEX IF EXISTS idx_bench_receipts_devices_return;
        DROP INDEX IF EXISTS idx_bench_receipts_devices_return_id;
    ",
        )
        .await?;

    // Use a single transaction for all deletions to speed up the process
    let transaction = client.transaction().await?;
    transaction
        .execute("DELETE FROM bench_receipts_devices", &[])
        .await?;
    transaction
        .execute("DELETE FROM bench_activities", &[])
        .await?;
    transaction
        .execute("DELETE FROM bench_receipts", &[])
        .await?;
    transaction
        .execute("DELETE FROM bench_devices", &[])
        .await?;
    transaction
        .execute("DELETE FROM bench_device_kinds", &[])
        .await?;
    transaction.execute("DELETE FROM bench_users", &[]).await?;
    transaction.execute("DELETE FROM bench_labs", &[]).await?;
    transaction.commit().await?;

    println!("Data cleanup complete");

    // Disable triggers temporarily to speed up inserts
    client
        .execute("SET session_replication_role = 'replica'", &[])
        .await?;

    println!("Creating labs...");
    {
        // Prepare all lab data at once
        let mut lab_values = Vec::with_capacity(num_labs);
        for i in 0..num_labs {
            let lab_id = Uuid::new_v4();
            let lab_name = format!("Lab {}", i + 1);
            lab_values.push(format!(
                "('{}', '{}', 'Room {}', 'Branch {}')",
                lab_id,
                lab_name,
                i + 100,
                (i % 5) + 1
            ));
        }

        // Insert all labs in a single query
        let query = format!(
            "INSERT INTO bench_labs (id, name, room, branch) VALUES {}",
            lab_values.join(", ")
        );

        client.execute(&query, &[]).await?;
        println!("Created {} labs", num_labs);
    }

    let lab_rows = client.query("SELECT id::text FROM bench_labs", &[]).await?;
    let lab_ids: Vec<String> = lab_rows.iter().map(|row| row.get(0)).collect();

    if lab_ids.is_empty() {
        return Err("Failed to create any labs".into());
    }

    println!("Creating users...");
    // Increase batch size for better performance
    let batch_size = 1000;
    for batch in 0..(num_users / batch_size + 1) {
        let start_idx = batch * batch_size;
        let end_idx = std::cmp::min((batch + 1) * batch_size, num_users);

        if start_idx >= end_idx {
            break;
        }

        let mut user_values = Vec::with_capacity(end_idx - start_idx);
        for i in start_idx..end_idx {
            let user_id = Uuid::new_v4();
            let user_name = format!("User {}", i + 1);
            let email = format!("user{}@example.com", i + 1);
            let image = json!({
                "url": format!("https://example.com/avatars/{}.jpg", i + 1)
            });

            user_values.push(format!(
                "('{}', '{}', '{}', '{}')",
                user_id, user_name, email, image
            ));
        }

        let query = format!(
            "INSERT INTO bench_users (id, name, email, image) VALUES {}",
            user_values.join(", ")
        );

        client.execute(&query, &[]).await?;

        if (batch + 1) % 10 == 0 || end_idx == num_users {
            println!("Created {}/{} users", end_idx, num_users);
        }
    }

    println!("Creating device kinds...");
    // Increase batch size for better performance
    let batch_size = 1000;
    for batch in 0..(num_device_kinds / batch_size + 1) {
        let start_idx = batch * batch_size;
        let end_idx = std::cmp::min((batch + 1) * batch_size, num_device_kinds);

        if start_idx >= end_idx {
            break;
        }

        let mut kind_values = Vec::with_capacity(end_idx - start_idx);
        for i in start_idx..end_idx {
            let kind_id = Uuid::new_v4();
            let kind_name = format!("Device Kind {}", i + 1);
            let image = json!({
                "url": format!("https://example.com/devices/{}.jpg", i + 1)
            });

            kind_values.push(format!(
                "('{}', '{}', '{}', false, '{{}}', '{{}}')",
                kind_id, kind_name, image
            ));
        }

        let query = format!(
            "INSERT INTO bench_device_kinds (id, name, image, is_borrowable_lab_only, allowed_borrow_roles, allowed_view_roles) VALUES {}",
            kind_values.join(", ")
        );

        client.execute(&query, &[]).await?;

        if (batch + 1) % 10 == 0 || end_idx == num_device_kinds {
            println!("Created {}/{} device kinds", end_idx, num_device_kinds);
        }
    }

    let kind_rows = client
        .query("SELECT id::text FROM bench_device_kinds", &[])
        .await?;
    let kind_ids: Vec<String> = kind_rows.iter().map(|row| row.get(0)).collect();

    if kind_ids.is_empty() {
        return Err("Failed to create any device kinds".into());
    }

    println!("Creating devices...");
    // Increase batch size significantly for devices
    let batch_size = 5000;
    // Make sure we have more healthy devices than other statuses
    let statuses = [
        "healthy",
        "healthy",
        "healthy",
        "borrowing",
        "broken",
        "lost",
    ];
    let total_batches = (num_devices + batch_size - 1) / batch_size;

    for batch in 0..total_batches {
        let start_idx = batch * batch_size;
        let end_idx = std::cmp::min((batch + 1) * batch_size, num_devices);

        if start_idx >= end_idx {
            break;
        }

        let mut device_values = Vec::with_capacity(end_idx - start_idx);
        for i in start_idx..end_idx {
            let device_id = Uuid::new_v4();
            let kind_idx = i % kind_ids.len();
            let lab_idx = i % lab_ids.len();
            let status_idx = i % statuses.len();
            let full_id = format!(
                "DEV-{}-{}",
                i,
                device_id.to_string().split('-').next().unwrap_or("")
            );

            device_values.push(format!(
                "('{}', '{}', '{}'::bench_device_status, '{}', '{}')",
                device_id, kind_ids[kind_idx], statuses[status_idx], lab_ids[lab_idx], full_id
            ));
        }

        let query = format!(
            "INSERT INTO bench_devices (id, kind, status, lab_id, full_id) VALUES {}",
            device_values.join(", ")
        );

        let result = client.execute(&query, &[]).await;
        if let Err(e) = result {
            println!("Error in batch {}/{}: {}", batch + 1, total_batches, e);
            return Err(e.into());
        }

        if (batch + 1) % 10 == 0 || end_idx == num_devices {
            println!(
                "Created {}/{} devices ({:.2}%)",
                end_idx,
                num_devices,
                (end_idx as f64 / num_devices as f64) * 100.0
            );
        }
    }

    // Re-enable triggers
    client
        .execute("SET session_replication_role = 'origin'", &[])
        .await?;

    // Create only necessary indexes after data insertion
    println!("Creating optimized indexes to speed up queries...");
    client
        .batch_execute(
            "
        -- Optimized index strategy for bench_devices
        -- Use a single covering index for the most common query patterns
        CREATE INDEX IF NOT EXISTS idx_bench_devices_status_id_kind_lab
            ON bench_devices(status, id, kind, lab_id);

        -- Specialized index for the borrow operation with INCLUDE to avoid additional lookups
        CREATE INDEX IF NOT EXISTS idx_bench_devices_status_healthy
            ON bench_devices(id)
            WHERE status = 'healthy'::bench_device_status;

        -- Specialized index for the return operation
        CREATE INDEX IF NOT EXISTS idx_bench_devices_status_borrowing
            ON bench_devices(id)
            WHERE status = 'borrowing'::bench_device_status;

        -- Index for bench_activities created_at (used in sorting)
        CREATE INDEX IF NOT EXISTS idx_bench_activities_created_at
            ON bench_activities(created_at);

        -- Optimized index for receipts_devices queries
        CREATE INDEX IF NOT EXISTS idx_bench_receipts_devices_borrowed_receipt
            ON bench_receipts_devices(borrowed_receipt_id, device_id);

        -- Optimized index for return queries
        CREATE INDEX IF NOT EXISTS idx_bench_receipts_devices_return
            ON bench_receipts_devices(device_id, returned_receipt_id)
            WHERE returned_receipt_id IS NULL;

        -- Index for get_random_borrowing_device_ids with INCLUDE to avoid lookups
        CREATE INDEX IF NOT EXISTS idx_bench_receipts_devices_return_id
            ON bench_receipts_devices(device_id)
            INCLUDE (borrow_id, expected_returned_at)
            WHERE return_id IS NULL;
    ",
        )
        .await?;

    println!("Indexes created successfully");

    println!(
        "Test data generation completed in {:?}",
        start_time.elapsed()
    );
    print_table_counts(&client).await?;

    // Check device status counts
    let status_counts = client
        .query(
            "SELECT status::text, COUNT(*) FROM bench_devices GROUP BY status ORDER BY status",
            &[],
        )
        .await?;

    println!("\n--- Device Status Counts ---");
    for row in &status_counts {
        let status: String = row.get(0);
        let count: i64 = row.get(1);
        println!("{}: {} devices", status, count);
    }
    println!("---------------------------\n");

    Ok(())
}

async fn fetch_ready_borrow_devices(
    app_state: &AppState,
    params: &QueryParams,
) -> Result<Vec<serde_json::Value>, StdError> {
    let client = app_state.db.get_client().await?;

    let limit = params.limit.unwrap_or(10) as i64;
    let offset = params.offset.unwrap_or(0) as i64;

    // Default to ordering by name
    let order_by_name = match params.order_by {
        Some(ref order) if !order.is_empty() => {
            let (field, is_asc) = &order[0];
            if field == "bench_device_kinds.name" {
                is_asc
            } else {
                &true
            }
        }
        _ => &true,
    };

    // Use prepared statement with parameters for better performance
    let sql = if *order_by_name {
        // Optimized query using indexes (idx_bench_devices_status) with ASC order
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
            bench_devices.status = 'healthy'::bench_device_status
            AND bench_devices.deleted_at IS NULL
        GROUP BY
            bench_device_kinds.id,
            bench_device_kinds.name,
            bench_device_kinds.image,
            bench_labs.name
        ORDER BY bench_device_kinds.name ASC
        LIMIT $1 OFFSET $2"
    } else {
        // Optimized query using indexes (idx_bench_devices_status) with DESC order
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
            bench_devices.status = 'healthy'::bench_device_status
            AND bench_devices.deleted_at IS NULL
        GROUP BY
            bench_device_kinds.id,
            bench_device_kinds.name,
            bench_device_kinds.image,
            bench_labs.name
        ORDER BY bench_device_kinds.name DESC
        LIMIT $1 OFFSET $2"
    };

    // Prepare and execute the statement
    let stmt = client.prepare(sql).await?;
    let rows = client.query(&stmt, &[&limit, &offset]).await?;

    let results: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let quantity: i64 = row.get("quantity");
            json!({
                "kind": row.get::<_, Uuid>("kind").to_string(),
                "name": row.get::<_, String>("name"),
                "image": row.get::<_, serde_json::Value>("image"),
                "quantity": quantity,
                "place": row.get::<_, String>("place")
            })
        })
        .collect();

    Ok(results)
}

async fn fetch_borrowing_devices(
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
        _ => "ORDER BY bench_activities.created_at DESC".to_string(),
    };

    // Optimized query using indexes (idx_bench_receipts_devices_return_id, idx_bench_activities_created_at)
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

    // Use prepared statement for better performance
    let stmt = client.prepare(&sql).await?;
    let rows = client.query(&stmt, &[]).await?;

    let results: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let borrowed_at: chrono::DateTime<chrono::Utc> = row.get("borrowed_at");
            let expected_returned_at: chrono::DateTime<chrono::Utc> =
                row.get("expected_returned_at");

            let borrower_image = match row.try_get::<_, serde_json::Value>("borrower_image") {
                Ok(img) => img,
                Err(_) => match row.try_get::<_, String>("borrower_image") {
                    Ok(img_str) => serde_json::from_str(&img_str).unwrap_or(json!({"url": ""})),
                    Err(_) => json!({"url": ""}),
                },
            };

            json!({
                "receipt_code": row.get::<_, Uuid>("receipt_code").to_string(),
                "borrower_name": row.get::<_, String>("borrower_name"),
                "borrower_image": borrower_image,
                "total_qty": row.get::<_, i64>("total_qty"),
                "returned_qty": row.get::<_, i64>("returned_qty"),
                "borrowed_place": row.get::<_, String>("borrowed_place"),
                "borrowed_at": borrowed_at.to_rfc3339(),
                "expected_returned_at": expected_returned_at.to_rfc3339(),
                "status": row.get::<_, String>("status"),
                "borrow_state": row.get::<_, String>("borrow_state")
            })
        })
        .collect();

    Ok(results)
}

async fn fetch_returned_devices(
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
        _ => "ORDER BY bench_activities.created_at DESC".to_string(),
    };

    // Optimized query using indexes (idx_bench_receipts_devices_return_id, idx_bench_activities_created_at)
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
            JOIN bench_users ON bench_receipts.actor_id = bench_users.id
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

    // Use prepared statement for better performance
    let stmt = client.prepare(&sql).await?;
    let rows = client.query(&stmt, &[]).await?;

    let results: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let returned_at: chrono::DateTime<chrono::Utc> = row.get("returned_at");

            let returned_image = match row.try_get::<_, serde_json::Value>("returned_image") {
                Ok(img) => img,
                Err(_) => match row.try_get::<_, String>("returned_image") {
                    Ok(img_str) => serde_json::from_str(&img_str).unwrap_or(json!({"url": ""})),
                    Err(_) => json!({"url": ""}),
                },
            };

            json!({
                "receipt_code": row.get::<_, Uuid>("receipt_code").to_string(),
                "returned_name": row.get::<_, String>("returned_name"),
                "returned_image": returned_image,
                "quantity": row.get::<_, i64>("quantity"),
                "returned_place": row.get::<_, String>("returned_place"),
                "returned_at": returned_at.to_rfc3339(),
                "status": row.get::<_, String>("status"),
                "note": row.get::<_, Option<String>>("note")
            })
        })
        .collect();

    Ok(results)
}

async fn create_receipt(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, StdError> {
    let mut client = app_state.db.get_client().await?;

    // Use READ COMMITTED isolation level for better performance
    let transaction = client
        .build_transaction()
        .isolation_level(tokio_postgres::IsolationLevel::ReadCommitted)
        .start()
        .await?;

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

    // Extract device IDs first
    let mut device_ids = Vec::with_capacity(devices.len());
    for device in devices {
        let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if device_id.is_empty() {
            continue;
        }
        let device_uuid = Uuid::parse_str(device_id)?;
        device_ids.push(device_uuid);
    }

    if device_ids.is_empty() {
        return Ok(json!({ "error": "No valid device IDs provided" }));
    }

    // Prepare parameters for the multi-step operation
    let borrower_uuid = Uuid::parse_str(borrower_id)?;
    let checker_uuid = Uuid::parse_str(borrow_checker_id)?;
    let lab_uuid = Uuid::parse_str(borrowed_lab_id)?;
    let now = chrono::Utc::now();
    let expected_returned_at = now + chrono::Duration::days(7);

    // Use a single CTE query to perform all operations in one database round-trip
    // This dramatically reduces latency and improves throughput
    let device_id_list: Vec<String> = device_ids
        .iter()
        .map(|uuid| format!("'{}'", uuid))
        .collect();

    let query = format!(
        "WITH
        -- First, lock and select available devices
        available_devices AS (
            SELECT id
            FROM bench_devices
            WHERE id IN ({})
            AND status = 'healthy'::bench_device_status
            FOR UPDATE SKIP LOCKED
        ),
        -- Create the receipt
        receipt AS (
            INSERT INTO bench_receipts (id, actor_id, checker_id, lab_id)
            VALUES ('{}', '{}', '{}', '{}')
            RETURNING id
        ),
        -- Create the activity
        activity AS (
            INSERT INTO bench_activities (id, type)
            VALUES (gen_random_uuid(), 'borrow'::bench_activity_type)
            RETURNING id
        ),
        -- Create receipt_devices entries
        receipt_devices AS (
            INSERT INTO bench_receipts_devices (
                borrowed_receipt_id, device_id, borrow_id,
                expected_returned_at, expected_returned_lab_id, prev_quality
            )
            SELECT
                (SELECT id FROM receipt),
                d.id,
                (SELECT id FROM activity),
                '{}'::timestamptz,
                '{}'::uuid,
                'healthy'::bench_device_status
            FROM available_devices d
            RETURNING device_id
        ),
        -- Update device status
        device_update AS (
            UPDATE bench_devices
            SET status = 'borrowing'::bench_device_status
            WHERE id IN (SELECT device_id FROM receipt_devices)
            RETURNING id
        )
        -- Return the count of processed devices
        SELECT COUNT(*) as processed_count FROM device_update",
        device_id_list.join(","),
        receipt_uuid,
        borrower_uuid,
        checker_uuid,
        lab_uuid,
        expected_returned_at.format("%Y-%m-%d %H:%M:%S%.f+00"),
        lab_uuid
    );

    // Execute the entire operation in a single query
    let result_row = transaction.query_one(&query, &[]).await?;
    let processed_count: i64 = result_row.get("processed_count");

    // Commit the transaction
    transaction.commit().await?;

    Ok(json!({
        "success": true,
        "id": receipt_id,
        "devices_processed": processed_count
    }))
}

async fn return_receipt(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, StdError> {
    let mut client = app_state.db.get_client().await?;

    // Use READ COMMITTED isolation level for better performance
    let transaction = client
        .build_transaction()
        .isolation_level(tokio_postgres::IsolationLevel::ReadCommitted)
        .start()
        .await?;

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

    // Extract device IDs and quality map
    let mut device_ids = Vec::with_capacity(devices.len());
    let mut device_quality_map = HashMap::new();

    for device in devices {
        let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if device_id.is_empty() {
            continue;
        }
        let device_uuid = Uuid::parse_str(device_id)?;
        device_ids.push(device_uuid);

        let after_quality = device
            .get("afterQuality")
            .and_then(|v| v.as_str())
            .unwrap_or("healthy");

        device_quality_map.insert(device_uuid, after_quality.to_string());
    }

    if device_ids.is_empty() {
        return Ok(json!({ "error": "No valid device IDs provided" }));
    }

    // Prepare parameters for the multi-step operation
    let returner_uuid = Uuid::parse_str(returner_id)?;
    let checker_uuid = Uuid::parse_str(return_checker_id)?;
    let lab_uuid = Uuid::parse_str(returned_lab_id)?;

    // Lock the devices and verify they are still in borrowing status and not already returned
    let device_id_list: Vec<String> = device_ids
        .iter()
        .map(|uuid| format!("'{}'", uuid))
        .collect();

    let lock_query = format!(
        "SELECT rd.device_id
         FROM bench_receipts_devices rd
         JOIN bench_devices d ON rd.device_id = d.id
         WHERE rd.device_id IN ({})
         AND rd.returned_receipt_id IS NULL
         AND d.status = 'borrowing'::bench_device_status
         FOR UPDATE SKIP LOCKED",
        device_id_list.join(",")
    );

    let locked_rows = transaction.query(&lock_query, &[]).await?;
    let locked_ids: Vec<Uuid> = locked_rows.iter().map(|row| row.get(0)).collect();

    // Check if we have any devices to return
    if locked_ids.is_empty() {
        return Ok(json!({
            "error": "No devices are available for return"
        }));
    }

    // We'll proceed with the devices we were able to lock
    let device_ids = locked_ids;

    // Insert receipt
    transaction
        .execute(
            "INSERT INTO bench_receipts (id, actor_id, checker_id, lab_id) VALUES ($1, $2, $3, $4)",
            &[&receipt_uuid, &returner_uuid, &checker_uuid, &lab_uuid],
        )
        .await?;

    // Create activity
    let activity_row = if let Some(note_text) = note {
        transaction
            .query_one(
                "INSERT INTO bench_activities (id, type, note)
                VALUES (gen_random_uuid(), 'return'::bench_activity_type, $1)
                RETURNING id",
                &[&note_text],
            )
            .await?
    } else {
        transaction
            .query_one(
                "INSERT INTO bench_activities (id, type)
                VALUES (gen_random_uuid(), 'return'::bench_activity_type)
                RETURNING id",
                &[],
            )
            .await?
    };
    let activity_uuid: Uuid = activity_row.get(0);

    // Process each device individually to avoid SQL syntax issues
    let mut processed_count = 0;

    for device_uuid in device_ids {
        let quality = device_quality_map
            .get(&device_uuid)
            .unwrap_or(&"healthy".to_string())
            .clone();

        // Use string interpolation for the enum type to avoid type conversion issues
        let update_query = format!(
            "UPDATE bench_receipts_devices
             SET returned_receipt_id = '{}',
                 return_id = '{}',
                 after_quality = '{}'::bench_device_status
             WHERE device_id = '{}'
             AND returned_receipt_id IS NULL",
            receipt_uuid,
            activity_uuid,
            quality,
            device_uuid
        );

        let updated = transaction.execute(&update_query, &[]).await?;

        if updated > 0 {
            // Use string interpolation for the enum type
            let device_update_query = format!(
                "UPDATE bench_devices
                 SET status = '{}'::bench_device_status
                 WHERE id = '{}'",
                quality,
                device_uuid
            );

            transaction.execute(&device_update_query, &[]).await?;

            processed_count += 1;
        }
    }

    // Commit the transaction
    transaction.commit().await?;

    Ok(json!({
        "success": true,
        "id": receipt_id,
        "devices_processed": processed_count
    }))
}

async fn get_random_healthy_device_ids(
    app_state: &AppState,
    count: usize,
) -> Result<Vec<String>, StdError> {
    let client = app_state.db.get_client().await?;

    // Use a more efficient query with specialized index
    // This avoids the need for a transaction and reduces database round-trips
    let query = format!(
        "SELECT id::text FROM bench_devices
         WHERE status = 'healthy'::bench_device_status
         ORDER BY random()
         LIMIT {}",
        count * 2 // Get more than needed to increase chances of success
    );

    // Execute the query directly without a transaction for better performance
    let rows = client.query(&query, &[]).await?;
    let mut device_ids: Vec<String> = rows.iter().map(|row| row.get(0)).collect();

    if device_ids.is_empty() {
        return Err("No healthy devices available".into());
    }

    // Shuffle and trim to requested count
    if device_ids.len() > count {
        use rand::seq::SliceRandom;
        device_ids.shuffle(&mut rng());
        device_ids.truncate(count);
    }

    Ok(device_ids)
}

async fn get_random_user_ids(app_state: &AppState, count: usize) -> Result<Vec<String>, StdError> {
    let client = app_state.db.get_client().await?;

    // Use a more efficient query without transaction
    // This reduces database round-trips and improves performance
    let query = format!(
        "SELECT id::text FROM bench_users
         ORDER BY random()
         LIMIT {}",
        count * 2 // Get more than needed to increase chances of success
    );

    // Execute the query directly without a transaction for better performance
    let rows = client.query(&query, &[]).await?;
    let mut user_ids: Vec<String> = rows.iter().map(|row| row.get(0)).collect();

    if user_ids.is_empty() {
        return Err("No users available".into());
    }

    // Shuffle and trim to requested count
    if user_ids.len() > count {
        use rand::seq::SliceRandom;
        user_ids.shuffle(&mut rng());
        user_ids.truncate(count);
    }

    Ok(user_ids)
}

async fn get_random_lab_id(app_state: &AppState) -> Result<String, StdError> {
    let client = app_state.db.get_client().await?;

    // Use a more efficient query without transaction
    // This reduces database round-trips and improves performance
    let row = client
        .query_one(
            "SELECT id::text FROM bench_labs ORDER BY random() LIMIT 1",
            &[],
        )
        .await?;

    let lab_id: String = row.get(0);
    Ok(lab_id)
}

async fn get_random_borrowing_device_ids(
    app_state: &AppState,
    count: usize,
) -> Result<Vec<String>, StdError> {
    let client = app_state.db.get_client().await?;

    // Use a more efficient query with specialized index
    // This avoids the need for a transaction and reduces database round-trips
    let query = format!(
        "SELECT device_id::text FROM bench_receipts_devices
         WHERE return_id IS NULL
         ORDER BY random()
         LIMIT {}",
        count * 2 // Get more than needed to increase chances of success
    );

    // Execute the query directly without a transaction for better performance
    let rows = client.query(&query, &[]).await?;
    let mut device_ids: Vec<String> = rows.iter().map(|row| row.get(0)).collect();

    if device_ids.is_empty() {
        return Err("No borrowing devices available".into());
    }

    // Shuffle and trim to requested count
    if device_ids.len() > count {
        use rand::seq::SliceRandom;
        device_ids.shuffle(&mut rng());
        device_ids.truncate(count);
    }

    Ok(device_ids)
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

    let random_user_ids = get_random_user_ids(&app_state, 100).await?;
    let random_lab_id = get_random_lab_id(&app_state).await?;

    for _ in 0..concurrency {
        let app_state_clone = app_state.clone();
        let counter_clone = counter.clone();
        let errors_clone = errors.clone();
        let latencies_clone = latencies.clone();
        let operation_clone = operation.to_string();
        let user_ids_clone = random_user_ids.clone();
        let lab_id_clone = random_lab_id.clone();

        let handle = task::spawn(async move {
            while Instant::now() < end_time {
                let request_start = Instant::now();
                let result = match operation_clone.as_str() {
                    "fetch_ready" => {
                        let params = QueryParams {
                            table: "bench_devices".to_string(),
                            columns: None,
                            conditions: None,
                            order_by: Some(vec![("bench_device_kinds.name".to_string(), true)]),
                            limit: Some(10),
                            offset: Some(0),
                            joins: None,
                        };
                        fetch_ready_borrow_devices(&app_state_clone, &params)
                            .await
                            .map(|_| ())
                    }
                    "fetch_borrowing" => {
                        let params = QueryParams {
                            table: "bench_receipts_devices".to_string(),
                            columns: None,
                            conditions: None,
                            order_by: Some(vec![(
                                "bench_activities.created_at".to_string(),
                                false,
                            )]),
                            limit: Some(10),
                            offset: Some(0),
                            joins: None,
                        };
                        fetch_borrowing_devices(&app_state_clone, &params)
                            .await
                            .map(|_| ())
                    }
                    "fetch_returned" => {
                        let params = QueryParams {
                            table: "bench_receipts_devices".to_string(),
                            columns: None,
                            conditions: None,
                            order_by: Some(vec![(
                                "bench_activities.created_at".to_string(),
                                false,
                            )]),
                            limit: Some(10),
                            offset: Some(0),
                            joins: None,
                        };
                        fetch_returned_devices(&app_state_clone, &params)
                            .await
                            .map(|_| ())
                    }
                    "create_borrow" => {
                        // Implement retry logic for serialization failures
                        const MAX_RETRIES: usize = 3;
                        let mut retry_count = 0;

                        loop {
                            let random_devices =
                                match get_random_healthy_device_ids(&app_state_clone, 2).await {
                                    Ok(devices) => devices,
                                    Err(_) => {
                                        let mut err_count = errors_clone.lock().unwrap();
                                        *err_count += 1;
                                        break Err("Failed to get healthy devices".into());
                                    }
                                };

                            if random_devices.len() < 2 {
                                let mut err_count = errors_clone.lock().unwrap();
                                *err_count += 1;
                                break Err("Not enough healthy devices available".into());
                            }

                            let user_idx = rng().random_range(0..user_ids_clone.len());

                            let params = InsertParams {
                                table: "bench_receipts".to_string(),
                                value: json!({
                                    "borrowerId": user_ids_clone[user_idx],
                                    "borrowCheckerId": user_ids_clone[(user_idx + 1) % user_ids_clone.len()],
                                    "borrowedLabId": lab_id_clone,
                                    "devices": [
                                        {
                                            "id": random_devices[0],
                                            "expectedReturnedAt": "NOW() + INTERVAL '7 days'",
                                            "prevQuality": "healthy"
                                        },
                                        {
                                            "id": random_devices[1],
                                            "expectedReturnedAt": "NOW() + INTERVAL '14 days'",
                                            "prevQuality": "healthy"
                                        }
                                    ]
                                }),
                            };

                            match create_receipt(&app_state_clone, &params).await {
                                Ok(_) => break Ok(()),
                                Err(e) => {
                                    // Check if it's a serialization failure that can be retried
                                    if retry_count < MAX_RETRIES
                                        && e.to_string().contains("could not serialize access")
                                    {
                                        retry_count += 1;
                                        // Small delay before retry to reduce contention
                                        tokio::time::sleep(Duration::from_millis(
                                            10 * retry_count as u64,
                                        ))
                                        .await;
                                        continue;
                                    } else {
                                        // Other error or too many retries
                                        break Err(e);
                                    }
                                }
                            }
                        }
                    }
                    "return_devices" => {
                        // Implement retry logic for serialization failures
                        const MAX_RETRIES: usize = 3;
                        let mut retry_count = 0;

                        loop {
                            let random_devices =
                                match get_random_borrowing_device_ids(&app_state_clone, 2).await {
                                    Ok(devices) => devices,
                                    Err(_) => {
                                        let mut err_count = errors_clone.lock().unwrap();
                                        *err_count += 1;
                                        break Err("Failed to get borrowing devices".into());
                                    }
                                };

                            if random_devices.len() < 2 {
                                let mut err_count = errors_clone.lock().unwrap();
                                *err_count += 1;
                                break Err("Not enough borrowing devices available".into());
                            }

                            let user_idx = rng().random_range(0..user_ids_clone.len());

                            let params = InsertParams {
                                table: "bench_receipts".to_string(),
                                value: json!({
                                    "returnerId": user_ids_clone[user_idx],
                                    "returnedCheckerId": user_ids_clone[(user_idx + 1) % user_ids_clone.len()],
                                    "returnedLabId": lab_id_clone,
                                    "devices": [
                                        {
                                            "id": random_devices[0],
                                            "afterQuality": "healthy"
                                        },
                                        {
                                            "id": random_devices[1],
                                            "afterQuality": "broken"
                                        }
                                    ],
                                    "note": "Returned after stress test"
                                }),
                            };

                            match return_receipt(&app_state_clone, &params).await {
                                Ok(_) => break Ok(()),
                                Err(e) => {
                                    // Check if it's a serialization failure that can be retried
                                    if retry_count < MAX_RETRIES
                                        && e.to_string().contains("could not serialize access")
                                    {
                                        retry_count += 1;
                                        // Small delay before retry to reduce contention
                                        tokio::time::sleep(Duration::from_millis(
                                            10 * retry_count as u64,
                                        ))
                                        .await;
                                        continue;
                                    } else {
                                        // Other error or too many retries
                                        break Err(e);
                                    }
                                }
                            }
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

async fn cleanup_test_data(app_state: &AppState) -> Result<(), StdError> {
    println!("Cleaning up test data...");
    let client = app_state.db.get_client().await?;

    // Drop indexes first to speed up deletion
    client
        .batch_execute(
            "
        DROP INDEX IF EXISTS idx_bench_devices_status_id_kind_lab;
        DROP INDEX IF EXISTS idx_bench_devices_status_healthy;
        DROP INDEX IF EXISTS idx_bench_devices_status_borrowing;
        DROP INDEX IF EXISTS idx_bench_activities_created_at;
        DROP INDEX IF EXISTS idx_bench_receipts_devices_borrowed_receipt;
        DROP INDEX IF EXISTS idx_bench_receipts_devices_return;
        DROP INDEX IF EXISTS idx_bench_receipts_devices_return_id;
    ",
        )
        .await?;

    // Delete all data
    client
        .execute("DELETE FROM bench_receipts_devices", &[])
        .await?;
    client.execute("DELETE FROM bench_activities", &[]).await?;
    client.execute("DELETE FROM bench_receipts", &[]).await?;
    client.execute("DELETE FROM bench_devices", &[]).await?;
    client
        .execute("DELETE FROM bench_device_kinds", &[])
        .await?;
    client.execute("DELETE FROM bench_users", &[]).await?;
    client.execute("DELETE FROM bench_labs", &[]).await?;

    println!("Test data cleanup completed");
    Ok(())
}

async fn run_stress_test(app_state: Arc<AppState>) -> Result<(), StdError> {
    println!("\n=== STARTING PERFORMANCE STRESS TEST ===\n");

    let (
        num_users,
        num_device_kinds,
        num_devices,
        num_labs,
        concurrent_requests,
        test_duration_secs,
        _,
    ) = get_config();

    generate_test_data(
        &app_state,
        num_users,
        num_device_kinds,
        num_devices,
        num_labs,
    )
    .await?;

    println!("\n=== RUNNING LOAD TESTS ===\n");

    println!("Testing fetch_ready_borrow_devices operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "fetch_ready",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    println!("\nTesting create_receipt (borrow) operation...");

    // Check if we have enough healthy devices before running the borrow test
    let client = app_state.db.get_client().await?;
    let count_stmt =
        "SELECT COUNT(*) FROM bench_devices WHERE status = 'healthy'::bench_device_status";
    let count_row = client.query_one(count_stmt, &[]).await?;
    let healthy_count: i64 = count_row.get(0);

    if healthy_count < 10 {
        println!(
            "Warning: Only {} healthy devices available. This may cause errors in the borrow test.",
            healthy_count
        );
    }

    println!("Healthy devices available: {}", healthy_count);

    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "create_borrow",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    println!("\nTesting fetch_borrowing_devices operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "fetch_borrowing",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    println!("\nTesting return_receipt operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "return_devices",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    println!("\nTesting fetch_returned_devices operation...");
    for &concurrency in concurrent_requests {
        measure_throughput(
            app_state.clone(),
            "fetch_returned",
            concurrency,
            test_duration_secs,
        )
        .await?;
    }

    cleanup_test_data(&app_state).await?;

    println!("\n=== STRESS TEST COMPLETED ===\n");
    Ok(())
}

fn benchmark_stress(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for stress test");
    let app_state = rt.block_on(setup_bench_env());
    let app_state_arc = Arc::new(app_state);

    let mut group = c.benchmark_group("Stress-Test");

    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(60));

    group.bench_function("Full Stress Test", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = run_stress_test(app_state_arc.clone()).await;
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_stress);
criterion_main!(benches);
