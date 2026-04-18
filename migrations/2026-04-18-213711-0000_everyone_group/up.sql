-- Your SQL goes here
INSERT INTO users
VALUES (0, "merlin", "sys");

INSERT INTO groups
VALUES (0, "sys.everyone", "A group containing all users", 0, NULL);

INSERT INTO group_users
SELECT user_id, 0 FROM users;
