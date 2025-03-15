use deadpool_postgres::{CreatePoolError, Manager, Pool};
use dotenvy::dotenv;
use futures::future::BoxFuture;
use serde::Deserialize;
use tokio_postgres::NoTls;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub max_connections: usize,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenv().ok();

        Ok(DatabaseConfig {
            host: std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("POSTGRES_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            user: std::env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: std::env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set"),
            database: std::env::var("POSTGRES_DB").expect("POSTGRES_DB must be set"),
            max_connections: std::env::var("POSTGRES_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database configuration error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Database pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("Database error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("Pool creation error: {0}")]
    CreatePool(#[from] CreatePoolError),
    #[error("Build error: {0}")]
    Build(String),
}

pub type DbResult<T> = Result<T, DbError>;

pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn new() -> DbResult<Self> {
        let config = DatabaseConfig::from_env()?;
        
        let pool_config = tokio_postgres::config::Config::new()
            .host(&config.host)
            .port(config.port)
            .user(&config.user)
            .password(&config.password)
            .dbname(&config.database)
            .to_owned();

        let mgr = Manager::new(pool_config, NoTls);
        let pool = Pool::builder(mgr)
            .max_size(config.max_connections)
            .build()
            .map_err(|e| DbError::Build(e.to_string()))?;

        let client = pool.get().await.map_err(|e| DbError::Pool(e))?;
        client.query("SELECT 1", &[]).await.map_err(|e| {
            eprintln!("Database connection test failed: {}", e);
            eprintln!("Please check your database credentials and ensure the database is running.");
            DbError::Postgres(e)
        })?;

        println!("Successfully connected to PostgreSQL!");
        
        Ok(Self { pool })
    }

    pub async fn get_client(&self) -> DbResult<deadpool_postgres::Client> {
        Ok(self.pool.get().await?)
    }

    #[allow(dead_code)]
    pub async fn with_connection<'a, F, T, E>(&'a self, operation: F) -> Result<T, E>
    where
        F: FnOnce(deadpool_postgres::Client) -> BoxFuture<'a, Result<T, E>>,
        E: From<DbError>,
    {
        let client = self.get_client().await?;
        operation(client).await
    }
}
