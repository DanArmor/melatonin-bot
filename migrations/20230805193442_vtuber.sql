-- Add migration script here
CREATE TABLE IF NOT EXISTS vtuber (
    id INTEGER PRIMARY KEY NOT NULL,
    first_name VARCHAR(256) NOT NULL,
    last_name VARCHAR(256) NOT NULL,
    emoji VARCHAR(256) NOT NULL,
    youtube_handle VARCHAR(256) NOT NULL,
    youtube_channel_id VARCHAR(256) NOT NULL
);