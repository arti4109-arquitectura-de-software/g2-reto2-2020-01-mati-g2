#![feature(type_alias_impl_trait)]
#![feature(async_closure)]

use rand::prelude::*;
use reqwest;
use reto2::{
    offers::{OfferEventRequest, OfferValue, Security, Side},
    routes,
    test_utils::{auth_test, availability_test},
    user::User,
    Ctx, CtxData,
};
use std::sync::Arc;
 
#[tokio::main]
async fn main() {
    let test_auth = false;
    if test_auth {
        let db = sled::Config::default().temporary(true).open().unwrap(); // sled::open("database.sled")?;
        let ctx: Ctx = Arc::new(CtxData::new(db, test_auth, None));
        tokio::spawn(auth_test(5, 10));
        warp::serve(routes(ctx.clone()))
            .run(([127, 0, 0, 1], 3030))
            .await;
    } else {
        let mut rng = rand::thread_rng();
        let error_on = rng.gen_range(1, 6);

        for i in 0..3 {
            let db = sled::Config::default().temporary(true).open().unwrap(); // sled::open("database.sled")?;
            let ctx: Ctx = Arc::new(CtxData::new(
                db,
                test_auth,
                if i == 2 { Some(error_on) } else { None },
            ));
            let f = warp::serve(routes(ctx.clone())).run(([127, 0, 0, 1], 3030 + i));
            if i == 2 {
                tokio::spawn(availability_test(5, 50));
                f.await;
            } else {
                tokio::spawn(f);
            }
        }
    }
}



