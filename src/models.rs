use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::{users,liked_videos,watched_videos};

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Clone, Serialize,Deserialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub email: String,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq, Serialize,Deserialize)]
#[diesel(belongs_to(User))]
#[diesel(table_name = liked_videos)]
pub struct LikedVideos {
    pub id: i32,
    pub title: String,
    pub video_id: i32,
    pub user_id: i32,
}

