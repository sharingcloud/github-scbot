CREATE TABLE IF NOT EXISTS repository (
    id serial PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    owner VARCHAR(255) NOT NULL,

    UNIQUE(name, owner)
);

CREATE TABLE IF NOT EXISTS pull_request (
    id serial PRIMARY KEY,
    repository_id INT NOT NULL,
    number INT NOT NULL,
    name VARCHAR(255) NOT NULL,
    automerge BOOLEAN NOT NULL,
    step VARCHAR(255) NOT NULL DEFAULT 'none',

    FOREIGN KEY(repository_id) REFERENCES repository(id),
    UNIQUE(repository_id, number)
);