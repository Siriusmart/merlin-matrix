-- This file should undo anything in `up.sql`
CREATE TABLE groups_old (
    id                  INTEGER NOT NULL,
    display             TEXT NOT NULL,
    owner_name          TEXT NOT NULL,
    owner_homeserver    TEXT NOT NULL,
    admin_group_id      INTEGER,

    PRIMARY KEY (id)
);

INSERT INTO groups_old
SELECT groups.group_id, groups.name, users.name, users.homeserver, groups.admin_group_id
FROM groups
JOIN users ON users.user_id = groups.owner_id;

DROP TABLE permissions;
DROP TABLE rooms;
DROP TABLE contexts;
DROP TABLE groups;
DROP TABLE users;


ALTER TABLE groups_old RENAME TO groups;
