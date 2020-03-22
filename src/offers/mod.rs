mod handler;
mod model;

use model::{OfferEvent, OfferEventRequest};

use crate::{utils::json_body, with_ctx, Ctx, auth, IpQueryParam};
use std::convert::Infallible;
use warp::{
    http::{header, Response, StatusCode},
    Filter, Rejection, Reply,
};

pub fn routes(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    make_offer(ctx)
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
                if let Err(e) = ctx.auth_manager.authorize(ip.ip.as_str(), cookie.as_str()) {
                    return Ok(Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header(header::SET_COOKIE, auth::DELETE_JWT_COOKIE)
                        .body("")
                        .unwrap());
                }

                let event = OfferEvent::from(event);
                handler::offer_event(event).await;
                Ok(Response::builder().status(StatusCode::OK).body("").unwrap())
            },
        )
}
