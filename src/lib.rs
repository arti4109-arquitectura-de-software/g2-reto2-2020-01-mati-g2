#![feature(async_closure)]

mod auth;
mod typed_tree;
pub mod user;
mod utils;

use auth::AuthRouter;
use warp::Filter;

pub struct RootRouter<'a> {
    db: sled::Db,
    auth_router: AuthRouter<'a>,
}

impl<'a> RootRouter<'a> {
    pub fn new(db: sled::Db) -> Self {
        RootRouter {
            db: db.clone(),
            auth_router: AuthRouter::new(db),
        }
    }
    pub fn routes(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + '_ {
        self.auth_router.routes()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
