#![feature(async_closure)]
#![feature(vec_remove_item)]

pub mod auth;
mod engine;
mod matches;
pub mod offers;
pub mod test_utils;
mod typed_tree;
pub mod user;
mod utils;

use auth::AuthManager;
use offers::OfferHandler;
use serde::Deserialize;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};

pub fn routes(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    auth::routes(ctx.clone()).or(offers::routes(ctx))
}

pub type Ctx = Arc<CtxData>;

pub fn with_ctx(
    ctx: Ctx,
) -> impl Filter<Extract = (Ctx,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ctx.clone())
}

pub struct CtxData {
    auth_manager: AuthManager,
    offer_handler: OfferHandler,
    test_auth: bool,
    error_on: Option<u32>,
    num_errors: AtomicU32,
}

impl CtxData {
    pub fn new(db: sled::Db, test_auth: bool, error_on: Option<u32>) -> Self {
        CtxData {
            auth_manager: AuthManager::new(db.clone(), jsonwebtoken::Validation::default()),
            offer_handler: OfferHandler::new(db),
            test_auth,
            error_on,
            num_errors: AtomicU32::new(0),
        }
    }
}

#[derive(Deserialize)]
pub struct IpQueryParam {
    ip: String,
}

pub mod prelude {
    pub use super::{
        bincode_des, bincode_ser, derive_key_of, derive_monotonic_key, derive_simple_struct,
        typed_tree::{KeyOf, MonotonicTypedTree, TypedTree},
        with_ctx, Ctx,
    };
}

#[cfg(test)]
mod tests {
    use reqwest;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
