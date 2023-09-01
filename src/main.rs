use handlers::report_action;
use log::{debug, error, info};
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
mod reported_stream;
mod user;
mod vtuber;

// Basic error handler
async fn error_handler<S: BotState>(_: Arc<API>, _: i64, _: State<S>, err: anyhow::Error) {
    error!("{}", err);
}

// Fetch streams in interval and notify users, when some stream will start soon
async fn notify_users(main_client: MainClient, timer_duration_sec: u64) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(timer_duration_sec));
    loop {
        interval.tick().await;
        info!("Fetching videos");

        let videos = match main_client.associate_video_vtuber().await {
            Ok(v) => v,
            Err(e) => {
                error!("Error during fetching: {}", e);
                main_client.send_alert(e).await;
                continue;
            }
        };
        info!("Final fetched amount: {}", videos.len());
        for stream in videos {
            match stream.video.channel.clone() {
                holodex::model::VideoChannel::Id(id) => debug!("Fetched stream(id): {}", id),
                holodex::model::VideoChannel::Min(min_info) => debug!(
                    "Fetched stream(min-info): {} / {}",
                    min_info.name,
                    min_info
                        .english_name
                        .unwrap_or(String::from("no english_name"))
                ),
            }

            match queries::is_stream_reported(main_client.get_pool(), &stream.video)
                .await
                .unwrap()
            {
                Some(_) => (),
                None => main_client.send_notification(stream).await,
            }
        }
        main_client.clean_reported_streams().await;
    }
}

//TODO: cleanup
//TODO: more log

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create bot state
    let bot_state = bot_init::init_app().await?;
    let holodex_api_key = bot_state.get_holodex_api_key();
    let timer_duration_sec = bot_state.get_timer_duration_sec();
    let monitoring_ip = bot_state.get_monitoring_ip();
    let alert_client = Arc::new(reqwest::Client::new());

    // Create client for mobot
    let client = mobot::Client::new(bot_state.get_telegram_bot_token());
    let mut router = mobot::Router::<config::MelatoninBotState>::new(client)
        .with_error_handler(error_handler)
        .with_state(bot_state);

    let commands = vec![
        BotCommand {
            command: "waves".into(),
            description: "Show list of waves".into(),
        },
        BotCommand {
            command: "about".into(),
            description: "Information about the bot".into(),
        },
        BotCommand {
            command: "start".into(),
            description: "Start of the conversation".into(),
        },
    ];
    // Create router
    router
        .api
        .set_my_commands(&mobot::api::SetMyCommandsRequest {
            commands,
            ..Default::default()
        })
        .await
        .unwrap();
    info!("Setuped router");

    // Create client for fetching videos and notifying users
    let main_client = main_client::MainClient::new(
        router.api.clone(),
        Arc::new(holodex::Client::new(&holodex_api_key)?),
        monitoring_ip,
        alert_client.clone(),
    );

    // Add routes
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
            mobot::Route::Message(mobot::Matcher::BotCommand(String::from("about"))),
            |e, s| async move { report_action(e, s, "about_handler").await },
        )
        .add_route(
            mobot::Route::Message(mobot::Matcher::BotCommand(String::from("about"))),
            crate::handlers::about_handler,
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
    info!("Routes were added");
    // Start notify-thread
    tokio::spawn(notify_users(main_client, timer_duration_sec));
    info!("Fetching thread was started");
    // Start bot
    info!("Bot was started");
    router.start().await;
    Ok(())
}
