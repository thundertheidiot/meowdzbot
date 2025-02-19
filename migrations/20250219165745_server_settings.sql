-- Add migration script here
CREATE TABLE server_settings (
    name TEXT PRIMARY KEY NOT NULL,
    addr TEXT NOT NULL,
    max_player_count INTEGER NOT NULL DEFAULT 16,
    legacy BOOLEAN NOT NULL DEFAULT TRUE,
    allow_upload_required BOOLEAN NOT NULL DEFAULT FALSE
);

INSERT INTO server_settings (name, addr)
       SELECT key, value FROM server_address;
