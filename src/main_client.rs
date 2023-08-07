use log::{error, info};
use std::sync::Arc;

use holodex::model::{
    builders::VideoFilterBuilder, ExtraVideoInfo, Language, Organisation, VideoSortingCriteria,
    VideoType,
};

pub struct MainClient {
    tg_api: Arc<mobot::API>,
    holodex_api: Arc<holodex::Client>,
}

impl MainClient {
    pub fn new(mobot_client: Arc<mobot::API>, holodex_client: Arc<holodex::Client>) -> Self {
        Self {
            tg_api: mobot_client,
            holodex_api: holodex_client,
        }
    }
    pub fn get_videos(&self) -> holodex::model::PaginatedResult<holodex::model::Video> {
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
        self.holodex_api.videos(&filter).unwrap()
    }
}
