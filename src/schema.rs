table! {
    blogpost (id) {
        id -> Uuid,
        date -> Date,
        published -> Bool,
        seo_name -> Text,
        title -> Text,
        summary -> Text,
        content -> Text,
        tweet_id -> Nullable<Text>,
    }
}

table! {
    request (id) {
        id -> Uuid,
        time -> Timestamp,
        url -> Text,
        headers -> Text,
        response_time -> Nullable<Float8>,
        finish_time -> Nullable<Float8>,
        status_code -> Nullable<Int2>,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Text,
        birthdate -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    blogpost,
    request,
    users,
);
