use std::str::FromStr;

use actix_utils::future;
use actix_web::{Error, FromRequest, dev, HttpRequest, error::{ErrorServiceUnavailable, ErrorUnauthorized}, HttpResponse};

use crate::models::Role;


pub struct SessionGuard {
    pub session: Option<String>,
}


impl FromRequest for SessionGuard {

    type Error = Error;
    type Future = future::Ready<Result<Self, Self::Error>>;


    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let sess_cookie = req.cookie("id");

        let session = sess_cookie.map(|c| c.value().to_string());

        future::ok(SessionGuard {session})
    }

    fn extract(req: &HttpRequest) -> Self::Future {
        Self::from_request(req, &mut dev::Payload::None)
    }
}

pub struct RoleGuard {
    pub role: Option<Role>
}

impl FromRequest for RoleGuard {
    type Error = Error;
    type Future = future::Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let role_cookie = req.cookie("role");
        let role = Role::from_str(role_cookie.unwrap().value());
        let role_opt: Option<Role> = role.ok();

        match role_opt {
            None => {
                todo!()
            },
            _ => {
                future::ok(RoleGuard { role: role_opt })
            }
            
        }

    }

    fn extract(req: &HttpRequest) -> Self::Future {
        Self::from_request(req, &mut dev::Payload::None)
    }
}


