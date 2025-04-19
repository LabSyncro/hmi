use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::error::Error;
use tauri::State;
use tokio_postgres::types::{FromSql, ToSql, Type};
use uuid::Uuid;

use crate::db::{connection::DbError, queries::builder::QueryBuilder, schema::DatabaseSchema};

use super::AppState;

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

fn to_camel_case(snake_str: &str) -> String {
    let mut result = String::with_capacity(snake_str.len());
    let mut capitalize_next = false;

    for c in snake_str.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn convert_json_keys_to_camel_case(value: JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (key, val) in obj {
                let camel_key = to_camel_case(&key);
                new_obj.insert(camel_key, convert_json_keys_to_camel_case(val));
            }
            JsonValue::Object(new_obj)
        }
        JsonValue::Array(arr) => JsonValue::Array(
            arr.into_iter()
                .map(convert_json_keys_to_camel_case)
                .collect(),
        ),
        _ => value,
    }
}

#[derive(Debug, Clone)]
pub struct PostgresEnum(String);

impl<'a> FromSql<'a> for PostgresEnum {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let s = String::from_utf8(raw.to_vec())?;
        Ok(PostgresEnum(s))
    }

    fn accepts(ty: &Type) -> bool {
        // Check if the type is a PostgreSQL enum by checking its object ID range
        // PostgreSQL custom types (including enums) have OIDs >= 16384
        ty.oid() >= 16384
    }
}

#[tauri::command]
pub async fn sync_schema(state: State<'_, AppState>) -> CommandResult<()> {
    let schema = DatabaseSchema::fetch(&state.db).await?;

    schema
        .save_to_file("db-schema.json")
        .map_err(|e| CommandError {
            message: format!("Failed to save schema: {}", e),
        })?;

    schema
        .generate_typescript_types("src/lib/db/types.ts")
        .map_err(|e| CommandError {
            message: format!("Failed to generate TypeScript types: {}", e),
        })?;

    *state.schema.lock().await = Some(schema);

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct JoinParams {
    pub table: String,
    pub left_column: String,
    pub right_column: String,
    pub kind: String, // "inner", "left", or "right"
    pub alias: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub table: String,
    pub columns: Option<Vec<String>>,
    pub conditions: Option<Vec<(String, serde_json::Value)>>,
    pub order_by: Option<Vec<(String, bool)>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub joins: Option<Vec<JoinParams>>,
}

#[tauri::command]
pub async fn query_table(
    state: State<'_, AppState>,
    params: QueryParams,
) -> CommandResult<Vec<serde_json::Value>> {
    let schema_guard = state.schema.lock().await;
    let schema = schema_guard.as_ref().ok_or_else(|| CommandError {
        message: "Database schema not initialized. Please restart the application.".to_string(),
    })?;

    let mut builder = QueryBuilder::new(schema, &params.table).ok_or_else(|| CommandError {
        message: format!("Table '{}' not found in the database schema", params.table),
    })?;

    if let Some(columns) = params.columns {
        builder = builder.select(&columns.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    }

    if let Some(joins) = params.joins {
        for join in joins {
            builder = match join.kind.as_str() {
                "inner" => builder.inner_join(
                    &join.table,
                    &join.left_column,
                    &join.right_column,
                    join.alias,
                ),
                "left" => builder.left_join(
                    &join.table,
                    &join.left_column,
                    &join.right_column,
                    join.alias,
                ),
                _ => builder, // Ignore unsupported join types
            };
        }
    }

    if let Some(conditions) = params.conditions {
        for (column, value) in conditions {
            match value {
                serde_json::Value::String(s) => {
                    let column_info = builder.get_column_type(&column);
                    if let Some(type_name) = column_info {
                        if type_name == "uuid" {
                            if let Ok(uuid) = uuid::Uuid::parse_str(&s) {
                                builder = builder.where_eq(&column, Some(uuid));
                            } else {
                                builder = builder.where_eq(&column, Some(s));
                            }
                        } else {
                            builder = builder.where_eq(&column, Some(s));
                        }
                    } else {
                        builder = builder.where_eq(&column, Some(s));
                    }
                }
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        builder = builder.where_eq(&column, Some(i));
                    } else if let Some(f) = n.as_f64() {
                        builder = builder.where_eq(&column, Some(f));
                    }
                }
                serde_json::Value::Bool(b) => builder = builder.where_eq(&column, Some(b)),
                serde_json::Value::Null => builder = builder.where_eq::<String>(&column, None),
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
                let name = to_camel_case(column.name());
                let value = match column.type_() {
                    &Type::VARCHAR | &Type::TEXT => {
                        let s: Option<String> = row.get(i);
                        match s {
                            Some(val) => serde_json::Value::String(val),
                            None => serde_json::Value::Null,
                        }
                    }
                    &Type::INT4 => {
                        let n: Option<i32> = row.get(i);
                        match n {
                            Some(val) => serde_json::Value::Number(val.into()),
                            None => serde_json::Value::Null,
                        }
                    }
                    &Type::INT8 => {
                        let n: Option<i64> = row.get(i);
                        match n {
                            Some(val) => serde_json::Value::Number(val.into()),
                            None => serde_json::Value::Null,
                        }
                    }
                    &Type::FLOAT8 => {
                        let n: Option<f64> = row.get(i);
                        serde_json::json!(n)
                    }
                    &Type::BOOL => {
                        let b: Option<bool> = row.get(i);
                        match b {
                            Some(val) => serde_json::Value::Bool(val),
                            None => serde_json::Value::Null,
                        }
                    }
                    &Type::TIMESTAMPTZ => {
                        let ts: Option<chrono::DateTime<chrono::Utc>> = row.get(i);
                        ts.map(|t| serde_json::Value::String(t.to_rfc3339()))
                            .unwrap_or(serde_json::Value::Null)
                    }
                    &Type::JSON | &Type::JSONB => match row.try_get::<_, Option<JsonValue>>(i) {
                        Ok(Some(json_val)) => convert_json_keys_to_camel_case(json_val),
                        Ok(None) => serde_json::Value::Null,
                        Err(_) => {
                            if let Ok(Some(json_str)) = row.try_get::<_, Option<String>>(i) {
                                if let Ok(parsed) = serde_json::from_str(&json_str) {
                                    convert_json_keys_to_camel_case(parsed)
                                } else {
                                    serde_json::Value::String(json_str)
                                }
                            } else {
                                serde_json::Value::Null
                            }
                        }
                    },
                    t if t.to_string().starts_with("_") => {
                        let arr: Option<Vec<String>> = row.try_get(i).ok().flatten();
                        match arr {
                            Some(val) => serde_json::Value::Array(
                                val.into_iter().map(serde_json::Value::String).collect(),
                            ),
                            None => serde_json::Value::Null,
                        }
                    }
                    t => {
                        let type_name = t.to_string();
                        if type_name == "uuid" {
                            match row.try_get::<_, uuid::Uuid>(i) {
                                Ok(uuid) => serde_json::Value::String(uuid.to_string()),
                                _ => {
                                    let uuid_str: Option<String> = row.try_get(i).ok().flatten();
                                    match uuid_str {
                                        Some(val) => serde_json::Value::String(val),
                                        None => serde_json::Value::Null,
                                    }
                                }
                            }
                        } else if t.oid() >= 16384 {
                            let enum_value: Result<Option<PostgresEnum>, _> = row.try_get(i);
                            match enum_value {
                                Ok(Some(PostgresEnum(val))) => serde_json::Value::String(val),
                                _ => {
                                    let str_val: Option<String> = row.try_get(i).ok().flatten();
                                    match str_val {
                                        Some(val) => serde_json::Value::String(val),
                                        None => serde_json::Value::Null,
                                    }
                                }
                            }
                        } else {
                            let val: Option<String> = row.try_get(i).ok().flatten();
                            match val {
                                Some(s) => serde_json::Value::String(s),
                                None => serde_json::Value::Null,
                            }
                        }
                    }
                };
                obj.insert(name, value);
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

    let (query, insert_params) = QueryBuilder::build_insert(&params.value, schema, &params.table)
        .ok_or_else(|| CommandError {
        message: format!("Failed to build insert query for table {}", params.table),
    })?;

    let client = state.db.get_client().await?;

    let params_slice: Vec<&(dyn ToSql + Sync)> = insert_params
        .iter()
        .map(|p| &**p as &(dyn ToSql + Sync))
        .collect();
    let row = client
        .query_one(&query, &params_slice)
        .await
        .map_err(DbError::from)?;

    let mut obj = serde_json::Map::new();
    for (i, column) in row.columns().iter().enumerate() {
        let name = to_camel_case(column.name());
        let value = match column.type_() {
            &Type::VARCHAR | &Type::TEXT => {
                let s: Option<String> = row.get(i);
                serde_json::Value::String(s.unwrap_or_default())
            }
            &Type::INT4 => {
                let n: Option<i32> = row.get(i);
                serde_json::Value::Number(n.unwrap_or_default().into())
            }
            &Type::INT8 => {
                let n: Option<i64> = row.get(i);
                serde_json::Value::Number(n.unwrap_or_default().into())
            }
            &Type::FLOAT8 => {
                let n: Option<f64> = row.get(i);
                serde_json::json!(n)
            }
            &Type::BOOL => {
                let b: Option<bool> = row.get(i);
                serde_json::Value::Bool(b.unwrap_or_default())
            }
            &Type::TIMESTAMPTZ => {
                let ts: Option<chrono::DateTime<chrono::Utc>> = row.get(i);
                ts.map(|t| serde_json::Value::String(t.to_rfc3339()))
                    .unwrap_or(serde_json::Value::Null)
            }
            &Type::JSON | &Type::JSONB => {
                let json: Option<JsonValue> = row.try_get(i).ok().flatten();
                convert_json_keys_to_camel_case(json.unwrap_or(JsonValue::Null))
            }
            _ => serde_json::Value::Null,
        };
        obj.insert(name, value);
    }

    Ok(serde_json::Value::Object(obj))
}

#[derive(Debug, Deserialize)]
pub struct RawQueryParams {
    #[allow(dead_code)]
    pub sql: String,
    #[allow(dead_code)]
    pub params: Option<Vec<serde_json::Value>>,
}

#[tauri::command]
pub async fn query_raw(
    state: State<'_, AppState>,
    params: RawQueryParams,
) -> CommandResult<Vec<serde_json::Value>> {
    let client = state.db.get_client().await?;

    let params_clone = params.params.clone();
    let param_values: Vec<Box<dyn ToSql + Send + Sync>> = params_clone
        .unwrap_or_default()
        .into_iter()
        .map(|v| match v {
            serde_json::Value::String(s) => {
                if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&s) {
                    Box::new(ts.with_timezone(&chrono::Utc)) as Box<dyn ToSql + Send + Sync>
                } else if let Ok(uuid_val) = Uuid::parse_str(&s) {
                    Box::new(uuid_val) as Box<dyn ToSql + Send + Sync>
                } else {
                    Box::new(s) as Box<dyn ToSql + Send + Sync>
                }
            }
            serde_json::Value::Array(arr) => {
                let vec: Vec<String> = arr
                    .iter()
                    .map(|v| v.as_str().unwrap_or_default().to_string())
                    .collect();
                Box::new(vec) as Box<dyn ToSql + Send + Sync>
            }
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Box::new(i) as Box<dyn ToSql + Send + Sync>
                } else if let Some(f) = n.as_f64() {
                    Box::new(f) as Box<dyn ToSql + Send + Sync>
                } else {
                    Box::new(0i64) as Box<dyn ToSql + Send + Sync>
                }
            }
            serde_json::Value::Bool(b) => Box::new(b) as Box<dyn ToSql + Send + Sync>,
            serde_json::Value::Null => {
                Box::new(Option::<String>::None) as Box<dyn ToSql + Send + Sync>
            }
            _ => Box::new("") as Box<dyn ToSql + Send + Sync>,
        })
        .collect();

    let params_slice: Vec<&(dyn ToSql + Sync)> = param_values
        .iter()
        .map(|p| &**p as &(dyn ToSql + Sync))
        .collect();

    let rows = client
        .query(&params.sql, params_slice.as_slice())
        .await
        .map_err(|e| CommandError {
            message: format!(
                "Query execution failed: {}. SQL: {}, Params: {:?}",
                e, params.sql, params.params
            ),
        })?;

    Ok(rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let name = to_camel_case(column.name());
                let value = match column.type_() {
                    &Type::VARCHAR | &Type::TEXT => {
                        let s: Option<String> = row.get(i);
                        match s {
                            Some(val) => serde_json::Value::String(val),
                            None => serde_json::Value::Null,
                        }
                    }
                    &Type::INT4 => {
                        let n: Option<i32> = row.get(i);
                        match n {
                            Some(val) => serde_json::Value::Number(val.into()),
                            None => serde_json::Value::Null,
                        }
                    }
                    &Type::INT8 => {
                        let n: Option<i64> = row.get(i);
                        match n {
                            Some(val) => serde_json::Value::Number(val.into()),
                            None => serde_json::Value::Null,
                        }
                    }
                    &Type::FLOAT8 => {
                        let n: Option<f64> = row.get(i);
                        match n {
                            Some(val) => serde_json::json!(val),
                            None => serde_json::Value::Null,
                        }
                    }
                    &Type::BOOL => {
                        let b: Option<bool> = row.get(i);
                        match b {
                            Some(val) => serde_json::Value::Bool(val),
                            None => serde_json::Value::Null,
                        }
                    }
                    &Type::TIMESTAMPTZ => {
                        let ts: Option<chrono::DateTime<chrono::Utc>> = row.get(i);
                        match ts {
                            Some(t) => serde_json::Value::String(t.to_rfc3339()),
                            None => serde_json::Value::Null,
                        }
                    }
                    &Type::JSON | &Type::JSONB => match row.try_get::<_, Option<JsonValue>>(i) {
                        Ok(Some(json_val)) => convert_json_keys_to_camel_case(json_val),
                        Ok(None) => serde_json::Value::Null,
                        Err(_) => {
                            if let Ok(Some(json_str)) = row.try_get::<_, Option<String>>(i) {
                                if let Ok(parsed) = serde_json::from_str(&json_str) {
                                    convert_json_keys_to_camel_case(parsed)
                                } else {
                                    serde_json::Value::String(json_str)
                                }
                            } else {
                                serde_json::Value::Null
                            }
                        }
                    },
                    t if t.to_string().starts_with("_") => {
                        let arr: Option<Vec<String>> = row.try_get(i).ok().flatten();
                        match arr {
                            Some(val) => serde_json::Value::Array(
                                val.into_iter().map(serde_json::Value::String).collect(),
                            ),
                            None => serde_json::Value::Null,
                        }
                    }
                    t => {
                        let type_name = t.to_string();
                        if type_name == "uuid" {
                            match row.try_get::<_, uuid::Uuid>(i) {
                                Ok(uuid) => serde_json::Value::String(uuid.to_string()),
                                _ => {
                                    let uuid_str: Option<String> = row.try_get(i).ok().flatten();
                                    match uuid_str {
                                        Some(val) => serde_json::Value::String(val),
                                        None => serde_json::Value::Null,
                                    }
                                }
                            }
                        } else if t.oid() >= 16384 {
                            let enum_value: Result<Option<PostgresEnum>, _> = row.try_get(i);
                            match enum_value {
                                Ok(Some(PostgresEnum(val))) => serde_json::Value::String(val),
                                _ => {
                                    let str_val: Option<String> = row.try_get(i).ok().flatten();
                                    match str_val {
                                        Some(val) => serde_json::Value::String(val),
                                        None => serde_json::Value::Null,
                                    }
                                }
                            }
                        } else {
                            let val: Option<String> = row.try_get(i).ok().flatten();
                            match val {
                                Some(s) => serde_json::Value::String(s),
                                None => serde_json::Value::Null,
                            }
                        }
                    }
                };
                obj.insert(name, value);
            }
            serde_json::Value::Object(obj)
        })
        .collect())
}
