-- Add migration script here
CREATE TABLE IF NOT EXISTS vtuber (
    id INTEGER PRIMARY KEY NOT NULL,
    first_name VARCHAR(256) NOT NULL,
    last_name VARCHAR(256) NOT NULL,
    username VARCHAR(256) NOT NULL,
    youtube_id VARCHAR(256) NOT NULL,
    corpo_id INTEGER NOT NULL,
    language_code VARCHAR(256) NOT NULL
);