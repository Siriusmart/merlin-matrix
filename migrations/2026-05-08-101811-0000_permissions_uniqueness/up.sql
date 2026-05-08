-- Your SQL goes here
PRAGMA foreign_keys = OFF;

CREATE TABLE permissions_new (
    permission_id       INTEGER NOT NULL,
    qualifier           TEXT NOT NULL UNIQUE,

    PRIMARY KEY (permission_id)
);

INSERT INTO permissions_new
SELECT * FROM permissions;

DROP TABLE permissions;

ALTER TABLE permissions_new RENAME TO permissions;

PRAGMA foreign_key_check;
PRAGMA foreign_keys = ON;
