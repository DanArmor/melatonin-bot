use handlers::{report_action, start_handler};
use log::{error, info};
use main_client::MainClient;
use mobot::API;

use mobot::api::BotCommand;
use mobot::handler::{BotState, State};

use std::sync::Arc;

mod bot_init;
mod config;
mod handlers;
mod main_client;
mod markup;
mod queries;
mod vtuber;
mod user;

async fn error_handler<S: BotState>(api: Arc<API>, chat_id: i64, _: State<S>, err: anyhow::Error) {
    error!("{}", err);
}


async fn notify_users(main_client: MainClient, timer_duration_sec: u64) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(timer_duration_sec));
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
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create bot state
    let bot_state = bot_init::init_app().await?;
    let holodex_api_key = bot_state.get_holodex_api_key();
    let timer_duration_sec = bot_state.get_timer_duration_sec();
    // Create client for mobot

    let client = mobot::Client::new(bot_state.get_telegram_bot_token());
    let mut router = mobot::Router::<config::MelatoninBotState>::new(client)
        .with_error_handler(error_handler)
        .with_state(bot_state);
    
    let commands = vec![BotCommand {
        command: "waves".into(),
        description: "Show list of waves".into(),
    }];
    router
        .api
        .set_my_commands(&mobot::api::SetMyCommandsRequest {
            commands,
            ..Default::default()
        })
        .await
        .unwrap();

    let main_client = main_client::MainClient::new(
        router.api.clone(),
        Arc::new(holodex::Client::new(&holodex_api_key)?),
    );

    router
        .add_route(
            mobot::Route::Message(mobot::Matcher::BotCommand(String::from("start"))),
            |e, s| async move { report_action(e, s, "start_handler").await },
        )
        .add_route(
            mobot::Route::Message(mobot::Matcher::BotCommand(String::from("start"))),
            crate::handlers::start_handler,
        );
    router
        .add_route(
            mobot::Route::Message(mobot::Matcher::BotCommand(String::from("waves"))),
            |e, s| async move { report_action(e, s, "info_handler").await },
        )
        .add_route(
            mobot::Route::Message(mobot::Matcher::BotCommand(String::from("waves"))),
            crate::handlers::info_handler,
        );
    router
        .add_route(
            mobot::Route::CallbackQuery(mobot::Matcher::Prefix(String::from("wave_"))),
            |e, s| async move { report_action(e, s, "wave_request").await },
        )
        .add_route(
            mobot::Route::CallbackQuery(mobot::Matcher::Prefix(String::from("wave_"))),
            crate::handlers::wave_handler,
        );
    router
        .add_route(
            mobot::Route::CallbackQuery(mobot::Matcher::Prefix(String::from("member_"))),
            |e, s| async move { report_action(e, s, "member_request").await },
        )
        .add_route(
            mobot::Route::CallbackQuery(mobot::Matcher::Prefix(String::from("member_"))),
            crate::handlers::member_handler,
        );
    // mobot::Route::Default doesn't work in that case
    // router
    //     .add_route(mobot::Route::Message(mobot::Matcher::Any), |e, s| async move {
    //         report_action(e, s, "any_handler").await
    //     })
    //     .add_route(mobot::Route::Message(mobot::Matcher::Any), crate::handlers::any_handler);

    tokio::spawn(notify_users(main_client, timer_duration_sec));
    router.start().await;
    Ok(())
}
