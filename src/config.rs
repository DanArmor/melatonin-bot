use log::{debug, error, info};
use mobot::BotState;
use serde::Deserialize;

use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Pool;
use sqlx::Sqlite;

use std::collections::HashMap;
use std::fs;
use tokio::sync::OnceCell;

use crate::queries;
use crate::queries::insert_vtuber;
use crate::vtuber;

// Config to keep secrets and stuff
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub holodex_api_key: String,
    pub telegram_bot_token: String,
    pub sql_connection_string: String,
    pub startup_data_path: String,
    pub max_connections: u32,
    pub timer_duration_sec: u64,
}

// Bot state, containts config data and pool of connections
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
    pub fn get_telegram_bot_token(&self) -> String {
        self.config.telegram_bot_token.clone()
    }
    pub fn get_holodex_api_key(&self) -> String {
        self.config.holodex_api_key.clone()
    }
    pub fn get_timer_duration_sec(&self) -> u64 {
        self.config.timer_duration_sec.clone()
    }
    pub fn get_pool(&self) -> Pool<Sqlite> {
        self.sql_pool.0.clone()
    }
    pub async fn init_startup_data(&self) -> Result<(), anyhow::Error> {
        let str_data = fs::read_to_string(self.config.startup_data_path.clone()).unwrap();
        let data: HashMap<String, serde_json::Value> = serde_json::from_str(&str_data).unwrap();
        let data: Vec<vtuber::VtuberWave> = serde_json::from_value(data["waves"].clone()).unwrap();
        for wave in data {
            for mut member in wave.members {
                member.wave_name = wave.name.clone();
                match queries::check_vtuber_exist(self.get_pool(), &member).await {
                    Ok(is_exist) => match is_exist {
                        true => (),
                        false => insert_vtuber(self.get_pool(), &member).await?,
                    },
                    Err(e) => panic!("{}", e),
                }
            }
        }
        Ok(())
    }
}

// Singleton to keep connections for all threads. Can't put into bot state directly, because
// we need to implement Default trait for all fields of state
static POOL: OnceCell<Pool<Sqlite>> = OnceCell::const_new();

// Initialize database with given path and max connection amount
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
    sqlx::migrate!("./migrations").run(&pool).await?;
    POOL.set(pool)?;
    Ok(())
}

// Wrapper for connection pool (Pool<Sqlite>) to provide Default
// trait implementation
#[derive(Debug, Clone)]
struct MyPool(Pool<Sqlite>);

unsafe impl Send for MyPool {}
unsafe impl Sync for MyPool {}

impl Default for MyPool {
    fn default() -> Self {
        MyPool(POOL.get().unwrap().clone())
    }
}
