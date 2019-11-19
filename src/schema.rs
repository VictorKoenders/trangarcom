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
    portfoliopost (id) {
        id -> Uuid,
        date -> Date,
        published -> Bool,
        seo_name -> Text,
        title -> Text,
        summary -> Text,
        summary_image -> Text,
        content -> Text,
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

allow_tables_to_appear_in_same_query!(blogpost, portfoliopost, request,);
