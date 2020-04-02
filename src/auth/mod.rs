mod handler;

use crate::{user::User, utils::json_body, with_ctx, Ctx, IpQueryParam};
pub use handler::AuthManager;
use serde::Serialize;
use std::convert::Infallible;
use warp::{
    http::{header, Response, StatusCode},
    reply, Filter, Rejection, Reply,
};

pub const JWT_COOKIE_NAME: &'static str = "JWT";
pub const DELETE_JWT_COOKIE: &'static str = "JWT=; Max-Age=0;";

pub fn routes(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    login(ctx.clone())
        .or(signup(ctx.clone()))
        .or(logout(ctx.clone()))
        .or(num_users(ctx))
}

fn num_users(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("num_users")
        .and(warp::get())
        .and(with_ctx(ctx))
        .and_then(async move |ctx: Ctx| -> Result<Response<_>, Infallible> {
            let num_users = ctx.auth_manager.get_num_users();
            Ok(Response::builder().body(format!("{}", num_users)).unwrap())
        })
}

fn login(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("login")
        .and(warp::post())
        .and(json_body::<User>(4))
        .and(warp::query::<IpQueryParam>())
        .and(with_ctx(ctx))
        .and_then(
            async move |user: User, ip: IpQueryParam, ctx: Ctx| -> Result<_, Infallible> {
                let json: reply::Json;
                let code;
                let cookie: Option<String>;
                match ctx.auth_manager.authenticate(&user, ip.ip.as_str()) {
                    Ok(_cookie) => {
                        // warp::http::Response::builder()
                        //     .header(header::SET_COOKIE, cookie)
                        //     .status(StatusCode::OK)
                        //     .body(reply)
                        //     .unwrap()
                        code = StatusCode::OK;
                        json = warp::reply::json(&"Logged in");
                        cookie = Some(_cookie);
                    }
                    Err(auth_err) => {
                        code = StatusCode::UNAUTHORIZED;
                        let message = match auth_err {
                            AuthenticateError::UserDoesNotExist => "User id doesn't exist",
                            AuthenticateError::IncorrectCombination => "Incorrect combination",
                        };
                        let err = ErrorMessage {
                            code: code.as_u16(),
                            message,
                        };
                        json = warp::reply::json(&err);
                        cookie = None;
                    }
                };
                let cookie_ref = cookie.as_ref().map_or(DELETE_JWT_COOKIE, |s| s.as_str());

                let r = reply::with_status(json, code);
                Ok(reply::with_header(r, header::SET_COOKIE, cookie_ref))
            },
        )
}

fn logout(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("logout")
        .and(warp::post())
        .and(warp::cookie(JWT_COOKIE_NAME))
        .and(with_ctx(ctx))
        .and_then(
            async move |cookie: String,
                        ctx: Ctx|
                        -> Result<warp::reply::WithHeader<_>, Infallible> {
                let reply = warp::reply::json(&"Logged out");
                Ok(ctx.auth_manager.logout(reply, &cookie).await)
            },
        )
}

fn signup(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("signup")
        .and(warp::post())
        .and(json_body::<User>(4))
        .and(warp::query::<IpQueryParam>())
        .and(with_ctx(ctx))
        .and_then(
            async move |user: User,
                        ip: IpQueryParam,
                        ctx: Ctx|
                        -> Result<Box<dyn Reply>, Infallible> {
                let reply = warp::reply::json(&"Signed up");
                let reply = warp::reply::with_status(reply, StatusCode::CREATED);

                Ok(
                    match ctx.auth_manager.signup(reply, user, ip.ip.as_str()).await {
                        Ok(reply) => Box::new(reply),
                        Err(err) => {
                            let code = StatusCode::EXPECTATION_FAILED;
                            let message = match err {
                                SignUpError::UserAlreadyCreated => "User taken",
                            };
                            Box::new(reply_error(code, message))
                        }
                    },
                )
            },
        )
}

fn reply_error(code: StatusCode, message: &'static str) -> reply::WithStatus<reply::Json> {
    let err = ErrorMessage {
        code: code.as_u16(),
        message,
    };
    let json = warp::reply::json(&err);
    warp::reply::with_status(json, code)
}

#[derive(Debug)]
pub enum SignUpError {
    UserAlreadyCreated,
}

#[derive(Debug)]
pub enum AuthorizeError {
    DifferentIp,
    InvalidToken,
    BlackListedToken,
}

#[derive(Debug)]
pub enum AuthenticateError {
    UserDoesNotExist,
    IncorrectCombination,
}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: &'static str,
}
