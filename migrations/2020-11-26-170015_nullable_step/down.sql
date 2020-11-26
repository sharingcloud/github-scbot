ALTER TABLE pull_request ALTER COLUMN check_status SET DEFAULT 'pass';
ALTER TABLE pull_request ALTER COLUMN check_status SET NOT NULL;
ALTER TABLE pull_request ALTER COLUMN step SET DEFAULT 'none'; 
ALTER TABLE pull_request ALTER COLUMN step SET NOT NULL;