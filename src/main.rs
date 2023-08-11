use log::{error, info};
use mobot::API;

use mobot::handler::{BotState, State};

use std::sync::Arc;

mod bot_init;
mod config;
mod main_client;
mod queries;
mod vtuber;

async fn error_handler<S: BotState>(api: Arc<API>, chat_id: i64, _: State<S>, err: anyhow::Error) {
    error!("{}", err);
}

// TODO : enforce constrains for foreign keys for sqlite by hand

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create bot state
    let bot_state = bot_init::init_app().await?;
    let holodex_api_key = bot_state.get_holodex_api_key();
    let timer_duration_sec = bot_state.get_timer_duration_sec();
    // Create client for mobot
    let client = mobot::Client::new(bot_state.get_telegram_bot_token().into());
    let mut router = mobot::Router::<config::MelatoninBotState>::new(client)
        .with_error_handler(error_handler)
        .with_state(bot_state);
    let main_client = main_client::MainClient::new(
        router.api.clone(),
        Arc::new(holodex::Client::new(&holodex_api_key)?),
    );
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(timer_duration_sec));
        loop {
            interval.tick().await;
            info!("Hello from timer");
            let results = main_client.get_videos();
            for stream in results {
                match stream.channel {
                    holodex::model::VideoChannel::Id(id) => info!("Res: id {}", id),
                    holodex::model::VideoChannel::Min(min_info) => info!(
                        "Res: {} {} / {}",
                        stream.title,
                        min_info.name,
                        min_info
                            .english_name
                            .unwrap_or(String::from("no english name"))
                    ),
                }
            }
        }
    });
    router.start().await;
    Ok(())
}
