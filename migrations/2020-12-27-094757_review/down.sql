DROP TABLE review;
ALTER TABLE pull_request ADD COLUMN required_reviewers TEXT NOT NULL DEFAULT '';
ALTER TABLE pull_request DROP COLUMN needed_reviewers_count;
ALTER TABLE repository DROP COLUMN default_needed_reviewers_count;
