
use actix_identity::Identity;
use actix_web::cookie::Cookie;
use bcrypt;
use actix_session::Session;
use actix_web::error::{InternalError, ErrorUnauthorized};

use actix_web::{HttpResponse, web, Responder, Error, Result, HttpRequest, HttpMessage};
use diesel::prelude::*;
use diesel::{QueryDsl, ExpressionMethods, SelectableHelper, PgConnection};
use serde::{Deserialize, Serialize};

use crate::{DbError, DbPool};
use crate::{models::User, schema::users};
use crate::db_actions::authenticate;


#[derive(Debug, Deserialize,Serialize,Clone)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}



pub fn validate_session(
    session: &Session
)
-> Result<String, HttpResponse> {
    let user_email: Option<String> = session.get("user_id").unwrap_or(None);
    // let auth: Option<String> = session.get("auth").unwrap_or(None);

    match user_email {
        Some(id) => {
            Ok(id)
        },
        None => Err(HttpResponse::Unauthorized().body("Unauthorized!"))
        
    }
}

pub async fn secret(
    session: Session
)
-> Result<impl Responder> {
    let auth = validate_session(&session)
        .map_err(|err| InternalError::from_response("",err));

    // let user_email: Option<String> = session.get("user_id").unwrap_or(None);
    Ok(auth)
}

pub async fn login(
    creds: web::Json<Credentials>,
    session: Session,
    pool: web::Data<DbPool>,
    req: HttpRequest
)
-> impl Responder {
    let creds = creds.into_inner();
    let cred_two = creds.clone();

    let resp = web::block(move || {
        let mut conn = pool.get()?;
        authenticate(creds, &mut conn)
    })
    .await
    .map_err(|err| ErrorUnauthorized(err));
    
    let user = resp.unwrap().unwrap();
    let hash_check = bcrypt::verify(cred_two.password, &user.password_hash).unwrap();
    match hash_check {
       true => {
             match session.get::<String>("user_id").unwrap() {
                Some(_key) => {
                    HttpResponse::Ok().body("All logged in, Welcome!")
                    // Redirect::to("/").using_status_code(StatusCode::FOUND)
                }
                None => {
                    session.insert("user_id", user.email).unwrap();
                    // session.insert("auth", "yo123").unwrap();
                    // Cookie::build("auth", "yo123")
                    //     .path("/")
                    //     .secure(true)
                    //     .http_only(false)
                    //     .finish();
                    // Identity::login(&req.extensions(), cred_two.email.to_owned()).unwrap();
                    HttpResponse::Ok().body("All logged in, Welcome!")
                    // Redirect::to("/").using_status_code(StatusCode::FOUND)
                }
            }

        },
        false => {
            HttpResponse::Unauthorized().body("Not Authenticated!")

        }
    }
}

pub async fn logout(
    session: Session,
    id: Identity
)
-> Result<impl Responder> {
    id.logout();
    session.clear();
    Ok(HttpResponse::Ok())
}
