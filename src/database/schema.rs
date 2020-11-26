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
    }
}

table! {
    repository (id) {
        id -> Int4,
        name -> Varchar,
        owner -> Varchar,
    }
}

joinable!(pull_request -> repository (repository_id));

allow_tables_to_appear_in_same_query!(pull_request, repository,);
