use super::super::schema::{DatabaseSchema, TableInfo};
use serde::Serialize;

#[derive(Debug)]
pub struct QueryBuilder<'a> {
    table: &'a TableInfo,
    conditions: Vec<String>,
    selected_columns: Option<Vec<String>>,
    order_by: Vec<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>>,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(schema: &'a DatabaseSchema, table_name: &str) -> Option<Self> {
        println!("Looking up table: {}", table_name);
        
        // Try to get the table info, first with the exact name, then try schema-qualified names
        let table = schema.tables.get(table_name).or_else(|| {
            println!("Table not found by exact name, searching in all schemas...");
            // If the table wasn't found by its name directly, try to find it in any schema
            let found = schema.tables.values().find(|t| t.name == table_name);
            if let Some(t) = found {
                println!("Found table in schema: {}", t.schema);
            } else {
                println!("Table not found in any schema");
            }
            found
        })?;

        Some(Self {
            table,
            conditions: Vec::new(),
            selected_columns: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
            params: Vec::new(),
        })
    }

    pub fn select(mut self, columns: &[&str]) -> Self {
        self.selected_columns = Some(columns.iter().map(|&s| s.to_string()).collect());
        self
    }

    pub fn where_eq<T: 'static + tokio_postgres::types::ToSql + Sync + Send>(
        mut self,
        column: &str,
        value: T,
    ) -> Self {
        let param_index = self.params.len() + 1;
        self.conditions
            .push(format!("{} = ${}", column, param_index));
        self.params.push(Box::new(value));
        self
    }

    #[allow(dead_code)]
    pub fn where_in<T: 'static + tokio_postgres::types::ToSql + Sync + Send>(
        mut self,
        column: &str,
        values: Vec<T>,
    ) -> Self {
        let param_indices: Vec<String> = (0..values.len())
            .map(|i| format!("${}", self.params.len() + i + 1))
            .collect();

        self.conditions
            .push(format!("{} IN ({})", column, param_indices.join(", ")));

        for value in values {
            self.params.push(Box::new(value));
        }
        self
    }

    pub fn order_by(mut self, column: &str, ascending: bool) -> Self {
        let direction = if ascending { "ASC" } else { "DESC" };
        self.order_by.push(format!("{} {}", column, direction));
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build_select(
        &self,
    ) -> (
        String,
        &Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>>,
    ) {
        let columns = match &self.selected_columns {
            Some(cols) => cols.join(", "),
            None => "*".to_string(),
        };

        let mut query = format!(
            "SELECT {} FROM {}.{}",
            columns, self.table.schema, self.table.name
        );

        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.conditions.join(" AND "));
        }

        if !self.order_by.is_empty() {
            query.push_str(" ORDER BY ");
            query.push_str(&self.order_by.join(", "));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        (query, &self.params)
    }

    pub fn build_insert<T: Serialize>(
        value: &T,
        schema: &DatabaseSchema,
        table_name: &str,
    ) -> Option<(
        String,
        Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>>,
    )> {
        let table = schema.tables.get(table_name)?;
        let value_map = serde_json::to_value(value).ok()?.as_object()?.clone();

        let mut columns = Vec::new();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut param_positions = Vec::new();

        for column in &table.columns {
            if let Some(value) = value_map.get(&column.name) {
                if !value.is_null() {
                    columns.push(column.name.clone());
                    param_positions.push(format!("${}", params.len() + 1));

                    // Convert serde_json::Value to postgres parameter
                    match value {
                        serde_json::Value::String(s) => params.push(Box::new(s.clone())),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                params.push(Box::new(i));
                            } else if let Some(f) = n.as_f64() {
                                params.push(Box::new(f));
                            }
                        }
                        serde_json::Value::Bool(b) => params.push(Box::new(*b)),
                        _ => continue,
                    }
                }
            }
        }

        let query = format!(
            "INSERT INTO {}.{} ({}) VALUES ({})",
            table.schema,
            table.name,
            columns.join(", "),
            param_positions.join(", ")
        );

        Some((query, params))
    }
}
