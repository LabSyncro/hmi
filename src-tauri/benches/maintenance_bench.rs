use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use hmi_lib::commands::db_commands::{InsertParams, QueryParams};

mod common;
use common::{ensure_bench_env, AppState};

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
            format!(
                "ORDER BY {} {}",
                field,
                if *is_asc { "ASC" } else { "DESC" }
            )
        }
        _ => "ORDER BY created_at DESC".to_string(),
    };

    let sql = format!(
        "WITH maintenance_data AS (
            SELECT
                m.id,
                m.status,
                u.name as technician_name,
                u.id as technician_id,
                l.name as lab_name,
                l.id as lab_id,
                a.created_at,
                m.finished_at,
                a.id as activity_id,
                a.note
            FROM
                bench_maintenance m
                JOIN bench_activities a ON m.id = a.id
                JOIN bench_users u ON m.technician_id = u.id
                JOIN bench_labs l ON m.lab_id = l.id
        ),
        device_counts AS (
            SELECT
                maintenance_id,
                COUNT(id) as device_count
            FROM
                bench_maintenance_devices
            GROUP BY
                maintenance_id
        )
        SELECT
            md.id::text,
            md.status,
            md.technician_name,
            md.technician_id::text,
            md.lab_name,
            md.lab_id::text,
            md.created_at,
            md.finished_at,
            COALESCE(dc.device_count, 0) as device_count,
            md.activity_id::text,
            md.note
        FROM
            maintenance_data md
            LEFT JOIN device_counts dc ON md.id = dc.maintenance_id
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

    let query = "
        WITH maintenance_data AS (
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
            WHERE m.id = $1
        ),
        device_data AS (
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
            WHERE md.maintenance_id = $1
        )
        SELECT
            json_build_object(
                'maintenance', (SELECT row_to_json(m) FROM maintenance_data m),
                'devices', (SELECT json_agg(d) FROM device_data d)
            ) as result";

    let row = match client.query_opt(query, &[&maintenance_uuid]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(json!({ "error": "Maintenance record not found" })),
        Err(err) => return Err(Box::new(err)),
    };

    let result: serde_json::Value = row.get("result");
    let maintenance = &result["maintenance"];
    let devices = &result["devices"];

    let created_at =
        chrono::DateTime::parse_from_rfc3339(maintenance["created_at"].as_str().unwrap_or(""))
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
            .with_timezone(&chrono::Utc);

    let finished_at = maintenance["finished_at"]
        .as_str()
        .map(|dt_str| {
            chrono::DateTime::parse_from_rfc3339(dt_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok()
        })
        .flatten();

    Ok(json!({
        "id": maintenance["id"],
        "status": maintenance["status"],
        "technicianName": maintenance["technician_name"],
        "technicianId": maintenance["technician_id"],
        "labName": maintenance["lab_name"],
        "labId": maintenance["lab_id"],
        "createdAt": created_at.to_rfc3339(),
        "finishedAt": finished_at.map(|dt| dt.to_rfc3339()),
        "note": maintenance["note"],
        "activityId": maintenance["activity_id"],
        "devices": devices
    }))
}

async fn create_maintenance(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let lab_id = params
        .value
        .get("labId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let technician_id = params
        .value
        .get("technicianId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let note = params.value.get("note").and_then(|v| v.as_str());

    let devices_vec = params
        .value
        .get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;

    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for maintenance" }));
    }

    let note_part = match note {
        Some(n) => format!("'{}'", n.replace("'", "''")),
        None => "NULL".to_string(),
    };

    let mut device_values = Vec::new();
    for device in devices {
        let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if device_id.is_empty() {
            continue;
        }

        let prev_status = device
            .get("prevStatus")
            .and_then(|v| v.as_str())
            .unwrap_or("healthy");
        let after_status = device
            .get("afterStatus")
            .and_then(|v| v.as_str())
            .unwrap_or("healthy");

        device_values.push(format!(
            "('{}', '{}'::bench_device_status, '{}'::bench_device_status)",
            device_id, prev_status, after_status
        ));
    }

    if device_values.is_empty() {
        return Ok(json!({ "error": "No valid devices specified for maintenance" }));
    }

    let device_values_str = device_values.join(", ");

    if lab_id.is_empty() || technician_id.is_empty() {
        return Ok(json!({ "error": "Missing required parameters: labId or technicianId" }));
    }

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
        ),
        device_data(device_id, prev_status, after_status) AS (
            VALUES {}
        ),
        insert_devices AS (
            INSERT INTO bench_maintenance_devices
            (maintenance_id, device_id, prev_status, after_status)
            SELECT
                (SELECT id FROM new_maintenance),
                dd.device_id,
                dd.prev_status,
                dd.after_status
            FROM device_data dd
            RETURNING device_id
        ),
        update_devices AS (
            UPDATE bench_devices
            SET status = 'maintaining'::bench_device_status
            WHERE id IN (SELECT device_id FROM insert_devices)
            RETURNING id
        )
        SELECT id::text FROM new_maintenance",
        note_part, lab_id, technician_id, device_values_str
    );

    let row = client.query_one(&query, &[]).await?;
    let maintenance_id = row.get::<_, String>(0);

    Ok(json!({
        "success": true,
        "id": maintenance_id
    }))
}

async fn finish_maintenance(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let maintenance_id = params
        .value
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if maintenance_id.is_empty() {
        return Ok(json!({ "error": "Maintenance ID is required" }));
    }

    let devices_vec = params
        .value
        .get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;

    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for maintenance completion" }));
    }

    let mut device_values = Vec::new();
    for device in devices {
        let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if device_id.is_empty() {
            continue;
        }

        let after_status = device
            .get("afterStatus")
            .and_then(|v| v.as_str())
            .unwrap_or("healthy");

        device_values.push(format!(
            "('{}', '{}'::bench_device_status)",
            device_id, after_status
        ));
    }

    if device_values.is_empty() {
        return Ok(json!({ "error": "No valid devices specified for maintenance completion" }));
    }

    let device_values_str = device_values.join(", ");

    let query = format!(
        "WITH update_maintenance AS (
            UPDATE bench_maintenance
            SET status = 'completed', finished_at = CURRENT_TIMESTAMP
            WHERE id = '{}'
            RETURNING id
        ),
        device_data(device_id, after_status) AS (
            VALUES {}
        ),
        update_maintenance_devices AS (
            UPDATE bench_maintenance_devices
            SET after_status = dd.after_status
            FROM device_data dd
            WHERE bench_maintenance_devices.device_id = dd.device_id
                AND bench_maintenance_devices.maintenance_id = '{}'
            RETURNING bench_maintenance_devices.device_id
        ),
        update_devices AS (
            UPDATE bench_devices
            SET status = dd.after_status
            FROM device_data dd
            WHERE bench_devices.id = dd.device_id
            RETURNING id
        )
        SELECT id FROM update_maintenance",
        maintenance_id, device_values_str, maintenance_id
    );

    let row = client.query_one(&query, &[]).await?;
    let result_id = row.get::<_, Uuid>(0).to_string();

    Ok(json!({
        "success": true,
        "id": result_id
    }))
}

fn benchmark_maintenance(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for maintenance benchmarks");

    let app_state = rt.block_on(async {
        // Use the ensure_bench_env function which handles setup and data population
        ensure_bench_env().await
    });

    let real_device_ids = rt.block_on(async {
        let client = app_state
            .db
            .get_client()
            .await
            .expect("Failed to get client");
        let query = "SELECT id::text FROM bench_devices WHERE status = 'healthy' LIMIT 3";
        let rows = client
            .query(query, &[])
            .await
            .expect("Failed to fetch device IDs");

        if rows.len() < 3 {
            let mut device_ids = rows
                .iter()
                .map(|row| row.get::<_, String>(0))
                .collect::<Vec<_>>();

            let kind_id = match client
                .query_opt("SELECT id FROM bench_device_kinds LIMIT 1", &[])
                .await
            {
                Ok(Some(row)) => row.get::<_, Uuid>(0),
                _ => Uuid::new_v4(),
            };

            let lab_id = match client
                .query_opt("SELECT id FROM bench_labs LIMIT 1", &[])
                .await
            {
                Ok(Some(row)) => row.get::<_, Uuid>(0),
                _ => Uuid::new_v4(),
            };

            while device_ids.len() < 3 {
                let device_id = Uuid::new_v4();
                let insert_query = "INSERT INTO bench_devices (id, full_id, kind, lab_id, status)
                    VALUES ($1, $2, $3, $4, 'healthy'::bench_device_status)
                    RETURNING id::text";

                match client
                    .query_one(
                        insert_query,
                        &[
                            &device_id,
                            &format!("DEV-TEST-{}", device_ids.len() + 1),
                            &kind_id,
                            &lab_id,
                        ],
                    )
                    .await
                {
                    Ok(row) => device_ids.push(row.get::<_, String>(0)),
                    Err(_) => device_ids.push(Uuid::new_v4().to_string()),
                }
            }

            device_ids
        } else {
            rows.iter()
                .map(|row| row.get::<_, String>(0))
                .collect::<Vec<_>>()
        }
    });

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

    // We no longer clean up tables to preserve the database state
    // and avoid recreating data for each benchmark run
    println!("Benchmark completed. Database state preserved for future runs.");
}

criterion_group!(benches, benchmark_maintenance);
criterion_main!(benches);
