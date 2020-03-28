mod engine_keyedheap;
pub mod offer_ord;

use crate::offers::{Offer, OfferEventKey, OfferEventKeyed, Side};
use crossbeam_channel::{self, Receiver, Sender};
use serde::{Serialize, Deserialize};
pub use engine_keyedheap::KeyedBinaryHeapEngine;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum MatchResult {
    Complete,
    Partial { offer: Offer, to_substract: u64 },
    None,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Matches {
    pub result: MatchResult,
    pub completed: Vec<Offer>,
}

pub trait EngineDataStruct: Sized {
    fn match_offer(
        &mut self,
        matches: &mut Vec<Offer>,
        offer: Offer,
        other: &mut Self,
    ) -> MatchResult;
    fn delete_key(&mut self, key: &OfferEventKey) -> bool;
    fn with_capacity(capacity: usize) -> Self;
}

pub struct Engine<T>
where
    T: EngineDataStruct,
{
    sell_offers: T,
    // market_sell_offers: Vec<MarketEngineOffer>,
    buy_offers: T,
    // market_buy_offers: Vec<MarketEngineOffer>,
    matches: Vec<Offer>,
    receiver: Receiver<OfferEventKeyed>,
    not_processed: Vec<OfferEventKeyed>,
    last_processed: Option<u64>,
    sender: Sender<Matches>,
}

impl<T> Engine<T>
where
    T: EngineDataStruct,
{
    pub fn new(receiver: Receiver<OfferEventKeyed>, sender: Sender<Matches>) -> Self {
        Engine {
            sell_offers: T::with_capacity(24),
            not_processed: Vec::new(),
            last_processed: None,
            // market_sell_offers: Vec::with_capacity(24),
            buy_offers: T::with_capacity(24),
            // market_buy_offers: Vec::with_capacity(24),
            matches: Vec::with_capacity(24),
            sender,
            receiver,
        }
    }

    pub fn start(&mut self) {
        let mut counter = 0u32;
        while let Ok(mut offer) = self.receiver.recv() {
            let seq = u64::from_be_bytes(offer.key().clone().into());
            if let Some(last_processed) = self.last_processed {
                if seq != last_processed + 1 {
                    self.not_processed.push(offer);
                    continue;
                }
            }
            self.last_processed = Some(seq);

            counter += 1;
            loop {
                match offer {
                    OfferEventKeyed::Add(offer) => {
                        let matches = self.process_offer(offer);
                        println!("Engine {} - Match: {:?}", counter, matches);

                        self.sender.send(matches).unwrap();
                    }
                    OfferEventKeyed::Delete(_, k) => {
                        let deleted = self.delete_offer(&k);
                        println!("Engine {} - Deleted {}", counter, deleted);
                    }
                }
                if let Some(o) = self.get_next() {
                    offer = o;
                } else {
                    break;
                }
            }
        }
    }

    fn get_next(&mut self) -> Option<OfferEventKeyed> {
        if let None = self.last_processed {
            return None;
        }
        let last_processed = self.last_processed.unwrap();
        let key = OfferEventKey((last_processed + 1).to_be_bytes());
        self.not_processed
            .remove_item(&OfferEventKeyed::Delete(key.clone(), key))
    }

    pub fn process_offer(&mut self, offer: Offer) -> Matches {
        let (same_offers, opposite_offers) = match offer.value.side {
            Side::Buy => (&mut self.buy_offers, &mut self.sell_offers),
            Side::Sell => (&mut self.sell_offers, &mut self.buy_offers),
        };

        let result = opposite_offers.match_offer(&mut self.matches, offer.clone(), same_offers);
        match &result {
            MatchResult::Complete => self.matches.push(offer),
            MatchResult::Partial { offer: o, .. } if o.key != offer.key => self.matches.push(offer),
            _ => {}
        }
        let completed: Vec<_> = self.matches.drain(..self.matches.len()).collect();

        Matches { completed, result }
    }

    pub fn delete_offer(&mut self, key: &OfferEventKey) -> bool {
        if self.buy_offers.delete_key(key) {
            true
        } else {
            self.sell_offers.delete_key(key)
        }
    }
}
