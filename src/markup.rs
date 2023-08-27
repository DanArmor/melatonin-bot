use mobot::*;
use sqlx::{Pool, Sqlite};

use crate::queries;

// Markup for selecting a wave
pub async fn waves_markup(pool: Pool<Sqlite>, tg_user_id: i64) -> api::ReplyMarkup {
    let waves = queries::get_amount_in_waves(pool, tg_user_id).await;
    match waves {
        Ok(waves) => api::ReplyMarkup::inline_keyboard_markup(
            waves
                .into_iter()
                .map(|x| {
                    vec![api::InlineKeyboardButton::from(format!(
                        "{} ({}/{})",
                        x.wave_name.clone(),
                        x.amount,
                        x.max_amount
                    ))
                    .with_callback_data(format!("wave_{}", x.wave_name))]
                })
                .collect(),
        ),
        Err(_) => {
            api::ReplyMarkup::inline_keyboard_markup(vec![vec![api::InlineKeyboardButton::from(
                "Извините, возникла ошибка. Попробуйте позже отправить команду /waves или /start",
            )]])
        }
    }
}

// Get status bedge for member
fn get_member_status_badge(is_selected: bool) -> &'static str {
    if is_selected {
        "✅"
    } else {
        ""
    }
}

// Markup for selecting members
pub async fn members_markup(
    pool: Pool<Sqlite>,
    tg_user_id: i64,
    wave_name: String,
) -> api::ReplyMarkup {
    let members = queries::get_wave_members(pool, tg_user_id, wave_name).await;
    match members {
        Ok(members) => {
            let mut members = members
                .into_iter()
                .map(|x| {
                    vec![api::InlineKeyboardButton::from(format!(
                        "{}{} {} {}",
                        get_member_status_badge(x.is_selected),
                        x.vtuber.first_name,
                        x.vtuber.last_name,
                        x.vtuber.emoji
                    ))
                    .with_callback_data(format!(
                        "member_{} {} wave_{}",
                        x.vtuber.first_name, x.vtuber.last_name, x.vtuber.wave_name
                    ))]
                })
                .collect::<Vec<_>>();
            members.push(vec![api::InlineKeyboardButton::from("Назад")
                .with_callback_data("member_back wave_none")]);
            api::ReplyMarkup::inline_keyboard_markup(members)
        }
        Err(_) => {
            api::ReplyMarkup::inline_keyboard_markup(vec![vec![api::InlineKeyboardButton::from(
                "Извините, возникла ошибка. Попробуйте позже отправить команду /waves или /start",
            )]])
        }
    }
}
