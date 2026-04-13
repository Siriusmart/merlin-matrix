-- Your SQL goes here
CREATE TABLE context_permissions (
    context_id          INTEGER NOT NULL,
    permission_id       INTEGER NOT NULL,
    priority            INTEGER NOT NULL,
    allowed             BOOLEAN NOT NULL,

    PRIMARY KEY (context_id, permission_id),
    FOREIGN KEY (context_id) REFERENCES contexts(context_id),
    FOREIGN KEY (permission_id) REFERENCES permissions(permission_id)
);
