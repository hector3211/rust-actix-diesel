use std::sync::Arc;
use actix_identity::Identity;

use actix_session::Session;
use actix_web::cookie::CookieBuilder;
use actix_web::{HttpResponse, web, Responder, Result, Error, HttpRequest, HttpMessage, get, post};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AppState;

use crate::db_actions::{authenticate, create_user, validate_email};
use crate::guards::SessionGuard;
use crate::models::{User, SwaggerErrorResponse};


#[derive(Debug, Deserialize,Serialize,Clone, ToSchema)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[utoipa::path(
    request_body = Credentials,
    responses(
        (
            status = 201,
            description = "Sign up a user",
            body = Credentials
        ),
        (
            status = 406,
            description = "Email Provided is not valid",
            body = SwaggerErrorResponse,
            example = json!(SwaggerErrorResponse::Conflict(String::from("Please Double check email, make sure it's valid")))
        )
    )
)]
#[post("/signup")]
pub async fn sign_up(
    creds: web::Json<Credentials>,
    session: Session,
    state: web::Data<Arc<AppState>>,
)
-> Result<impl Responder> {
    let creds = creds.into_inner();


    let user = web::block(move || {
        let mut conn = state.pool.get()?;
        create_user(&mut conn, creds)
    })
    .await?;

    let user: User = user.unwrap();
    session.insert("user", user.id.clone()).unwrap();
    session.insert("role", user.role.clone()).unwrap();
    Ok(HttpResponse::Created().json(user))
}

#[utoipa::path(
    request_body = Credentials,
    responses(
        (
            status = 200,
            description = "Log in a user",
        ),
        (
        status = 404,
        description = "User Not Found",
        body = SwaggerErrorResponse,
        example = json!(SwaggerErrorResponse::NotFound(String::from("User Not Found")))
        ),
    )
)]
#[post("/login")]
pub async fn login(
    creds: web::Json<Credentials>,
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    session: Session
)
-> Result<impl Responder> {
    let creds = creds.into_inner();
    let cred_two = creds.clone();

    let resp = web::block(move || {
        let mut conn = state.pool.get()?;
        authenticate(creds, &mut conn)
    })
    .await?;
    
    let user: User = resp.unwrap();

    let hash_check = bcrypt::verify(cred_two.password, &user.password_hash).unwrap();
    match hash_check {
       true => {
            Identity::login(&req.extensions(), user.email.into()).unwrap();
            let cookie = CookieBuilder::new("role", user.role.unwrap())
                .secure(true)
                .http_only(true)
                .finish();
            Ok(HttpResponse::Ok().cookie(cookie).finish())

        },
        false => {
            Ok(HttpResponse::Unauthorized().body("Not Authenticated!"))

        }
    }
}

#[utoipa::path(
    responses(
        (
            status = 200,
            description = "Log out a user",
        ),
        (
            status = 404,
            description = "User Not Found",
            body = SwaggerErrorResponse,
            example = json!(SwaggerErrorResponse::NotFound(String::from("User Not Found")))
        ),
    )
)]
#[post("/logout")]
pub async fn logout(
    // session: Session,
    user: Identity
)
-> Result<impl Responder> {
    user.logout();
    Ok(HttpResponse::Ok().body("Successfully Loged Out"))
}

fn _generate_key()
-> String {
    let api_key:String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    return api_key;
}

pub fn validate_session(sess_guard: SessionGuard)-> Result<HttpResponse, Error> {
    if let Some(session) = sess_guard.session {
            Ok(HttpResponse::Ok().body(format!("Session: {}", session)))
        } else {
            Ok(HttpResponse::Unauthorized().body("Unauthorized!"))
    }
        
}

pub async fn secret(sess_guard: SessionGuard) -> Result<impl Responder> {
    let auth = validate_session(sess_guard).unwrap();
    Ok(auth)
}


pub async fn apikey_to_state(
    state: web::Data<Arc<AppState>>,
    key: &str
) {
    let mut keys = state.api_keys.lock().unwrap();
    keys.push(key.to_string());
}

