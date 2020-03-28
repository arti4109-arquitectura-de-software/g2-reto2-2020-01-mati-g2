mod handler;
mod model;

pub use {
    crate::engine::Matches,
    handler::OfferHandler,
    model::{
        Offer, OfferEvent, OfferEventKey, OfferEventKeyed, OfferEventRequest, OfferValue, Security,
        Side,
    },
};

use crate::{auth, utils::json_body, with_ctx, Ctx, IpQueryParam};
use std::convert::Infallible;
use warp::{
    http::{header, Response, StatusCode},
    Filter, Rejection, Reply,
};

pub fn routes(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    make_offer(ctx.clone()).or(inner_make_offer(ctx))
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
                let ans3 = ctx.offer_handler.offer_event(event).unwrap();

                if ans1 != ans2 || ans2 != ans3 || ans3 != ans1 {
                    println!("ERROR in offer processing");
                } else {
                    println!("Good match");
                    ctx.offer_handler.send_matches(ans3);
                }
                Ok(Response::builder().status(StatusCode::OK).body("ok").unwrap())
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
                let m = ctx.offer_handler.offer_event(event).unwrap();
                Ok(warp::reply::json(&m))
            },
        )
}
