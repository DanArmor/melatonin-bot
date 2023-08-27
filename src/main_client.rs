use crate::{config::MyPool, reported_stream::ReportedStream};
use chrono;
use chrono::Timelike;
use log::{debug};
use mobot::api::{ParseMode, SendPhotoRequest};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

use holodex::model::{
    builders::VideoFilterBuilder, ExtraVideoInfo, Language, Organisation, VideoSortingCriteria,
    VideoType,
};

pub struct MainClient {
    // Telegram api client
    pub tg_api: Arc<mobot::API>,
    // Holodex api client
    pub holodex_api: Arc<holodex::Client>,
    // Global sql connection pool
    sql_pool: MyPool,
}

pub struct VtuberVideo {
    // Vtuber, associated with video
    pub vtuber: crate::vtuber::Vtuber,
    // Video of the vtuber
    pub video: holodex::model::Video,
}

impl MainClient {
    pub fn new(mobot_client: Arc<mobot::API>, holodex_client: Arc<holodex::Client>) -> Self {
        Self {
            tg_api: mobot_client,
            holodex_api: holodex_client,
            sql_pool: MyPool::default(),
        }
    }
    // Get sql pool
    pub fn get_pool(&self) -> Pool<Sqlite> {
        self.sql_pool.0.clone()
    }
    // Get stream, that will start soon
    pub fn get_videos(&self) -> Vec<holodex::model::Video> {
        let filter = VideoFilterBuilder::new()
            .organisation(Organisation::Nijisanji)
            .language(&[Language::English])
            .video_type(VideoType::Stream)
            .max_upcoming_hours(1)
            .include(&[ExtraVideoInfo::Description, ExtraVideoInfo::ChannelStats])
            .sort_by(VideoSortingCriteria::StartScheduled)
            .status(&[holodex::model::VideoStatus::Upcoming])
            .limit(50)
            .build();
        self.holodex_api
            .videos(&filter)
            .unwrap()
            .into_iter()
            .filter(|x| {
                x.available_at.naive_utc() - chrono::Utc::now().naive_utc()
                    < chrono::Duration::minutes(22)
            })
            .collect()
    }
    // Remove passed streams that users have been notified about
    pub async fn clean_reported_streams(&self) {
        let reported_streams = sqlx::query_as!(ReportedStream, "SELECT * FROM reported_stream")
            .fetch_all(&self.get_pool())
            .await
            .unwrap();
        let time_now = chrono::Utc::now().naive_utc();
        for stream in reported_streams {
            if stream.scheduled_start < time_now {
                sqlx::query!("DELETE FROM reported_stream WHERE id = ?", stream.id)
                    .execute(&self.get_pool())
                    .await
                    .unwrap();
            }
        }
    }
    // Associate fetched videos with vtubers. Drop videos, that don't belong to any vtuber in db
    pub async fn associate_video_vtuber(&self) -> Vec<VtuberVideo> {
        // Fetch vector of vtubers
        let vtubers = sqlx::query_as!(crate::vtuber::Vtuber, "SELECT * FROM vtuber")
            .fetch_all(&self.get_pool())
            .await
            .unwrap();

        // Connect videos with vtubers. Filter out videos, that don't belong to any vtuber in db
        self.get_videos()
            .into_iter()
            .filter_map(|video| {
                match vtubers
                    .iter()
                    .position(|vtuber| vtuber.youtube_channel_id == video.channel.id().to_string())
                {
                    Some(index) => Some(VtuberVideo {
                        vtuber: vtubers[index].clone(),
                        video,
                    }),
                    None => None,
                }
            })
            .collect()
    }
    // Notify all subscribed users about the stream
    pub async fn send_notification(&self, stream: VtuberVideo) {
        // Get all users, that subscribed to this vtuber
        let users = sqlx::query_as!(
            crate::user::User,
            r#"SELECT user.* FROM user JOIN user_vtuber ON user_vtuber.vtuber_id = ?"#,
            stream.vtuber.id
        )
        .fetch_all(&self.get_pool())
        .await
        .unwrap();

        // Notify every user
        for user in &users {
            // Get GMT+3 datetime
            let local_date_gmt3 =
                stream.video.available_at.naive_utc() + chrono::Duration::hours(3);
            // Send thumbnail and text-message
            let res = self
                .tg_api
                .send_photo(
                    &SendPhotoRequest::new_external_url(
                        user.tg_chat_id,
                        format!(
                            "https://img.youtube.com/vi/{}/0.jpg",
                            stream.video.id.to_string()
                        ),
                    )
                    .with_caption(format!(
                        "Стрим {} {} начнется через \\~20 минут\n\
                        \n\
                        [▶️ Ссылка на стрим](https://www.youtube.com/watch?v={})\n\
                        Начало: {:02}:{:02} \\(GMT\\+3 Europe/Moscow\\)",
                        stream.vtuber.first_name,
                        stream.vtuber.last_name,
                        stream.video.id.to_string(),
                        local_date_gmt3.hour(),
                        local_date_gmt3.minute()
                    ))
                    .with_parse_mode(ParseMode::MarkdownV2),
                )
                .await;
            debug!("User-notify: {:?}", res);
        }
        if !users.is_empty() {
            crate::queries::insert_reported_stream(self.get_pool(), &stream.video, &stream.vtuber)
                .await
                .unwrap();
        }
    }
}
