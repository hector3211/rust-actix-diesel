use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::{Key, SameSite};
use actix_web::{
    App,
    Error,
    HttpResponse,
    HttpServer,
    error::ErrorInternalServerError,
    get,
    post,
    web,
};

use tracing::info;

use diesel::{
    r2d2::{self,ConnectionManager},
    PgConnection
};
use dotenv::dotenv;
use std::env;
use std::io;
pub mod schema;
pub mod models;
pub mod auth;
pub mod db_actions;

use crate::auth::{login, secret};
use crate::db_actions::{
    get_everything,
    get_user_info,
    validate_email,
    create_user,
    create_liked_videos
};

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
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                .cookie_http_only(false)
                .cookie_same_site(SameSite::Strict)
                .build(),
            )
            .app_data(web::Data::new(pool.clone()))
            .service(new_user)
            .service(new_video_liked)
            .service(user_info)
            .service(user_data)
            .service(login)
            .service(secret)
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


     Ok(HttpResponse::Ok().json(info))
    
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


#[post("/user/{user_email}/{user_password}")]
async fn new_user(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
)
-> Result<HttpResponse, Error> {
    let (email, password) = path.into_inner();
    match validate_email(&email) {
        Ok(_) => {
            let user = web::block(move|| {
                let mut conn = pool.get()?;
                create_user(&mut conn,&email,password)
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

