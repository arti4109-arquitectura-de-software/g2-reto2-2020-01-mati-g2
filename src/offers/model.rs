use crate::{
    bincode_des, bincode_ser, derive_key_of, derive_monotonic_key, derive_simple_struct,
    typed_tree::KeyOf,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Hash)]
pub struct OfferEventKey([u8; 8]);
derive_monotonic_key!(OfferEventKey);

#[derive(Deserialize)]
pub enum OfferEventRequest {
    Delete(u64),
    Add(OfferValue),
}

impl From<OfferEventRequest> for OfferEvent {
    fn from(o: OfferEventRequest) -> Self {
        match o {
            OfferEventRequest::Add(v) => OfferEvent::Add(v),
            OfferEventRequest::Delete(v) => OfferEvent::Delete(OfferEventKey(v.to_be_bytes())),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum OfferEvent {
    Delete(OfferEventKey),
    Add(OfferValue),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum OfferEventKeyed {
    Delete(OfferEventKey, OfferEventKey),
    Add(OfferEventKey, OfferValue),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OfferValue {
    pub security: Security,
    pub side: Side,
    pub amount: u64,
    pub price: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Security {
    BTC,
    USD,
    COP,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Sell,
    Buy,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::Sell => Side::Buy,
            Side::Buy => Side::Sell,
        }
    }
}

derive_key_of!(OfferEventKey, OfferEvent, "OfferEvent", 2);
