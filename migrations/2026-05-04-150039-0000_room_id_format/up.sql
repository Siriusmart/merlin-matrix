-- Your SQL goes here
PRAGMA foreign_keys = OFF;

CREATE TABLE rooms_new(
    room_id             INTEGER NOT NULL,
    m_room_id           TEXT NOT NULL UNIQUE,
    context_id          INTEGER,

    PRIMARY KEY (room_id),
    FOREIGN KEY (context_id) REFERENCES contexts(context_id)
);

INSERT INTO rooms_new
SELECT room_id, m_room_id || ':' || m_room_homeserver, context_id FROM rooms;

DROP TABLE rooms;

ALTER TABLE rooms_new RENAME TO rooms;

PRAGMA foreign_key_check;
PRAGMA foreign_keys = ON;
