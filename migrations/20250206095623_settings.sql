-- Add migration script here
CREATE TABLE settings (
       id INTEGER PRIMARY KEY CHECK (id=1),
       external_redirector_address TEXT
)
