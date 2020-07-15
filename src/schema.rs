table! {
    match_admin (id) {
        id -> Int4,
        questions -> Array<Text>,
        user_id -> Int4,
    }
}

table! {
    match_groups (id) {
        id -> Int4,
        members -> Array<Int4>,
        match_id -> Int4,
    }
}

table! {
    match_responses (id) {
        id -> Int4,
        answers -> Array<Bool>,
        match_id -> Int4,
        user_id -> Int4,
    }
}

table! {
    user (id) {
        id -> Int4,
        discord_id -> Int4,
        languages -> Array<Text>,
        group_state -> Int4,
    }
}

joinable!(match_admin -> user (user_id));
joinable!(match_groups -> match_admin (match_id));
joinable!(match_responses -> match_admin (match_id));
joinable!(match_responses -> user (user_id));

allow_tables_to_appear_in_same_query!(
    match_admin,
    match_groups,
    match_responses,
    user,
);
