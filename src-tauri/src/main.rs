// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;

mod commands;
mod db;

use commands::AppState;
use db::connection::Database;
use db::schema::DatabaseSchema;

#[tokio::main]
async fn main() {
    let db = Database::new()
        .await
        .expect("Failed to initialize database");

    if let Err(e) = sync_schema_cli().await {
        eprintln!("Warning: Failed to sync schema: {}", e);
    }

    let schema = DatabaseSchema::fetch(&db)
        .await
        .expect("Failed to fetch database schema");

    println!("Database schema fetched successfully!");

    let state = AppState {
        db,
        schema: Arc::new(Mutex::new(Some(schema))),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::db_commands::sync_schema,
            commands::db_commands::query_table,
            commands::db_commands::insert_into_table
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn sync_schema_cli() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new().await?;
    let schema = DatabaseSchema::fetch(&db).await?;

    let workspace_dir = std::env::current_dir()?
        .parent()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find project root directory",
            )
        })?
        .to_path_buf();

    let types_dir = workspace_dir.join("src").join("types").join("db");
    std::fs::create_dir_all(&types_dir)?;

    println!("Saving schema to files...");

    let schema_path = types_dir.join("schema.json");
    schema.save_to_file(&schema_path)?;
    println!("✓ Schema saved to src/types/db/schema.json");

    let types_path = types_dir.join("generated.ts");
    schema.generate_typescript_types(&types_path)?;
    println!("✓ TypeScript types generated in src/types/db/generated.ts");

    Ok(())
}
