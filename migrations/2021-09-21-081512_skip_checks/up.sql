ALTER TABLE repository ADD COLUMN default_enable_qa BOOL NOT NULL default true;
ALTER TABLE repository ADD COLUMN default_enable_checks BOOL NOT NULL default true;
