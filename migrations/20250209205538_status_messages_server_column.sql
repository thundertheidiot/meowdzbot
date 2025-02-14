-- Add migration script here
ALTER TABLE status_messages
      ADD COLUMN server_name TEXT NOT NULL DEFAULT 'meow';
