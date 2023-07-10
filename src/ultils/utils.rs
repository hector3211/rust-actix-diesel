use std::sync::Arc;
use actix_web::{HttpResponse, web, Responder, Result, Error, HttpRequest, HttpMessage, get, post};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::AppState;
use crate::guards::SessionGuard;
pub fn generate_key()
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
