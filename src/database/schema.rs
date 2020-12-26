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
        required_reviewers -> Text,
    }
}

table! {
    repository (id) {
        id -> Int4,
        name -> Varchar,
        owner -> Varchar,
        pr_title_validation_regex -> Text,
    }
}

joinable!(pull_request -> repository (repository_id));

allow_tables_to_appear_in_same_query!(pull_request, repository,);
