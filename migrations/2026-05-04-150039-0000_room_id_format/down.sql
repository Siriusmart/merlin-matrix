-- This file should undo anything in `up.sql`
PRAGMA foreign_keys = OFF;

CREATE TABLE rooms_old(
    room_id             INTEGER NOT NULL,
    m_room_id           TEXT NOT NULL,
    m_room_id           TEXT NOT NULL,
    context_id          INTEGER,

    PRIMARY KEY (room_id),
    FOREIGN KEY (context_id) REFERENCES contexts(context_id)
);

INSERT INTO rooms_old
SELECT
    room_id,
    substr(m_room_id, 1, instr(m_room_id, ':') - 1),
    substr(m_room_id, instr(m_room_id, ':') + 1),
    context_id
FROM rooms;

DROP TABLE rooms;

ALTER TABLE rooms_old RENAME TO rooms;

PRAGMA foreign_key_check;
PRAGMA foreign_keys = ON;
