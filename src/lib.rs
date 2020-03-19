use jsonwebtoken::{self as jwt, DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use std::time;
use warp::http::header;

mod typed_tree;
pub mod user;
mod utils;

use typed_tree::TypedTree;
use user::{BlackListed, BlackListedKey, User};

const SECRET: &'static str = "secret";

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    exp: u64, // seconds
    user_id: String,
    ip: String,
}

pub struct AuthManager<'a> {
    user_db: sled::Db,
    blacklist_db: sled::Db,
    jwt_encoding_key: EncodingKey,
    jwt_decoding_key: DecodingKey<'a>,
    jwt_validation: jwt::Validation,
    jwt_header: jwt::Header,
}

impl<'a> AuthManager<'a> {
    pub fn new(user_db: sled::Db, blacklist_db: sled::Db, jwt_validation: jwt::Validation) -> Self {
        let algorithm = jwt_validation.algorithms[0].clone();
        AuthManager {
            user_db,
            blacklist_db,
            jwt_encoding_key: EncodingKey::from_secret(SECRET.as_ref()),
            jwt_decoding_key: DecodingKey::from_secret(SECRET.as_ref()),
            jwt_validation,
            jwt_header: jwt::Header::new(algorithm),
        }
    }

    pub async fn authenticate<R>(&self, reply: R, user: User) -> Result<(), &'static str>
    where
        R: warp::Reply,
    {
        let pers_user = self.user_db.get_typed(&user.key()).unwrap();
        if let Some(pers_user) = pers_user {
            let hashed_pasword = bcrypt::hash(&user.password, bcrypt::DEFAULT_COST).unwrap();
            if bcrypt::verify(&pers_user.password, &hashed_pasword).unwrap() {
                let claims = Claims {
                    exp: now_plus_duration(time::Duration::from_secs(12 * 60 * 60)),
                    user_id: user.id,
                    ip: "11".to_string(),
                };
                let cookie =
                    jwt::encode(&self.jwt_header, &claims, &self.jwt_encoding_key).unwrap();

                self.set_cookie(reply, &cookie);
                Ok(())
            } else {
                Err("User or email incorrect")
            }
        } else {
            Err("User doesn't exist")
        }
    }

    pub fn authorize(&self, ip: &str, cookie: &str) -> bool {
        let data = jwt::decode::<Claims>(cookie, &self.jwt_decoding_key, &self.jwt_validation);
        if let Ok(data) = data {
            if data.claims.ip == ip {
                if !self.is_in_blacklist(cookie) {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    

    pub async fn logout<R>(&self, reply: R, cookie: &str) -> Result<warp::reply::WithHeader<R>, R>
    where
        R: warp::Reply,
    {
        let data = jwt::decode::<Claims>(cookie, &self.jwt_decoding_key, &self.jwt_validation);
        if let Ok(data) = data {
            let key = BlackListedKey(cookie);
            let value = BlackListed {
                exp: data.claims.exp,
            };
            self.blacklist_db.insert_typed(&key, value).unwrap();
            self.blacklist_db.flush_async().await.unwrap();
            Ok(warp::reply::with_header(
                reply,
                header::SET_COOKIE,
                format!("JWT=''; SameSite=Lax; HttpOnly"),
            ))
        } else {
            Err(reply)
        }
    }

    fn set_cookie<R>(&self, reply: R, cookie: &str) -> warp::reply::WithHeader<R>
    where
        R: warp::Reply,
    {
        warp::reply::with_header(
            reply,
            header::SET_COOKIE,
            format!("JWT={}; SameSite=Lax; HttpOnly", cookie),
        )
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
