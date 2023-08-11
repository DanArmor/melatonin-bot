use mobot::*;
use sqlx::{Pool, Sqlite};

use crate::queries;

pub async fn waves_markup(pool: Pool<Sqlite>) -> api::ReplyMarkup {
    let res = queries::get_waves_names(pool).await;
    match res {
        Ok(waves) => api::ReplyMarkup::inline_keyboard_markup(
            waves
                .into_iter()
                .map(|x| {
                    vec![api::InlineKeyboardButton::from(x.clone())
                        .with_callback_data(format!("wave_{}", x))]
                })
                .collect(),
        ),
        Err(e) => {
            api::ReplyMarkup::inline_keyboard_markup(vec![vec![api::InlineKeyboardButton::from(
                "test",
            )]])
        }
    }
}

pub async fn members_markup(pool: Pool<Sqlite>, wave_name: String) -> api::ReplyMarkup {
    let res = queries::get_wave_members(pool, wave_name).await;
    match res {
        Ok(members) => {
            let mut members = members
                .into_iter()
                .map(|x| {
                    vec![api::InlineKeyboardButton::from(format!(
                        "{} {} {}",
                        x.first_name, x.last_name, x.emoji
                    ))
                    .with_callback_data(format!("member_{}{}", x.first_name, x.last_name))]
                })
                .collect::<Vec<_>>();
            members.push(vec![
                api::InlineKeyboardButton::from("Назад").with_callback_data("member_back")
            ]);
            api::ReplyMarkup::inline_keyboard_markup(members)
        }
        Err(e) => {
            api::ReplyMarkup::inline_keyboard_markup(vec![vec![api::InlineKeyboardButton::from(
                "test",
            )]])
        }
    }
}
