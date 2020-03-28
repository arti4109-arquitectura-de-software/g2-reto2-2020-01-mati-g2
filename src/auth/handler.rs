use jsonwebtoken::{self as jwt, DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use std::time;
use warp::{http::header, Reply};

use crate::auth::{
    AuthenticateError, AuthorizeError, SignUpError, DELETE_JWT_COOKIE, JWT_COOKIE_NAME,
};
use crate::prelude::*;
use crate::typed_tree::{KeyOf, TypedTree};
use crate::user::{BlackListed, BlackListedKey, User, UserKey};

const SECRET: &'static str = "secret";

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    exp: u64, // seconds
    user_id: String,
    ip: String,
}

pub struct AuthManager {
    user_db: sled::Tree,
    blacklist_db: sled::Tree,
    jwt_encoding_key: EncodingKey,
    jwt_decoding_key: DecodingKey<'static>,
    jwt_validation: jwt::Validation,
    jwt_header: jwt::Header,
    _blacklist_interval_handle: tokio::task::JoinHandle<()>,
}

impl AuthManager {
    pub fn new(db: sled::Db, jwt_validation: jwt::Validation) -> Self {
        let algorithm = jwt_validation.algorithms[0].clone();
        let blacklist_db = db.open_tree(<BlackListedKey as KeyOf>::PREFIX).unwrap();

        let mut interval = tokio::time::interval(time::Duration::new(60 * 5, 0)); // 5 min
        let interval_db = blacklist_db.clone();
        let handle = tokio::spawn(async move {
            loop {
                interval.tick().await;
                clear_blacklist(&interval_db).await;
            }
        });

        AuthManager {
            user_db: db.open_tree(<UserKey as KeyOf>::PREFIX).unwrap(),
            blacklist_db: blacklist_db.clone(),
            jwt_encoding_key: EncodingKey::from_secret(SECRET.as_ref()),
            jwt_decoding_key: DecodingKey::from_secret(SECRET.as_ref()),
            jwt_validation,
            jwt_header: jwt::Header::new(algorithm),
            _blacklist_interval_handle: handle,
        }
    }

    pub fn authenticate(&self, user: &User, ip: &str) -> Result<String, AuthenticateError> {
        let pers_user = self.user_db.get_typed(&user.key()).unwrap();
        if let Some(pers_user) = pers_user {
            if bcrypt::verify(&user.password, &pers_user.password).unwrap() {
                Ok(self.make_cookie(pers_user.id, ip.to_string()))
            } else {
                Err(AuthenticateError::IncorrectCombination)
            }
        } else {
            Err(AuthenticateError::UserDoesNotExist)
        }
    }

    pub fn authorize(&self, ip: &str, cookie: &str) -> Result<(), AuthorizeError> {
        let data = jwt::decode::<Claims>(cookie, &self.jwt_decoding_key, &self.jwt_validation);
        if let Ok(data) = data {
            if data.claims.ip == ip {
                if self.is_in_blacklist(cookie) {
                    Err(AuthorizeError::BlackListedToken)
                } else {
                    Ok(())
                }
            } else {
                Err(AuthorizeError::DifferentIp)
            }
        } else {
            Err(AuthorizeError::InvalidToken)
        }
    }

    pub async fn signup<R>(
        &self,
        reply: R,
        mut user: User,
        ip: &str,
    ) -> Result<impl warp::Reply, SignUpError>
    where
        R: warp::Reply,
    {
        let mut start = time::Instant::now();
        let hashed_pasword = bcrypt::hash(&user.password, 9).unwrap();
        println!("bcrypt: {:?}", start.elapsed());

        user.password = hashed_pasword;
        let user_id = user.id.clone();

        start = time::Instant::now();
        let comp_swap_result = self
            .user_db
            .compare_and_swap(
                &UserKey(user_id.as_str()),
                None as Option<&[u8]>,
                Some(user),
            )
            .unwrap();
        println!("comp_and_swap: {:?}", start.elapsed());

        match comp_swap_result {
            Ok(()) => {
                start = time::Instant::now();
                self.user_db.flush_async().await.unwrap();
                println!("flush: {:?}", start.elapsed());
                Ok(self.set_cookie(reply, user_id, ip.to_string()))
            }
            Err(_) => Err(SignUpError::UserAlreadyCreated),
        }
    }

    pub async fn logout<R>(&self, reply: R, cookie: &str) -> warp::reply::WithHeader<R>
    where
        R: Reply,
    {
        let data = jwt::decode::<Claims>(cookie, &self.jwt_decoding_key, &self.jwt_validation);
        if let Ok(data) = data {
            let key = BlackListedKey(cookie);
            let value = BlackListed {
                exp: data.claims.exp,
            };
            self.blacklist_db.insert_typed(&key, value).unwrap();
            self.blacklist_db.flush_async().await.unwrap();
            self.remove_cookie(reply)
        } else {
            self.remove_cookie(reply)
        }
    }

    pub fn remove_cookie<R>(&self, reply: R) -> warp::reply::WithHeader<R>
    where
        R: warp::Reply,
    {
        warp::reply::with_header(reply, header::SET_COOKIE, DELETE_JWT_COOKIE)
    }

    fn make_cookie(&self, user_id: String, ip: String) -> String {
        let max_age_secs = 12 * 60 * 60; // 12 hours
        let claims = Claims {
            exp: now_plus_duration(time::Duration::from_secs(max_age_secs)),
            user_id,
            ip,
        };
        let cookie = jwt::encode(&self.jwt_header, &claims, &self.jwt_encoding_key).unwrap();

        format!(
            "{}={}; Max-Age={}; SameSite=Lax; HttpOnly",
            JWT_COOKIE_NAME,
            cookie,
            max_age_secs * 1000
        )
    }

    fn set_cookie<R>(&self, reply: R, user_id: String, ip: String) -> warp::reply::WithHeader<R>
    where
        R: warp::Reply,
    {
        let cookie = self.make_cookie(user_id, ip);

        warp::reply::with_header(reply, header::SET_COOKIE, cookie)
    }

    fn is_in_blacklist(&self, cookie: &str) -> bool {
        let key = BlackListedKey(cookie);
        let black = self.blacklist_db.get_typed(&key).unwrap();
        if let Some(black) = black {
            let not_expired = now_in_secs() > black.exp;
            not_expired
        } else {
            false
        }
    }
}

fn now_in_secs() -> u64 {
    let start = time::SystemTime::now();
    start
        .duration_since(time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

fn now_plus_duration(duration: time::Duration) -> u64 {
    let start = time::SystemTime::now().checked_add(duration).unwrap();
    start
        .duration_since(time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

async fn clear_blacklist(blacklist_db: &sled::Tree) {
    let now = now_in_secs();
    let mut batch = sled::Batch::default();
    let mut count = 0;
    blacklist_db.iter().for_each(|res| {
        let (key, v) = res.unwrap();
        let v: BlackListed = bincode_des!(v.as_ref()).unwrap();
        if now > v.exp {
            count += 1;
            batch.remove(key);
        }
    });
    println!("blacklist: {} deleted", count);
    blacklist_db.apply_batch(batch).unwrap()
}
