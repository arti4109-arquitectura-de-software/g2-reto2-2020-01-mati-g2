use jsonwebtoken::{self as jwt, DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use std::time;
use warp::{http::header, Reply};

use crate::auth::{AuthenticateError, AuthorizeError, SignUpError};
use crate::typed_tree::{KeyOf, TypedTree};
use crate::user::{BlackListed, BlackListedKey, User, UserKey};

const SECRET: &'static str = "secret";

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    exp: u64, // seconds
    user_id: String,
    ip: String,
}

#[derive(Clone)]
pub struct AuthManager {
    user_db: sled::Tree,
    blacklist_db: sled::Tree,
    jwt_encoding_key: EncodingKey,
    jwt_decoding_key: DecodingKey<'static>,
    jwt_validation: jwt::Validation,
    jwt_header: jwt::Header,
}

impl AuthManager {
    pub fn new(db: sled::Db, jwt_validation: jwt::Validation) -> Self {
        let algorithm = jwt_validation.algorithms[0].clone();
        AuthManager {
            user_db: db.open_tree(<UserKey as KeyOf>::PREFIX).unwrap(),
            blacklist_db: db.open_tree(<BlackListedKey as KeyOf>::PREFIX).unwrap(),
            jwt_encoding_key: EncodingKey::from_secret(SECRET.as_ref()),
            jwt_decoding_key: DecodingKey::from_secret(SECRET.as_ref()),
            jwt_validation,
            jwt_header: jwt::Header::new(algorithm),
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
        println!("bcrypt: {:?}", time::Instant::now().duration_since(start));

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
        println!(
            "comp_and_swap: {:?}",
            time::Instant::now().duration_since(start)
        );

        match comp_swap_result {
            Ok(()) => {
                start = time::Instant::now();
                self.user_db.flush_async().await.unwrap();
                println!("flush: {:?}", time::Instant::now().duration_since(start));
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
        warp::reply::with_header(
            reply,
            header::SET_COOKIE,
            format!("JWT=; Max-Age=0; SameSite=Lax; HttpOnly"),
        )
    }

    fn make_cookie(&self, user_id: String, ip: String) -> String {
        let max_age_secs = 12 * 60 * 60;
        let claims = Claims {
            exp: now_plus_duration(time::Duration::from_secs(max_age_secs)),
            user_id,
            ip,
        };
        let cookie = jwt::encode(&self.jwt_header, &claims, &self.jwt_encoding_key).unwrap();

        format!(
            "JWT={}; Max-Age={}; SameSite=Lax; HttpOnly",
            cookie,
            max_age_secs * 1000
        )
    }

    fn set_cookie<R>(&self, reply: R, user_id: String, ip: String) -> warp::reply::WithHeader<R>
    where
        R: warp::Reply,
    {
        let max_age_secs = 12 * 60 * 60;
        let claims = Claims {
            exp: now_plus_duration(time::Duration::from_secs(max_age_secs)),
            user_id,
            ip,
        };
        let cookie = jwt::encode(&self.jwt_header, &claims, &self.jwt_encoding_key).unwrap();

        warp::reply::with_header(
            reply,
            header::SET_COOKIE,
            format!(
                "JWT={}; Max-Age={}; SameSite=Lax; HttpOnly",
                cookie,
                max_age_secs * 1000
            ),
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
