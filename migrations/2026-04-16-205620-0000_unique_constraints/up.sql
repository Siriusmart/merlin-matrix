-- Your SQL goes here
PRAGMA foreign_keys = OFF;

CREATE TABLE contexts_new (
    context_id              INTEGER NOT NULL,
    name                    TEXT COLLATE NOCASE UNIQUE,
    description             TEXT NOT NULL,
    owner_id                INTEGER NOT NULL,
    admin_group_id          INTEGER,

    PRIMARY KEY (context_id),
    FOREIGN KEY (owner_id) REFERENCES users(user_id),
    FOREIGN KEY (admin_group_id) REFERENCES groups(group_id)
);

INSERT INTO contexts_new
SELECT * FROM contexts;

DROP TABLE contexts;

ALTER TABLE contexts_new RENAME TO contexts;

PRAGMA foreign_key_check;

CREATE TABLE groups_new (
    group_id                INTEGER NOT NULL,
    name                    TEXT COLLATE NOCASE UNIQUE,
    description             TEXT NOT NULL,
    owner_id                INTEGER NOT NULL,
    admin_group_id          INTEGER,

    PRIMARY KEY (group_id),
    FOREIGN KEY (owner_id) REFERENCES users(user_id),
    FOREIGN KEY (admin_group_id) REFERENCES groups_new(group_id)
);

INSERT INTO groups_new
SELECT group_id, name, "", owner_id, admin_group_id FROM groups;

DROP TABLE groups;

ALTER TABLE groups_new RENAME TO groups;

PRAGMA foreign_key_check;

CREATE TABLE rooms_new (
    room_id                 INTEGER NOT NULL,
    m_room_id               TEXT NOT NULL,
    m_room_homeserver       TEXT NOT NULL,
    context_id              INTEGER,

    PRIMARY KEY (room_id),
    FOREIGN KEY (context_id) REFERENCES contexts(context_id),
    UNIQUE (m_room_id, m_room_homeserver)
);


INSERT INTO rooms_new
SELECT * FROM rooms;

DROP TABLE rooms;

ALTER TABLE rooms_new RENAME TO rooms;

PRAGMA foreign_key_check;

CREATE TABLE users_new (
    user_id                 INTEGER NOT NULL,
    m_user_id               TEXT NOT NULL,
    m_user_homeserver       TEXT NOT NULL,

    PRIMARY KEY (user_id),
    UNIQUE (m_user_id, m_user_homeserver)
);


INSERT INTO users_new
SELECT * FROM users;

DROP TABLE users;

ALTER TABLE users_new RENAME TO users;

PRAGMA foreign_key_check;

PRAGMA foreign_keys = ON;
