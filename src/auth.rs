use std::sync::Arc;
use actix_identity::Identity;
use bcrypt;
use actix_session::Session;
use actix_web::{HttpResponse, web, Responder, Result, Error, HttpRequest, HttpMessage};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::AppState;

use crate::db_actions::{authenticate, create_user};
use crate::guards::SessionGuard;
use crate::models::User;


#[derive(Debug, Deserialize,Serialize,Clone)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

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
    session.insert("user", user.id).unwrap();
    Ok(HttpResponse::Created().json(user))
}

pub async fn login(
    creds: web::Json<Credentials>,
    state: web::Data<Arc<AppState>>,
    req: HttpRequest
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
            Ok(HttpResponse::Ok().body("Your Back in action!"))

        },
        false => {
            Ok(HttpResponse::Unauthorized().body("Not Authenticated!"))

        }
    }
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

pub async fn logout(
    // session: Session,
    user: Identity
)
-> Result<impl Responder> {
    user.logout();
    Ok(HttpResponse::Ok())
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
