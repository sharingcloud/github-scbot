CREATE TABLE IF NOT EXISTS review (
    id serial PRIMARY KEY,
    pull_request_id INT NOT NULL,
    username VARCHAR(255) NOT NULL,
    state VARCHAR(255) NOT NULL,
    required BOOLEAN NOT NULL,

    FOREIGN KEY(pull_request_id) REFERENCES pull_request(id),
    UNIQUE(pull_request_id, username)
);

ALTER TABLE pull_request DROP COLUMN required_reviewers;
ALTER TABLE pull_request ADD COLUMN needed_reviewers_count INT NOT NULL DEFAULT 2;
ALTER TABLE repository ADD COLUMN default_needed_reviewers_count INT NOT NULL DEFAULT 2;
