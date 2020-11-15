table! {
    pull_request (id) {
        id -> Integer,
        repository_id -> Integer,
        number -> Integer,
        name -> Text,
        automerge -> Bool,
        step -> Text,
    }
}

table! {
    repository (id) {
        id -> Integer,
        name -> Text,
        owner -> Text,
    }
}

joinable!(pull_request -> repository (repository_id));

allow_tables_to_appear_in_same_query!(pull_request, repository,);
