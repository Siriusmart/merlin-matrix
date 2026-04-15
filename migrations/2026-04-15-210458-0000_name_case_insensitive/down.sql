-- This file should undo anything in `up.sql`
PRAGMA foreign_keys = OFF;

CREATE TABLE groups_old (
    group_id            INTEGER NOT NULL,
    name                TEXT NOT NULL,
    owner_id            INTEGER NOT NULL,
    admin_group_id      INTEGER,

    PRIMARY KEY (group_id),
    FOREIGN KEY (owner_id) REFERENCES users(user_id),
    FOREIGN KEY (admin_group_id) REFERENCES groups_old(group_id)
);

INSERT INTO groups_old
SELECT group_id, name, owner_id, admin_group_id
FROM groups;

DROP TABLE groups;

ALTER TABLE groups_old RENAME TO groups;

PRAGMA foreign_key_check;
