-- Your SQL goes here
PRAGMA foreign_keys = OFF;

CREATE TABLE groups_new (
    group_id            INTEGER NOT NULL,
    name                TEXT NOT NULL COLLATE NOCASE,
    owner_id            INTEGER NOT NULL,
    admin_group_id      INTEGER,

    PRIMARY KEY (group_id),
    FOREIGN KEY (owner_id) REFERENCES users(user_id),
    FOREIGN KEY (admin_group_id) REFERENCES groups_new(group_id)
);

INSERT INTO groups_new
SELECT group_id, name, owner_id, admin_group_id
FROM groups;

DROP TABLE groups;

ALTER TABLE groups_new RENAME TO groups;

PRAGMA foreign_key_check;

PRAGMA foreign_keys = ON;
