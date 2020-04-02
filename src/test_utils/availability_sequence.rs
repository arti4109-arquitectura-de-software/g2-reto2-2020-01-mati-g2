use super::*;
use crate::{
    offers::{OfferEventRequest, OfferValue, Security, Side},
    user::User,
};

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
