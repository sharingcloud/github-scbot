CREATE TABLE IF NOT EXISTS account (
    username VARCHAR(255) PRIMARY KEY,
    is_admin BOOLEAN NOT NULL
);

ALTER TABLE pull_request ADD COLUMN creator VARCHAR(255) NOT NULL default 'ghost';
