mod handler;
mod model;

use crate::{auth, utils::json_body, with_ctx, Ctx, IpQueryParam};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
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
        .or(set_cookie(ctx))
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
                    let ans3 = ctx.offer_handler.offer_event(event).unwrap().await;

                    ctx.offer_handler.send_matches(ans3);
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body("ok")
                        .unwrap())
                } else {
                    let client = reqwest::Client::new();
                    let (r1, r2) = futures::future::join(
                        client
                            .post("http://127.0.0.1:3031/offers_inner")
                            .json(&event)
                            .send(),
                        client
                            .post("http://127.0.0.1:3032/offers_inner")
                            .json(&event)
                            .send(),
                    )
                    .await;

                    let (ans1, ans2) = futures::future::join(
                        r1.unwrap().json::<Matches>(),
                        r2.unwrap().json::<Matches>(),
                    )
                    .await;
                    let (ans1, ans2) = (ans1.unwrap(), ans2.unwrap());

                    let event = OfferEvent::from(event);
                    let ans3 = ctx.offer_handler.offer_event(event).unwrap().await;

                    if ans1 != ans2 || ans2 != ans3 || ans3 != ans1 {
                        println!("ERROR in offer processing");
                        Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
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
        .and(json_body::<OfferEventRequest>(6))
        .and(with_ctx(ctx))
        .and_then(
            async move |event: OfferEventRequest, ctx: Ctx| -> Result<_, Infallible> {
                let event = OfferEvent::from(event);
                let mut m = ctx.offer_handler.offer_event(event).unwrap().await;
                if let Some(error_on) = ctx.error_on {
                    if error_on as u64 == u64::from(m.key.clone()) {
                        m.result = match m.result {
                            MatchResult::Complete => MatchResult::None,
                            MatchResult::None => MatchResult::Complete,
                            MatchResult::Partial{..} => MatchResult::None,
                        };
                    }
                }
                Ok(warp::reply::json(&m))
            },
        )
}

#[derive(Serialize, Deserialize)]
pub struct CookieSetter {
    cookie: String,
}
fn set_cookie(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("set_cookie")
        .and(warp::post())
        .and(warp::query::<CookieSetter>())
        .and(with_ctx(ctx))
        .and_then(
            async move |cookie: CookieSetter, ctx: Ctx| -> Result<_, Infallible> {
                Ok(warp::reply::with_header(
                    warp::reply::json(&""),
                    header::SET_COOKIE,
                    format!("JWT={};", cookie.cookie),
                ))
            },
        )
}
