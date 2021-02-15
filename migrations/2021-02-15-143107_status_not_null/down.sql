ALTER TABLE pull_request ALTER COLUMN check_status DROP NOT NULL;
ALTER TABLE pull_request ALTER COLUMN check_status DROP DEFAULT;
ALTER TABLE pull_request ALTER COLUMN qa_status DROP NOT NULL;
ALTER TABLE pull_request ALTER COLUMN qa_status DROP DEFAULT;
