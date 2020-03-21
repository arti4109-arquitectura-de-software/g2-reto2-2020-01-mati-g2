#![feature(async_closure)]

pub mod auth;
mod typed_tree;
pub mod user;
mod utils;

use auth::AuthManager;
use std::sync::Arc;
use warp::{Filter, Reply, Rejection};

pub fn routes(ctx: Ctx) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    auth::routes(ctx)
}

pub type Ctx = Arc<CtxData>;

pub fn with_ctx(
    ctx: Ctx,
) -> impl Filter<Extract = (Ctx,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ctx.clone())
}

pub fn new_ctx(db: sled::Db) -> Ctx {
    Arc::new(CtxData::new(db))
}
pub struct CtxData {
    auth_manager: AuthManager,
}

impl CtxData {
    pub fn new(db: sled::Db) -> Self {
        CtxData {
            auth_manager: AuthManager::new(db, jsonwebtoken::Validation::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
