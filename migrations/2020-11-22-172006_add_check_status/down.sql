PRAGMA foreign_keys=off;

CREATE TABLE pull_request_new (
    id INTEGER NOT NULL PRIMARY KEY,
    repository_id INTEGER NOT NULL,
    number INTEGER NOT NULL,
    name VARCHAR NOT NULL,
    automerge BOOLEAN NOT NULL,
    step VARCHAR NOT NULL,

    FOREIGN KEY(repository_id) REFERENCES repository(id),
    UNIQUE(repository_id, number)
);

INSERT INTO pull_request_new
SELECT id, repository_id, number, name, automerge, step FROM pull_request;

DROP TABLE pull_request;
ALTER TABLE pull_request_new RENAME TO pull_request;

PRAGMA foreign_keys=on;