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
