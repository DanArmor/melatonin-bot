use crate::config::MelatoninBotState;
use crate::markup::{self, members_markup};
use crate::queries;
use anyhow::anyhow;
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
    .with_reply_markup(markup::waves_markup(s.get().read().await.get_pool(), id).await)).await?;
    let pool = s.get().read().await.get_pool();
    let user = e.update.from_user().unwrap();
    match queries::insert_user(pool, user, e.update.chat_id().unwrap()).await {
        Ok(_) => Ok(Action::Done),
        Err(e) => Err(anyhow!(e)),
    }
}

pub async fn about_handler(e: Event, s: State<MelatoninBotState>) -> Result<Action, anyhow::Error> {
    let id = get_user_id(&e)?;
    e.api
        .send_message(&SendMessageRequest::new(
            e.update.chat_id()?,
            "Бот, напоминающий о стримах выбранных втуберов NijiEN за 15-20 минут до начала\n\
            Жалобы/предложения - @DanArmor\n\
            Код бота: https://github.com/DanArmor/melatonin-bot\n\
            Если что-то не работает - попробуйте команду /start\n\
            Если и это не помогло - напишите админу",
        ))
        .await?;
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
    .with_reply_markup(markup::waves_markup(s.get().read().await.get_pool(), id).await)).await?;
    Ok(Action::Done)
}

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
                members_markup(s.get().read().await.get_pool(), id, String::from(wave_name)).await,
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
    let data = e.update.get_callback_query()?.data.clone().unwrap();
    let (member_name, wave_name) = data
        .strip_prefix("member_")
        .unwrap()
        .split_once(" wave_")
        .unwrap();
    match member_name {
        "back" => {
            e.api
            .edit_message_text(
                &EditMessageTextRequest::new(String::from("Данный бот напоминает о стримах выбранных вами втуберов Nijisanji EN за 15-20 до начала стрима. Выберите волну"))
                    .with_chat_id(e.update.chat_id()?)
                    .with_message_id(e.update.message_id()?),
            )
            .await?;
            e.api
                .edit_message_reply_markup(
                    &EditMessageReplyMarkupRequest::new(
                        markup::waves_markup(s.get().read().await.get_pool(), id).await,
                    )
                    .with_chat_id(e.update.chat_id()?)
                    .with_message_id(e.update.message_id()?),
                )
                .await?;
        }
        _ => {
            let (first_name, last_name) = member_name.split_once(" ").unwrap();
            let pool = s.get().read().await.get_pool();
            let vtuber_id = sqlx::query!(
                "SELECT id FROM vtuber WHERE first_name = ? AND last_name = ?",
                first_name,
                last_name
            )
            .fetch_one(&pool)
            .await
            .unwrap()
            .id;
            queries::update_user_vtuber(pool, id, vtuber_id).await?;
            e.api
                .edit_message_reply_markup(
                    &EditMessageReplyMarkupRequest::new(
                        markup::members_markup(
                            s.get().read().await.get_pool(),
                            id,
                            String::from(wave_name),
                        )
                        .await,
                    )
                    .with_chat_id(e.update.chat_id()?)
                    .with_message_id(e.update.message_id()?),
                )
                .await?;
        }
    }
    Ok(Action::Done)
}
