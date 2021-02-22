CREATE TABLE IF NOT EXISTS repository (
    id serial NOT NULL,
    "name" varchar(255) NOT NULL,
    "owner" varchar(255) NOT NULL,
    default_strategy varchar(255) NOT NULL,
    default_needed_reviewers_count int4 NOT NULL,
    pr_title_validation_regex text NOT NULL,

    CONSTRAINT repository_name_owner_key UNIQUE (name, owner),
    CONSTRAINT repository_pkey PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS pull_request (
    id serial NOT NULL,
    repository_id int4 NOT NULL,
    "number" int4 NOT NULL,
    creator varchar(255) NOT NULL,
    name varchar(255) NOT NULL,
    base_branch varchar(255) NOT NULL,
    head_branch varchar(255) NOT NULL,
    step varchar(255) NULL,
    check_status varchar(255) NOT NULL,
    qa_status varchar(255) NOT NULL,
    needed_reviewers_count int4 NOT NULL,
    status_comment_id int4 NOT NULL,
    automerge bool NOT NULL,
    wip bool NOT NULL,
    "locked" bool NOT NULL,
    merged bool NOT NULL,
    closed bool NOT NULL,

    CONSTRAINT pull_request_pkey PRIMARY KEY (id),
    CONSTRAINT pull_request_repository_id_number_key UNIQUE (repository_id, number),
    CONSTRAINT pull_request_repository_id_fkey FOREIGN KEY (repository_id) REFERENCES repository(id)
);

CREATE TABLE IF NOT EXISTS review (
    id serial NOT NULL,
    pull_request_id int4 NOT NULL,
    username varchar(255) NOT NULL,
    state varchar(255) NOT NULL,
    required bool NOT NULL,
    "valid" bool NOT NULL,

    CONSTRAINT review_pkey PRIMARY KEY (id),
    CONSTRAINT review_pull_request_id_username_key UNIQUE (pull_request_id, username),
    CONSTRAINT review_pull_request_id_fkey FOREIGN KEY (pull_request_id) REFERENCES pull_request(id)
);

CREATE TABLE IF NOT EXISTS merge_rule (
    id serial NOT NULL,
    repository_id int4 NOT NULL,
    base_branch varchar(255) NOT NULL,
    head_branch varchar(255) NOT NULL,
    strategy varchar(255) NOT NULL,

    CONSTRAINT merge_rule_pkey PRIMARY KEY (id),
    CONSTRAINT merge_rule_repository_id_base_branch_head_branch_key UNIQUE (repository_id, base_branch, head_branch),
    CONSTRAINT merge_rule_repository_id_fkey FOREIGN KEY (repository_id) REFERENCES repository(id)
);

CREATE TABLE IF NOT EXISTS account (
    username varchar(255) NOT NULL,
    is_admin bool NOT NULL,

    CONSTRAINT account_pkey PRIMARY KEY (username)
);

CREATE TABLE IF NOT EXISTS external_account (
    username varchar(255) NOT NULL,
    public_key text NOT NULL,
    private_key text NOT NULL,

    CONSTRAINT external_account_pkey PRIMARY KEY (username)
);

CREATE TABLE IF NOT EXISTS external_account_right (
    username varchar(255) NOT NULL,
    repository_id int4 NOT NULL,

    CONSTRAINT external_account_right_pkey PRIMARY KEY (username, repository_id),
    CONSTRAINT external_account_right_repository_id_fkey FOREIGN KEY (repository_id) REFERENCES repository(id),
    CONSTRAINT external_account_right_username_fkey FOREIGN KEY (username) REFERENCES external_account(username)
);

CREATE TABLE IF NOT EXISTS history_webhook (
    id serial NOT NULL,
    repository_id int4 NOT NULL,
    pull_request_id int4 NOT NULL,
    received_at timestamptz NOT NULL,
    username varchar(255) NOT NULL,
    event_key varchar(255) NOT NULL,
    payload text NOT NULL,

    CONSTRAINT history_webhook_pkey PRIMARY KEY (id),
    CONSTRAINT history_webhook_pull_request_id_fkey FOREIGN KEY (pull_request_id) REFERENCES pull_request(id),
    CONSTRAINT history_webhook_repository_id_fkey FOREIGN KEY (repository_id) REFERENCES repository(id)
);
