ALTER TABLE repository ADD COLUMN pr_title_validation_regex TEXT NOT NULL DEFAULT '';
ALTER TABLE pull_request ADD COLUMN required_reviewers TEXT NOT NULL DEFAULT '';