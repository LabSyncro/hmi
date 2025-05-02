use dotenvy::dotenv;
use std::sync::Arc;
use tokio::sync::Mutex;

use hmi_lib::db::connection::Database;
use hmi_lib::db::schema::DatabaseSchema;

pub struct AppState {
    pub db: Database,
    #[allow(dead_code)]
    pub schema: Arc<Mutex<Option<DatabaseSchema>>>,
}

#[allow(dead_code)]
pub async fn ensure_bench_env() -> AppState {
    dotenv().ok();
    let db = Database::new()
        .await
        .expect("Failed to connect to test database for benchmark");

    let client = db.get_client().await.expect("Failed to get client");

    let tables_exist = client
        .query_one(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables
                WHERE table_name = 'bench_users'
            )",
            &[],
        )
        .await
        .map(|row| row.get::<_, bool>(0))
        .unwrap_or(false);

    if !tables_exist {
        println!("Tables don't exist. Setting up test tables...");
        setup_test_tables(&db)
            .await
            .expect("Failed to set up test tables");
    }

    let users_count = client
        .query_one("SELECT COUNT(*) FROM bench_users", &[])
        .await
        .map(|row| row.get::<_, i64>(0))
        .unwrap_or(0);

    let device_kinds_count = client
        .query_one("SELECT COUNT(*) FROM bench_device_kinds", &[])
        .await
        .map(|row| row.get::<_, i64>(0))
        .unwrap_or(0);

    let devices_count = client
        .query_one("SELECT COUNT(*) FROM bench_devices", &[])
        .await
        .map(|row| row.get::<_, i64>(0))
        .unwrap_or(0);

    if users_count != 1000 || device_kinds_count != 2000 || devices_count != 50000 {
        println!("Database doesn't have the correct number of records:");
        println!("  - Users: {} (should be 1000)", users_count);
        println!("  - Device kinds: {} (should be 2000)", device_kinds_count);
        println!("  - Devices: {} (should be 50000)", devices_count);
        println!("Recreating test data...");

        client
            .batch_execute(
                "TRUNCATE bench_shipments_devices, bench_shipments,
                bench_maintenance_devices, bench_maintenance,
                bench_receipts_devices, bench_receipts,
                bench_inventory_assessments_devices, bench_inventory_assessments,
                bench_activities, bench_devices, bench_device_kinds,
                bench_categories, bench_labs, bench_users CASCADE;",
            )
            .await
            .expect("Failed to truncate tables");

        populate_large_test_data(&db, 1000, 2000, 50000, 10)
            .await
            .expect("Failed to populate large test data");
    } else {
        println!("Successfully connected to PostgreSQL!");
        println!("Database already contains the correct test data:");
        println!("  - 1000 users");
        println!("  - 2000 device kinds");
        println!("  - 50000 devices");
        println!("  - 10 labs");
    }

    let schema = DatabaseSchema::fetch(&db)
        .await
        .expect("Failed to fetch schema for benchmark");

    AppState {
        db,
        schema: Arc::new(Mutex::new(Some(schema))),
    }
}

pub async fn setup_test_tables(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let client = db.get_client().await?;

    let create_tables = r#"
    -- Create test types
    DO $$
    BEGIN
        IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'bench_device_status') THEN
            CREATE TYPE bench_device_status AS ENUM (
                'healthy', 'broken', 'discarded', 'lost',
                'assessing', 'shipping', 'maintaining', 'borrowing'
            );
        END IF;

        IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'bench_activity_type') THEN
            CREATE TYPE bench_activity_type AS ENUM (
                'assessment', 'borrow', 'return', 'maintenance', 'shipment'
            );
        END IF;
    END $$;

    -- Users table for testing
    CREATE TABLE IF NOT EXISTS bench_users (
        id UUID PRIMARY KEY,
        name TEXT NOT NULL,
        email TEXT UNIQUE,
        image TEXT,
        deleted_at TIMESTAMPTZ
    );

    -- Labs table for testing
    CREATE TABLE IF NOT EXISTS bench_labs (
        id UUID PRIMARY KEY,
        name TEXT,
        room TEXT NOT NULL,
        branch TEXT NOT NULL,
        deleted_at TIMESTAMPTZ
    );

    -- Categories table for testing
    CREATE TABLE IF NOT EXISTS bench_categories (
        id UUID PRIMARY KEY,
        name TEXT NOT NULL,
        deleted_at TIMESTAMPTZ
    );

    -- Device kinds table for testing
    CREATE TABLE IF NOT EXISTS bench_device_kinds (
        id UUID PRIMARY KEY,
        name TEXT NOT NULL,
        unit TEXT,
        brand TEXT,
        manufacturer TEXT,
        description TEXT,
        image JSONB,
        is_borrowable_lab_only BOOLEAN NOT NULL DEFAULT false,
        allowed_borrow_roles TEXT[] NOT NULL DEFAULT '{}',
        allowed_view_roles TEXT[] NOT NULL DEFAULT '{}',
        category_id UUID REFERENCES bench_categories(id),
        deleted_at TIMESTAMPTZ
    );

    -- Devices table for testing
    CREATE TABLE IF NOT EXISTS bench_devices (
        id UUID PRIMARY KEY,
        full_id TEXT NOT NULL,
        kind UUID REFERENCES bench_device_kinds(id),
        lab_id UUID REFERENCES bench_labs(id),
        status bench_device_status NOT NULL DEFAULT 'healthy'::bench_device_status,
        accessory_for_kind_id UUID REFERENCES bench_device_kinds(id),
        deleted_at TIMESTAMPTZ
    );

    -- Activities table for testing
    CREATE TABLE IF NOT EXISTS bench_activities (
        id UUID PRIMARY KEY,
        type bench_activity_type NOT NULL,
        note TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    -- Inventory assessments table for testing
    CREATE TABLE IF NOT EXISTS bench_inventory_assessments (
        id UUID PRIMARY KEY REFERENCES bench_activities(id),
        lab_id UUID REFERENCES bench_labs(id),
        accountant_id UUID REFERENCES bench_users(id),
        status TEXT NOT NULL DEFAULT 'assessing',
        finished_at TIMESTAMPTZ
    );

    -- Inventory assessments devices table for testing
    CREATE TABLE IF NOT EXISTS bench_inventory_assessments_devices (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        assessing_id UUID REFERENCES bench_inventory_assessments(id),
        device_id UUID REFERENCES bench_devices(id),
        prev_status bench_device_status,
        after_status bench_device_status,
        UNIQUE(assessing_id, device_id)
    );

    -- Receipts table for testing
    CREATE TABLE IF NOT EXISTS bench_receipts (
        id UUID PRIMARY KEY,
        actor_id UUID REFERENCES bench_users(id),
        checker_id UUID REFERENCES bench_users(id),
        lab_id UUID REFERENCES bench_labs(id)
    );

    -- Receipts devices table for testing
    CREATE TABLE IF NOT EXISTS bench_receipts_devices (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        borrowed_receipt_id UUID REFERENCES bench_receipts(id),
        returned_receipt_id UUID REFERENCES bench_receipts(id),
        device_id UUID REFERENCES bench_devices(id),
        borrow_id UUID REFERENCES bench_activities(id),
        return_id UUID REFERENCES bench_activities(id),
        expected_returned_at TIMESTAMPTZ,
        expected_returned_lab_id UUID REFERENCES bench_labs(id),
        prev_quality bench_device_status,
        after_quality bench_device_status,
        note TEXT
    );

    -- Maintenance table for testing
    CREATE TABLE IF NOT EXISTS bench_maintenance (
        id UUID PRIMARY KEY REFERENCES bench_activities(id),
        lab_id UUID REFERENCES bench_labs(id),
        technician_id UUID REFERENCES bench_users(id),
        status TEXT NOT NULL DEFAULT 'in_progress',
        finished_at TIMESTAMPTZ
    );

    -- Maintenance devices table for testing
    CREATE TABLE IF NOT EXISTS bench_maintenance_devices (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        maintenance_id UUID REFERENCES bench_maintenance(id),
        device_id UUID REFERENCES bench_devices(id),
        prev_status bench_device_status,
        after_status bench_device_status,
        UNIQUE(maintenance_id, device_id)
    );

    -- Shipments table for testing
    CREATE TABLE IF NOT EXISTS bench_shipments (
        id UUID PRIMARY KEY REFERENCES bench_activities(id),
        from_lab_id UUID REFERENCES bench_labs(id),
        to_lab_id UUID REFERENCES bench_labs(id),
        shipper_id UUID REFERENCES bench_users(id),
        status TEXT NOT NULL DEFAULT 'preparing',
        finished_at TIMESTAMPTZ
    );

    -- Shipments devices table for testing
    CREATE TABLE IF NOT EXISTS bench_shipments_devices (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        shipment_id UUID REFERENCES bench_shipments(id),
        device_id UUID REFERENCES bench_devices(id),
        prev_status bench_device_status,
        after_status bench_device_status,
        UNIQUE(shipment_id, device_id)
    );
    "#;

    client.batch_execute(create_tables).await?;

    Ok(())
}

async fn populate_large_test_data(
    db: &Database,
    num_users: usize,
    num_device_kinds: usize,
    num_devices: usize,
    num_labs: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = db.get_client().await?;

    println!("Generating test data with:");
    println!("  - {} users", num_users);
    println!("  - {} device kinds", num_device_kinds);
    println!("  - {} devices", num_devices);
    println!("  - {} labs", num_labs);

    client
        .execute("SET session_replication_role = 'replica'", &[])
        .await?;

    println!("Creating users...");
    let batch_size = 1000;
    for batch in 0..((num_users) / batch_size + 1) {
        let start_idx = batch * batch_size;
        let end_idx = std::cmp::min((batch + 1) * batch_size, num_users);

        if start_idx >= end_idx {
            break;
        }

        let mut user_values = Vec::with_capacity(end_idx - start_idx);
        for i in start_idx..end_idx {
            let user_id = uuid::Uuid::new_v4();
            let user_name = format!("User {}", i + 1);
            let email = format!("user{}@example.com", i + 1);
            let image = serde_json::json!({
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

    println!("Creating labs...");
    let mut lab_values = Vec::with_capacity(num_labs);
    for i in 0..num_labs {
        let lab_id = uuid::Uuid::new_v4();
        let lab_name = format!("Lab {}", i + 1);
        lab_values.push(format!(
            "('{}', '{}', 'Room {}', 'Branch {}')",
            lab_id,
            lab_name,
            i + 100,
            (i % 5) + 1
        ));
    }

    let query = format!(
        "INSERT INTO bench_labs (id, name, room, branch) VALUES {}",
        lab_values.join(", ")
    );

    client.execute(&query, &[]).await?;
    println!("Created {} labs", num_labs);

    println!("Creating device kinds...");
    let batch_size = 1000;
    for batch in 0..((num_device_kinds) / batch_size + 1) {
        let start_idx = batch * batch_size;
        let end_idx = std::cmp::min((batch + 1) * batch_size, num_device_kinds);

        if start_idx >= end_idx {
            break;
        }

        let mut kind_values = Vec::with_capacity(end_idx - start_idx);
        for i in start_idx..end_idx {
            let kind_id = uuid::Uuid::new_v4();
            let kind_name = format!("Device Kind {}", i + 1);
            let image = serde_json::json!({
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

    println!("Creating categories...");
    let categories = [
        ("Computers", uuid::Uuid::new_v4()),
        ("Networks", uuid::Uuid::new_v4()),
        ("Peripherals", uuid::Uuid::new_v4()),
    ];

    let mut category_values = Vec::with_capacity(categories.len());
    for (name, id) in &categories {
        category_values.push(format!("('{}', '{}')", id, name));
    }

    let query = format!(
        "INSERT INTO bench_categories (id, name) VALUES {}",
        category_values.join(", ")
    );

    client.execute(&query, &[]).await?;
    println!("Created {} categories", categories.len());

    println!("Assigning categories to device kinds...");
    let category_ids: Vec<uuid::Uuid> = categories.iter().map(|(_, id)| *id).collect();

    let kind_rows = client
        .query("SELECT id FROM bench_device_kinds", &[])
        .await?;
    let kind_ids: Vec<uuid::Uuid> = kind_rows
        .iter()
        .map(|row| row.get::<_, uuid::Uuid>(0))
        .collect();

    let batch_size = 500;
    for (i, chunk) in kind_ids.chunks(batch_size).enumerate() {
        let mut updates = Vec::new();

        for (j, kind_id) in chunk.iter().enumerate() {
            let category_idx = (i * batch_size + j) % 3;
            updates.push(format!("('{}', '{}')", kind_id, category_ids[category_idx]));
        }

        if !updates.is_empty() {
            let update_query = format!(
                "UPDATE bench_device_kinds AS dk
                 SET category_id = u.category_id::uuid
                 FROM (VALUES {}) AS u(id, category_id)
                 WHERE dk.id = u.id::uuid",
                updates.join(", ")
            );

            client.execute(&update_query, &[]).await?;
        }
    }

    println!("Assigned categories to device kinds");

    let lab_rows = client.query("SELECT id::text FROM bench_labs", &[]).await?;
    let lab_ids: Vec<String> = lab_rows.iter().map(|row| row.get(0)).collect();

    let kind_rows = client
        .query("SELECT id::text FROM bench_device_kinds", &[])
        .await?;
    let kind_ids: Vec<String> = kind_rows.iter().map(|row| row.get(0)).collect();

    if lab_ids.is_empty() || kind_ids.is_empty() {
        return Err("Failed to create labs or device kinds".into());
    }

    println!("Creating devices...");
    let batch_size = 5000;
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
            let device_id = uuid::Uuid::new_v4();
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

    client
        .execute("SET session_replication_role = 'origin'", &[])
        .await?;

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

    let tables = [
        "bench_labs",
        "bench_users",
        "bench_device_kinds",
        "bench_devices",
    ];

    println!("\n--- Current Database State ---");
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

#[allow(dead_code)]
pub async fn cleanup_test_tables(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let client = db.get_client().await?;

    let drop_tables = r#"
    DROP TABLE IF EXISTS bench_shipments_devices;
    DROP TABLE IF EXISTS bench_shipments;
    DROP TABLE IF EXISTS bench_maintenance_devices;
    DROP TABLE IF EXISTS bench_maintenance;
    DROP TABLE IF EXISTS bench_receipts_devices;
    DROP TABLE IF EXISTS bench_receipts;
    DROP TABLE IF EXISTS bench_inventory_assessments_devices;
    DROP TABLE IF EXISTS bench_inventory_assessments;
    DROP TABLE IF EXISTS bench_activities;
    DROP TABLE IF EXISTS bench_devices;
    DROP TABLE IF EXISTS bench_device_kinds;
    DROP TABLE IF EXISTS bench_categories;
    DROP TABLE IF EXISTS bench_labs;
    DROP TABLE IF EXISTS bench_users;

    -- Drop custom types
    DROP TYPE IF EXISTS bench_device_status;
    DROP TYPE IF EXISTS bench_activity_type;
    "#;

    client.batch_execute(drop_tables).await?;

    Ok(())
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        let rt = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime for AppState clone");
        let db = rt.block_on(async {
            Database::new()
                .await
                .expect("Failed to connect to database in AppState clone")
        });

        AppState {
            db,
            schema: self.schema.clone(),
        }
    }
}
