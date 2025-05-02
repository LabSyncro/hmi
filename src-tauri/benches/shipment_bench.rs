use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use hmi_lib::commands::db_commands::{InsertParams, QueryParams};

mod common;
use common::{ensure_bench_env, AppState};

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
            format!(
                "ORDER BY {} {}",
                field,
                if *is_asc { "ASC" } else { "DESC" }
            )
        }
        _ => "ORDER BY created_at DESC".to_string(),
    };

    let sql = format!(
        "WITH shipment_data AS (
            SELECT
                s.id,
                s.status,
                s.shipper_id,
                s.from_lab_id,
                s.to_lab_id,
                a.created_at,
                s.finished_at,
                a.id as activity_id,
                a.note
            FROM
                bench_shipments s
                JOIN bench_activities a ON s.id = a.id
        ),
        user_data AS (
            SELECT
                u.id,
                u.name
            FROM
                bench_users u
        ),
        lab_data AS (
            SELECT
                l.id,
                l.name
            FROM
                bench_labs l
        ),
        device_counts AS (
            SELECT
                shipment_id,
                COUNT(id) as device_count
            FROM
                bench_shipments_devices
            GROUP BY
                shipment_id
        )
        SELECT
            sd.id::text,
            sd.status,
            u.name as shipper_name,
            sd.shipper_id::text,
            l_from.name as from_lab_name,
            sd.from_lab_id::text,
            l_to.name as to_lab_name,
            sd.to_lab_id::text,
            sd.created_at,
            sd.finished_at,
            COALESCE(dc.device_count, 0) as device_count,
            sd.activity_id::text,
            sd.note
        FROM
            shipment_data sd
            JOIN user_data u ON sd.shipper_id = u.id
            JOIN lab_data l_from ON sd.from_lab_id = l_from.id
            JOIN lab_data l_to ON sd.to_lab_id = l_to.id
            LEFT JOIN device_counts dc ON sd.id = dc.shipment_id
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

    let query = "
        WITH shipment_data AS (
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
            WHERE s.id = $1
        ),
        device_data AS (
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
            WHERE sd.shipment_id = $1
        )
        SELECT
            json_build_object(
                'shipment', (SELECT row_to_json(s) FROM shipment_data s),
                'devices', (SELECT json_agg(d) FROM device_data d)
            ) as result";

    let row = match client.query_opt(query, &[&shipment_uuid]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(json!({ "error": "Shipment not found" })),
        Err(err) => return Err(Box::new(err)),
    };

    let result: serde_json::Value = row.get("result");
    let shipment = &result["shipment"];
    let devices = &result["devices"];

    let created_at =
        chrono::DateTime::parse_from_rfc3339(shipment["created_at"].as_str().unwrap_or(""))
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
            .with_timezone(&chrono::Utc);

    let finished_at = shipment["finished_at"]
        .as_str()
        .map(|dt_str| {
            chrono::DateTime::parse_from_rfc3339(dt_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok()
        })
        .flatten();

    Ok(json!({
        "id": shipment["id"],
        "status": shipment["status"],
        "shipperName": shipment["shipper_name"],
        "shipperId": shipment["shipper_id"],
        "fromLabName": shipment["from_lab_name"],
        "fromLabId": shipment["from_lab_id"],
        "toLabName": shipment["to_lab_name"],
        "toLabId": shipment["to_lab_id"],
        "createdAt": created_at.to_rfc3339(),
        "finishedAt": finished_at.map(|dt| dt.to_rfc3339()),
        "note": shipment["note"],
        "activityId": shipment["activity_id"],
        "devices": devices
    }))
}

async fn create_shipment(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let from_lab_id = params
        .value
        .get("fromLabId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let to_lab_id = params
        .value
        .get("toLabId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let shipper_id = params
        .value
        .get("shipperId")
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
        return Ok(json!({ "error": "No devices specified for shipment" }));
    }

    if from_lab_id.is_empty() || to_lab_id.is_empty() || shipper_id.is_empty() {
        return Ok(
            json!({ "error": "Missing required parameters: fromLabId, toLabId, or shipperId" }),
        );
    }

    if from_lab_id == to_lab_id {
        return Ok(json!({ "error": "Source and destination labs must be different" }));
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
        return Ok(json!({ "error": "No valid devices specified for shipment" }));
    }

    let device_values_str = device_values.join(", ");

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
        ),
        device_data(device_id, prev_status, after_status) AS (
            VALUES {}
        ),
        insert_devices AS (
            INSERT INTO bench_shipments_devices
            (shipment_id, device_id, prev_status, after_status)
            SELECT
                (SELECT id FROM new_shipment),
                dd.device_id::uuid,
                dd.prev_status,
                dd.after_status
            FROM device_data dd
            RETURNING device_id
        ),
        update_devices AS (
            UPDATE bench_devices
            SET status = 'shipping'::bench_device_status
            WHERE id IN (SELECT device_id FROM insert_devices)
            RETURNING id
        )
        SELECT id::text FROM new_shipment",
        note_part, from_lab_id, to_lab_id, shipper_id, device_values_str
    );

    let row = client.query_one(&query, &[]).await?;
    let shipment_id = row.get::<_, String>(0);

    Ok(json!({
        "success": true,
        "id": shipment_id
    }))
}

async fn complete_shipment(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let shipment_id = params
        .value
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if shipment_id.is_empty() {
        return Ok(json!({ "error": "Shipment ID is required" }));
    }

    let devices_vec = params
        .value
        .get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;

    if devices.is_empty() {
        let query = format!(
            "WITH shipment_info AS (
                SELECT to_lab_id
                FROM bench_shipments
                WHERE id = '{}'::uuid
            ),
            shipment_devices AS (
                SELECT device_id
                FROM bench_shipments_devices
                WHERE shipment_id = '{}'::uuid
            ),
            update_devices AS (
                UPDATE bench_devices
                SET status = 'healthy'::bench_device_status,
                    lab_id = (SELECT to_lab_id FROM shipment_info)
                WHERE id IN (SELECT device_id FROM shipment_devices)
                RETURNING id
            ),
            update_shipment AS (
                UPDATE bench_shipments
                SET status = 'completed', finished_at = CURRENT_TIMESTAMP
                WHERE id = '{}'::uuid
                RETURNING id
            )
            SELECT id::text FROM update_shipment",
            shipment_id, shipment_id, shipment_id
        );

        let row = match client.query_opt(&query, &[]).await? {
            Some(row) => row,
            None => return Ok(json!({ "error": "Shipment not found or could not be updated" })),
        };

        let result_id = row.get::<_, String>(0);

        return Ok(json!({
            "success": true,
            "id": result_id
        }));
    } else {
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
            return Ok(json!({ "error": "No valid devices specified for shipment completion" }));
        }

        let device_values_str = device_values.join(", ");

        let query = format!(
            "WITH shipment_info AS (
                SELECT to_lab_id
                FROM bench_shipments
                WHERE id = '{}'::uuid
            ),
            device_data(device_id, after_status) AS (
                VALUES {}
            ),
            update_shipment_devices AS (
                UPDATE bench_shipments_devices
                SET after_status = dd.after_status
                FROM device_data dd
                WHERE bench_shipments_devices.device_id = dd.device_id::uuid
                    AND bench_shipments_devices.shipment_id = '{}'::uuid
                RETURNING bench_shipments_devices.device_id
            ),
            update_devices AS (
                UPDATE bench_devices
                SET status = dd.after_status,
                    lab_id = (SELECT to_lab_id FROM shipment_info)
                FROM device_data dd
                WHERE bench_devices.id = dd.device_id::uuid
                RETURNING id
            ),
            update_shipment AS (
                UPDATE bench_shipments
                SET status = 'completed', finished_at = CURRENT_TIMESTAMP
                WHERE id = '{}'::uuid
                RETURNING id
            )
            SELECT id::text FROM update_shipment",
            shipment_id, device_values_str, shipment_id, shipment_id
        );

        let row = match client.query_opt(&query, &[]).await? {
            Some(row) => row,
            None => return Ok(json!({ "error": "Shipment not found or could not be updated" })),
        };

        let result_id = row.get::<_, String>(0);

        return Ok(json!({
            "success": true,
            "id": result_id
        }));
    }
}

fn benchmark_shipment(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for shipment benchmarks");

    let app_state = rt.block_on(async { ensure_bench_env().await });

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
                            &format!("DEV-SHIP-{}", device_ids.len() + 1),
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

    // Get real lab and user IDs from the database
    let (from_lab_id, to_lab_id, shipper_id) = rt.block_on(async {
        let client = app_state
            .db
            .get_client()
            .await
            .expect("Failed to get client");

        // Get two different lab IDs
        let labs = client
            .query("SELECT id::text FROM bench_labs ORDER BY id LIMIT 2", &[])
            .await
            .unwrap_or_default();

        let from_lab_id = if labs.len() > 0 {
            labs[0].get::<_, String>(0)
        } else {
            "".to_string()
        };

        let to_lab_id = if labs.len() > 1 {
            labs[1].get::<_, String>(0)
        } else if labs.len() > 0 {
            // If we only have one lab, create a new one
            let new_lab_id = Uuid::new_v4();
            let _ = client.execute(
                "INSERT INTO bench_labs (id, name, room, branch) VALUES ($1, 'Benchmark Lab', 'Room 101', 'Branch 1')",
                &[&new_lab_id]
            ).await;
            new_lab_id.to_string()
        } else {
            "".to_string()
        };

        let shipper_id = client
            .query_one("SELECT id::text FROM bench_users LIMIT 1", &[])
            .await
            .map(|row| row.get::<_, String>(0))
            .unwrap_or_else(|_| "".to_string());

        (from_lab_id, to_lab_id, shipper_id)
    });

    if from_lab_id.is_empty() || to_lab_id.is_empty() || shipper_id.is_empty() {
        println!("Warning: Could not find labs or shipper for benchmarking");
    }

    let shipment_id = rt.block_on(async {
        let create_params = InsertParams {
            table: "bench_shipments".to_string(),
            value: json!({
                "fromLabId": from_lab_id,
                "toLabId": to_lab_id,
                "shipperId": shipper_id,
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
        order_by: Some(vec![("created_at".to_string(), false)]),
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
            "fromLabId": from_lab_id,
            "toLabId": to_lab_id,
            "shipperId": shipper_id,
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

    println!("Benchmark completed. Database state preserved for future runs.");
}

criterion_group!(benches, benchmark_shipment);
criterion_main!(benches);
