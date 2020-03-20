mod handler;

use crate::{user::User, utils::json_body};
use handler::AuthManager;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use warp::{http::StatusCode, *};

#[derive(Clone)]
pub struct AuthRouter<'a> {
    db: sled::Db,
    auth_manager: Arc<AuthManager<'a>>,
}

impl<'a> AuthRouter<'a> {
    pub fn new(db: sled::Db) -> Self {
        AuthRouter {
            db: db.clone(),
            auth_manager: Arc::new(AuthManager::new(
                db.clone(),
                db,
                jsonwebtoken::Validation::default(),
            )),
        }
    }
    pub fn routes(self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + 'a {
        self.clone().login().or(self.signup())
    }

    fn login(self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + 'a {
      let v = self.clone();
        warp::path!("login")
            .and(warp::post())
            .and(json_body::<User>(4))
            .and_then(async move |user: User| {
              let c = &self;
                let reply = warp::reply::json(&"Logged in");
                c.clone().auth_manager.authenticate(reply, &user, "123")
            })
    }

    fn signup(self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + 'a {
        let v = &self;
        warp::path!("signup")
            .and(warp::post())
            .and(json_body::<User>(4))
            .and_then(|mut user: User| async move  {
                let c = v.clone();
                let reply = warp::reply::json(&"Logged in");
                let reply = warp::reply::with_status(reply, StatusCode::CREATED);
                c.auth_manager.clone().signup(reply, user, "123").await
            })
    }
}

#[derive(Debug)]
pub enum SignUpError {
    UserAlreadyCreated,
}
impl reject::Reject for SignUpError {}

#[derive(Debug)]
pub enum AuthorizeError {
    DifferentIp,
    InvalidToken,
    BlackListedToken,
}
impl reject::Reject for AuthorizeError {}

#[derive(Debug)]
pub enum AuthenticateError {
    UserDoesNotExist,
    IncorrectCombination,
}
impl reject::Reject for AuthenticateError {}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: &'static str,
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;
    if let Some(err) = err.find::<SignUpError>() {
        code = StatusCode::EXPECTATION_FAILED;
        message = "User taken";
    } else if let Some(err) = err.find::<AuthorizeError>() {
        code = StatusCode::UNAUTHORIZED;
        message = match err {
            AuthorizeError::DifferentIp => "Changed IP",
            _ => "Invalid Token",
        }
    } else if let Some(err) = err.find::<AuthenticateError>() {
        code = StatusCode::UNAUTHORIZED;
        message = match err {
            AuthenticateError::UserDoesNotExist => "User Id Doesn't exist",
            AuthenticateError::IncorrectCombination => "Incorrect Combination",
        }
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });
    Ok(warp::reply::with_status(json, code))
}
