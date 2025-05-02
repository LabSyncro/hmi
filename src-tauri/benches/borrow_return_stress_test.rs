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

use hmi_lib::commands::db_commands::{InsertParams, QueryParams};

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

#[allow(dead_code)]
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

async fn fetch_ready_borrow_devices(
    app_state: &AppState,
    params: &QueryParams,
) -> Result<Vec<serde_json::Value>, StdError> {
    let client = app_state.db.get_client().await?;

    let limit = params.limit.unwrap_or(10) as i64;
    let offset = params.offset.unwrap_or(0) as i64;

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

    let sql = if *order_by_name {
        "WITH healthy_devices AS (
            SELECT
                kind,
                lab_id,
                COUNT(*) as quantity
            FROM
                bench_devices
            WHERE
                status = 'healthy'::bench_device_status
                AND deleted_at IS NULL
            GROUP BY
                kind, lab_id
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
            kind,
            name,
            image,
            quantity,
            place,
            SUM(quantity) OVER() as total_count
        FROM
            device_info
        ORDER BY name ASC
        LIMIT $1 OFFSET $2"
    } else {
        "WITH healthy_devices AS (
            SELECT
                kind,
                lab_id,
                COUNT(*) as quantity
            FROM
                bench_devices
            WHERE
                status = 'healthy'::bench_device_status
                AND deleted_at IS NULL
            GROUP BY
                kind, lab_id
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
            kind,
            name,
            image,
            quantity,
            place,
            SUM(quantity) OVER() as total_count
        FROM
            device_info
        ORDER BY name DESC
        LIMIT $1 OFFSET $2"
    };

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
    let order_field = match params.order_by {
        Some(ref order) if !order.is_empty() => {
            let (field, is_asc) = &order[0];
            if field == "bench_activities.created_at" {
                if *is_asc {
                    "borrowed_at ASC"
                } else {
                    "borrowed_at DESC"
                }
            } else {
                field.as_str()
            }
        }
        _ => "borrowed_at DESC",
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
        ORDER BY {}
        LIMIT {} OFFSET {}",
        order_field, limit, offset
    );

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
    let order_field = match params.order_by {
        Some(ref order) if !order.is_empty() => {
            let (field, is_asc) = &order[0];
            if field == "bench_activities.created_at" {
                if *is_asc {
                    "returned_at ASC"
                } else {
                    "returned_at DESC"
                }
            } else {
                field.as_str()
            }
        }
        _ => "returned_at DESC",
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
        ORDER BY {}
        LIMIT {} OFFSET {}",
        order_field, limit, offset
    );

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
        .unwrap_or("");
    let borrow_checker_id = params
        .value
        .get("borrowCheckerId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let borrowed_lab_id = params
        .value
        .get("borrowedLabId")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let devices_vec = params
        .value
        .get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;

    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for borrowing" }));
    }

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

    if borrower_id.is_empty() || borrow_checker_id.is_empty() || borrowed_lab_id.is_empty() {
        return Ok(
            json!({ "error": "Missing required parameters: borrowerId, borrowCheckerId, or borrowedLabId" }),
        );
    }

    let borrower_uuid = Uuid::parse_str(borrower_id)?;
    let checker_uuid = Uuid::parse_str(borrow_checker_id)?;
    let lab_uuid = Uuid::parse_str(borrowed_lab_id)?;
    let now = chrono::Utc::now();
    let expected_returned_at = now + chrono::Duration::days(7);

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

    let result_row = transaction.query_one(&query, &[]).await?;
    let processed_count: i64 = result_row.get("processed_count");

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
    let client = app_state.db.get_client().await?;

    let receipt_uuid = Uuid::new_v4();
    let receipt_id = receipt_uuid.to_string();
    let activity_uuid = Uuid::new_v4();

    let returner_id = params
        .value
        .get("returnerId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let return_checker_id = params
        .value
        .get("returnedCheckerId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let returned_lab_id = params
        .value
        .get("returnedLabId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let note = params.value.get("note").and_then(|v| v.as_str());

    let devices = params
        .value
        .get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for return" }));
    }

    let devices_json = serde_json::Value::Array(devices.clone());

    let mut query = String::from(
        "
        WITH
        -- Parse the input JSON
        input_data AS (
            SELECT
                $1::uuid as receipt_id,
                $2::uuid as activity_id,
                $3::uuid as returner_id,
                $4::uuid as checker_id,
                $5::uuid as lab_id
        ),
        -- Parse the devices JSON array
        device_data AS (
            SELECT
                (d->>'id')::uuid as device_id,
                COALESCE((d->>'afterQuality')::bench_device_status, 'healthy'::bench_device_status) as quality
            FROM
                json_array_elements($6) as d
        ),
        -- Create the receipt
        receipt AS (
            INSERT INTO bench_receipts (id, actor_id, checker_id, lab_id)
            SELECT receipt_id, returner_id, checker_id, lab_id FROM input_data
            RETURNING id
        ),
        -- Create the activity
        activity AS (
            INSERT INTO bench_activities (id, type",
    );

    if note.is_some() {
        query.push_str(
            ", note) SELECT activity_id, 'return'::bench_activity_type, $7 FROM input_data",
        );
    } else {
        query.push_str(") SELECT activity_id, 'return'::bench_activity_type FROM input_data");
    }

    query.push_str(
        "
            RETURNING id
        ),
        -- Lock and update devices in one step
        locked_devices AS (
            SELECT rd.device_id, d.quality
            FROM bench_receipts_devices rd
            JOIN bench_devices bd ON rd.device_id = bd.id
            JOIN device_data d ON rd.device_id = d.device_id
            WHERE rd.returned_receipt_id IS NULL
            AND bd.status = 'borrowing'::bench_device_status
            FOR UPDATE SKIP LOCKED
        ),
        -- Update receipts_devices
        updated_receipts AS (
            UPDATE bench_receipts_devices
            SET
                returned_receipt_id = (SELECT id FROM receipt),
                return_id = (SELECT id FROM activity),
                after_quality = ld.quality
            FROM
                locked_devices ld
            WHERE
                bench_receipts_devices.device_id = ld.device_id
                AND bench_receipts_devices.returned_receipt_id IS NULL
            RETURNING bench_receipts_devices.device_id
        ),
        -- Update device status
        updated_devices AS (
            UPDATE bench_devices
            SET status = ld.quality
            FROM
                locked_devices ld
            WHERE
                bench_devices.id = ld.device_id
            RETURNING bench_devices.id
        )
        -- Return the count and receipt ID
        SELECT
            (SELECT id FROM receipt) as receipt_id,
            COUNT(*) as processed_count
        FROM updated_devices
    ",
    );

    if returner_id.is_empty() || return_checker_id.is_empty() || returned_lab_id.is_empty() {
        return Ok(
            json!({ "error": "Missing required parameters: returnerId, returnedCheckerId, or returnedLabId" }),
        );
    }

    let returner_uuid = Uuid::parse_str(returner_id)?;
    let checker_uuid = Uuid::parse_str(return_checker_id)?;
    let lab_uuid = Uuid::parse_str(returned_lab_id)?;

    let result = if let Some(note_text) = note {
        client
            .query_one(
                &query,
                &[
                    &receipt_uuid,
                    &activity_uuid,
                    &returner_uuid,
                    &checker_uuid,
                    &lab_uuid,
                    &devices_json,
                    &note_text,
                ],
            )
            .await?
    } else {
        client
            .query_one(
                &query,
                &[
                    &receipt_uuid,
                    &activity_uuid,
                    &returner_uuid,
                    &checker_uuid,
                    &lab_uuid,
                    &devices_json,
                ],
            )
            .await?
    };

    let processed_count: i64 = result.get("processed_count");

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

    let query = format!(
        "SELECT id::text FROM bench_devices
         WHERE status = 'healthy'::bench_device_status
         ORDER BY random()
         LIMIT {}",
        count * 2
    );

    let rows = client.query(&query, &[]).await?;
    let mut device_ids: Vec<String> = rows.iter().map(|row| row.get(0)).collect();

    if device_ids.is_empty() {
        return Err("No healthy devices available".into());
    }

    if device_ids.len() > count {
        use rand::seq::SliceRandom;
        device_ids.shuffle(&mut rng());
        device_ids.truncate(count);
    }

    Ok(device_ids)
}

async fn get_random_user_ids(app_state: &AppState, count: usize) -> Result<Vec<String>, StdError> {
    let client = app_state.db.get_client().await?;

    let query = format!(
        "SELECT id::text FROM bench_users
         ORDER BY random()
         LIMIT {}",
        count * 2
    );

    let rows = client.query(&query, &[]).await?;
    let mut user_ids: Vec<String> = rows.iter().map(|row| row.get(0)).collect();

    if user_ids.is_empty() {
        return Err("No users available".into());
    }

    if user_ids.len() > count {
        use rand::seq::SliceRandom;
        user_ids.shuffle(&mut rng());
        user_ids.truncate(count);
    }

    Ok(user_ids)
}

async fn get_random_lab_id(app_state: &AppState) -> Result<String, StdError> {
    let client = app_state.db.get_client().await?;

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

    let query = format!(
        "SELECT device_id::text FROM bench_receipts_devices
         WHERE return_id IS NULL
         ORDER BY random()
         LIMIT {}",
        count * 2
    );

    let rows = client.query(&query, &[]).await?;
    let mut device_ids: Vec<String> = rows.iter().map(|row| row.get(0)).collect();

    if device_ids.is_empty() {
        return Err("No borrowing devices available".into());
    }

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
                                        break Err(e);
                                    }
                                }
                            }
                        }
                    }
                    "return_devices" => {
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

    println!("Cleaning up existing data...");
    match cleanup_test_tables(&app_state.db).await {
        Ok(_) => println!("Data cleanup complete"),
        Err(e) => return Err(format!("Error cleaning up test tables: {}", e).into()),
    }

    populate_large_test_data(
        &app_state.db,
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

    println!("Cleaning up test data...");
    match cleanup_test_tables(&app_state.db).await {
        Ok(_) => println!("Test data cleanup complete"),
        Err(e) => return Err(format!("Error cleaning up test tables: {}", e).into()),
    }

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
