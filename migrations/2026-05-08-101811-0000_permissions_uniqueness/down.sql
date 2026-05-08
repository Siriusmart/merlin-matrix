-- This file should undo anything in `up.sql`
PRAGMA foreign_keys = OFF;

CREATE TABLE permissions_old (
    permission_id       INTEGER NOT NULL,
    qualifier           TEXT NOT NULL,

    PRIMARY KEY (permission_id)
);

INSERT INTO permissions_old
SELECT * FROM permissions;

DROP TABLE permissions;

ALTER TABLE permissions_old RENAME TO permissions;

PRAGMA foreign_key_check;
PRAGMA foreign_keys = ON;
