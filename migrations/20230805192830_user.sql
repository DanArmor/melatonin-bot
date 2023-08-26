-- Add migration script here
CREATE TABLE IF NOT EXISTS user (
    first_name VARCHAR(256) NOT NULL,
    last_name VARCHAR(256),
    username VARCHAR(256),
    tg_user_id INTEGER PRIMARY KEY NOT NULL,
    tg_chat_id INTEGER NOT NULL
);