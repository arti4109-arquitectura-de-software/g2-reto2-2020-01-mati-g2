mod auth_sequence;
mod availability_sequence;
pub mod requester;

pub use auth_sequence::start_petitions_auth;

use crate::auth::PathBody;
use futures::future::{BoxFuture, FutureExt, TryFutureExt};
use requester::SerType;
use reqwest::Response;

const LOGIN_ROUTE: &str = "http://127.0.0.1:3030/login?ip=";
const SIGNUP_ROUTE: &str = "http://127.0.0.1:3030/signup?ip=";
const LOGOUT_ROUTE: &str = "http://127.0.0.1:3030/logout";
const OFFERS_ROUTE: &str = "http://127.0.0.1:3030/offers?ip=";
const CONFIG_PATH_ROUTE: &str = "http://127.0.0.1:3030/config_path";
const CONFIG_ROUTE: &str = "http://127.0.0.1:3030/config";
const SET_COOKIE_ROUTE: &str = "http://127.0.0.1:3030/set_cookie?cookie=";

pub async fn auth_test(n_processes: u32, n_requests: u32) {
    std::thread::sleep(std::time::Duration::from_millis(3000));
    let futs: Vec<BoxFuture<'_, ()>> = (0..n_processes)
        .map(|_| {
            let re = requester::Requester::new(None);
            re.start_auth(n_requests).boxed()
        })
        .collect();

    futures::future::join_all(futs).await;
    println!("Ended testing");
    let resp = reqwest::get("http://127.0.0.1:3030/num_users")
        .await
        .unwrap();
    println!("number users: {}", resp.text().await.unwrap());
}

pub async fn flexibility_test(n_processes: u32, n_requests: u32) {
    std::thread::sleep(std::time::Duration::from_millis(3000));
    println!("Test Started //////////////");

    let mut main_requester = requester::Requester::new(None);

    println!("Json /////////////////////////////////");
    main_requester.config_path("./assets/json.wasm").await;

    // Json
    let formats = [SerType::Json];
    let futs: Vec<BoxFuture<'_, ()>> = (0..n_processes)
        .map(|i| {
            let re = requester::Requester::new(Some(formats[(i % formats.len() as u32) as usize]));
            re.start_flexibility(n_requests).boxed()
        })
        .collect();

    futures::future::join_all(futs).await;

    main_requester.send_offer_ser(Some(SerType::MsgPack)).await;
    main_requester.send_offer_ser(Some(SerType::Cbor)).await;

    println!("Cbor, Json /////////////////////////////////");
    main_requester
        .config_path("./assets/no_avro_msg-pack.wasm")
        .await;

    // Cbor, Json
    let formats = [SerType::Cbor, SerType::Json];
    let futs: Vec<BoxFuture<'_, ()>> = (0..n_processes)
        .map(|i| {
            let re = requester::Requester::new(Some(formats[(i % formats.len() as u32) as usize]));
            re.start_flexibility(n_requests).boxed()
        })
        .collect();
    futures::future::join_all(futs).await;

    main_requester.send_offer_ser(Some(SerType::MsgPack)).await;

    println!("ALL FORMATS /////////////////////////////////");
    main_requester
        .config_path("./assets/all_formats.wasm")
        .await;

    // ALL
    let formats = [SerType::Cbor, SerType::Json, SerType::MsgPack];
    let futs: Vec<BoxFuture<'_, ()>> = (0..n_processes)
        .map(|i| {
            let re = requester::Requester::new(Some(formats[(i % formats.len() as u32) as usize]));
            re.start_flexibility(n_requests).boxed()
        })
        .collect();

    futures::future::join_all(futs).await;

    println!("Test Ended //////////////////");

    let mut json_cbor_times = Vec::new();
    for _ in 1..20 {
        json_cbor_times.push(
            main_requester
                .config_path("./assets/no_avro_msg-pack.wasm")
                .await,
        );
    }
    println!("{:?}", json_cbor_times);

    let mut all_formats_times = Vec::new();
    for _ in 1..20 {
        all_formats_times.push(
            main_requester
                .config_path("./assets/all_formats.wasm")
                .await,
        );
    }
    println!("{:?}", all_formats_times);

    let mut json_times = Vec::new();
    for _ in 1..20 {
        json_times.push(main_requester.config_path("./assets/json.wasm").await);
    }

    println!("{:?}", json_times);
}

pub async fn availability_test(n_processes: u32, n_requests: u32) {
    std::thread::sleep(std::time::Duration::from_millis(3000));

    let futs: Vec<BoxFuture<'_, ()>> = (0..n_processes)
        .map(|_| {
            let re = requester::Requester::new(None);
            re.start_availability(n_requests).boxed()
        })
        .collect();

    futures::future::join_all(futs).await;
    println!("Ended testing");
    let (resp1, resp2, resp3) = futures::future::join3(
        reqwest::get("http://127.0.0.1:3030/num_errors").and_then(|f| f.text()),
        reqwest::get("http://127.0.0.1:3031/num_errors").and_then(|f| f.text()),
        reqwest::get("http://127.0.0.1:3032/num_errors").and_then(|f| f.text()),
    )
    .await;
    let resp1: u32 = resp1.unwrap().parse().unwrap();
    let resp2: u32 = resp2.unwrap().parse().unwrap();
    let resp3: u32 = resp3.unwrap().parse().unwrap();

    assert_eq!(resp3 + resp2, resp1);
    println!("number generated errors: {}", resp1);
}
