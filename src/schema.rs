// @generated automatically by Diesel CLI.

diesel::table! {
    liked_videos (id) {
        id -> Int4,
        title -> Text,
        video_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
    }
}

diesel::table! {
    watched_videos (id) {
        id -> Int4,
        title -> Text,
        video_id -> Int4,
        user_id -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    liked_videos,
    users,
    watched_videos,
);
