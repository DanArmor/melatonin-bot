use crate::config::MelatoninBotState;
use mobot::handler::State;
use mobot::*;

fn get_user_id(e: &Event) -> Result<i64, anyhow::Error> {
    Ok(e.update.from_user()?.id)
}

pub async fn start_handler(e: Event, s: State<MelatoninBotState>) -> Result<Action, anyhow::Error> {
    let id = get_user_id(&e)?;
    e.send_message("Здравствуйте, данный бот напоминает о стримах выбранных вами втуберов Nijisanji EN за 15-20 до начала стрима").await?;
    Ok(Action::Done)
}
