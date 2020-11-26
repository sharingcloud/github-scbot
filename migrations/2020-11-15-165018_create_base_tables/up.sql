CREATE TABLE repository (
    id INTEGER NOT NULL PRIMARY KEY,
    name VARCHAR NOT NULL,
    owner VARCHAR NOT NULL,

    UNIQUE(name, owner)
);

CREATE TABLE pull_request (
    id INTEGER NOT NULL PRIMARY KEY,
    repository_id INTEGER NOT NULL,
    number INTEGER NOT NULL,
    name VARCHAR NOT NULL,
    automerge BOOLEAN NOT NULL,
    step VARCHAR NOT NULL,

    FOREIGN KEY(repository_id) REFERENCES repository(id),
    UNIQUE(repository_id, number)
);