-- This file should undo anything in `up.sql`
DELETE FROM users
WHERE user_id = 0;

DELETE FROM group_users
WHERE group_id = 0;

DELETE FROM groups
WHERE group_id = 0;
