use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tokio_postgres::types::ToSql;

use crate::db::{
    connection::{Database, DbError},
    queries::builder::QueryBuilder,
    schema::DatabaseSchema,
};

pub struct AppState {
    pub db: Database,
    pub schema: Arc<Mutex<Option<DatabaseSchema>>>,
}

#[derive(Debug, Serialize)]
pub struct CommandError {
    message: String,
}

impl From<DbError> for CommandError {
    fn from(error: DbError) -> Self {
        CommandError {
            message: error.to_string(),
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

#[tauri::command]
pub async fn sync_schema(state: State<'_, AppState>) -> CommandResult<()> {
    let schema = DatabaseSchema::fetch(&state.db).await?;

    // Save schema to file
    schema
        .save_to_file("db-schema.json")
        .map_err(|e| CommandError {
            message: format!("Failed to save schema: {}", e),
        })?;

    // Generate TypeScript types
    schema
        .generate_typescript_types("src/lib/db/types.ts")
        .map_err(|e| CommandError {
            message: format!("Failed to generate TypeScript types: {}", e),
        })?;

    // Update schema in state
    *state.schema.lock().await = Some(schema);

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub table: String,
    pub columns: Option<Vec<String>>,
    pub conditions: Option<Vec<(String, serde_json::Value)>>,
    pub order_by: Option<Vec<(String, bool)>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub async fn query_table(
    state: State<'_, AppState>,
    params: QueryParams,
) -> CommandResult<Vec<serde_json::Value>> {
    let schema_guard = state.schema.lock().await;
    let schema = schema_guard.as_ref().ok_or_else(|| CommandError {
        message: "Schema not synchronized".to_string(),
    })?;

    let mut builder = QueryBuilder::new(schema, &params.table).ok_or_else(|| CommandError {
        message: format!("Table {} not found", params.table),
    })?;

    if let Some(columns) = params.columns {
        builder = builder.select(&columns.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    }

    if let Some(conditions) = params.conditions {
        for (column, value) in conditions {
            match value {
                serde_json::Value::String(s) => builder = builder.where_eq(&column, s),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        builder = builder.where_eq(&column, i);
                    } else if let Some(f) = n.as_f64() {
                        builder = builder.where_eq(&column, f);
                    }
                }
                serde_json::Value::Bool(b) => builder = builder.where_eq(&column, b),
                _ => continue,
            }
        }
    }

    if let Some(order) = params.order_by {
        for (column, ascending) in order {
            builder = builder.order_by(&column, ascending);
        }
    }

    if let Some(limit) = params.limit {
        builder = builder.limit(limit);
    }

    if let Some(offset) = params.offset {
        builder = builder.offset(offset);
    }

    let (query, params) = builder.build_select();
    let client = state.db.get_client().await?;

    let params_slice: Vec<&(dyn ToSql + Sync)> =
        params.iter().map(|p| &**p as &(dyn ToSql + Sync)).collect();
    let rows = client
        .query(&query, &params_slice)
        .await
        .map_err(DbError::from)?;

    Ok(rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let name = column.name();
                let value = match column.type_() {
                    &tokio_postgres::types::Type::VARCHAR | &tokio_postgres::types::Type::TEXT => {
                        let s: Option<String> = row.get(i);
                        serde_json::Value::String(s.unwrap_or_default())
                    }
                    &tokio_postgres::types::Type::INT4 => {
                        let n: Option<i32> = row.get(i);
                        serde_json::Value::Number(n.unwrap_or_default().into())
                    }
                    &tokio_postgres::types::Type::INT8 => {
                        let n: Option<i64> = row.get(i);
                        serde_json::Value::Number(n.unwrap_or_default().into())
                    }
                    &tokio_postgres::types::Type::FLOAT8 => {
                        let n: Option<f64> = row.get(i);
                        serde_json::json!(n)
                    }
                    &tokio_postgres::types::Type::BOOL => {
                        let b: Option<bool> = row.get(i);
                        serde_json::Value::Bool(b.unwrap_or_default())
                    }
                    &tokio_postgres::types::Type::TIMESTAMPTZ => {
                        let ts: Option<chrono::DateTime<chrono::Utc>> = row.get(i);
                        ts.map(|t| serde_json::Value::String(t.to_rfc3339()))
                            .unwrap_or(serde_json::Value::Null)
                    }
                    _ => serde_json::Value::Null,
                };
                obj.insert(name.to_string(), value);
            }
            serde_json::Value::Object(obj)
        })
        .collect())
}

#[derive(Debug, Deserialize)]
pub struct InsertParams {
    pub table: String,
    pub value: serde_json::Value,
}

#[tauri::command]
pub async fn insert_into_table(
    state: State<'_, AppState>,
    params: InsertParams,
) -> CommandResult<serde_json::Value> {
    let schema_guard = state.schema.lock().await;
    let schema = schema_guard.as_ref().ok_or_else(|| CommandError {
        message: "Schema not synchronized".to_string(),
    })?;

    let (query, params) = QueryBuilder::build_insert(&params.value, schema, &params.table)
        .ok_or_else(|| CommandError {
            message: format!("Failed to build insert query for table {}", params.table),
        })?;

    let client = state.db.get_client().await?;

    let params_slice: Vec<&(dyn ToSql + Sync)> =
        params.iter().map(|p| &**p as &(dyn ToSql + Sync)).collect();
    let row = client
        .query_one(&query, &params_slice)
        .await
        .map_err(DbError::from)?;

    let mut obj = serde_json::Map::new();
    for (i, column) in row.columns().iter().enumerate() {
        let name = column.name();
        let value = match column.type_() {
            &tokio_postgres::types::Type::VARCHAR | &tokio_postgres::types::Type::TEXT => {
                let s: Option<String> = row.get(i);
                serde_json::Value::String(s.unwrap_or_default())
            }
            &tokio_postgres::types::Type::INT4 => {
                let n: Option<i32> = row.get(i);
                serde_json::Value::Number(n.unwrap_or_default().into())
            }
            &tokio_postgres::types::Type::INT8 => {
                let n: Option<i64> = row.get(i);
                serde_json::Value::Number(n.unwrap_or_default().into())
            }
            &tokio_postgres::types::Type::FLOAT8 => {
                let n: Option<f64> = row.get(i);
                serde_json::json!(n)
            }
            &tokio_postgres::types::Type::BOOL => {
                let b: Option<bool> = row.get(i);
                serde_json::Value::Bool(b.unwrap_or_default())
            }
            &tokio_postgres::types::Type::TIMESTAMPTZ => {
                let ts: Option<chrono::DateTime<chrono::Utc>> = row.get(i);
                ts.map(|t| serde_json::Value::String(t.to_rfc3339()))
                    .unwrap_or(serde_json::Value::Null)
            }
            _ => serde_json::Value::Null,
        };
        obj.insert(name.to_string(), value);
    }

    Ok(serde_json::Value::Object(obj))
}
