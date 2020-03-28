use crate::engine::{Engine, KeyedBinaryHeapEngine, Matches};
use crate::matches::{MatchKey, MatchPersistor};
use crate::offers::{OfferEvent, OfferEventKeyed};
use crate::prelude::*;
use crossbeam_channel::{unbounded, Sender};
use std::sync::atomic::AtomicU64;
use std::thread;

pub struct OfferHandler {
    offers_db: sled::Tree,
    offer_counter: AtomicU64,
    sender_offer: Sender<OfferEventKeyed>,
}

impl OfferHandler {
    pub fn new(db: sled::Db) -> Self {
        let (s_offer, r_offer) = unbounded::<OfferEventKeyed>();
        let (s_matches, r_matches) = unbounded::<Matches>();

        let mut engine = Engine::<KeyedBinaryHeapEngine>::new(r_offer, s_matches);
        let mut persistor = MatchPersistor::new(r_matches, db.clone());

        let _engine_handle = thread::spawn(move || engine.start());
        let _persistor_handle = thread::spawn(move || persistor.start());

        let mut offers_db = db.open_tree(<MatchKey as KeyOf>::PREFIX).unwrap();
        let offer_counter = AtomicU64::from(
            <sled::Tree as MonotonicTypedTree<MatchKey>>::get_max_key(&mut offers_db).unwrap(),
        );

        Self {
            offers_db,
            offer_counter,
            sender_offer: s_offer,
        }
    }

    pub async fn offer_event(&self, event: OfferEvent) -> sled::Result<()> {
        let (key, _) = self
            .offers_db
            .insert_monotonic_atomic(&self.offer_counter, event.clone())?;
        println!("{:?}", event);
        self.sender_offer
            .send(OfferEventKeyed::from_event(key, event))
            .expect("Error on send offer though channel.");
        Ok(())
    }
}
