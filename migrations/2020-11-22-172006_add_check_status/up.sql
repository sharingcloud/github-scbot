ALTER TABLE pull_request ADD COLUMN check_status VARCHAR(255) NOT NULL;
ALTER TABLE pull_request ALTER COLUMN check_status SET DEFAULT 'pass';