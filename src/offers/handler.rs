use crate::engine::{Engine, KeyedBinaryHeapEngine, MatchResult, Matches};
use crate::matches::{MatchKey, MatchPersistor};
use crate::offers::{OfferEvent, OfferEventKey, OfferEventKeyed};
use crate::prelude::*;
use crossbeam_channel::{unbounded, Sender};
use std::sync::atomic::AtomicU64;
use std::{collections::HashMap, thread};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

pub struct OfferHandler {
    offers_db: sled::Tree,
    pub offer_counter: AtomicU64,
    sender_offer: Sender<OfferEventKeyed>,
    s_matches: Sender<Matches>,
    subscriptions: Arc<Mutex<HashMap<OfferEventKey, WaitMatches>>>,
}

impl OfferHandler {
    pub fn new(db: sled::Db) -> Self {
        let (s_offer, r_offer) = unbounded::<OfferEventKeyed>();
        let (s_matches, r_matches) = unbounded::<Matches>();
        let (s_matches2, r_matches2) = unbounded::<Matches>();

        let mut engine = Engine::<KeyedBinaryHeapEngine>::new(r_offer, s_matches);

        let _engine_handle = thread::spawn(move || engine.start());
        let mut persistor = MatchPersistor::new(r_matches2, db.clone());
        let _persistor_handle = thread::spawn(move || persistor.start());

        let mut offers_db = db.open_tree(<MatchKey as KeyOf>::PREFIX).unwrap();
        let offer_counter = AtomicU64::from(
            <sled::Tree as MonotonicTypedTree<MatchKey>>::get_max_key(&mut offers_db).unwrap(),
        );

        let subscriptions = Arc::new(Mutex::from(HashMap::<OfferEventKey, WaitMatches>::new()));
        let subscriptions2 = subscriptions.clone();
        thread::spawn(move || {
            while let Ok(m) = r_matches.recv() {
                let f = {
                    let mut _s = subscriptions2.lock().unwrap();
                    _s.remove(&m.key).unwrap()
                };

                f.complete(m);
            }
        });

        Self {
            offers_db,
            offer_counter,
            sender_offer: s_offer,
            s_matches: s_matches2,
            subscriptions,
        }
    }

    pub async fn persist_offer(&self, event: OfferEvent) -> sled::Result<OfferEventKey> {
        let (key, _): (OfferEventKey, _) = self
            .offers_db
            .insert_monotonic_atomic(&self.offer_counter, event.clone())?;
        self.offers_db.flush_async().await?;
        Ok(key)
    }

    pub fn send_offer(&self, event: OfferEventKeyed) -> impl Future<Output = Matches> {
        let fut = WaitMatches::new();
        {
            let mut m = self.subscriptions.lock().unwrap();
            m.insert(event.key().clone(), fut.clone());
        }
        self.sender_offer
            .send(event)
            .expect("Error on send offer though channel.");
        fut
    }

    pub fn send_matches(&self, matches: Matches) {
        if let MatchResult::None = matches.result {
        } else {
            self.s_matches.send(matches).unwrap();
        }
    }
}

#[derive(Clone)]
struct WaitMatches {
    state: Arc<Mutex<WaitMatchesState>>,
}

impl WaitMatches {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::from(WaitMatchesState {
                matches: None,
                waker: None,
            })),
        }
    }

    fn complete(&self, matches: Matches) {
        let mut state = self.state.lock().unwrap();

        state.matches = Some(matches);
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}
struct WaitMatchesState {
    matches: Option<Matches>,
    waker: Option<Waker>,
}

impl Future for WaitMatches {
    type Output = Matches;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Look at the shared state to see if the timer has already completed.
        let mut state = self.get_mut().state.lock().unwrap();

        if let Some(m) = state.matches.take() {
            Poll::Ready(m)
        } else {
            // Set waker so that the thread can wake up the current task
            // when the timer has completed, ensuring that the future is polled
            // again and sees that `completed = true`.
            //
            // It's tempting to do this once rather than repeatedly cloning
            // the waker each time. However, the `TimerFuture` can move between
            // tasks on the executor, which could cause a stale waker pointing
            // to the wrong task, preventing `TimerFuture` from waking up
            // correctly.
            //
            // N.B. it's possible to check for this using the `Waker::will_wake`
            // function, but we omit that here to keep things simple.
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
