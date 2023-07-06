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
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 60]
        password_hash -> Bpchar,
        #[max_length = 5]
        role -> Nullable<Varchar>,
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

diesel::joinable!(liked_videos -> users (user_id));
diesel::joinable!(watched_videos -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    liked_videos,
    users,
    watched_videos,
);
