use super::super::schema::DatabaseSchema;
use serde::Serialize;
use tokio_postgres::types::ToSql;

pub struct QueryBuilder<'a> {
    pub schema: &'a DatabaseSchema,
    pub table: String,
    conditions: Vec<(String, Box<dyn ToSql + Sync + Send>)>,
    selected_columns: Option<Vec<String>>,
    order_by: Vec<(String, bool)>,
    limit: Option<i64>,
    offset: Option<i64>,
    joins: Vec<JoinClause>,
}

#[derive(Debug)]
pub struct JoinClause {
    table: String,
    alias: Option<String>,
    kind: JoinType,
    conditions: Vec<(String, String)>, // (left_column, right_column)
}

#[derive(Debug)]
pub enum JoinType {
    Inner,
    Left,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(schema: &'a DatabaseSchema, table: &str) -> Option<Self> {
        let table_exists = schema.tables.contains_key(table);
        if !table_exists {
            return None;
        }

        Some(QueryBuilder {
            schema,
            table: table.to_string(),
            conditions: Vec::new(),
            selected_columns: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
            joins: Vec::new(),
        })
    }

    pub fn get_column_type(&self, column_name: &str) -> Option<String> {
        self.schema.tables.get(&self.table).and_then(|table| {
            table
                .columns
                .iter()
                .find(|col| col.name == column_name)
                .map(|col| col.type_name.clone())
        })
    }

    pub fn select(mut self, columns: &[&str]) -> Self {
        self.selected_columns = Some(columns.iter().map(|&s| s.to_string()).collect());
        self
    }

    pub fn where_eq<T: 'static + tokio_postgres::types::ToSql + Sync + Send>(
        mut self,
        column: &str,
        value: Option<T>,
    ) -> Self {
        match value {
            Some(v) => {
                let param_index = self.conditions.len() + 1;
                self.conditions.push((
                    format!("{}.{} = ${}", self.table, column, param_index),
                    Box::new(v),
                ));
            }
            None => {
                self.conditions
                    .push((format!("{}.{} IS NULL", self.table, column), Box::new("")));
            }
        }
        self
    }

    #[allow(dead_code)]
    pub fn where_in<T: 'static + tokio_postgres::types::ToSql + Sync + Send>(
        mut self,
        column: &str,
        values: Vec<T>,
    ) -> Self {
        let param_indices: Vec<String> = (0..values.len())
            .map(|i| format!("${}", self.conditions.len() + i + 1))
            .collect();

        self.conditions.push((
            format!("{} IN ({})", column, param_indices.join(", ")),
            Box::new(values),
        ));

        self
    }

    pub fn order_by(mut self, column: &str, ascending: bool) -> Self {
        self.order_by.push((column.to_string(), ascending));
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

    pub fn inner_join(
        mut self,
        table: &str,
        left_column: &str,
        right_column: &str,
        alias: Option<String>,
    ) -> Self {
        self.joins.push(JoinClause {
            table: table.to_string(),
            alias,
            kind: JoinType::Inner,
            conditions: vec![(left_column.to_string(), right_column.to_string())],
        });
        self
    }

    pub fn left_join(
        mut self,
        table: &str,
        left_column: &str,
        right_column: &str,
        alias: Option<String>,
    ) -> Self {
        self.joins.push(JoinClause {
            table: table.to_string(),
            alias,
            kind: JoinType::Left,
            conditions: vec![(left_column.to_string(), right_column.to_string())],
        });
        self
    }

    pub fn build_select(&mut self) -> (String, Vec<Box<dyn ToSql + Sync + Send>>) {
        let mut query = String::from("SELECT ");

        if let Some(ref columns) = self.selected_columns {
            let column_list: Vec<String> = columns
                .iter()
                .map(|c| {
                    if c.contains('.') || c.to_lowercase().contains(" as ") {
                        c.clone()
                    } else {
                        format!("{}.{}", self.table, c)
                    }
                })
                .collect();
            query.push_str(&column_list.join(", "));
        } else {
            let mut all_columns = vec![format!("{}.* ", self.table)];

            for join in &self.joins {
                let table_name = join.alias.as_ref().unwrap_or(&join.table);
                all_columns.push(format!("{}.* ", table_name));
            }

            query.push_str(&all_columns.join(", "));
        }

        query.push_str(&format!(" FROM {}", self.table));

        for join in &self.joins {
            let join_type = match join.kind {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
            };

            let table_alias = if let Some(ref alias) = join.alias {
                format!(" AS {}", alias)
            } else {
                String::new()
            };

            query.push_str(&format!(" {} {}{}", join_type, join.table, table_alias));

            if !join.conditions.is_empty() {
                let join_conditions: Vec<String> = join
                    .conditions
                    .iter()
                    .map(|(left, right)| {
                        format!(
                            "{}.{} = {}.{}",
                            self.table,
                            left,
                            join.alias.as_ref().unwrap_or(&join.table),
                            right
                        )
                    })
                    .collect();
                query.push_str(" ON ");
                query.push_str(&join_conditions.join(" AND "));
            }
        }

        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .conditions
                .iter()
                .map(|(condition, _)| condition.clone())
                .collect();
            query.push_str(&conditions.join(" AND "));
        }

        if !self.order_by.is_empty() {
            query.push_str(" ORDER BY ");
            let order_clauses: Vec<String> = self
                .order_by
                .iter()
                .map(|(column, asc)| {
                    format!(
                        "{}.{} {}",
                        self.table,
                        column,
                        if *asc { "ASC" } else { "DESC" }
                    )
                })
                .collect();
            query.push_str(&order_clauses.join(", "));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let params = std::mem::take(&mut self.conditions)
            .into_iter()
            .filter(|(condition, _)| !condition.ends_with("IS NULL"))
            .map(|(_, v)| v)
            .collect();

        (query, params)
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
