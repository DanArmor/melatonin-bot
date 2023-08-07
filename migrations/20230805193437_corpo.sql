-- Add migration script here
CREATE TABLE IF NOT EXISTS corpo (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR(256) NOT NULL
);