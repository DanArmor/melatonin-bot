use log::{debug, error, info};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Sqlite;
use sqlx::{any, migrate::MigrateDatabase};
use std::fs;

mod bot_init;
mod config;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create bot state
    let bot_state = bot_init::init_app().await?;
    // Create client for mobot
    let client = mobot::Client::new(bot_state.get_telegram_bot_token().into());

    Ok(())
}
