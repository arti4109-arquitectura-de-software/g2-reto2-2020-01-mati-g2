use super::*;
use crate::{
  offers::{OfferEventRequest, OfferValue, Security, Side},
  user::User,
};

pub async fn start_petitions_auth() {
  std::thread::sleep(std::time::Duration::from_millis(3000));
  let ip1 = "1";
  let user1 = User {
      id: "user1".to_string(),
      password: "user1".to_string(),
  };
  let ip2 = "2";
  let user2 = User {
      id: "user2".to_string(),
      password: "user2".to_string(),
  };
  let offer_event = OfferEventRequest::Add(OfferValue {
      security: Security::BTC,
      side: Side::Buy,
      amount: 8,
      price: Some(5),
  });

  let client = reqwest::Client::builder()
      .cookie_store(true)
      .build()
      .unwrap();

  // Sign up
  let r = client
      .post(&format!("{}{}", SIGNUP_ROUTE, ip1))
      .json(&user1)
      .send()
      .await
      .unwrap();
  assert_eq!(r.status(), 201);

  // Get valid cookie
  let valid_cookie = r.cookies().next().unwrap();
  let valid_cookie = if valid_cookie.name() == "JWT" {
      valid_cookie.value()
  } else {
      panic!("Wrong cookie");
  };

  // Create offer
  let r = client
      .post(&format!("{}{}", OFFERS_ROUTE, ip1))
      .json(&offer_event)
      .send()
      .await
      .unwrap();
  assert_eq!(200, r.status());

  // Log out
  let r = client.post(LOGOUT_ROUTE).send().await.unwrap();
  assert_eq!(200, r.status());

  // Logged out
  let r = client
      .post(&format!("{}{}", OFFERS_ROUTE, ip1))
      .json(&offer_event)
      .send()
      .await
      .unwrap();
  assert_eq!(400, r.status());

  // Set valid cookie
  let r = client
      .post(&format!("{}{}", SET_COOKIE_ROUTE, valid_cookie))
      .send()
      .await
      .unwrap();
  assert_eq!(200, r.status());

  // Black listed
  let r = client
      .post(&format!("{}{}", OFFERS_ROUTE, ip1))
      .json(&offer_event)
      .send()
      .await
      .unwrap();
  assert_eq!(401, r.status());

  // Log in
  let r = client
      .post(&format!("{}{}", LOGIN_ROUTE, ip1))
      .json(&user1)
      .send()
      .await
      .unwrap();
  assert_eq!(r.status(), 200);

  // Wrong ip
  let r = client
      .post(&format!("{}{}", OFFERS_ROUTE, ip2))
      .json(&offer_event)
      .send()
      .await
      .unwrap();
  assert_eq!(r.status(), 401);

  // Log in unregistered
  let r = client
      .post(&format!("{}{}", LOGIN_ROUTE, ip1))
      .json(&user2)
      .send()
      .await
      .unwrap();
  assert_eq!(r.status(), 401);

  // Log in wrong password
  let mut user1_mod = user1.clone();
  user1_mod.password = "nn".to_string();
  let r = client
      .post(&format!("{}{}", LOGIN_ROUTE, ip1))
      .json(&user1_mod)
      .send()
      .await
      .unwrap();
  assert_eq!(r.status(), 401);
}