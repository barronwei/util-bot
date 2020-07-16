table! {
    match_admin (id) {
        id -> Int4,
        user_id -> Int4,
        status -> Bool,
    }
}

table! {
    match_groups (id) {
        id -> Int4,
        match_id -> Int4,
        members -> Array<Int4>,
    }
}

table! {
    match_responses (id) {
        id -> Int4,
        match_id -> Int4,
        user_id -> Int4,
    }
}

table! {
    pool_questions (id) {
        id -> Int4,
        pool_id -> Int4,
        question -> Text,
    }
}

table! {
    pool_responses (id) {
        id -> Int4,
        response_id -> Int4,
        answer -> Text,
    }
}

table! {
    user (id) {
        id -> Int4,
        discord_id -> Int4,
        languages -> Array<Text>,
        pool_state -> Int4,
    }
}

joinable!(match_admin -> user (user_id));
joinable!(match_groups -> match_admin (match_id));
joinable!(match_responses -> match_admin (match_id));
joinable!(match_responses -> user (user_id));
joinable!(pool_questions -> match_admin (pool_id));
joinable!(pool_responses -> match_responses (response_id));

allow_tables_to_appear_in_same_query!(
    match_admin,
    match_groups,
    match_responses,
    pool_questions,
    pool_responses,
    user,
);
