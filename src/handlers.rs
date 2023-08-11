use crate::config::MelatoninBotState;
use crate::markup::{self, members_markup};
use log::info;
use mobot::api::{EditMessageReplyMarkupRequest, EditMessageTextRequest, SendMessageRequest};
use mobot::handler::State;
use mobot::*;
use std::future::Future;

fn get_user_id(e: &Event) -> Result<i64, anyhow::Error> {
    Ok(e.update.from_user()?.id)
}

pub fn report_action<'a>(
    e: Event,
    s: State<MelatoninBotState>,
    text: &'a str,
) -> impl Future<Output = Result<Action, anyhow::Error>> + 'a {
    (|e: Event, s: State<MelatoninBotState>| async move {
        let id = get_user_id(&e)?;
        info!("UserID<{}>. Action<{}>", id, text);
        Ok(Action::Next)
    })(e, s)
}

pub async fn start_handler(e: Event, s: State<MelatoninBotState>) -> Result<Action, anyhow::Error> {
    let id = get_user_id(&e)?;
    e
    .api
    .send_message(
        &SendMessageRequest::new(e.update.chat_id()?, "Здравствуйте, данный бот напоминает о стримах выбранных вами втуберов Nijisanji EN за 15-20 минут до начала стрима. Выберите волну")
    .with_reply_markup(markup::waves_markup(s.get().read().await.get_pool()).await)).await?;
    Ok(Action::Done)
}

pub async fn any_handler(e: Event, s: State<MelatoninBotState>) -> Result<Action, anyhow::Error> {
    let id = get_user_id(&e)?;
    e.api
        .send_message(&SendMessageRequest::new(
            e.update.chat_id()?,
            "Команда не распознана",
        ))
        .await?;
    Ok(Action::Done)
}

pub async fn info_handler(e: Event, s: State<MelatoninBotState>) -> Result<Action, anyhow::Error> {
    let id = get_user_id(&e)?;
    e
    .api
    .send_message(
        &SendMessageRequest::new(e.update.chat_id()?, "Данный бот напоминает о стримах выбранных вами втуберов Nijisanji EN за 15-20 минут до начала стрима. Выберите волну")
    .with_reply_markup(markup::waves_markup(s.get().read().await.get_pool()).await)).await?;
    Ok(Action::Done)
}

// "back" => {
//     e.api
//     .edit_message_text(
//         &EditMessageTextRequest::new(String::from("Данный бот напоминает о стримах выбранных вами втуберов Nijisanji EN за 15-20 до начала стрима. Выберите волну"))
//             .with_chat_id(e.update.chat_id()?)
//             .with_message_id(e.update.message_id()?),
//     )
//     .await?;
//     e.api
//         .edit_message_reply_markup(
//             &EditMessageReplyMarkupRequest::new(
//                 markup::waves_markup(s.get().read().await.get_pool()).await,
//             )
//             .with_chat_id(e.update.chat_id()?)
//             .with_message_id(e.update.message_id()?),
//         )
//         .await?;
// }

pub async fn wave_handler(e: Event, s: State<MelatoninBotState>) -> Result<Action, anyhow::Error> {
    let id = get_user_id(&e)?;
    let wave_name = e.update.get_callback_query()?.data.clone().unwrap();
    let wave_name = wave_name.strip_prefix("wave_").unwrap();
    e.api
        .edit_message_text(
            &EditMessageTextRequest::new(String::from("Выберите втубера"))
                .with_chat_id(e.update.chat_id()?)
                .with_message_id(e.update.message_id()?),
        )
        .await?;
    e.api
        .edit_message_reply_markup(
            &EditMessageReplyMarkupRequest::new(
                members_markup(s.get().read().await.get_pool(), String::from(wave_name)).await,
            )
            .with_chat_id(e.update.chat_id()?)
            .with_message_id(e.update.message_id()?),
        )
        .await?;
    Ok(Action::Done)
}

pub async fn member_handler(
    e: Event,
    s: State<MelatoninBotState>,
) -> Result<Action, anyhow::Error> {
    let id = get_user_id(&e)?;
    e
    .api
    .send_message(
        &SendMessageRequest::new(e.update.chat_id()?, "Здравствуйте, данный бот напоминает о стримах выбранных вами втуберов Nijisanji EN за 15-20 минут до начала стрима")
    .with_reply_markup(markup::waves_markup(s.get().read().await.get_pool()).await)).await?;
    Ok(Action::Done)
}
