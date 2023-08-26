use anyhow::anyhow;
use sqlx::error;
use sqlx::sqlite::SqliteRow;
use sqlx::Pool;
use sqlx::Row;
use sqlx::Sqlite;

use crate::vtuber;
use crate::vtuber::Vtuber;

pub async fn insert_user(
    pool: Pool<Sqlite>,
    user: &mobot::api::User,
    chat_id: i64,
) -> Result<(), anyhow::Error> {
    match sqlx::query!(
        r#"INSERT INTO user (first_name, last_name, username, tg_user_id, tg_chat_id)
        VALUES (?, ?, ?, ?, ?)"#,
        user.first_name,
        user.last_name,
        user.username,
        user.id,
        chat_id
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(e)),
    }
}

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

pub struct UserVtubers {
    pub vtuber: vtuber::Vtuber,
    pub is_selected: bool,
}

fn user_vtubers_from_row(row: SqliteRow) -> UserVtubers {
    let vtuber = Vtuber {
        id: row.get("id"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        wave_name: row.get("wave_name"),
        emoji: row.get("emoji"),
        youtube_channel_id: row.get("youtube_channel_id"),
        youtube_handle: row.get("youtube_handle"),
    };
    UserVtubers {
        vtuber: vtuber,
        is_selected: row.get("is_selected"),
    }
}

pub async fn get_wave_members(
    pool: Pool<Sqlite>,
    tg_user_id: i64,
    wave_name: String,
) -> Result<Vec<UserVtubers>, anyhow::Error> {
    match sqlx::query(
        r#"SELECT
            vtuber.id AS "id",
            vtuber.first_name,
            vtuber.last_name,
            vtuber.wave_name,
            vtuber.emoji,
            vtuber.youtube_channel_id,
            vtuber.youtube_handle,
            IIF(user_vtuber.user_id IS NOT NULL, true, false) AS "is_selected" 
        FROM
            vtuber
            LEFT JOIN (
            SELECT
                *
            FROM
                user_vtuber
            WHERE
                user_vtuber.user_id = ?
            ) user_vtuber ON vtuber.id = user_vtuber.vtuber_id
        WHERE
            vtuber.wave_name = ?;"#,
    )
    .bind(tg_user_id)
    .bind(wave_name)
    .fetch_all(&pool)
    .await
    {
        Ok(members) => Ok(members.into_iter().map(user_vtubers_from_row).collect()),
        Err(e) => Err(anyhow!(e)),
    }
}

pub async fn update_user_vtuber(
    pool: Pool<Sqlite>,
    tg_user_id: i64,
    vtuber_id: i64,
) -> Result<(), anyhow::Error> {
    match sqlx::query!(
        r#"SELECT id FROM user_vtuber
        WHERE user_id = ? AND vtuber_id = ?"#,
        tg_user_id,
        vtuber_id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => {
            sqlx::query!(r#"DELETE FROM user_vtuber WHERE id = ?"#, row.id)
                .execute(&pool)
                .await?;
            Ok(())
        }
        Err(e) => match e {
            error::Error::RowNotFound => {
                sqlx::query!(
                    r#"INSERT INTO user_vtuber (user_id, vtuber_id) VALUES (?, ?)"#,
                    tg_user_id,
                    vtuber_id
                )
                .execute(&pool)
                .await?;
                Ok(())
            }
            _ => Err(e.into()),
        },
    }
}
