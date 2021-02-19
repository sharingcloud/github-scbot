CREATE TABLE IF NOT EXISTS history_webhook (
    id serial PRIMARY KEY,
    repository_id INT NOT NULL,
    pull_request_id INT NOT NULL,
    username VARCHAR(255) NOT NULL,
    received_at TIMESTAMP WITH TIME ZONE NOT NULL,
    event_key VARCHAR(255) NOT NULL,
    payload TEXT NOT NULL,

    FOREIGN KEY(repository_id) REFERENCES repository(id),
    FOREIGN KEY(pull_request_id) REFERENCES pull_request(id)
);
