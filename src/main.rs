use log::{debug, error, info};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Sqlite;
use sqlx::{any, migrate::MigrateDatabase};
use std::fs;

mod bot_init;
mod config;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    bot_init::init_app().await?;

    Ok(())
}
