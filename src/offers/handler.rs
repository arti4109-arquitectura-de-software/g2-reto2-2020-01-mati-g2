use crate::engine::{Engine, KeyedBinaryHeapEngine, Matches, MatchResult};
use crate::matches::{MatchKey, MatchPersistor, };
use crate::offers::{OfferEvent, OfferEventKeyed, OfferEventKey};
use crate::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::atomic::AtomicU64;
use std::thread;

pub struct OfferHandler {
    offers_db: sled::Tree,
    pub offer_counter: AtomicU64,
    sender_offer: Sender<OfferEventKeyed>,
    r_matches: Receiver<Matches>,
    s_matches: Sender<Matches>,
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

        Self {
            offers_db,
            offer_counter,
            sender_offer: s_offer,
            s_matches: s_matches2,
            r_matches,
        }
    }

    pub fn offer_event(&self, event: OfferEvent) -> sled::Result<(OfferEventKey, Matches)> {
        let (key, _): (OfferEventKey, _) = self
            .offers_db
            .insert_monotonic_atomic(&self.offer_counter, event.clone())?;
        println!("{:?}", event);
        self.sender_offer
            .send(OfferEventKeyed::from_event(key.clone(), event))
            .expect("Error on send offer though channel.");
        Ok((key, self.r_matches.recv().unwrap()))
    }

    pub fn send_matches(&self, matches: Matches) {
        if let MatchResult::None = matches.result {
        } else {
            self.s_matches.send(matches).unwrap();
        }
    }
}
