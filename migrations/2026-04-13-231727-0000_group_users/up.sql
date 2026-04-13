-- Your SQL goes here
CREATE TABLE group_users (
    user_id         INTEGER NOT NULL,
    group_id        INTEGER NOT NULL,

    PRIMARY KEY (user_id, group_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (group_id) REFERENCES groups(group_id)
);
