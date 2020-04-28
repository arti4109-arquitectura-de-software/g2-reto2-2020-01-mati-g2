#![feature(type_alias_impl_trait)]
#![feature(async_closure)]

use rand::prelude::*;
use reto2::{
    routes,
    test_utils::{auth_test, availability_test, flexibility_test},
    Ctx, CtxData,
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let test_auth = true;
    let test_flexibility = true;
    
    if test_auth {
        let db = sled::Config::default().temporary(true).open().unwrap(); // sled::open("database.sled")?;
        let ctx: Ctx = Arc::new(CtxData::new(db, test_auth, None));
        if test_flexibility {
            tokio::spawn(flexibility_test(10, 50));
        } else {
            tokio::spawn(auth_test(10, 10));
        }
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
                tokio::spawn(availability_test(10, 10));
                f.await;
            } else {
                tokio::spawn(f);
            }
        }
    }
}
