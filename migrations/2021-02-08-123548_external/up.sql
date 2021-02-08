CREATE TABLE IF NOT EXISTS external_account (
    username VARCHAR(255) PRIMARY KEY,
    public_key TEXT NOT NULL,
    private_key TEXT NOT NULL,

    UNIQUE(username)
);

CREATE TABLE IF NOT EXISTS external_account_right (
    username VARCHAR(255),
    repository_id INT,

    PRIMARY KEY(username, repository_id),
    FOREIGN KEY(username) REFERENCES external_account(username),
    FOREIGN KEY(repository_id) REFERENCES repository(id),
    UNIQUE(username, repository_id)
);
