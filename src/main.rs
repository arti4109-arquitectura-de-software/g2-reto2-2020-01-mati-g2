#![feature(type_alias_impl_trait)]
#![feature(async_closure)]

use reto2::user::User;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::{Filter};
 
#[tokio::main]
async fn main() {
    let routes = all_routes();
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

type InfallibleR<T> = Result<T, Infallible>;

#[derive(Deserialize, Serialize)]
enum Side {
    Buy,
    Sell,
}

#[derive(Deserialize, Serialize)]
struct OfferReq {
    side: Side,
    amount: u32,
    limit: Option<u32>,
}

fn json_body<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone
where
    for<'de> T: Deserialize<'de> + Send,
{
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn all_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    login().or(signup())
}

fn login() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("login")
        .and(warp::post())
        .and(json_body::<User>())
        .and_then(async move |user: User| {
            let reply = warp::reply::json(&"Good");
            InfallibleR::Ok(reply)
        })
}

fn signup() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("signup")
        .and(warp::post())
        .and(json_body::<User>())
        .and_then(async move |json: User| InfallibleR::Ok(warp::reply::json(&vec!["Good"])))
}
