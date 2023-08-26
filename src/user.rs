use serde::Deserialize;
use serde;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct User {
    #[serde(skip)]
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub tg_user_id: i64,
    pub tg_chat_id: i64,
}