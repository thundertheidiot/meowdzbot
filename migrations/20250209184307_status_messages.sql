-- Add migration script here
CREATE TABLE status_messages (
       id INTEGER PRIMARY KEY AUTOINCREMENT,
       channel_id TEXT NOT NULL CHECK (channel_id GLOB '[0-9]*'),
       message_id TEXT NOT NULL CHECK (message_id GLOB '[0-9]*')
)
