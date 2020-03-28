use crate::{derive_offer_ord, engine::offer_ord::OfferOrdSigned};
use crate::{
    engine::{EngineDataStruct, MatchResult},
    offers::{Offer, OfferEventKey, Side},
};
use keyed_priority_queue::KeyedPriorityQueue;

#[derive(Eq, PartialOrd, Clone, Debug)]
pub struct EngineOfferKBH {
    price: Option<i64>,
    key: [u8; 8],
    amount: u64,
}
derive_offer_ord!(OfferOrdSigned, EngineOfferKBH, cmp_max);

pub type KeyedBinaryHeapEngine = KeyedPriorityQueue<OfferEventKey, EngineOfferKBH>;

impl EngineDataStruct for KeyedBinaryHeapEngine {
    fn delete_key(&mut self, key: &OfferEventKey) -> bool {
        match self.remove(key) {
            Some(_) => true,
            None => false,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }

    fn match_offer(
        &mut self,
        matches: &mut Vec<Offer>,
        offer: Offer,
        other: &mut Self,
    ) -> MatchResult {
        let mut excedent = offer.value.amount;
        let opposite_side = offer.opposite_side();

        if let Some(price) = offer.value.price {
            let price = match offer.value.side {
                Side::Buy => price as i64,
                Side::Sell => -(price as i64),
            };

            while let Some((_, o)) = self.peek() {
                if let Some(p) = o.price {
                    if  p > price {
                        break;
                    }
                }
                let (k, mut o) = self.pop().unwrap();
                if o.amount > excedent {
                    let new_offer = o.into_offer(opposite_side, offer.value.security);
                    o.amount -= excedent;
                    self.push(k, o);

                    return MatchResult::Partial {
                        offer: new_offer,
                        to_substract: excedent,
                    };
                }

                matches.push(o.into_offer(opposite_side, offer.value.security));
                if o.amount == excedent {
                    return MatchResult::Complete;
                } else {
                    excedent -= o.amount;
                }
            }
        } else {
            while let Some((k, mut o)) = self.pop() {
                if o.amount > excedent {
                    let new_offer = o.into_offer(opposite_side, offer.value.security);
                    o.amount -= excedent;
                    self.push(k, o);

                    return MatchResult::Partial {
                        offer: new_offer,
                        to_substract: excedent,
                    };
                }

                matches.push(o.into_offer(opposite_side, offer.value.security));
                if o.amount == excedent {
                    return MatchResult::Complete;
                } else {
                    excedent -= o.amount;
                }
            }
        }

        let new_offer = EngineOfferKBH {
            price: EngineOfferKBH::price_from_offer(&offer),
            amount: excedent,
            key: *offer.key.as_ref(),
        };
        other.push(offer.key.clone(), new_offer);

        if offer.value.amount == excedent {
            MatchResult::None
        } else {
            let to_substract = offer.value.amount - excedent;
            MatchResult::Partial {
                offer,
                to_substract,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::{Engine, Matches},
        offers::{Offer, OfferEventKeyed, OfferValue, Security, Side},
    };

    #[test]
    fn new_keyed_priority_queue() {}

    #[test]
    fn engine_test() {
        let (_sender_offer, receiver_offer) = crossbeam_channel::unbounded::<OfferEventKeyed>();
        let (sender_matches, _receiver_matches) = crossbeam_channel::unbounded::<Matches>();
        let mut engine = Engine::<KeyedBinaryHeapEngine>::new(receiver_offer, sender_matches);
        let offer = Offer {
            key: u64::to_be_bytes(0).into(),
            value: OfferValue {
                side: Side::Buy,
                security: Security::BTC,
                amount: 10,
                price: None,
            },
        };
        engine.process_offer(offer);
        let offer = Offer {
            key: u64::to_be_bytes(1).into(),
            value: OfferValue {
                side: Side::Buy,
                security: Security::BTC,
                amount: 5,
                price: Some(32),
            },
        };
        let matches = engine.process_offer(offer);
        println!("{:?}", matches);

        let offer = Offer {
            key: u64::to_be_bytes(2).into(),
            value: OfferValue {
                side: Side::Sell,
                security: Security::BTC,
                amount: 8,
                price: None,
            },
        };
        let matches = engine.process_offer(offer);
        println!("{:?}", matches);

        let offer = Offer {
            key: u64::to_be_bytes(3).into(),
            value: OfferValue {
                side: Side::Sell,
                security: Security::BTC,
                amount: 6,
                price: Some(33),
            },
        };
        let matches = engine.process_offer(offer);
        println!("{:?}", matches);
    }
}
