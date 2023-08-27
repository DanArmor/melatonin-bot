use serde::Deserialize;
use serde;
use chrono;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ReportedStream {
    #[serde(skip)]
    pub id: i64,
    pub video_id: String,
    pub vtuber_id: i64,
    pub scheduled_start: chrono::NaiveDateTime,
}