-- Your SQL goes here
CREATE TABLE users (
    user_id             INTEGER NOT NULL,
    name                TEXT NOT NULL,
    homeserver          TEXT NOT NULL,

    PRIMARY KEY (user_id)
);

CREATE TABLE groups_new (
    group_id            INTEGER NOT NULL,
    name                TEXT NOT NULL,
    owner_id            INTEGER NOT NULL,
    admin_group_id      INTEGER,

    PRIMARY KEY (group_id),
    FOREIGN KEY (owner_id) REFERENCES users(user_id),
    FOREIGN KEY (admin_group_id) REFERENCES groups_new(group_id)
);

CREATE TABLE contexts (
    context_id          INTEGER NOT NULL,
    name                TEXT NOT NULL,
    description         TEXT NOT NULL,
    owner_id            INTEGER NOT NULL,
    admin_group_id      INTEGER,

    PRIMARY KEY (context_id),
    FOREIGN KEY (owner_id) REFERENCES users(user_id),
    FOREIGN KEY (admin_group_id) REFERENCES groups_new(group_id)
);

CREATE TABLE rooms (
    room_id                 INTEGER NOT NULL,
    matrix_room_id          TEXT NOT NULL,
    matrix_room_homeserver  TEXT NOT NULL,
    context_id              INTEGER,

    PRIMARY KEY (room_id),
    FOREIGN KEY (context_id) REFERENCES contexts(context_id)
);

CREATE TABLE permissions (
    permission_id       INTEGER NOT NULL,
    qualifier           TEXT NOT NULL,

    PRIMARY KEY (permission_id)
);


INSERT INTO users (name, homeserver)
SELECT DISTINCT owner_name, owner_homeserver
FROM groups;

INSERT INTO groups_new
SELECT groups.id, groups.display, users.user_id, groups.admin_group_id
FROM groups
JOIN users ON users.name = groups.owner_name
              AND users.homeserver = groups.owner_homeserver;

DROP TABLE groups;

ALTER TABLE groups_new RENAME TO groups;
