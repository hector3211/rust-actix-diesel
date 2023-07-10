use actix_cors::Cors;
use actix_files::{NamedFile, Files};
use actix_identity::{IdentityMiddleware, Identity};
use actix_session::{
    config::{PersistentSession, CookieContentSecurity},
    storage::CookieSessionStore,
    SessionMiddleware, Session
};
use actix_web::{
    cookie::{Key,SameSite},
    App,
    HttpResponse,
    HttpServer,
    Result,
    error::{ErrorInternalServerError, HttpError},
    web, Responder, HttpRequest, Error, get,
};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use auth::sign_up;
use cookie::time::Duration;
use guards::SessionGuard;
use models::VideoType;
use tracing::info;


use diesel::{
    r2d2::{self,ConnectionManager},
    PgConnection
};
use dotenv::dotenv;
use std::{env, sync::{Arc, Mutex}, path::PathBuf};
use std::io;
pub mod schema;
pub mod models;
pub mod auth;
pub mod db_actions;
pub mod guards;
pub mod ultils;

use crate::{
    auth::{login,logout},
    models::{SwaggerErrorResponse, Role}
};
use crate::db_actions::{
    get_everything,
    create_liked_videos
};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbError = Box<dyn std::error::Error + Send + Sync>;

const ONEDAY: Duration = Duration::days(1);
const _ONEMIN: Duration = Duration::minutes(1);

pub struct AppState {
    pub pool: DbPool,
    pub api_keys: Mutex<Vec<String>>,
}


#[actix_web::main]
async fn main() -> io::Result<()> {
    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // info!("staring server at http://localhost:8080");
    let key = Key::generate();
     dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("Database url in .env must be set dude!");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    let state = Arc::new(AppState {
        pool,
        api_keys: Mutex::new(Vec::new()),
    });

    #[derive(OpenApi)]
    #[openapi(
        paths(
            auth::sign_up,
            auth::login,
            auth::logout,
            user_data
        ),
        components (
            schemas(
                models::LikedVideos,
                models::WatchedVideos,
                models::UserWithVideos,
                models::User,
                models::SwaggerErrorResponse,
                auth::Credentials
            )
        )
    )]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();

    HttpServer::new(move|| {
          let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .expose_any_header();
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(), key.clone(),
                )
                .cookie_http_only(true)
                .cookie_content_security(CookieContentSecurity::Private)
                .cookie_same_site(SameSite::Strict)
                .session_lifecycle(PersistentSession::default().session_ttl(ONEDAY))
                .build(),
            )
            .wrap(cors)
            .app_data(web::Data::new(state.clone()))
            .service(sign_up)
            .service(login)
            .service(logout)
            .service(user_data)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi.clone()),
            )
            // .service(
            //     Files::new("/","./static")
            //     .show_files_listing()
            //         .index_file("index.html")
            //         .use_last_modified(true)
            // )
            // .route("/", web::get().to(index))
            // .route("/logot", web::post().to(logout))
            // .service(web::scope("/user")
            //     .route("/secret", web::get().to(secret))
            //     .route("/info/{user_id}", web::get().to(user_data))
            //     .route("/video/{id}/{title}/{vid_id}/{video_type}", web::post().to(new_video))
            // )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}



async fn index(
    _user: Option<Identity>, 
    _req: HttpRequest
) -> Result<impl Responder> {
    // if let Some(_user) = user {
        let path: PathBuf = "./static/index.html".parse().unwrap();
        Ok(NamedFile::open(path).unwrap())
    // } else {
    //     Err(HttpResponse::Unauthorized().body("not signed in or logged in"))
    // }

}


#[utoipa::path(
    responses(
        (
            status = 200,
            description = "Fetches a specific user",
            body = UserWithVideos
        ),
        (
            status = 401,
            description = "Not authorized",
            body = SwaggerErrorResponse,
            example = json!(SwaggerErrorResponse::Unauthorized(String::from("Not authorized User")))
        ),
    )
)]
#[get("/user/{id}")]
async fn user_data(
    state: web::Data<Arc<AppState>>,
    path: web::Path<i32>,
    session_guard: SessionGuard,
    session: Session
)
-> Result<HttpResponse> {
    if let Some(_sess) =  session_guard.session {
        if let Some(role) = session.get::<String>("role").unwrap() {
            let user_role = Role::try_from(role).unwrap();
            match user_role {
                Role::ADMIN => {
                    let user_id = path.into_inner();
                    let info = web::block(move || {
                        let mut conn = state.pool.get()?;
                        get_everything(&mut conn, user_id)

                    })
                    .await?;
                    Ok(HttpResponse::Ok().json(info.unwrap()))
                },
                Role::User => {
                    Ok(HttpResponse::Unauthorized().body("Not authorized!"))
                }
            }
        } else {
        Ok(HttpResponse::Unauthorized().body("Not authorized!"))
        }
    } else {
        Ok(HttpResponse::Unauthorized().body("Not authorized!"))
    }
    
}

// async fn new_user(
//     state: web::Data<Arc<AppState>>,
//     path: web::Path<(String, String)>,
// )
// -> Result<HttpResponse> {
//     let (email, password) = path.into_inner();
//     match validate_email(&email) {
//         Ok(_) => {
//             let user = web::block(move|| {
//                 let mut conn = state.pool.get()?;
//                 create_user(&mut conn,&email,password)
//             })
//                 .await?
//                 .map_err(|err| ErrorInternalServerError(err))?;
//
//             Ok(HttpResponse::Ok().json(user))
//         },
//             Err(err) => {
//             Ok(err)
//         }
//     }
//    
// }


async fn new_video(
    state: web::Data<Arc<AppState>>,
    path: web::Path<(i32,String,i32,String)>,
)
-> Result<HttpResponse> {
    let (id,title,vid_id,video_type) = path.into_inner();
    let video_type =  video_type.parse::<VideoType>().expect("expected this from str to work!");
    info!("video type : {:?}",video_type);
    match video_type {
        VideoType::LIKED => {
            let liked_vid = web::block(move|| {
                let mut conn = state.pool.get()?;
                create_liked_videos(&mut conn, id,title.as_str(),vid_id,models::VideoType::LIKED)
            })
                .await?
                .map_err(|err| ErrorInternalServerError(err))?;

            Ok(HttpResponse::Ok().json(liked_vid))
        }
        VideoType::WATCHED => {
            let watched_vid = web::block(move|| {
                let mut conn = state.pool.get()?;
                create_liked_videos(&mut conn, id,title.as_str(),vid_id,models::VideoType::WATCHED)
            })
                .await?
                .map_err(|err| ErrorInternalServerError(err))?;

            Ok(HttpResponse::Ok().json(watched_vid))
        },
    }
}
