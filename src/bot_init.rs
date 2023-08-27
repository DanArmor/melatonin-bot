use crate::config::*;
use log::debug;
use std::fs;

// Initialize env_logger for the app
pub fn init_logger() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    debug!("Logger initialized.");
}

// Create new bot state from config
pub fn init_bot_state(config: Config) -> MelatoninBotState {
    let state = MelatoninBotState::new(config);
    debug!("Bot state initialized");
    state
}

// Read config from disk and create according struct
fn init_config() -> Config {
    let data = fs::read_to_string("dev.json").unwrap();
    debug!("Config was read from file");
    serde_json::from_str(&data).unwrap()
}

// Initialize entire app and return bot state
pub async fn init_app() -> anyhow::Result<MelatoninBotState> {
    debug!("Initializing app. . .");
    init_logger();
    let config = init_config();

    init_db(config.sql_connection_string.clone(), config.max_connections).await?;
    let state = init_bot_state(config);
    state.init_startup_data().await?;
    Ok(state)
}
