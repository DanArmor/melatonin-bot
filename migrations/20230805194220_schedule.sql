-- Add migration script here
CREATE TABLE IF NOT EXISTS reported_stream (
    id INTEGER PRIMARY KEY NOT NULL,
    video_id VARCHAR(64) NOT NULL,
    vtuber_id INTEGER NOT NULL,
    scheduled_start DATETIME NOT NULL,
    FOREIGN KEY(vtuber_id) REFERENCES vtuber(id)
);