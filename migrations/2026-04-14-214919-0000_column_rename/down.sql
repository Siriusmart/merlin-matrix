-- This file should undo anything in `up.sql`
ALTER TABLE rooms
RENAME COLUMN m_room_id TO matrix_room_id;

ALTER TABLE rooms
RENAME COLUMN m_room_homeserver TO matrix_room_homeserver;

ALTER TABLE users
RENAME COLUMN m_user_id TO name;

ALTER TABLE users
RENAME COLUMN m_user_homeserver TO homeserver;
