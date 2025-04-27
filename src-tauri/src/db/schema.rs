use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::connection::{Database, DbResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub type_name: String,
    pub is_nullable: bool,
    pub is_primary: bool,
    pub default_value: Option<String>,
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableInfo {
    pub name: String,
    pub schema: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseSchema {
    pub tables: HashMap<String, TableInfo>,
    pub enums: HashMap<String, EnumInfo>,
    pub version: String,
}

impl DatabaseSchema {
    pub async fn fetch(db: &Database) -> DbResult<Self> {
        let client = db.get_client().await?;

        let enum_query = r#"
            SELECT 
                t.typname as enum_name,
                array_agg(e.enumlabel ORDER BY e.enumsortorder) as enum_values
            FROM pg_type t
            JOIN pg_enum e ON t.oid = e.enumtypid
            JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace
            GROUP BY t.typname, n.nspname
            HAVING n.nspname = 'public'
        "#;

        let enum_rows = client.query(enum_query, &[]).await?;
        let mut enums = HashMap::new();

        for row in enum_rows {
            let name: String = row.get("enum_name");
            let values: Vec<String> = row.get("enum_values");
            enums.insert(name.clone(), EnumInfo { name, values });
        }

        let table_query = r#"
            SELECT 
                t.table_schema,
                t.table_name,
                c.column_name,
                c.data_type,
                c.udt_name,
                c.is_nullable,
                c.column_default,
                tc.constraint_type
            FROM information_schema.tables t
            JOIN information_schema.columns c 
                ON c.table_schema = t.table_schema 
                AND c.table_name = t.table_name
            LEFT JOIN information_schema.key_column_usage kcu
                ON kcu.table_schema = t.table_schema
                AND kcu.table_name = t.table_name
                AND kcu.column_name = c.column_name
            LEFT JOIN information_schema.table_constraints tc
                ON tc.constraint_schema = t.table_schema
                AND tc.constraint_name = kcu.constraint_name
            WHERE t.table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY t.table_schema, t.table_name, c.ordinal_position;
        "#;

        let rows = client.query(table_query, &[]).await?;

        let mut tables: HashMap<String, TableInfo> = HashMap::new();
        let mut table_columns: HashMap<String, Vec<ColumnInfo>> = HashMap::new();
        let mut table_primary_keys: HashMap<String, Vec<String>> = HashMap::new();

        for row in rows {
            let schema: String = row.get("table_schema");
            let table_name: String = row.get("table_name");
            let column_name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            let udt_name: String = row.get("udt_name");
            let is_nullable: String = row.get("is_nullable");
            let default_value: Option<String> = row.get("column_default");
            let constraint_type: Option<String> = row.get("constraint_type");

            let full_table_name = format!("{}.{}", schema, table_name);

            let enum_values = if data_type == "USER-DEFINED" {
                enums.get(&udt_name).map(|e| e.values.clone())
            } else {
                None
            };

            let column = ColumnInfo {
                name: column_name.clone(),
                type_name: if data_type == "USER-DEFINED" {
                    udt_name.clone()
                } else {
                    data_type
                },
                is_nullable: is_nullable == "YES",
                is_primary: constraint_type.as_deref() == Some("PRIMARY KEY"),
                default_value,
                enum_values,
            };

            table_columns
                .entry(full_table_name.clone())
                .or_default()
                .push(column);

            if constraint_type.as_deref() == Some("PRIMARY KEY") {
                table_primary_keys
                    .entry(full_table_name.clone())
                    .or_default()
                    .push(column_name);
            }

            if !tables.contains_key(&full_table_name) {
                let table_info = TableInfo {
                    name: table_name.clone(),
                    schema: schema.clone(),
                    columns: Vec::new(),
                    primary_keys: Vec::new(),
                };
                tables.insert(full_table_name.clone(), table_info.clone());
                tables.insert(table_name, table_info);
            }
        }

        for (full_name, table) in tables.iter_mut() {
            if let Some(columns) = table_columns.get(full_name) {
                table.columns = columns.clone();
            }
            if let Some(primary_keys) = table_primary_keys.get(full_name) {
                table.primary_keys = primary_keys.clone();
            }
        }

        let version: String = client.query_one("SELECT version()", &[]).await?.get(0);

        Ok(DatabaseSchema {
            tables,
            enums,
            version,
        })
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    #[allow(dead_code)]
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let contents = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    pub fn generate_typescript_types<P: AsRef<Path>>(&self, output_path: P) -> std::io::Result<()> {
        let mut typescript = String::new();

        typescript.push_str("// This file is auto-generated. Do not edit manually.\n\n");

        for (enum_name, enum_info) in &self.enums {
            typescript.push_str(&format!("export enum {} {{\n", pascal_case(enum_name)));
            for value in &enum_info.values {
                typescript.push_str(&format!(
                    "  {} = '{}',\n",
                    value.replace('-', "_").to_uppercase(),
                    value
                ));
            }
            typescript.push_str("}\n\n");
        }

        let mut seen_tables = std::collections::HashMap::new();

        for (full_name, table) in &self.tables {
            if !full_name.contains('.')
                && self
                    .tables
                    .contains_key(&format!("{}.{}", table.schema, table.name))
            {
                continue;
            }

            let interface_name = if seen_tables.contains_key(&table.name) {
                format!(
                    "{}_{}",
                    pascal_case(&table.name),
                    pascal_case(&table.schema)
                )
            } else {
                pascal_case(&table.name)
            };
            seen_tables.insert(table.name.clone(), interface_name.clone());

            typescript.push_str(&format!("export interface {} {{\n", interface_name));

            let mut seen_columns: std::collections::HashSet<String> =
                std::collections::HashSet::new();

            for column in &table.columns {
                let clean_name = column.name.replace('\u{200B}', "");
                if !seen_columns.insert(clean_name.clone()) {
                    continue;
                }

                let ts_type = if let Some(_enum_values) = &column.enum_values {
                    pascal_case(&column.type_name)
                } else if column.type_name.starts_with('_') || column.type_name.contains("ARRAY") {
                    let base_type = if column.type_name.starts_with('_') {
                        &column.type_name[1..]
                    } else {
                        column.type_name.trim_end_matches("[]")
                    };
                    format!("{}[]", pg_type_to_typescript(base_type))
                } else {
                    pg_type_to_typescript(&column.type_name)
                };

                let nullable = if column.is_nullable { " | null" } else { "" };
                typescript.push_str(&format!(
                    "  {}: {}{}\n",
                    camel_case(&clean_name),
                    ts_type,
                    nullable
                ));
            }

            typescript.push_str("}\n\n");
        }

        fs::write(output_path, typescript)
    }
}

fn pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut result = first.to_uppercase().to_string();
                    result.push_str(&chars.as_str().to_lowercase());
                    result
                }
            }
        })
        .collect()
}

fn camel_case(s: &str) -> String {
    let pascal = pascal_case(s);
    pascal
        .chars()
        .next()
        .unwrap_or_default()
        .to_lowercase()
        .to_string()
        + &pascal[1..]
}

fn pg_type_to_typescript(pg_type: &str) -> String {
    match pg_type {
        "integer" | "smallint" | "bigint" | "serial" | "bigserial" => "number".to_string(),
        "numeric" | "decimal" | "real" | "double precision" => "number".to_string(),
        "character varying" | "text" | "character" | "varchar" => "string".to_string(),
        "boolean" => "boolean".to_string(),
        "timestamp with time zone"
        | "timestamp without time zone"
        | "timestamp"
        | "timestamptz" => "Date".to_string(),
        "json" | "jsonb" => "any".to_string(),
        "uuid" => "string".to_string(),
        "bytea" => "unknown".to_string(),
        t if t.contains("enum") => pascal_case(t.trim_end_matches("enum")),
        t if t.starts_with('_') => {
            let base_type = &t[1..];
            format!("{}[]", pg_type_to_typescript(base_type))
        }
        _ => "string".to_string(), // Default to string for unknown types
    }
}
