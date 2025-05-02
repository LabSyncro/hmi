use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use hmi_lib::commands::db_commands::QueryParams;

mod common;
use common::{ensure_bench_env, AppState};

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
            format!(
                "ORDER BY {} {}",
                field,
                if *is_asc { "ASC" } else { "DESC" }
            )
        }
        _ => "ORDER BY d.full_id ASC".to_string(),
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
        "brand": row.get::<_, Option<String>>(4),
        "manufacturer": row.get::<_, Option<String>>(5),
        "description": row.get::<_, Option<String>>(6),
        "labName": row.get::<_, String>(7),
        "labId": row.get::<_, String>(8),
        "status": row.get::<_, String>(9),
        "categoryName": row.try_get::<_, Option<String>>(10).unwrap_or(None).unwrap_or_default(),
        "categoryId": row.try_get::<_, Option<String>>(11).unwrap_or(None).unwrap_or_default()
    }))
}

async fn get_device_receipt_by_id(
    app_state: &AppState,
    device_id: &str,
    lab_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    if device_id.is_empty() || lab_id.is_empty() {
        return Ok(json!({ "error": "Missing device ID or lab ID" }));
    }

    let device_uuid = match Uuid::parse_str(device_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid device ID format" })),
    };

    let lab_uuid = match Uuid::parse_str(lab_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid lab ID format" })),
    };

    let sql = "
        SELECT
            d.id,
            d.full_id,
            d.status::text,
            d.kind,
            d.lab_id,
            dk.image,
            dk.unit,
            dk.name AS device_name,
            dk.allowed_borrow_roles,
            dk.allowed_view_roles,
            dk.brand,
            dk.manufacturer,
            dk.description,
            dk.is_borrowable_lab_only,
            c.name AS category_name,
            l.room,
            l.branch,
            r.id AS receipt_id,
            a.created_at AS borrowed_at,
            rd.expected_returned_at,
            rd.prev_quality::text,
            bl.name AS borrowed_lab,
            rl.name AS expected_return_lab,
            actor.id AS borrower_id,
            actor.name AS borrower_name,
            actor.image AS borrower_image
        FROM
            bench_devices d
            LEFT JOIN bench_device_kinds dk ON d.kind = dk.id
            LEFT JOIN bench_labs l ON d.lab_id = l.id
            LEFT JOIN bench_categories c ON dk.category_id = c.id
            LEFT JOIN bench_receipts_devices rd ON d.id = rd.device_id AND rd.returned_receipt_id IS NULL
            LEFT JOIN bench_receipts r ON rd.borrowed_receipt_id = r.id
            LEFT JOIN bench_users actor ON r.actor_id = actor.id
            LEFT JOIN bench_labs bl ON r.lab_id = bl.id
            LEFT JOIN bench_labs rl ON rd.expected_returned_lab_id = rl.id
            LEFT JOIN bench_activities a ON rd.borrow_id = a.id
        WHERE
            d.id = $1
            AND d.deleted_at IS NULL
    ";

    let row = match client.query_opt(sql, &[&device_uuid]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(json!({ "error": "Device not found" })),
        Err(err) => return Err(Box::new(err)),
    };

    let device_lab_id: Uuid = row.get("lab_id");
    if device_lab_id != lab_uuid {
        return Ok(json!({ "error": "Device does not belong to this lab" }));
    }

    let borrower_id: Option<Uuid> = row.try_get("borrower_id").ok();
    let borrowed_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("borrowed_at").ok();

    Ok(json!({
        "fullId": row.get::<_, String>("full_id"),
        "status": row.get::<_, String>("status"),
        "prevQuality": row.try_get::<_, String>("prev_quality").ok(),
        "image": row.get::<_, serde_json::Value>("image"),
        "unit": row.try_get::<_, String>("unit").unwrap_or_default(),
        "deviceName": row.get::<_, String>("device_name"),
        "allowedBorrowRoles": row.get::<_, Vec<String>>("allowed_borrow_roles"),
        "allowedViewRoles": row.get::<_, Vec<String>>("allowed_view_roles"),
        "brand": row.get::<_, Option<String>>("brand"),
        "manufacturer": row.get::<_, Option<String>>("manufacturer"),
        "description": row.get::<_, Option<String>>("description"),
        "isBorrowableLabOnly": row.get::<_, bool>("is_borrowable_lab_only"),
        "categoryName": row.get::<_, String>("category_name"),
        "labRoom": row.get::<_, String>("room"),
        "labBranch": row.get::<_, String>("branch"),
        "kind": row.get::<_, Uuid>("kind").to_string(),
        "receiptId": row.try_get::<_, Uuid>("receipt_id").ok().map(|id| id.to_string()),
        "borrower": borrower_id.map(|id| {
            json!({
                "id": id.to_string(),
                "name": row.get::<_, String>("borrower_name"),
                "image": row.get::<_, serde_json::Value>("borrower_image")
            })
        }),
        "borrowedAt": borrowed_at.map(|dt| dt.to_rfc3339()),
        "expectedReturnedAt": row.try_get::<_, chrono::DateTime<chrono::Utc>>("expected_returned_at")
            .ok()
            .map(|dt| dt.to_rfc3339()),
        "borrowedLab": row.try_get::<_, String>("borrowed_lab").ok(),
        "expectedReturnLab": row.try_get::<_, String>("expected_return_lab").ok()
    }))
}

async fn get_device_audit_by_id(
    app_state: &AppState,
    device_id: &str,
    lab_id: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    if device_id.is_empty() || lab_id.is_empty() {
        return Ok(json!({ "error": "Missing device ID or lab ID" }));
    }

    let device_uuid = match Uuid::parse_str(device_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid device ID format" })),
    };

    let lab_uuid = match Uuid::parse_str(lab_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid lab ID format" })),
    };

    let sql = "
        WITH active_assessment AS (
            SELECT
                ia.id
            FROM bench_inventory_assessments ia
            WHERE ia.status = 'assessing'
                AND ia.finished_at IS NULL
                AND ia.lab_id = $2
            LIMIT 1
        )
        SELECT
            d.id,
            d.full_id,
            CASE
                WHEN d.status = 'assessing' THEN COALESCE(iad.prev_status, d.status)
                ELSE d.status
            END::text as status,
            iad.after_status::text as audit_condition,
            d.status::text as current_status,
            d.kind,
            d.lab_id,
            dk.image,
            dk.unit,
            dk.name AS device_name,
            dk.is_borrowable_lab_only,
            l.room,
            l.branch,
            c.name AS category_name
        FROM
            bench_devices d
            LEFT JOIN bench_device_kinds dk ON d.kind = dk.id
            LEFT JOIN bench_labs l ON d.lab_id = l.id
            LEFT JOIN bench_categories c ON dk.category_id = c.id
            LEFT JOIN active_assessment aa ON true
            LEFT JOIN bench_inventory_assessments_devices iad ON iad.device_id = d.id AND iad.assessing_id = aa.id
        WHERE
            d.id = $1
            AND d.deleted_at IS NULL
    ";

    let row = match client.query_opt(sql, &[&device_uuid, &lab_uuid]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(json!({ "error": "Device not found" })),
        Err(err) => return Err(Box::new(err)),
    };

    let device_lab_id: Uuid = row.get("lab_id");
    if device_lab_id != lab_uuid {
        return Ok(json!({ "error": "Device does not belong to this lab" }));
    }

    Ok(json!({
        "id": row.get::<_, Uuid>("id").to_string(),
        "fullId": row.get::<_, String>("full_id"),
        "status": row.get::<_, String>("status"),
        "currentStatus": row.get::<_, String>("current_status"),
        "auditCondition": row.try_get::<_, String>("audit_condition").unwrap_or_default(),
        "image": {
            "mainImage": row.get::<_, serde_json::Value>("image").get("mainImage").and_then(|v| v.as_str()).unwrap_or("")
        },
        "unit": row.try_get::<_, String>("unit").unwrap_or_default(),
        "deviceName": row.get::<_, String>("device_name"),
        "isBorrowableLabOnly": row.get::<_, bool>("is_borrowable_lab_only"),
        "labRoom": row.get::<_, String>("room"),
        "labBranch": row.get::<_, String>("branch"),
        "kind": row.get::<_, Uuid>("kind").to_string(),
        "categoryName": row.get::<_, String>("category_name")
    }))
}

async fn get_device_maintenance_by_id(
    app_state: &AppState,
    device_id: &str,
    lab_id: Option<&str>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    if device_id.is_empty() {
        return Ok(json!({ "error": "Missing device ID" }));
    }

    let device_uuid = match Uuid::parse_str(device_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid device ID format" })),
    };

    let lab_uuid = if let Some(lab_id) = lab_id {
        match Uuid::parse_str(lab_id) {
            Ok(uuid) => Some(uuid),
            Err(_) => return Ok(json!({ "error": "Invalid lab ID format" })),
        }
    } else {
        None
    };

    let sql = "
        SELECT
            d.id,
            CASE
                WHEN d.status = 'maintaining' THEN COALESCE(md.prev_status, d.status)
                ELSE d.status
            END::text as status,
            d.status::text as current_status,
            md.after_status::text as outcome,
            d.kind,
            d.lab_id,
            dk.image,
            dk.unit,
            dk.name AS device_name,
            dk.is_borrowable_lab_only,
            l.room,
            l.branch,
            m.id as maintenance_id,
            m.technician_id as technician_id,
            u.name as technician_name,
            a.note as notes,
            a.created_at
        FROM
            bench_devices d
            LEFT JOIN bench_device_kinds dk ON d.kind = dk.id
            LEFT JOIN bench_labs l ON d.lab_id = l.id
            LEFT JOIN bench_maintenance_devices md ON d.id = md.device_id
            LEFT JOIN bench_maintenance m ON md.maintenance_id = m.id
            LEFT JOIN bench_users u ON m.technician_id = u.id
            LEFT JOIN bench_activities a ON m.id = a.id
        WHERE
            d.id = $1
            AND d.deleted_at IS NULL
        ORDER BY
            a.created_at DESC
        LIMIT 1
    ";

    let row = match client.query_opt(sql, &[&device_uuid]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(json!({ "error": "Device not found" })),
        Err(err) => return Err(Box::new(err)),
    };

    if let Some(lab_uuid) = lab_uuid {
        let device_lab_id: Uuid = row.get("lab_id");
        if device_lab_id != lab_uuid {
            return Ok(json!({ "error": "Device does not belong to this lab" }));
        }
    }

    let created_at: Option<chrono::DateTime<chrono::Utc>> =
        row.try_get("created_at").unwrap_or(None);
    let room: String = row.get("room");
    let branch: String = row.get("branch");
    let location = if !room.is_empty() && !branch.is_empty() {
        format!("{}, {}", room, branch)
    } else {
        "".to_string()
    };

    Ok(json!({
        "id": row.get::<_, Uuid>("id").to_string(),
        "maintenanceId": row.try_get::<_, Uuid>("maintenance_id").ok().map(|id| id.to_string()),
        "technician": row.try_get::<_, Uuid>("technician_id").ok().map(|id| {
            json!({
                "id": id.to_string(),
                "name": row.try_get::<_, String>("technician_name").unwrap_or_default()
            })
        }),
        "status": row.get::<_, String>("status"),
        "currentStatus": row.get::<_, String>("current_status"),
        "outcome": row.try_get::<_, String>("outcome").unwrap_or_default(),
        "kind": row.get::<_, Uuid>("kind").to_string(),
        "deviceName": row.get::<_, String>("device_name"),
        "image": row.get::<_, serde_json::Value>("image"),
        "unit": row.try_get::<_, String>("unit").unwrap_or_default(),
        "isBorrowableLabOnly": row.get::<_, bool>("is_borrowable_lab_only"),
        "location": location,
        "notes": row.try_get::<_, String>("notes").ok(),
        "createdAt": created_at.map(|dt| dt.to_rfc3339())
    }))
}

async fn get_device_shipment_by_id(
    app_state: &AppState,
    device_id: &str,
    lab_id: Option<&str>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = app_state.db.get_client().await?;

    if device_id.is_empty() {
        return Ok(json!({ "error": "Missing device ID" }));
    }

    let device_uuid = match Uuid::parse_str(device_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(json!({ "error": "Invalid device ID format" })),
    };

    let lab_uuid = if let Some(lab_id) = lab_id {
        match Uuid::parse_str(lab_id) {
            Ok(uuid) => Some(uuid),
            Err(_) => return Ok(json!({ "error": "Invalid lab ID format" })),
        }
    } else {
        None
    };

    let sql = "
        SELECT
            d.id,
            d.full_id,
            d.status::text,
            d.kind,
            d.lab_id,
            dk.image,
            dk.unit,
            dk.name AS device_name,
            dk.allowed_borrow_roles,
            dk.allowed_view_roles,
            dk.brand,
            dk.manufacturer,
            dk.description,
            dk.is_borrowable_lab_only,
            c.name AS category_name,
            l.room,
            l.branch,
            sd.prev_status::text as prev_condition,
            sd.after_status::text as after_condition,
            s.id as shipment_id,
            s.status as shipment_status,
            s_start.name AS source_location,
            s_arrive.name AS destination_location,
            sender.name AS sender_name,
            receiver.name AS receiver_name
        FROM
            bench_devices d
            LEFT JOIN bench_device_kinds dk ON d.kind = dk.id
            LEFT JOIN bench_labs l ON d.lab_id = l.id
            LEFT JOIN bench_categories c ON dk.category_id = c.id
            LEFT JOIN bench_shipments_devices sd ON d.id = sd.device_id
            LEFT JOIN bench_shipments s ON sd.shipment_id = s.id
            LEFT JOIN bench_labs s_start ON s.from_lab_id = s_start.id
            LEFT JOIN bench_labs s_arrive ON s.to_lab_id = s_arrive.id
            LEFT JOIN bench_users sender ON s.shipper_id = sender.id
            LEFT JOIN bench_users receiver ON s.shipper_id = receiver.id
        WHERE
            d.id = $1
            AND d.deleted_at IS NULL
        ORDER BY
            s.id DESC
        LIMIT 1
    ";

    let row = match client.query_opt(sql, &[&device_uuid]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(json!({ "error": "Device not found" })),
        Err(err) => return Err(Box::new(err)),
    };

    if let Some(lab_uuid) = lab_uuid {
        let device_lab_id: Uuid = row.get("lab_id");
        if device_lab_id != lab_uuid {
            return Ok(json!({ "error": "Device does not belong to this lab" }));
        }
    }

    Ok(json!({
        "status": row.get::<_, String>("status"),
        "prevCondition": row.try_get::<_, String>("prev_condition").ok(),
        "afterCondition": row.try_get::<_, String>("after_condition").ok(),
        "shipmentId": row.try_get::<_, Uuid>("shipment_id").ok().map(|id| id.to_string()),
        "sourceLocation": row.try_get::<_, String>("source_location").ok(),
        "destinationLocation": row.try_get::<_, String>("destination_location").ok(),
        "senderName": row.try_get::<_, String>("sender_name").ok(),
        "receiverName": row.try_get::<_, String>("receiver_name").ok(),
        "image": row.get::<_, serde_json::Value>("image"),
        "unit": row.try_get::<_, String>("unit").unwrap_or_default(),
        "deviceName": row.get::<_, String>("device_name"),
        "isBorrowableLabOnly": row.get::<_, bool>("is_borrowable_lab_only")
    }))
}

async fn get_device_inventory_by_kind(
    app_state: &AppState,
    kind_id: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
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

async fn get_device_audit_history(
    app_state: &AppState,
    device_id: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
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
            ia.id AS \"assessmentId\",
            u.id AS \"auditor_id\",
            u.name AS \"auditor_name\",
            u.email AS \"auditor_email\",
            u.image AS \"auditor_avatar\",
            a.created_at AS \"auditDate\",
            ia.status AS \"auditResult\",
            a.note AS \"notes\",
            iad.prev_status AS \"prevStatus\",
            iad.after_status AS \"afterStatus\"
        FROM
            bench_devices d
            JOIN bench_inventory_assessments_devices iad ON d.id = iad.device_id
            JOIN bench_inventory_assessments ia ON iad.assessing_id = ia.id
            JOIN bench_activities a ON ia.id = a.id
            LEFT JOIN bench_users u ON ia.accountant_id = u.id
        WHERE
            d.id = $1
            AND d.deleted_at IS NULL
        ORDER BY
            a.created_at DESC
    ";

    let rows = client.query(sql, &[&device_uuid]).await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let results = rows
        .iter()
        .map(|row| {
            let audit_date: chrono::DateTime<chrono::Utc> = row.get("auditDate");

            json!({
                "id": row.get::<_, Uuid>("id").to_string(),
                "fullId": row.get::<_, String>("fullId"),
                "auditor": {
                    "id": row.get::<_, Uuid>("auditor_id").to_string(),
                    "name": row.get::<_, String>("auditor_name"),
                    "email": row.get::<_, String>("auditor_email"),
                    "avatar": row.get::<_, serde_json::Value>("auditor_avatar")
                },
                "auditDate": audit_date.to_rfc3339(),
                "auditResult": row.get::<_, String>("auditResult"),
                "notes": row.try_get::<_, String>("notes").ok(),
                "prevStatus": row.try_get::<_, String>("prevStatus").ok(),
                "afterStatus": row.try_get::<_, String>("afterStatus").ok()
            })
        })
        .collect();

    Ok(results)
}

async fn get_device_transport_history(
    app_state: &AppState,
    device_id: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
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
            s.id AS \"shipmentId\",
            start_lab.name AS \"sourceLocation\",
            arrive_lab.name AS \"destinationLocation\",
            a_from.created_at AS \"transportDate\",
            s.status,
            sender.id AS \"sender_id\",
            sender.name AS \"sender_name\",
            sender.email AS \"sender_email\",
            sender.image AS \"sender_avatar\",
            receiver.id AS \"receiver_id\",
            receiver.name AS \"receiver_name\",
            receiver.email AS \"receiver_email\",
            receiver.image AS \"receiver_avatar\"
        FROM
            bench_devices d
            JOIN bench_shipments_devices sd ON d.id = sd.device_id
            JOIN bench_shipments s ON sd.shipment_id = s.id
            JOIN bench_labs start_lab ON s.from_lab_id = start_lab.id
            JOIN bench_labs arrive_lab ON s.to_lab_id = arrive_lab.id
            LEFT JOIN bench_activities a_from ON s.id = a_from.id
            LEFT JOIN bench_users sender ON s.shipper_id = sender.id
            LEFT JOIN bench_users receiver ON s.shipper_id = receiver.id
        WHERE
            d.id = $1
            AND d.deleted_at IS NULL
        ORDER BY
            a_from.created_at DESC
    ";

    let rows = client.query(sql, &[&device_uuid]).await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let results = rows
        .iter()
        .map(|row| {
            let transport_date: chrono::DateTime<chrono::Utc> = row.get("transportDate");
            let sender_id: Option<Uuid> = row.try_get("sender_id").ok();
            let receiver_id: Option<Uuid> = row.try_get("receiver_id").ok();

            json!({
                "id": row.get::<_, Uuid>("id").to_string(),
                "fullId": row.get::<_, String>("fullId"),
                "sourceLocation": row.get::<_, String>("sourceLocation"),
                "destinationLocation": row.get::<_, String>("destinationLocation"),
                "transportDate": transport_date.to_rfc3339(),
                "status": row.get::<_, String>("status"),
                "sender": sender_id.map(|id| {
                    json!({
                        "id": id.to_string(),
                        "name": row.get::<_, String>("sender_name"),
                        "email": row.get::<_, String>("sender_email"),
                        "avatar": row.get::<_, serde_json::Value>("sender_avatar")
                    })
                }),
                "receiver": receiver_id.map(|id| {
                    json!({
                        "id": id.to_string(),
                        "name": row.get::<_, String>("receiver_name"),
                        "email": row.get::<_, String>("receiver_email"),
                        "avatar": row.get::<_, serde_json::Value>("receiver_avatar")
                    })
                })
            })
        })
        .collect();

    Ok(results)
}

async fn get_device_borrow_history(
    app_state: &AppState,
    device_id: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
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

async fn get_device_maintenance_history(
    app_state: &AppState,
    device_id: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
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
            a.note AS \"maintenanceReason\",
            m.status,
            u.id AS \"technician_id\",
            u.name AS \"technician_name\",
            u.email AS \"technician_email\",
            u.image AS \"technician_avatar\",
            a.created_at AS \"maintenanceStartDate\",
            m.finished_at AS \"finishedAt\"
        FROM
            bench_devices d
            JOIN bench_maintenance_devices md ON d.id = md.device_id
            JOIN bench_maintenance m ON md.maintenance_id = m.id
            JOIN bench_activities a ON m.id = a.id
            LEFT JOIN bench_users u ON m.technician_id = u.id
        WHERE
            d.id = $1
            AND d.deleted_at IS NULL
        ORDER BY
            a.created_at DESC
    ";

    let rows = client.query(sql, &[&device_uuid]).await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let results = rows.iter().map(|row| {
        let maintenance_start_date: chrono::DateTime<chrono::Utc> = row.get("maintenanceStartDate");
        let finished_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("finishedAt").ok();

        let expected_completion_date = finished_at.map(|dt| dt.to_rfc3339()).unwrap_or_else(|| {
            let expected_date = maintenance_start_date + chrono::Duration::days(14);
            expected_date.to_rfc3339()
        });

        json!({
            "id": row.get::<_, Uuid>("id").to_string(),
            "fullId": row.get::<_, String>("fullId"),
            "maintenanceReason": row.get::<_, Option<String>>("maintenanceReason").unwrap_or_else(|| "Bảo trì định kỳ".to_string()),
            "status": row.get::<_, String>("status"),
            "technician": {
                "id": row.get::<_, Uuid>("technician_id").to_string(),
                "name": row.get::<_, String>("technician_name"),
                "email": row.get::<_, String>("technician_email"),
                "avatar": row.get::<_, serde_json::Value>("technician_avatar")
            },
            "maintenanceStartDate": maintenance_start_date.to_rfc3339(),
            "finishedAt": finished_at.map(|dt| dt.to_rfc3339()),
            "expectedCompletionDate": expected_completion_date,
            "notes": row.try_get::<_, String>("maintenanceReason").ok()
        })
    }).collect();

    Ok(results)
}

fn benchmark_device(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime for device benchmarks");

    let app_state = rt.block_on(async { ensure_bench_env().await });

    let (test_device_id, kind_id, lab_id) = rt.block_on(async {
        let client = app_state.db.get_client().await.unwrap();

        let device_query = "SELECT d.id::text, d.kind, d.lab_id
                           FROM bench_devices d
                           WHERE d.status = 'healthy'
                           LIMIT 1";

        match client.query_opt(device_query, &[]).await {
            Ok(Some(row)) => {
                let device_id = row.get::<_, String>(0);
                let kind_id = row.get::<_, Uuid>(1);
                let lab_id = row.get::<_, Uuid>(2);
                (device_id, kind_id, lab_id)
            }
            _ => {
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

                let device_id = Uuid::new_v4();
                let insert_query = "INSERT INTO bench_devices (id, full_id, kind, lab_id, status)
                    VALUES ($1, $2, $3, $4, $5::bench_device_status)
                    RETURNING id::text";

                match client
                    .query_one(
                        insert_query,
                        &[
                            &device_id,
                            &format!("DEV-BENCH-{}", device_id.simple()),
                            &kind_id,
                            &lab_id,
                            &"healthy",
                        ],
                    )
                    .await
                {
                    Ok(row) => (row.get::<_, String>(0), kind_id, lab_id),
                    Err(_) => ("".to_string(), kind_id, lab_id),
                }
            }
        }
    });

    let mut group = c.benchmark_group("Device Operations");

    // Benchmark 1: Fetch devices
    let fetch_params = QueryParams {
        table: "bench_devices".to_string(),
        columns: None,
        conditions: None,
        order_by: Some(vec![("d.full_id".to_string(), true)]),
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

        // Benchmark for device borrow history
        group.bench_function(BenchmarkId::new("Fetch Device Borrow History", 1), |b| {
            b.to_async(&rt).iter(|| async {
                match get_device_borrow_history(&app_state, &test_device_id).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Device Borrow History benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        });

        // Benchmark for device maintenance history
        group.bench_function(
            BenchmarkId::new("Fetch Device Maintenance History", 1),
            |b| {
                b.to_async(&rt).iter(|| async {
                    match get_device_maintenance_history(&app_state, &test_device_id).await {
                        Ok(result) => {
                            black_box(result);
                        }
                        Err(err) => {
                            eprintln!(
                                "Error in Fetch Device Maintenance History benchmark: {}",
                                err
                            );
                            black_box(Vec::<serde_json::Value>::new());
                        }
                    }
                });
            },
        );

        // Benchmark for device receipt by ID
        let lab_id_str = lab_id.to_string();
        group.bench_function(BenchmarkId::new("Fetch Device Receipt", 1), |b| {
            b.to_async(&rt).iter(|| async {
                match get_device_receipt_by_id(&app_state, &test_device_id, &lab_id_str).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Device Receipt benchmark: {}", err);
                        black_box(json!({"error": err.to_string()}));
                    }
                }
            });
        });

        // Benchmark for device audit by ID
        group.bench_function(BenchmarkId::new("Fetch Device Audit", 1), |b| {
            b.to_async(&rt).iter(|| async {
                match get_device_audit_by_id(&app_state, &test_device_id, &lab_id_str).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Device Audit benchmark: {}", err);
                        black_box(json!({"error": err.to_string()}));
                    }
                }
            });
        });

        // Benchmark for device shipment by ID
        group.bench_function(BenchmarkId::new("Fetch Device Shipment", 1), |b| {
            b.to_async(&rt).iter(|| async {
                match get_device_shipment_by_id(&app_state, &test_device_id, Some(&lab_id_str))
                    .await
                {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Device Shipment benchmark: {}", err);
                        black_box(json!({"error": err.to_string()}));
                    }
                }
            });
        });

        // Benchmark for device maintenance by ID
        group.bench_function(BenchmarkId::new("Fetch Device Maintenance", 1), |b| {
            b.to_async(&rt).iter(|| async {
                match get_device_maintenance_by_id(&app_state, &test_device_id, Some(&lab_id_str))
                    .await
                {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Device Maintenance benchmark: {}", err);
                        black_box(json!({"error": err.to_string()}));
                    }
                }
            });
        });

        // Benchmark for device audit history
        group.bench_function(BenchmarkId::new("Fetch Device Audit History", 1), |b| {
            b.to_async(&rt).iter(|| async {
                match get_device_audit_history(&app_state, &test_device_id).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Device Audit History benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        });

        // Benchmark for device transport history
        group.bench_function(BenchmarkId::new("Fetch Device Transport History", 1), |b| {
            b.to_async(&rt).iter(|| async {
                match get_device_transport_history(&app_state, &test_device_id).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Device Transport History benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        });

        let kind_id_str = kind_id.to_string();

        // Benchmark for device inventory by kind
        group.bench_function(BenchmarkId::new("Fetch Device Inventory By Kind", 1), |b| {
            b.to_async(&rt).iter(|| async {
                match get_device_inventory_by_kind(&app_state, &kind_id_str).await {
                    Ok(result) => {
                        black_box(result);
                    }
                    Err(err) => {
                        eprintln!("Error in Fetch Device Inventory By Kind benchmark: {}", err);
                        black_box(Vec::<serde_json::Value>::new());
                    }
                }
            });
        });
    }

    group.finish();

    println!("Benchmark completed. Database state preserved for future runs.");
}

criterion_group!(benches, benchmark_device);
criterion_main!(benches);
