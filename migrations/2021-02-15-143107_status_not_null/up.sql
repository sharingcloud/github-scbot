ALTER TABLE pull_request ALTER COLUMN check_status SET DEFAULT 'waiting';
ALTER TABLE pull_request ALTER COLUMN check_status SET NOT NULL;
ALTER TABLE pull_request ALTER COLUMN qa_status SET DEFAULT 'waiting';
ALTER TABLE pull_request ALTER COLUMN qa_status SET NOT NULL;
