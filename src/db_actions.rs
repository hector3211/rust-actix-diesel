use diesel::prelude::*;
use diesel::PgConnection;
use crate::models::User;
use crate::schema::liked_videos;
use crate::schema::users;
use crate::{models::{UserWithVideos, LikedVideos, WatchedVideos}, DbError};
use actix_web::HttpResponse;



pub fn validate_email(email: &str) -> Result<(), HttpResponse> {
    if !email.contains(".com") && !email.contains("@") {
        return Err(HttpResponse::NotAcceptable().body("Email provided is invalid! please check email"))
    }
    Ok(())

}

fn hash_password(password: &str) -> String {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .expect("Failed to hash password!")
}

pub fn get_everything(
    conn: &mut PgConnection,
    id: i32
)
-> Result<UserWithVideos, DbError>  {
    let user: User = users::table
        .filter(users::id.eq(id))
        .select(User::as_select())
        .get_result(conn)?;

    let liked_vids: Vec<LikedVideos> = LikedVideos::belonging_to(&user)
        .select(LikedVideos::as_select())
        .load(conn)?;

    let watched_vids: Vec<WatchedVideos> = WatchedVideos::belonging_to(&user)
        .select(WatchedVideos::as_select())
        .load(conn)?;

    let data = UserWithVideos {
         user,
        liked_videos: liked_vids,
        watched_videos: watched_vids
    };

    Ok(data)
}

pub fn get_user_info(
    conn: &mut PgConnection,
    id: i32
)
-> Result<Vec<LikedVideos>, DbError> {
    let user: User = users::table
        .filter(users::id.eq(id))
        .select(User::as_select())
        .get_result(conn)?;

    let videos: Vec<LikedVideos> = LikedVideos::belonging_to(&user)
        .select(LikedVideos::as_select())
        .load(conn)?;

    Ok(videos)
}

pub fn create_user(
    conn: &mut PgConnection,
    email: &str,
    pass: String
)
-> Result<User, DbError> {
    let hashed_password = hash_password(&pass);
    let user = diesel::insert_into(users::table)
        .values((
            users::email.eq(email),
            users::password_hash.eq(hashed_password)
        ))
        .returning(User::as_returning())
        .get_result(conn)?;

    Ok(user)
}

pub fn create_liked_videos(
    conn: &mut PgConnection,
    id: i32,
    title: &str,
    vid_id: i32
)
-> Result<LikedVideos, DbError> {
    let liked_vids = diesel::insert_into(liked_videos::table)
        .values((
            liked_videos::title.eq(title),
            liked_videos::video_id.eq(vid_id),
            liked_videos::user_id.eq(id),
        ))
        .returning(LikedVideos::as_returning())
        .get_result(conn)?;

    Ok(liked_vids)
}
