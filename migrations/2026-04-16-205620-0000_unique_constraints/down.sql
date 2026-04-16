-- This file should undo anything in `up.sql`
PRAGMA foreign_keys = OFF;

CREATE TABLE contexts_old (
    context_id              INTEGER NOT NULL,
    name                    TEXT COLLATE NOCASE,
    description             TEXT NOT NULL,
    owner_id                INTEGER NOT NULL,
    admin_group_id          INTEGER,

    PRIMARY KEY (context_id),
    FOREIGN KEY (owner_id) REFERENCES users(user_id),
    FOREIGN KEY (admin_group_id) REFERENCES groups(group_id)
);

INSERT INTO contexts_old
SELECT * FROM contexts;

DROP TABLE contexts;

ALTER TABLE contexts_old RENAME TO contexts;

PRAGMA foreign_key_check;

CREATE TABLE groups_old (
    group_id                INTEGER NOT NULL,
    name                    TEXT COLLATE NOCASE,
    description             TEXT NOT NULL,
    owner_id                INTEGER NOT NULL,
    admin_group_id          INTEGER,

    PRIMARY KEY (group_id),
    FOREIGN KEY (owner_id) REFERENCES users(user_id),
    FOREIGN KEY (admin_group_id) REFERENCES groups_old(group_id)
);

INSERT INTO groups_old
SELECT group_id, name,  owner_id, admin_group_id FROM groups;

DROP TABLE groups;

ALTER TABLE groups_old RENAME TO groups;

PRAGMA foreign_key_check;

CREATE TABLE rooms_old (
    room_id                 INTEGER NOT NULL,
    m_room_id               TEXT NOT NULL,
    m_room_homeserver       TEXT NOT NULL,
    context_id              INTEGER,

    PRIMARY KEY (room_id),
    FOREIGN KEY (context_id) REFERENCES contexts(context_id),
);


INSERT INTO rooms_old
SELECT * FROM rooms;

DROP TABLE rooms;

ALTER TABLE rooms_old RENAME TO rooms;

PRAGMA foreign_key_check;

CREATE TABLE users_old (
    user_id                 INTEGER NOT NULL,
    m_user_id               TEXT NOT NULL,
    m_user_homeserver       TEXT NOT NULL,

    PRIMARY KEY (user_id),
);


INSERT INTO users_old
SELECT * FROM users;

DROP TABLE users;

ALTER TABLE users_old RENAME TO users;

PRAGMA foreign_key_check;

PRAGMA foreign_keys = ON;
