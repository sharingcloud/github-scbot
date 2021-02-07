DROP TABLE merge_rule;

ALTER TABLE repository DROP COLUMN default_strategy;
ALTER TABLE pull_request DROP COLUMN base_branch;
ALTER TABLE pull_request DROP COLUMN head_branch;
