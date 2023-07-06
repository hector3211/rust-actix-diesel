use std::str::FromStr;

use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::{users,liked_videos,watched_videos};



#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Clone, Serialize,Deserialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: String,
    pub role: Option<String>
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq, Serialize,Deserialize)]
#[diesel(belongs_to(User))]
#[diesel(table_name = liked_videos)]
pub struct LikedVideos {
    pub id: i32,
    pub title: String,
    pub video_id: i32,
    pub user_id: i32
}


#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq, Serialize,Deserialize)]
#[diesel(belongs_to(User))]
#[diesel(table_name = watched_videos)]
pub struct WatchedVideos {
    pub id: i32,
    pub title: String,
    pub video_id: i32,
    pub user_id: i32
}


#[derive(Serialize)]
pub struct UserWithVideos {
    #[serde(flatten)]
    pub user: User,
    pub liked_videos: Vec<LikedVideos>,
    pub watched_videos: Vec<WatchedVideos>
}

#[derive(Deserialize,Serialize,Clone)]
pub struct PostUser {
    pub pass: String,
}


#[derive(Deserialize,Serialize,Clone,Debug)]
pub enum VideoType {
    WATCHED,
    LIKED
}


impl FromStr for VideoType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "liked" => {
                Ok(VideoType::LIKED)
            },
            "watched" => {
                Ok(VideoType::LIKED)
            },
                _ => {
                Err(anyhow::Error::msg("Couldn't convert"))
            }
        }
    }
}

#[derive(Deserialize,Serialize,Debug)]
pub enum VideoTypeResult {
    WATCHED(WatchedVideos),
    LIKED(LikedVideos)
}
