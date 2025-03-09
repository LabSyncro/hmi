use std::sync::Arc;
use tokio::sync::Mutex;
use crate::db::connection::Database;
use crate::db::schema::DatabaseSchema;

#[allow(dead_code)]
pub struct AppState {
    pub db: Database,
    pub schema: Arc<Mutex<Option<DatabaseSchema>>>,
}

pub mod db_commands; 