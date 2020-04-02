#![feature(type_alias_impl_trait)]
#![feature(async_closure)]

use rand::prelude::*;
use reqwest;
use reto2::{
    offers::{OfferEventRequest, OfferValue, Security, Side},
    routes,
    test_utils::auth_test,
    user::User,
    Ctx, CtxData,
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let test_auth = true;
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
        println!("error_on: {}", error_on);

        for i in 0..3 {
            let db = sled::Config::default().temporary(true).open().unwrap(); // sled::open("database.sled")?;
            let ctx: Ctx = Arc::new(CtxData::new(
                db,
                test_auth,
                if i == 2 { Some(error_on) } else { None },
            ));
            let f = warp::serve(routes(ctx.clone())).run(([127, 0, 0, 1], 3030 + i));
            if i == 2 {
                tokio::spawn(start_petitions_disp(error_on));
                f.await;
            } else {
                tokio::spawn(f);
            }
        }
    }
}

const LOGIN_ROUTE: &str = "http://127.0.0.1:3030/login?ip=";
const SIGNUP_ROUTE: &str = "http://127.0.0.1:3030/signup?ip=";
const LOGOUT_ROUTE: &str = "http://127.0.0.1:3030/logout";
const OFFERS_ROUTE: &str = "http://127.0.0.1:3030/offers?ip=";
const SET_COOKIE_ROUTE: &str = "http://127.0.0.1:3030/set_cookie?cookie=";

async fn start_petitions_disp(error_on: u32) {
    let mut count = 0;
    std::thread::sleep(std::time::Duration::from_millis(3000));
    let ip = "1";
    let user1 = User {
        id: "user1".to_string(),
        password: "user1".to_string(),
    };
    let mut offer_event = OfferValue {
        security: Security::BTC,
        side: Side::Sell,
        amount: 8,
        price: Some(5),
    };

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    // Sign up
    let r = client
        .post(&format!("{}{}", SIGNUP_ROUTE, ip))
        .json(&user1)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);

    let r = create_offer(&client, ip, &offer_event).await;
    count += 1;
    if count == error_on {
        assert_eq!(500, r.status());
        return;
    } else {
        assert_eq!(200, r.status());
    }

    offer_event.side = Side::Buy;
    offer_event.amount = 6;
    let r = create_offer(&client, ip, &offer_event).await;
    count += 1;
    if count == error_on {
        assert_eq!(500, r.status());
        return;
    } else {
        assert_eq!(200, r.status());
    }

    offer_event.amount = 6;
    offer_event.price = None;
    let r = create_offer(&client, ip, &offer_event).await;
    count += 1;
    if count == error_on {
        assert_eq!(500, r.status());
        return;
    } else {
        assert_eq!(200, r.status());
    }

    offer_event.side = Side::Sell;
    offer_event.price = Some(3);
    let r = create_offer(&client, ip, &offer_event).await;
    count += 1;
    if count == error_on {
        assert_eq!(500, r.status());
        return;
    } else {
        assert_eq!(200, r.status());
    }

    offer_event.side = Side::Buy;
    offer_event.amount = 12;
    offer_event.price = Some(3);
    let r = create_offer(&client, ip, &offer_event).await;
    count += 1;
    if count == error_on {
        assert_eq!(500, r.status());
        return;
    } else {
        assert_eq!(200, r.status());
    }
}

async fn create_offer(
    client: &reqwest::Client,
    ip: &str,
    offer_event: &OfferValue,
) -> reqwest::Response {
    let r = client
        .post(&format!("{}{}", OFFERS_ROUTE, ip))
        .json(&OfferEventRequest::Add(offer_event.clone()))
        .send()
        .await
        .unwrap();
    r
}
