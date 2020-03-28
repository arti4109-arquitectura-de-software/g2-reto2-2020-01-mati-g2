#![feature(async_closure)]
#![feature(vec_remove_item)]

pub mod auth;
mod engine;
mod matches;
mod offers;
mod typed_tree;
pub mod user;
mod utils;

use auth::AuthManager;
use offers::OfferHandler;
use serde::Deserialize;
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
}

impl CtxData {
    pub fn new(db: sled::Db) -> Self {
        CtxData {
            auth_manager: AuthManager::new(db.clone(), jsonwebtoken::Validation::default()),
            offer_handler: OfferHandler::new(db),
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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
