table! {
    account (username) {
        username -> Varchar,
        is_admin -> Bool,
    }
}

table! {
    external_account (username) {
        username -> Varchar,
        public_key -> Text,
        private_key -> Text,
    }
}

table! {
    external_account_right (username, repository_id) {
        username -> Varchar,
        repository_id -> Int4,
    }
}

table! {
    history_webhook (id) {
        id -> Int4,
        repository_id -> Int4,
        pull_request_id -> Int4,
        received_at -> Timestamptz,
        username -> Varchar,
        event_key -> Varchar,
        payload -> Text,
    }
}

table! {
    merge_rule (id) {
        id -> Int4,
        repository_id -> Int4,
        base_branch -> Varchar,
        head_branch -> Varchar,
        strategy -> Varchar,
    }
}

table! {
    pull_request (id) {
        id -> Int4,
        repository_id -> Int4,
        number -> Int4,
        creator -> Varchar,
        name -> Varchar,
        base_branch -> Varchar,
        head_branch -> Varchar,
        step -> Nullable<Varchar>,
        check_status -> Varchar,
        qa_status -> Varchar,
        needed_reviewers_count -> Int4,
        status_comment_id -> Int4,
        automerge -> Bool,
        wip -> Bool,
        locked -> Bool,
        merged -> Bool,
        closed -> Bool,
    }
}

table! {
    repository (id) {
        id -> Int4,
        name -> Varchar,
        owner -> Varchar,
        default_strategy -> Varchar,
        default_needed_reviewers_count -> Int4,
        pr_title_validation_regex -> Text,
    }
}

table! {
    review (id) {
        id -> Int4,
        pull_request_id -> Int4,
        username -> Varchar,
        state -> Varchar,
        required -> Bool,
        valid -> Bool,
    }
}

joinable!(external_account_right -> external_account (username));
joinable!(external_account_right -> repository (repository_id));
joinable!(history_webhook -> pull_request (pull_request_id));
joinable!(history_webhook -> repository (repository_id));
joinable!(merge_rule -> repository (repository_id));
joinable!(pull_request -> repository (repository_id));
joinable!(review -> pull_request (pull_request_id));

allow_tables_to_appear_in_same_query!(
    account,
    external_account,
    external_account_right,
    history_webhook,
    merge_rule,
    pull_request,
    repository,
    review,
);
