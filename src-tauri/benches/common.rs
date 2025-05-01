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

pub async fn setup_bench_env() -> AppState {
    dotenv().ok();
    let db = Database::new()
        .await
        .expect("Failed to connect to test database for benchmark");

    setup_test_tables(&db)
        .await
        .expect("Failed to set up test tables");

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

    populate_test_data(db).await?;

    Ok(())
}

pub async fn populate_test_data(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let client = db.get_client().await?;

    let insert_users = r#"
    INSERT INTO bench_users (id, name, email, image)
    VALUES 
        ('11111111-1111-1111-1111-111111111111', 'Test User 1', 'user1@test.com', '{"url": "https://example.com/avatar1.jpg"}'),
        ('22222222-2222-2222-2222-222222222222', 'Test User 2', 'user2@test.com', '{"url": "https://example.com/avatar2.jpg"}'),
        ('33333333-3333-3333-3333-333333333333', 'Test User 3', 'user3@test.com', '{"url": "https://example.com/avatar3.jpg"}')
    ON CONFLICT (id) DO NOTHING;
    "#;

    let insert_labs = r#"
    INSERT INTO bench_labs (id, name, room, branch)
    VALUES 
        ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Lab 1', 'Room 101', 'Branch A'),
        ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Lab 2', 'Room 202', 'Branch B'),
        ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'Lab 3', 'Room 303', 'Branch C')
    ON CONFLICT (id) DO NOTHING;
    "#;

    let insert_categories = r#"
    INSERT INTO bench_categories (id, name)
    VALUES 
        ('dddddddd-dddd-dddd-dddd-dddddddddddd', 'Computers'),
        ('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'Networks'),
        ('ffffffff-ffff-ffff-ffff-ffffffffffff', 'Peripherals')
    ON CONFLICT (id) DO NOTHING;
    "#;

    let insert_device_kinds = r#"
    INSERT INTO bench_device_kinds (id, name, unit, brand, manufacturer, description, image, category_id, allowed_borrow_roles, allowed_view_roles)
    VALUES 
        ('aaaaaaaa-0000-4000-a000-000000000001', 'Laptop', 'unit', 'Dell', 'Dell Inc.', 'Standard laptop', '{"mainImage": "laptop.jpg"}', 'dddddddd-dddd-dddd-dddd-dddddddddddd', '{student,teacher}', '{student,teacher,admin}'),
        ('bbbbbbbb-0000-4000-a000-000000000002', 'Router', 'unit', 'Cisco', 'Cisco Systems', 'Network router', '{"mainImage": "router.jpg"}', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', '{teacher}', '{student,teacher,admin}'),
        ('cccccccc-0000-4000-a000-000000000003', 'Mouse', 'unit', 'Logitech', 'Logitech Inc.', 'Computer mouse', '{"mainImage": "mouse.jpg"}', 'ffffffff-ffff-ffff-ffff-ffffffffffff', '{student,teacher}', '{student,teacher,admin}')
    ON CONFLICT (id) DO NOTHING;
    "#;

    let insert_devices = r#"
    WITH device_data AS (
        SELECT 
            gen_random_uuid() as id,
            'DEV-' || lpad(n::text, 5, '0') as full_id,
            'aaaaaaaa-0000-4000-a000-000000000001'::uuid as kind,
            'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'::uuid as lab_id,
            'healthy'::bench_device_status as status
        FROM generate_series(1, 34) as n
        UNION ALL
        SELECT 
            gen_random_uuid() as id,
            'DEV-' || lpad((n+34)::text, 5, '0') as full_id,
            'bbbbbbbb-0000-4000-a000-000000000002'::uuid as kind,
            'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'::uuid as lab_id,
            'healthy'::bench_device_status as status
        FROM generate_series(1, 33) as n
        UNION ALL
        SELECT 
            gen_random_uuid() as id,
            'DEV-' || lpad((n+67)::text, 5, '0') as full_id,
            'cccccccc-0000-4000-a000-000000000003'::uuid as kind,
            'cccccccc-cccc-cccc-cccc-cccccccccccc'::uuid as lab_id,
            'healthy'::bench_device_status as status
        FROM generate_series(1, 33) as n
    )
    INSERT INTO bench_devices (id, full_id, kind, lab_id, status)
    SELECT id, full_id, kind, lab_id, status
    FROM device_data
    ON CONFLICT (id) DO NOTHING;
    "#;

    client
        .batch_execute(insert_users)
        .await
        .map_err(|e| format!("Failed to insert users: {}", e))?;
    client
        .batch_execute(insert_labs)
        .await
        .map_err(|e| format!("Failed to insert labs: {}", e))?;
    client
        .batch_execute(insert_categories)
        .await
        .map_err(|e| format!("Failed to insert categories: {}", e))?;
    client
        .batch_execute(insert_device_kinds)
        .await
        .map_err(|e| format!("Failed to insert device kinds: {}", e))?;
    client
        .batch_execute(insert_devices)
        .await
        .map_err(|e| format!("Failed to insert devices: {}", e))?;

    Ok(())
}

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
