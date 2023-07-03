use actix_identity::{IdentityMiddleware, Identity};
use actix_session::{
    config::PersistentSession,
    storage::CookieSessionStore,
    SessionMiddleware
};
use actix_web::{
    cookie::{Key,SameSite},
    App,
    HttpResponse,
    HttpServer,
    Result,
    error::{ErrorInternalServerError, self},
    web,
    guard, Responder,
};

use cookie::time::Duration;
use models::VideoType;
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

use crate::auth::{
    login,
    secret,
    logout
};
use crate::db_actions::{
    get_everything,
    validate_email,
    create_user,
    create_liked_videos
};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbError = Box<dyn std::error::Error + Send + Sync>;

const ONEDAY: Duration = Duration::days(1);



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
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                .cookie_name("HO-auth".to_owned())
                .cookie_http_only(false)
                .cookie_same_site(SameSite::Strict)
                .session_lifecycle(PersistentSession::default().session_ttl(ONEDAY))
                .build(),
            )
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::get().to(index))
            .route("/login", web::post().to(login))
            .route("/logot", web::post().to(logout))
            .service(web::scope("/user")
                .route("/secret", web::get().to(secret))
                .route("/info/{user_id}", web::get().to(user_data))
                .route("/{user_email}/{user_password}", web::post().to(new_user))
                .route("/video/{id}/{title}/{vid_id}/{video_type}", web::post().to(new_video))
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn index(identity: Option<Identity>) -> actix_web::Result<impl Responder> {
    let id = match identity.map(|id| id.id()) {
        None => "anonymous".to_owned(),
        Some(Ok(id)) => id,
        Some(Err(err)) => return Err(error::ErrorInternalServerError(err)),
    };

    Ok(format!("Hello {id}"))
}


async fn user_data(
    pool: web::Data<DbPool>,
    path: web::Path<i32>
)
-> Result<HttpResponse> {
    let user_id = path.into_inner();
    let info = web::block(move || {
        let mut conn = pool.get()?;
        get_everything(&mut conn, user_id)

    })
    .await?
    .map_err(|err| ErrorInternalServerError(err))?;


     Ok(HttpResponse::Ok().json(info))
    
}

async fn new_user(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
)
-> Result<HttpResponse> {
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


async fn new_video(
    pool: web::Data<DbPool>,
    path: web::Path<(i32,String,i32,String)>,
)
-> Result<HttpResponse> {
    let (id,title,vid_id,video_type) = path.into_inner();
    let video_type =  video_type.parse::<VideoType>().expect("expected this from str to work!");
    info!("video type : {:?}",video_type);
    match video_type {
        VideoType::LIKED => {
            let liked_vid = web::block(move|| {
                let mut conn = pool.get()?;
                create_liked_videos(&mut conn, id,title.as_str(),vid_id,models::VideoType::LIKED)
            })
                .await?
                .map_err(|err| ErrorInternalServerError(err))?;

            Ok(HttpResponse::Ok().json(liked_vid))
        }
        VideoType::WATCHED => {
            let watched_vid = web::block(move|| {
                let mut conn = pool.get()?;
                create_liked_videos(&mut conn, id,title.as_str(),vid_id,models::VideoType::WATCHED)
            })
                .await?
                .map_err(|err| ErrorInternalServerError(err))?;

            Ok(HttpResponse::Ok().json(watched_vid))
        },
    }
}
