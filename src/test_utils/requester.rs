use super::*;
use crate::{
    offers::{OfferEventRequest, OfferValue, Security, Side},
    user::User,
};
use futures::future::{BoxFuture, FutureExt};
use rand::prelude::*;

pub struct Requester {
    authorized: bool,
    client: reqwest::Client,
    valid_cookie: Option<String>,
    user_cred: Option<User>,
    valid_ip: String,
}

impl Requester {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();

        Requester {
            authorized: false,
            client,
            user_cred: None,
            valid_cookie: None,
            valid_ip: "d".to_string(),
        }
    }

    pub async fn start(mut self, n_requests: u32) {
        for _ in 1..n_requests {
            let index: usize = rand::thread_rng().gen_range(1, 9);
            self.call_method(index).await;
        }
    }

    fn call_method(&mut self, index: usize) -> BoxFuture<'_, ()> {
        match index {
            1 => self.signup().boxed(),
            2 => self.login().boxed(),
            3 => self.login_unregistered().boxed(),
            4 => self.login_wrong_password().boxed(),
            5 => self.send_offer().boxed(),
            6 => self.send_offer_wrong_ip().boxed(),
            7 => self.send_offer_blacklisted().boxed(),
            8 => self.logout().boxed(),
            _ => panic!("wrong index"),
        }
    }

    async fn signup(&mut self) -> () {
        let user = random_user();
        let response = self
            .client
            .post(&format!("{}{}", SIGNUP_ROUTE, self.valid_ip))
            .json(&user)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 201);
        self.user_cred = Some(user);
        self.authorized = true;
        self.update_valid_cookie(response);
    }

    async fn login(&mut self) {
        let user = if let Some(user) = &self.user_cred {
            user
        } else {
            self.signup().await;
            self.user_cred.as_ref().unwrap()
        };
        let response = self
            .client
            .post(&format!("{}{}", LOGIN_ROUTE, self.valid_ip))
            .json(user)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        self.authorized = true;
        self.update_valid_cookie(response);
    }

    async fn login_unregistered(&mut self) {
        let user = if let Some(user) = &self.user_cred {
            user
        } else {
            self.signup().await;
            self.user_cred.as_ref().unwrap()
        };
        let mut user = user.clone();
        user.password = "wrong".to_string();

        let response = self
            .client
            .post(&format!("{}{}", LOGIN_ROUTE, self.valid_ip))
            .json(&user)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 401);
        self.authorized = false;
    }

    async fn login_wrong_password(&mut self) {
        let user = random_user();
        let response = self
            .client
            .post(&format!("{}{}", LOGIN_ROUTE, self.valid_ip))
            .json(&user)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 401);
        self.authorized = false;
    }

    async fn send_offer(&mut self) {
        let offer_event = random_offer();
        let r = self
            .client
            .post(&format!("{}{}", OFFERS_ROUTE, self.valid_ip))
            .json(&offer_event)
            .send()
            .await
            .unwrap();
        if self.authorized {
            assert_eq!(200, r.status());
        } else {
            assert!(400 == r.status() || 401 == r.status());
        }
    }

    async fn send_offer_wrong_ip(&mut self) {
        if !self.authorized {
            self.login().await;
        }
        let offer_event = random_offer();
        let r = self
            .client
            .post(&format!("{}{}", OFFERS_ROUTE, "wrong_ip"))
            .json(&offer_event)
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 401);
        self.authorized = false;
    }

    async fn send_offer_blacklisted(&mut self) {
        if !self.authorized {
            self.login().await;
        }

        self.logout().await;

        let valid_cookie = self.valid_cookie.as_ref().unwrap();

        // Set previously valid cookie
        let r = self
            .client
            .post(&format!("{}{}", SET_COOKIE_ROUTE, valid_cookie))
            .send()
            .await
            .unwrap();
        assert_eq!(200, r.status());

        let offer_event = random_offer();
        // Black listed
        let r = self
            .client
            .post(&format!("{}{}", OFFERS_ROUTE, self.valid_ip))
            .json(&offer_event)
            .send()
            .await
            .unwrap();
        assert_eq!(401, r.status());
    }

    async fn logout(&mut self) {
        let r = self.client.post(LOGOUT_ROUTE).send().await.unwrap();
        if self.authorized {
            assert_eq!(200, r.status());
        } else {
            assert_eq!(400, r.status());
        }
        self.authorized = false;
    }

    fn update_valid_cookie(&mut self, r: reqwest::Response) {
        let valid_cookie = r.cookies().next().unwrap();
        self.valid_cookie = if valid_cookie.name() == "JWT" {
            Some(valid_cookie.value().to_string())
        } else {
            panic!(format!("Wrong cookie: {}", valid_cookie.name()));
        };
    }
}

fn random_user() -> User {
    let mut rng = rand::thread_rng();
    let (id, password): (u64, u64) = rng.gen();
    User {
        id: id.to_string(),
        password: password.to_string(),
    }
}

fn random_offer() -> OfferEventRequest {
    let mut rng = rand::thread_rng();
    let (is_buy, with_price): (bool, bool) = rng.gen();

    OfferEventRequest::Add(OfferValue {
        security: Security::BTC,
        side: if is_buy { Side::Buy } else { Side::Sell },
        amount: rng.gen_range(50u64, 100),
        price: if with_price {
            Some(rng.gen_range(50u64, 100))
        } else {
            None
        },
    })
}
