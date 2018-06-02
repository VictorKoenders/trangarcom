table! {
    blogpost (id) {
        id -> Uuid,
        date -> Date,
        published -> Bool,
        seo_name -> Text,
        title -> Text,
        summary -> Text,
        content -> Text,
    }
}

table! {
    request (id) {
        id -> Uuid,
        time -> Timestamp,
        url -> Text,
        remote_ip -> Text,
        headers -> Text,
        response_time -> Nullable<Float8>,
        finish_time -> Nullable<Float8>,
    }
}

allow_tables_to_appear_in_same_query!(
    blogpost,
    request,
);
