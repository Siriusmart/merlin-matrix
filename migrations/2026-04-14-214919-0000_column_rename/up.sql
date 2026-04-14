-- Your SQL goes here
ALTER TABLE rooms
RENAME COLUMN matrix_room_id TO m_room_id;

ALTER TABLE rooms
RENAME COLUMN matrix_room_homeserver TO m_room_homeserver;

ALTER TABLE users
RENAME COLUMN name TO m_user_id;

ALTER TABLE users
RENAME COLUMN homeserver TO m_user_homeserver;
