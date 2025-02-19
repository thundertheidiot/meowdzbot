-- Add migration script here
ALTER TABLE settings ADD COLUMN activity_server_identifier TEXT;
ALTER TABLE settings ADD COLUMN activity_server_max_players INTEGER;
