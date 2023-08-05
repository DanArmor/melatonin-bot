-- Add migration script here
CREATE TABLE IF NOT EXISTS user_vtuber (
    id INTEGER PRIMARY KEY NOT NULL,
    vtuber_id INTEGER NOT NULL,
    scheduled_start DATETIME NOT NULL,
    video_link VARCHAR(256) NOT NULL
);