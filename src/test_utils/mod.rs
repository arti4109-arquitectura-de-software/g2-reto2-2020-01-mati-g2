mod auth_sequence;
pub mod requester;

pub use auth_sequence::start_petitions_auth;

use futures::future::{BoxFuture, FutureExt};

const LOGIN_ROUTE: &str = "http://127.0.0.1:3030/login?ip=";
const SIGNUP_ROUTE: &str = "http://127.0.0.1:3030/signup?ip=";
const LOGOUT_ROUTE: &str = "http://127.0.0.1:3030/logout";
const OFFERS_ROUTE: &str = "http://127.0.0.1:3030/offers?ip=";
const SET_COOKIE_ROUTE: &str = "http://127.0.0.1:3030/set_cookie?cookie=";

pub async fn auth_test(n_processes: u32, n_requests: u32) {
    std::thread::sleep(std::time::Duration::from_millis(3000));
    let futs: Vec<BoxFuture<'_, ()>> = (1..n_processes)
        .map(|_| {
            let re = requester::Requester::new();
            re.start(n_requests).boxed()
        })
        .collect();

    futures::future::join_all(futs).await;
    println!("Ended testing");
    let resp = reqwest::get("http://127.0.0.1:3030/num_users")
        .await
        .unwrap();
    println!("number users: {}", resp.text().await.unwrap());
}
