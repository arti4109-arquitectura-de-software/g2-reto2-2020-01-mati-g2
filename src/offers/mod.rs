mod handler;
mod model;

use crate::{auth, utils::json_body, with_ctx, Ctx, IpQueryParam};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::atomic;
use warp::{
    http::{header, Response, StatusCode},
    Filter, Rejection, Reply,
};
pub use {
    crate::engine::{MatchResult, Matches},
    handler::OfferHandler,
    model::{
        Offer, OfferEvent, OfferEventKey, OfferEventKeyed, OfferEventRequest, OfferValue, Security,
        Side,
    },
};

pub fn routes(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    make_offer(ctx.clone())
        .or(inner_make_offer(ctx.clone()))
        .or(set_cookie(ctx.clone()))
        .or(num_errors(ctx))
}

fn make_offer(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("offers")
        .and(warp::post())
        .and(warp::cookie(auth::JWT_COOKIE_NAME))
        .and(warp::query::<IpQueryParam>())
        .and(json_body::<OfferEventRequest>(6))
        .and(with_ctx(ctx))
        .and_then(
            async move |cookie: String,
                        ip: IpQueryParam,
                        event: OfferEventRequest,
                        ctx: Ctx|
                        -> Result<Response<_>, Infallible> {
                if let Err(_e) = ctx.auth_manager.authorize(ip.ip.as_str(), cookie.as_str()) {
                    return Ok(Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header(header::SET_COOKIE, auth::DELETE_JWT_COOKIE)
                        .body("")
                        .unwrap());
                }
                if ctx.test_auth {
                    let event = OfferEvent::from(event);
                    let key = ctx
                        .offer_handler
                        .persist_offer(event.clone())
                        .await
                        .unwrap();

                    let event = OfferEventKeyed::from_event(key, event);
                    let ans3 = ctx.offer_handler.send_offer(event).await;
                    ctx.offer_handler.send_matches(ans3);
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body("ok")
                        .unwrap())
                } else {
                    let event = OfferEvent::from(event);
                    let key = ctx
                        .offer_handler
                        .persist_offer(event.clone())
                        .await
                        .unwrap();

                    let event = OfferEventKeyed::from_event(key, event);
                    let ans3 = ctx.offer_handler.send_offer(event.clone());

                    let client = reqwest::Client::new();
                    let (r1, r2) = futures::future::join(
                        client
                            .post("http://127.0.0.1:3031/offers_inner")
                            .json(&event.clone())
                            .send(),
                        client
                            .post("http://127.0.0.1:3032/offers_inner")
                            .json(&event)
                            .send(),
                    )
                    .await;

                    let ((ans1, ans2), ans3) = futures::future::join(
                        futures::future::join(
                            r1.unwrap().json::<Matches>(),
                            r2.unwrap().json::<Matches>(),
                        ),
                        ans3,
                    )
                    .await;

                    let (ans1, ans2) = (ans1.unwrap(), ans2.unwrap());

                    if ans1 != ans2 || ans2 != ans3 || ans3 != ans1 {
                        println!("ERROR in offer processing");
                        if ans1 != ans3 {
                            ctx.num_errors
                                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                        if ans2 != ans3 {
                            ctx.num_errors
                                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body("error")
                            .unwrap())
                    } else {
                        println!("Good match");
                        ctx.offer_handler.send_matches(ans3);
                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body("ok")
                            .unwrap())
                    }
                }
            },
        )
}

fn inner_make_offer(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("offers_inner")
        .and(warp::post())
        .and(json_body::<OfferEventKeyed>(6))
        .and(with_ctx(ctx))
        .and_then(
            async move |event: OfferEventKeyed, ctx: Ctx| -> Result<_, Infallible> {
                let mut m = ctx.offer_handler.send_offer(event).await;
                let mut r = rand::thread_rng();
                if r.gen_bool(0.01) {
                    ctx.num_errors
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    m.result = match m.result {
                        MatchResult::Complete => MatchResult::None,
                        MatchResult::None => MatchResult::Complete,
                        MatchResult::Partial { .. } => MatchResult::None,
                    };
                }
                Ok(warp::reply::json(&m))
            },
        )
}

fn num_errors(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("num_errors")
        .and(warp::get())
        .and(with_ctx(ctx))
        .and_then(async move |ctx: Ctx| -> Result<Response<_>, Infallible> {
            Ok(Response::builder()
                .body(ctx.num_errors.load(atomic::Ordering::SeqCst).to_string())
                .unwrap())
        })
}

#[derive(Serialize, Deserialize)]
pub struct CookieSetter {
    cookie: String,
}
fn set_cookie(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("set_cookie")
        .and(warp::post())
        .and(warp::query::<CookieSetter>())
        .and_then(async move |cookie: CookieSetter| -> Result<_, Infallible> {
            Ok(warp::reply::with_header(
                warp::reply::json(&""),
                header::SET_COOKIE,
                format!("JWT={};", cookie.cookie),
            ))
        })
}
