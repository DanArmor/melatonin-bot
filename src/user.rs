use serde;
use serde::Deserialize;

// User struct
#[derive(Deserialize, Clone, Debug, Default)]
pub struct User {
    #[serde(skip)]
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub tg_user_id: i64,
    pub tg_chat_id: i64,
}
