CREATE TABLE IF NOT EXISTS pull_request_rule (
    id serial NOT NULL,
    repository_id int4 NOT NULL,
    name varchar(255) NOT NULL,
    conditions text NOT NULL,
    actions text NOT NULL,

    CONSTRAINT pull_request_rule_pkey PRIMARY KEY (id),
    CONSTRAINT pull_request_rule_repository_id_name_key UNIQUE (repository_id, name),
    CONSTRAINT pull_request_rule_repository_id_fkey FOREIGN KEY (repository_id) REFERENCES repository(id) ON DELETE CASCADE
);
