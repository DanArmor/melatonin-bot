use serde;
use serde::Deserialize;

// Vtuber struct
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Vtuber {
    #[serde(skip)]
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip)]
    pub wave_name: String,
    pub emoji: String,
    pub youtube_channel_id: String,
    pub youtube_handle: String,
}

// Represents vtuber wave
#[derive(Deserialize, Clone, Debug, Default)]
pub struct VtuberWave {
    pub name: String,
    pub members: Vec<Vtuber>,
}
