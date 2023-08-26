-- Add migration script here
CREATE TABLE IF NOT EXISTS user_vtuber (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    vtuber_id INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES user(tg_user_id),
    FOREIGN KEY(vtuber_id) REFERENCES vtuber(id)
);