use chrono;
use serde;
use serde::Deserialize;

// Stream that users were notified about
#[derive(Deserialize, Clone, Debug, Default)]
pub struct ReportedStream {
    #[serde(skip)]
    pub id: i64,
    // Youtube video id
    pub video_id: String,
    pub vtuber_id: i64,
    pub scheduled_start: chrono::NaiveDateTime,
}
