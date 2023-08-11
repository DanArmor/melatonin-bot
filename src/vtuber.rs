use serde::Deserialize;
use serde;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Vtuber {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip)]
    pub wave_name: String,
    pub emoji: String,
    pub youtube_channel_id: String,
    pub youtube_handle: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct VtuberWave {
    pub name: String,
    pub members: Vec<Vtuber>,
}
