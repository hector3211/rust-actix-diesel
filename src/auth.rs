use bcrypt;
use actix_session::Session;
use actix_web::error::{InternalError, ErrorUnauthorized};
use actix_web::http::Error;
use actix_web::{HttpResponse, post, web, Responder, get};
use diesel::prelude::*;
use diesel::{QueryDsl, ExpressionMethods, SelectableHelper, PgConnection};
use serde::{Deserialize, Serialize};

use crate::{DbError, DbPool};
use crate::{models::User, schema::users};


#[derive(Debug, Deserialize,Serialize,Clone)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}


pub fn authenticate(
    creds: Credentials,
    conn: &mut PgConnection
) 
-> Result<User, DbError> {
    // let hash_check = bcrypt::hash(creds.password, DEFAULT_COST).unwrap();
    let user: User = users::table
        .filter(users::email.eq(&creds.email))
        .select(User::as_select())
        .get_result(conn)?;


    // let hash_check = bcrypt::verify(creds.password, &user.password_hash).unwrap();

    Ok(user)
}

pub fn validate_session(
    session: &Session
)
-> Result<String, HttpResponse> {
    let user_email: Option<String> = session.get("user_id").unwrap_or(None);

    match user_email {
        Some(id) => {
            session.renew();
            Ok(id)
        },
        None => Err(HttpResponse::Unauthorized().body("Unauthorized!"))
        
    }
}

#[get("/user/secret")]
pub async fn secret(
    session: Session
)
-> Result<impl Responder, Error> {
    let _ = validate_session(&session)
        .map_err(|err| InternalError::from_response("",err));

    Ok("secret renewed")
}

#[post("/user/login")]
pub async fn login(
    creds: web::Json<Credentials>,
    session: Session,
    pool: web::Data<DbPool>
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
                }
                None => {
                    session.insert("user_id", user.email).unwrap();
                    HttpResponse::Ok().body("All logged in, Welcome!")
                }
            }

        },
        false => {
            HttpResponse::Unauthorized().body("Not Authenticated!")
        }
    }


}
