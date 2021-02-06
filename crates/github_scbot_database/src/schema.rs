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
    }
}

table! {
    repository (id) {
        id -> Int4,
        name -> Varchar,
        owner -> Varchar,
        pr_title_validation_regex -> Text,
        default_needed_reviewers_count -> Int4,
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

joinable!(pull_request -> repository (repository_id));
joinable!(review -> pull_request (pull_request_id));

allow_tables_to_appear_in_same_query!(pull_request, repository, review,);
