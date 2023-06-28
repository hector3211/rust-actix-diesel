use actix_web::App;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::error::ErrorInternalServerError;
use actix_web::get;
use actix_web::post;
use actix_web::web;
use diesel::{
    prelude::*,
    r2d2::{self,ConnectionManager}
};
use diesel::PgConnection;
use dotenv::dotenv;
use std::env;
use std::io;
pub mod schema;
pub mod models;

use  crate::schema::*;
use crate::models::*;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbError = Box<dyn std::error::Error + Send + Sync>;



#[actix_web::main]
async fn main() -> io::Result<()> {
     dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("Database url in .env must be set dude!");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    HttpServer::new(move|| {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(new_user)
            .service(new_video_liked)
            .service(user_info)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


#[get("/user/videos/{user_id}")]
async fn user_info(
    pool: web::Data<DbPool>,
    path: web::Path<i32>
)
-> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let info = web::block(move || {
        let mut conn = pool.get()?;
        get_user_info(&mut conn, user_id)

    })
    .await?
    .map_err(|err| ErrorInternalServerError(err))?;

     Ok(HttpResponse::Ok().json(info))
    
}

fn get_user_info(
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

#[post("/user/{user_email}")]
async fn new_user(
    pool: web::Data<DbPool>,
    path: web::Path<String>
)
-> Result<HttpResponse, Error> {
    let email = path.into_inner();
    let user = web::block(move|| {
        let mut conn = pool.get()?;
        create_user(&mut conn,email.as_str())
    })
    .await?
    .map_err(|err| ErrorInternalServerError(err))?;

     Ok(HttpResponse::Ok().json(user))
    
}
fn create_user(
    conn: &mut PgConnection,
    email: &str
)
-> Result<User, DbError> {
    let user = diesel::insert_into(users::table)
        .values(users::email.eq(email))
        .returning(User::as_returning())
        .get_result(conn)?;

    Ok(user)
}


#[post("/user/videos/{id}/{title}/{vid_id}/")]
async fn new_video_liked(
    pool: web::Data<DbPool>,
    path: web::Path<(i32,String,i32)>
)
-> Result<HttpResponse, Error>{
    let (id,title,vid_id) = path.into_inner();
    let liked_vids = web::block(move|| {
        let mut conn = pool.get()?;
        create_liked_videos(&mut conn, id,title.as_str(),vid_id)
    })
    .await?
    .map_err(|err| ErrorInternalServerError(err))?;

     Ok(HttpResponse::Ok().json(liked_vids))
}

fn create_liked_videos(
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
