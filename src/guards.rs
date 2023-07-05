use actix_session::Session;
use actix_utils::future;
use actix_web::{Error, FromRequest, dev, cookie::Cookie, HttpRequest};


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


async fn extract_session_key( session: Session) -> Option<String> {
    let api_key = session.get::<String>("api-key").unwrap();
    return api_key;

}
