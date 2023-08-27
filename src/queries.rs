use anyhow::anyhow;
use lazy_static::lazy_static;
use sqlx::error;
use sqlx::sqlite::SqliteRow;
use sqlx::Pool;
use sqlx::Row;
use sqlx::Sqlite;
use std::collections::HashMap;

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

pub async fn check_reported_stream(
    pool: Pool<Sqlite>,
    video: &holodex::model::Video,
) -> Result<Option<crate::reported_stream::ReportedStream>, anyhow::Error> {
    let video_id = video.id.to_string();
    match sqlx::query_as!(
        crate::reported_stream::ReportedStream,
        r#"SELECT * FROM reported_stream
        WHERE video_id = ?"#,
        video_id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(stream) => Ok(Some(stream)),
        Err(e) => match e {
            error::Error::RowNotFound => Ok(None),
            _ => Err(e.into()),
        },
    }
}

pub async fn insert_reported_stream(
    pool: Pool<Sqlite>,
    video: &holodex::model::Video,
    vtuber: &vtuber::Vtuber,
) -> Result<(), anyhow::Error> {
    let video_id = video.id.to_string();
    let scheduled_time = video.available_at.naive_utc();
    match sqlx::query!(
        "INSERT INTO reported_stream (video_id, vtuber_id, scheduled_start) VALUES (?, ?, ?)",
        video_id,
        vtuber.id,
        scheduled_time
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(e)),
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

#[derive(Debug)]
pub struct WaveAmount {
    pub wave_name: String,
    pub amount: i64,
    pub max_amount: i64,
}

// TODO: find some other solution
// Map waves to debute order
lazy_static! {
    static ref NijiENMap: HashMap<&'static str, i64> = vec![
        ("LazuLight", 1),
        ("OBSYDIA", 2),
        ("Ethyria", 3),
        ("Luxiem", 4),
        ("Noctyx", 5),
        ("ILUNA", 6),
        ("XSOLEIL", 7),
        ("Krisis", 8),
    ]
    .into_iter()
    .collect();
}

pub async fn get_amount_in_waves(
    pool: Pool<Sqlite>,
    tg_user_id: i64,
) -> Result<Vec<WaveAmount>, anyhow::Error> {
    match sqlx::query(
        r#"
    SELECT
        vtuber.wave_name,
        COUNT(user_vtuber.id) as 'amount',
        COUNT(vtuber.id) as 'max_amount'
    FROM
        vtuber
        LEFT JOIN (
        SELECT
            *
        FROM
            user_vtuber
        WHERE
            user_vtuber.user_id = ?
        ) user_vtuber ON user_vtuber.vtuber_id = vtuber.id
    GROUP BY
        vtuber.wave_name;"#,
    )
    .bind(tg_user_id)
    .fetch_all(&pool)
    .await
    {
        Ok(members) => {
            let mut waves = members
                .into_iter()
                .map(|row| WaveAmount {
                    wave_name: row.get("wave_name"),
                    amount: row.get("amount"),
                    max_amount: row.get("max_amount"),
                })
                .collect::<Vec<WaveAmount>>();
            // Sort by debut order
            waves.sort_by(|x, y| {
                NijiENMap[x.wave_name.as_str()].cmp(&NijiENMap[y.wave_name.as_str()])
            });
            Ok(waves)
        }
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
