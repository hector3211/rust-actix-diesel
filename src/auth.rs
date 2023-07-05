

use std::sync::Arc;

use actix_identity::Identity;
use actix_web::cookie::Cookie;

use bcrypt;
use actix_session::Session;
use actix_web::error::{InternalError, ErrorUnauthorized};

use actix_web::{HttpResponse, web, Responder, Result, Error};


use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tracing::log::info;

use crate::AppState;

use crate::db_actions::authenticate;
use crate::guards::SessionGuard;


#[derive(Debug, Deserialize,Serialize,Clone)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}



pub fn validate_session(
    sess_guard: SessionGuard
)
-> Result<HttpResponse, Error> {
    // let auth: Option<String> = session.get("auth").unwrap_or(None);

    if let Some(session) = sess_guard.session {
            Ok(HttpResponse::Ok().body(format!("Session: {}", session)))
        } else {
            Ok(HttpResponse::Unauthorized().body("Unauthorized!"))
    }
        
}

pub async fn secret(
    sess_guard: SessionGuard
)
-> Result<impl Responder> {
    let auth = validate_session(sess_guard).unwrap();

    // let user_email: Option<String> = session.get("user_id").unwrap_or(None);
    Ok(auth)
}

pub async fn login(
    creds: web::Json<Credentials>,
    session: Session,
    state: web::Data<Arc<AppState>>,
)
-> HttpResponse {
    let creds = creds.into_inner();
    let cred_two = creds.clone();

    let mut conn = state.pool.get().unwrap();
    let resp = web::block(move || {
        authenticate(creds, &mut conn)
    })
    .await
    .map_err(|err| ErrorUnauthorized(err))
    .expect("web block functin failed!");
    
    let user = resp.unwrap();
    let hash_check = bcrypt::verify(cred_two.password, &user.password_hash).unwrap();
    match hash_check {
       true => {
             match session.get::<String>("user").unwrap() {
                Some(key) => {
                    if key == cred_two.email {
                        HttpResponse::Ok().body("Your Back in action!")
                    } else {
                        todo!()
                    }
                }
                None => {
                    session.insert("user", cred_two.email).unwrap();
                    HttpResponse::Found().body("All logged in, Welcome!")
                }
            }

        },
        false => {
            HttpResponse::Unauthorized().body("Not Authenticated!")

        }
    }
}

pub async fn apikey_to_state(
    state: web::Data<Arc<AppState>>,
    key: &str
) {
    let mut keys = state.api_keys.lock().unwrap();
    keys.push(key.to_string());
}

pub async fn logout(
    session: Session,
)
-> Result<impl Responder> {
    session.clear();
    Ok(HttpResponse::Ok())
}

fn generate_key()
-> String {
    let api_key:String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    return api_key;
}
