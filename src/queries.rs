use sqlx::error;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::vtuber;

pub async fn check_vtuber_exist(
    pool: Pool<Sqlite>,
    member: &vtuber::Vtuber,
) -> Result<bool, anyhow::Error> {
    match sqlx::query!(
        r#"SELECT first_name FROM vtuber
        WHERE first_name = ? AND last_name = ?"#,
        member.first_name,
        member.last_name
    )
    .fetch_one(&pool)
    .await
    {
        Ok(_) => Ok(true),
        Err(e) => match e {
            error::Error::RowNotFound => Ok(false),
            _ => Err(e.into()),
        },
    }
}

pub async fn insert_vtuber(
    pool: Pool<Sqlite>,
    member: &vtuber::Vtuber,
) -> Result<(), anyhow::Error> {
    match sqlx::query!(
        r#"INSERT INTO vtuber (first_name, last_name, emoji, wave_name, youtube_handle, youtube_channel_id)
        VALUES (?, ?, ?, ?, ?, ?)"#,
        member.first_name,
        member.last_name,
        member.wave_name,
        member.emoji,
        member.youtube_handle,
        member.youtube_channel_id
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}
