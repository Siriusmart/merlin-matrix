-- Your SQL goes here
CREATE TABLE groups (
    id                  INTEGER NOT NULL,
    display             TEXT NOT NULL,
    owner_name          TEXT NOT NULL,
    owner_homeserver    TEXT NOT NULL,
    admin_group_id      INTEGER,

    PRIMARY KEY (id)
);
