use anyhow::anyhow;
use sqlx::error;
use sqlx::Pool;
use sqlx::Row;
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
        member.emoji,
        member.wave_name,
        member.youtube_handle,
        member.youtube_channel_id
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(e)),
    }
}

pub async fn get_waves_names(pool: Pool<Sqlite>) -> Result<Vec<String>, anyhow::Error> {
    match sqlx::query(r#"SELECT wave_name as name FROM vtuber GROUP BY wave_name"#)
        .fetch_all(&pool)
        .await
    {
        Ok(members) => Ok(members
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect()),
        Err(e) => Err(anyhow!(e)),
    }
}

pub async fn get_wave_members(
    pool: Pool<Sqlite>,
    wave_name: String,
) -> Result<Vec<vtuber::Vtuber>, anyhow::Error> {
    match sqlx::query_as!(
        vtuber::Vtuber,
        r#"SELECT * FROM vtuber WHERE wave_name = ?"#,
        wave_name
    )
    .fetch_all(&pool)
    .await
    {
        Ok(members) => Ok(members),
        Err(e) => Err(anyhow!(e)),
    }
}
