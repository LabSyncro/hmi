use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use hmi_lib::commands::db_commands::{InsertParams, QueryParams};

mod common;
use common::{ensure_bench_env, AppState};

async fn fetch_assessments(
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
        _ => "ORDER BY bench_activities.created_at DESC".to_string(),
    };

    let sql = format!(
        "WITH assessment_data AS (
            SELECT
                ia.id,
                ia.status,
                u.name as accountant_name,
                u.id as accountant_id,
                l.name as lab_name,
                l.id as lab_id,
                a.created_at as created_at,
                ia.finished_at,
                a.id as activity_id
            FROM
                bench_inventory_assessments ia
                JOIN bench_activities a ON ia.id = a.id
                JOIN bench_users u ON ia.accountant_id = u.id
                JOIN bench_labs l ON ia.lab_id = l.id
        ),
        device_counts AS (
            SELECT
                assessing_id,
                COUNT(id) as device_count
            FROM
                bench_inventory_assessments_devices
            GROUP BY
                assessing_id
        )
        SELECT
            ad.id::text,
            ad.status,
            ad.accountant_name,
            ad.accountant_id::text,
            ad.lab_name,
            ad.lab_id::text,
            ad.created_at,
            ad.finished_at,
            COALESCE(dc.device_count, 0) as device_count,
            ad.activity_id::text
        FROM
            assessment_data ad
            LEFT JOIN device_counts dc ON ad.id = dc.assessing_id
        {} LIMIT {} OFFSET {}",
        order_clause, limit, offset
    );

    let rows = client.query(&sql, &[]).await?;

    let results = rows
        .iter()
        .map(|row| {
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let finished_at: Option<chrono::DateTime<chrono::Utc>> = row.get("finished_at");

            json!({
                "id": row.get::<_, String>(0),
                "status": row.get::<_, String>(1),
                "accountantName": row.get::<_, String>(2),
                "accountantId": row.get::<_, String>(3),
                "labName": row.get::<_, String>(4),
                "labId": row.get::<_, String>(5),
                "createdAt": created_at.to_rfc3339(),
                "finishedAt": finished_at.map(|dt| dt.to_rfc3339()),
                "deviceCount": row.get::<_, i64>(8),
                "activityId": row.get::<_, String>(9)
            })
        })
        .collect();

    Ok(results)
}

async fn fetch_assessment_details(
    app_state: &AppState,
    params: &QueryParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let assessment_id = params
        .conditions
        .as_ref()
        .and_then(|conds| conds.first())
        .and_then(|(_, val)| val.as_str())
        .unwrap_or("");

    if assessment_id.is_empty() {
        return Ok(json!({ "error": "Assessment ID is required" }));
    }

    let assessment_uuid = match Uuid::parse_str(assessment_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid assessment ID format" })),
    };

    let query = "
        WITH assessment_data AS (
            SELECT
                ia.id::text,
                ia.status,
                u.name as accountant_name,
                u.id::text as accountant_id,
                l.name as lab_name,
                l.id::text as lab_id,
                a.created_at as created_at,
                ia.finished_at,
                a.note,
                a.id::text as activity_id
            FROM
                bench_inventory_assessments ia
                JOIN bench_activities a ON ia.id = a.id
                JOIN bench_users u ON ia.accountant_id = u.id
                JOIN bench_labs l ON ia.lab_id = l.id
            WHERE ia.id = $1
        ),
        device_data AS (
            SELECT
                iad.id::text,
                d.id::text as device_id,
                d.full_id as device_full_id,
                dk.name as device_kind_name,
                iad.prev_status::text,
                iad.after_status::text
            FROM
                bench_inventory_assessments_devices iad
                JOIN bench_devices d ON iad.device_id = d.id
                JOIN bench_device_kinds dk ON d.kind = dk.id
            WHERE iad.assessing_id = $1
        )
        SELECT
            json_build_object(
                'assessment', (SELECT row_to_json(a) FROM assessment_data a),
                'devices', (SELECT json_agg(d) FROM device_data d)
            ) as result";

    let row = match client.query_opt(query, &[&assessment_uuid]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(json!({ "error": "Assessment not found" })),
        Err(err) => return Err(Box::new(err)),
    };

    let result: serde_json::Value = row.get("result");
    let assessment = &result["assessment"];
    let devices = &result["devices"];

    let created_at =
        chrono::DateTime::parse_from_rfc3339(assessment["created_at"].as_str().unwrap_or(""))
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
            .with_timezone(&chrono::Utc);

    let finished_at = assessment["finished_at"]
        .as_str()
        .map(|dt_str| {
            chrono::DateTime::parse_from_rfc3339(dt_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok()
        })
        .flatten();

    Ok(json!({
        "id": assessment["id"],
        "status": assessment["status"],
        "accountantName": assessment["accountant_name"],
        "accountantId": assessment["accountant_id"],
        "labName": assessment["lab_name"],
        "labId": assessment["lab_id"],
        "createdAt": created_at.to_rfc3339(),
        "finishedAt": finished_at.map(|dt| dt.to_rfc3339()),
        "note": assessment["note"],
        "activityId": assessment["activity_id"],
        "devices": devices
    }))
}

async fn create_assessment(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let lab_id = match params.value.get("labId").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(json!({ "error": "Lab ID is required" })),
    };
    let accountant_id = match params.value.get("accountantId").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(json!({ "error": "Accountant ID is required" })),
    };
    let note = params.value.get("note").and_then(|v| v.as_str());

    let devices_vec = params
        .value
        .get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;

    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for assessment" }));
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

        let prev_status = match device.get("prevStatus").and_then(|v| v.as_str()) {
            Some(status) if !status.is_empty() => status,
            _ => continue,
        };
        let after_status = match device.get("afterStatus").and_then(|v| v.as_str()) {
            Some(status) if !status.is_empty() => status,
            _ => continue,
        };

        device_values.push(format!(
            "('{}', '{}'::bench_device_status, '{}'::bench_device_status)",
            device_id, prev_status, after_status
        ));
    }

    if device_values.is_empty() {
        return Ok(json!({ "error": "No valid devices specified for assessment" }));
    }

    let device_values_str = device_values.join(", ");

    let query = format!(
        "WITH new_activity AS (
            INSERT INTO bench_activities (id, type, note)
            VALUES (gen_random_uuid(), 'assessment'::bench_activity_type, {})
            RETURNING id
        ),
        new_assessment AS (
            INSERT INTO bench_inventory_assessments (id, lab_id, accountant_id, status)
            SELECT
                id,
                '{}'::uuid,
                '{}'::uuid,
                'assessing'
            FROM new_activity
            RETURNING id
        ),
        device_data(device_id, prev_status, after_status) AS (
            VALUES {}
        ),
        insert_devices AS (
            INSERT INTO bench_inventory_assessments_devices
            (assessing_id, device_id, prev_status, after_status)
            SELECT
                (SELECT id FROM new_assessment),
                dd.device_id,
                dd.prev_status,
                dd.after_status
            FROM device_data dd
            RETURNING device_id
        ),
        update_devices AS (
            UPDATE bench_devices
            SET status = 'assessing'::bench_device_status
            WHERE id IN (SELECT device_id FROM insert_devices)
            RETURNING id
        )
        SELECT id::text FROM new_assessment",
        note_part, lab_id, accountant_id, device_values_str
    );

    let row = client.query_one(&query, &[]).await?;
    let assessment_id = row.get::<_, String>(0);

    Ok(json!({
        "success": true,
        "id": assessment_id
    }))
}

async fn finish_assessment(
    app_state: &AppState,
    params: &InsertParams,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    let assessment_id = params
        .value
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if assessment_id.is_empty() {
        return Ok(json!({ "error": "Assessment ID is required" }));
    }

    let devices_vec = params
        .value
        .get("devices")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let devices = &devices_vec;

    if devices.is_empty() {
        return Ok(json!({ "error": "No devices specified for assessment completion" }));
    }

    let mut device_values = Vec::new();
    for device in devices {
        let device_id = device.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if device_id.is_empty() {
            continue;
        }

        let after_status = match device.get("afterStatus").and_then(|v| v.as_str()) {
            Some(status) if !status.is_empty() => status,
            _ => continue,
        };

        device_values.push(format!(
            "('{}', '{}'::bench_device_status)",
            device_id, after_status
        ));
    }

    if device_values.is_empty() {
        return Ok(json!({ "error": "No valid devices specified for assessment completion" }));
    }

    let device_values_str = device_values.join(", ");

    let query = format!(
        "WITH update_assessment AS (
            UPDATE bench_inventory_assessments
            SET status = 'completed', finished_at = CURRENT_TIMESTAMP
            WHERE id = '{}'
            RETURNING id
        ),
        device_data(device_id, after_status) AS (
            VALUES {}
        ),
        update_assessment_devices AS (
            UPDATE bench_inventory_assessments_devices
            SET after_status = dd.after_status
            FROM device_data dd
            WHERE bench_inventory_assessments_devices.device_id = dd.device_id
                AND bench_inventory_assessments_devices.assessing_id = '{}'
            RETURNING bench_inventory_assessments_devices.device_id
        ),
        update_devices AS (
            UPDATE bench_devices
            SET status = dd.after_status
            FROM device_data dd
            WHERE bench_devices.id = dd.device_id
            RETURNING id
        )
        SELECT id FROM update_assessment",
        assessment_id, device_values_str, assessment_id
    );

    let row = client.query_one(&query, &[]).await?;
    let result_id = row.get::<_, Uuid>(0).to_string();

    Ok(json!({
        "success": true,
        "id": result_id
    }))
}

fn benchmark_audit(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for audit benchmarks");

    let app_state = rt.block_on(ensure_bench_env());

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
            println!("Warning: Not enough healthy devices found for benchmarking");
            return Vec::new();
        }

        rows.iter()
            .map(|row| row.get::<_, String>(0))
            .collect::<Vec<_>>()
    });

    let assessment_id = if real_device_ids.len() >= 2 {
        rt.block_on(async {
            let client = app_state
                .db
                .get_client()
                .await
                .expect("Failed to get client");
            let lab_id = client
                .query_one("SELECT id::text FROM bench_labs LIMIT 1", &[])
                .await
                .map(|row| row.get::<_, String>(0))
                .unwrap_or_else(|_| "".to_string());

            let accountant_id = client
                .query_one("SELECT id::text FROM bench_users LIMIT 1", &[])
                .await
                .map(|row| row.get::<_, String>(0))
                .unwrap_or_else(|_| "".to_string());

            if lab_id.is_empty() || accountant_id.is_empty() {
                println!("Warning: Could not find lab or accountant for benchmarking");
                return "".to_string();
            }

            let create_params = InsertParams {
                table: "bench_inventory_assessments".to_string(),
                value: json!({
                    "labId": lab_id,
                    "accountantId": accountant_id,
                    "note": "Test assessment for benchmarking",
                    "devices": [
                        {
                            "id": real_device_ids[0],
                            "prevStatus": "healthy",
                            "afterStatus": "healthy"
                        },
                        {
                            "id": real_device_ids[1],
                            "prevStatus": "healthy",
                            "afterStatus": "broken"
                        }
                    ]
                }),
            };

            match create_assessment(&app_state, &create_params).await {
                Ok(result) => result
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                Err(e) => {
                    println!(
                        "Warning: Failed to create assessment for benchmarking: {}",
                        e
                    );
                    "".to_string()
                }
            }
        })
    } else {
        println!("Warning: Not enough device IDs for assessment benchmarking");
        "".to_string()
    };

    let mut group = c.benchmark_group("Audit Operations");

    // Benchmark 1: Fetch assessments
    let fetch_params = QueryParams {
        table: "bench_inventory_assessments".to_string(),
        columns: None,
        conditions: None,
        order_by: Some(vec![("bench_activities.created_at".to_string(), false)]),
        limit: Some(10),
        offset: Some(0),
        joins: None,
    };

    group.bench_with_input(
        BenchmarkId::new("Fetch Assessments", 10),
        &fetch_params,
        |b, p| {
            b.to_async(&rt).iter(|| async {
                match fetch_assessments(&app_state, p).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Assessments benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        },
    );

    // Benchmark 2: Fetch assessment details
    if !assessment_id.is_empty() {
        let details_params = QueryParams {
            table: "bench_inventory_assessments".to_string(),
            columns: None,
            conditions: Some(vec![("id".to_string(), json!(assessment_id))]),
            order_by: None,
            limit: None,
            offset: None,
            joins: None,
        };

        group.bench_with_input(
            BenchmarkId::new("Fetch Assessment Details", 1),
            &details_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match fetch_assessment_details(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Fetch Assessment Details benchmark: {}", err);
                            black_box(json!({"error": err.to_string()}));
                        }
                    }
                });
            },
        );
    }

    // Benchmark 3: Create assessment - only run if we have real device IDs
    let _create_benchmark = if real_device_ids.len() >= 3 {
        let (lab_id, accountant_id) = rt.block_on(async {
            let client = app_state
                .db
                .get_client()
                .await
                .expect("Failed to get client");
            let lab_id = client
                .query_one("SELECT id::text FROM bench_labs LIMIT 1", &[])
                .await
                .map(|row| row.get::<_, String>(0))
                .unwrap_or_else(|_| "".to_string());

            let accountant_id = client
                .query_one("SELECT id::text FROM bench_users LIMIT 1", &[])
                .await
                .map(|row| row.get::<_, String>(0))
                .unwrap_or_else(|_| "".to_string());

            (lab_id, accountant_id)
        });

        if lab_id.is_empty() || accountant_id.is_empty() {
            println!("Warning: Could not find lab or accountant for create benchmark");
            false
        } else {
            let create_params = InsertParams {
                table: "bench_inventory_assessments".to_string(),
                value: json!({
                    "labId": lab_id,
                    "accountantId": accountant_id,
                    "note": "Benchmark test assessment",
                    "devices": [
                        {
                            "id": real_device_ids[0],
                            "prevStatus": "healthy",
                            "afterStatus": "healthy"
                        },
                        {
                            "id": real_device_ids[1],
                            "prevStatus": "healthy",
                            "afterStatus": "broken"
                        },
                        {
                            "id": real_device_ids[2],
                            "prevStatus": "healthy",
                            "afterStatus": "lost"
                        }
                    ]
                }),
            };

            group.bench_with_input(
                BenchmarkId::new("Create Assessment", 3),
                &create_params,
                |b, p| {
                    b.to_async(&rt).iter(|| async {
                        match create_assessment(&app_state, p).await {
                            Ok(result) => {
                                black_box(result);
                            }
                            Err(err) => {
                                eprintln!("Error in Create Assessment benchmark: {}", err);
                                black_box(json!({"error": err.to_string()}));
                            }
                        }
                    });
                },
            );

            true
        }
    } else {
        println!("Warning: Not enough device IDs for create assessment benchmark");
        false
    };

    // Benchmark 4: Finish assessment
    if !assessment_id.is_empty() && real_device_ids.len() >= 2 {
        let finish_params = InsertParams {
            table: "bench_inventory_assessments".to_string(),
            value: json!({
                "id": assessment_id,
                "devices": [
                    {
                        "id": real_device_ids[0],
                        "afterStatus": "healthy"
                    },
                    {
                        "id": real_device_ids[1],
                        "afterStatus": "broken"
                    }
                ]
            }),
        };

        group.bench_with_input(
            BenchmarkId::new("Finish Assessment", 2),
            &finish_params,
            |b, p| {
                b.to_async(&rt).iter(|| async {
                    match finish_assessment(&app_state, p).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!("Error in Finish Assessment benchmark: {}", err);
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

criterion_group!(benches, benchmark_audit);
criterion_main!(benches);
