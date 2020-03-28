#![feature(type_alias_impl_trait)]
#![feature(async_closure)]

use reqwest;
use reto2::{
    offers::{OfferEventKey, OfferEventRequest, OfferValue, Security, Side},
    routes,
    user::User,
    Ctx, CtxData,
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    for i in 0..3 {
        let db = sled::Config::default().temporary(true).open().unwrap(); // sled::open("database.sled")?;
        let ctx: Ctx = Arc::new(CtxData::new(db));
        let f = warp::serve(routes(ctx.clone())).run(([127, 0, 0, 1], 3030 + i));
        if i == 2 {
            tokio::spawn(start_petitions());
            f.await;
        } else {
            tokio::spawn(f);
        }
    }
}

const LOGIN_ROUTE: &str = "http://127.0.0.1:3030/login?ip=";
const SIGNUP_ROUTE: &str = "http://127.0.0.1:3030/signup?ip=";
const LOGOUT_ROUTE: &str = "http://127.0.0.1:3030/logout";
const OFFERS_ROUTE: &str = "http://127.0.0.1:3030/offers?ip=";

async fn start_petitions() {
    std::thread::sleep(std::time::Duration::from_millis(3000));
    let ip = "1";
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let r = client
        .post(&format!("{}{}", SIGNUP_ROUTE, ip))
        .json(&User {
            id: "user2".to_string(),
            password: "user2".to_string(),
        })
        .send()
        .await
        .unwrap();
    println!("status: {} {}", r.status(), r.text().await.unwrap());

    let r = client
        .post(&format!("{}{}", OFFERS_ROUTE, ip))
        .json(&OfferEventRequest::Add(OfferValue {
            security: Security::BTC,
            side: Side::Buy,
            amount: 8,
            price: Some(5),
        }))
        .send()
        .await
        .unwrap();
    println!("status: {} {}", r.status(), r.text().await.unwrap());

    let r = client.post(LOGOUT_ROUTE).send().await.unwrap();
    println!("status: {} {}", r.status(), r.text().await.unwrap());
}
