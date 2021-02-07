CREATE TABLE IF NOT EXISTS merge_rule (
    id serial PRIMARY KEY,
    repository_id INT NOT NULL,
    base_branch VARCHAR(255) NOT NULL,
    head_branch VARCHAR(255) NOT NULL,
    strategy VARCHAR(255) NOT NULL,

    FOREIGN KEY(repository_id) REFERENCES repository(id),
    UNIQUE(repository_id, base_branch, head_branch)
);

ALTER TABLE repository ADD COLUMN default_strategy VARCHAR(255) NOT NULL DEFAULT 'merge';
ALTER TABLE pull_request ADD COLUMN base_branch VARCHAR(255) NOT NULL DEFAULT 'unknown';
ALTER TABLE pull_request ADD COLUMN head_branch VARCHAR(255) NOT NULL DEFAULT 'unknown';
