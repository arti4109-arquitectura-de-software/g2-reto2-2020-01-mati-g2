#![feature(type_alias_impl_trait)]
#![feature(async_closure)]

use reto2::{RootRouter};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc};
use warp::{http::StatusCode, Filter};

#[tokio::main]
async fn main() {
    let db = sled::Config::default().temporary(true).open().unwrap(); // sled::open("database.sled")?;
    let router = RootRouter::new(db);
    let routes = router.routes();
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




