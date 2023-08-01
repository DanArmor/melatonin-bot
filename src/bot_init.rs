use crate::config::*;
use log::{debug, error, info};
use std::fs;

pub fn init_logger() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    debug!("Logger initialized.");
}

pub fn init_bot_state(config: Config) -> MelatoninBotState {
    let state = MelatoninBotState::new(config);
    state
}

fn init_config() -> Config {
    let data = fs::read_to_string("dev.json").unwrap();
    serde_json::from_str(&data).unwrap()
}

pub async fn init_app() -> anyhow::Result<MelatoninBotState> {
    init_logger();
    let config = init_config();

    init_db(config.sql_connection_string.clone(), config.max_connections).await?;
    let state = init_bot_state(config);
    Ok(state)
}
