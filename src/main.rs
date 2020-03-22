#![feature(type_alias_impl_trait)]
#![feature(async_closure)]

use reto2::{routes, Ctx, CtxData};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let db = sled::Config::default().temporary(true).open().unwrap(); // sled::open("database.sled")?;
    let ctx: Ctx = Arc::new(CtxData::new(db));
    warp::serve(routes(ctx)).run(([127, 0, 0, 1], 3030)).await;
}
