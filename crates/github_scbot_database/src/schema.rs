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
        name -> Varchar,
        automerge -> Bool,
        step -> Nullable<Varchar>,
        check_status -> Nullable<Varchar>,
        status_comment_id -> Int4,
        qa_status -> Nullable<Varchar>,
        wip -> Bool,
        needed_reviewers_count -> Int4,
        locked -> Bool,
        merged -> Bool,
        base_branch -> Varchar,
        head_branch -> Varchar,
        closed -> Bool,
    }
}

table! {
    repository (id) {
        id -> Int4,
        name -> Varchar,
        owner -> Varchar,
        pr_title_validation_regex -> Text,
        default_needed_reviewers_count -> Int4,
        default_strategy -> Varchar,
    }
}

table! {
    review (id) {
        id -> Int4,
        pull_request_id -> Int4,
        username -> Varchar,
        state -> Varchar,
        required -> Bool,
    }
}

joinable!(external_account_right -> external_account (username));
joinable!(external_account_right -> repository (repository_id));
joinable!(merge_rule -> repository (repository_id));
joinable!(pull_request -> repository (repository_id));
joinable!(review -> pull_request (pull_request_id));

allow_tables_to_appear_in_same_query!(
    external_account,
    external_account_right,
    merge_rule,
    pull_request,
    repository,
    review,
);
