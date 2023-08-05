use log::{debug, error, info};
use mobot::API;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Sqlite;
use sqlx::{any, migrate::MigrateDatabase};
use std::fs;

use mobot::handler::{BotState, State};

use std::sync::Arc;

mod bot_init;
mod config;

async fn error_handler<S: BotState>(api: Arc<API>, chat_id: i64, _: State<S>, err: anyhow::Error) {
    error!("{}", err);
}

// TODO : enforce constrains for foreign keys for sqlite by hand

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create bot state
    let bot_state = bot_init::init_app().await?;
    // Create client for mobot
    let client = mobot::Client::new(bot_state.get_telegram_bot_token().into());
    let mut router = mobot::Router::<config::MelatoninBotState>::new(client)
        .with_error_handler(error_handler)
        .with_state(bot_state);

    Ok(())
}
