use actix_web::body;
use bcrypt;
use actix_hash::{BodyBlake2b, BodyHash, BodyHashParts};
use actix_session::{
    Session,
    SessionMiddleware,
};
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::{CookieJar,Key,Cookie};
use actix_web::{
    App,
    Error,
    HttpResponse,
    HttpServer,
    error::{ErrorInternalServerError, ErrorNotAcceptable},
    get,
    post,
    web,
    middleware::Logger
};

use tracing::info;

use diesel::{
    prelude::*,
    r2d2::{self,ConnectionManager},
    PgConnection
};
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
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    info!("staring server at http://localhost:8080");
    let key = Key::generate();
     dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("Database url in .env must be set dude!");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    HttpServer::new(move|| {
        App::new()
            .wrap(
                SessionMiddleware::new(CookieSessionStore::default(), key.clone())
            )
            .app_data(web::Data::new(pool.clone()))
            .service(new_user)
            .service(new_video_liked)
            .service(user_info)
            .service(user_data)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


#[get("/user/info/{user_id}")]
async fn user_data(
    pool: web::Data<DbPool>,
    path: web::Path<i32>
)
-> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let info = web::block(move || {
        let mut conn = pool.get()?;
        get_everything(&mut conn, user_id)

    })
    .await?
    .map_err(|err| ErrorInternalServerError(err))?;

    // for development 
    let pass = info.user.password_hash.clone();
    println!("did user password match: {}",bcrypt::verify("password1234", &pass).unwrap());

     Ok(HttpResponse::Ok().json(info))
    
}

fn get_everything(
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

fn validate_email(email: &str) -> Result<(), HttpResponse> {
    if !email.contains(".com") && !email.contains("@") {
        return Err(HttpResponse::NotAcceptable().body("Email provided is invalid! please check email"))
    }
    Ok(())

}

fn hash_password(password: &str) -> String {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .expect("Failed to hash password!")
}

#[post("/user/{user_email}")]
async fn new_user(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    body: web::Json<PostUser>
)
-> Result<HttpResponse, Error> {
    let email = path.into_inner();
    let pass = body.pass.clone();
    match validate_email(&email) {
        Ok(_) => {
            let user = web::block(move|| {
                let mut conn = pool.get()?;
                create_user(&mut conn,&email,pass)
            })
                .await?
                .map_err(|err| ErrorInternalServerError(err))?;

            Ok(HttpResponse::Ok().json(user))
        },
            Err(err) => {
            Ok(err)
        }
    }
    
}
fn create_user(
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
