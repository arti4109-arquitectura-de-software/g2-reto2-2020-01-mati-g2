#![feature(type_alias_impl_trait)]
#![feature(async_closure)]

use reto2::{routes, new_ctx, Ctx};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let db = sled::Config::default().temporary(true).open().unwrap(); // sled::open("database.sled")?;
    let ctx: Ctx = new_ctx(db);
    warp::serve(routes(ctx)).run(([127, 0, 0, 1], 3030)).await;
}

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
