use std::sync::Arc;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{HttpResponse, web, Responder, Result, HttpRequest, HttpMessage, post};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::AppState;
use crate::db_actions::{authenticate, create_user, validate_email};
use crate::models::SwaggerErrorResponse;


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


    let resp = web::block(move || {
        let mut conn = state.pool.get()?;
        create_user(&mut conn, creds)
    })
    .await?;

    if let Ok(user) = resp {
        session.insert("user", user.id.clone()).unwrap();
        session.insert("role", user.role.clone()).unwrap();
        Ok(HttpResponse::Created().json(user))
    } else {
        Ok(HttpResponse::InternalServerError().body("Internal Server Error!"))
    }
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
    if validate_email(&creds.email).is_ok() {
        let resp = web::block(move || {
            let mut conn = state.pool.get()?;
            authenticate(creds, &mut conn)
        })
        .await?;
        
        if let Ok(user) = resp {
            let hash_check = bcrypt::verify(cred_two.password, &user.password_hash);
            match hash_check.is_ok() {
                true => {
                    Identity::login(&req.extensions(), user.email.into()).unwrap();
                    // let cookie = CookieBuilder::new("role", user.role.unwrap())
                    //     .secure(true)
                    //     .http_only(true)
                    //     .finish();
                    session.insert("role", user.role).unwrap();
                    Ok(HttpResponse::Ok().body("Back In Action!"))

                },
                false => {
                    Ok(HttpResponse::NotFound().body("User not found, please double check credintails"))
                }
            }
        } else {
            Ok(HttpResponse::NotFound().body("User not found, please double check credintails"))
        }
    } else {
        Ok(HttpResponse::NotAcceptable().body("Email Provided was invalid!"))
    }

}

#[utoipa::path(
    responses(
        (
            status = 200,
            description = "Successfully Logged Out User",
        ),
        (
            status = 404,
            description = "Session Not Found",
            body = SwaggerErrorResponse,
            example = json!(SwaggerErrorResponse::NotFound(String::from("Session Not Found")))
        ),
    )
)]
#[post("/logout")]
pub async fn logout(
    user: Identity
)
-> Result<impl Responder> {
    user.logout();
    Ok(HttpResponse::Ok().body("Successfully Loged Out"))
}

