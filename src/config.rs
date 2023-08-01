use log::{debug, error, info};
use mobot::BotState;
use serde::Deserialize;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};
use tokio::sync::OnceCell;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub holodex_api_key: String,
    pub telegram_bot_token: String,
    pub sql_connection_string: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Default, BotState)]
pub struct MelatoninBotState {
    config: Config,
    sql_pool: MyPool,
}

impl MelatoninBotState {
    pub fn new(config: Config) -> Self {
        MelatoninBotState {
            config: config,
            sql_pool: MyPool::default(),
        }
    }
    pub fn get_pool(&self) -> Pool<Sqlite> {
        self.sql_pool.0.clone()
    }
}

static POOL: OnceCell<Pool<Sqlite>> = OnceCell::const_new();

pub async fn init_db(db_path: String, max_conn: u32) -> Result<(), anyhow::Error> {
    if !Sqlite::database_exists(&db_path).await.unwrap_or(false) {
        match Sqlite::create_database(&db_path).await {
            Ok(_) => info!("{} was created", db_path),
            Err(e) => error!("{}", e),
        }
    } else {
        info!("{} already exists", db_path)
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(max_conn)
        .connect(&db_path)
        .await?;
    POOL.set(pool)?;
    Ok(())
}

#[derive(Debug, Clone)]
struct MyPool(Pool<Sqlite>);

unsafe impl Send for MyPool {}
unsafe impl Sync for MyPool {}

impl Default for MyPool {
    fn default() -> Self {
        MyPool(POOL.get().unwrap().clone())
    }
}
