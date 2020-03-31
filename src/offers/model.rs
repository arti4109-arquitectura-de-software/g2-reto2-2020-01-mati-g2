use crate::{
    bincode_des, bincode_ser, derive_key_of, derive_monotonic_key, derive_simple_struct,
    typed_tree::KeyOf,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Hash)]
pub struct OfferEventKey(pub [u8; 8]);
derive_monotonic_key!(OfferEventKey);

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum OfferEvent {
    Delete(OfferEventKey),
    Add(OfferValue),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum OfferEventKeyed {
    Delete(OfferEventKey, OfferEventKey),
    Add(Offer),
}

impl PartialEq for OfferEventKeyed {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl OfferEventKeyed {
    pub fn from_event(key: OfferEventKey, event: OfferEvent) -> Self {
        match event {
            OfferEvent::Add(value) => Self::Add(Offer { key, value }),
            OfferEvent::Delete(k) => Self::Delete(key, k),
        }
    }
    pub fn key(&self) -> &OfferEventKey {
        match self {
            OfferEventKeyed::Add(o) => &o.key,
            OfferEventKeyed::Delete(k, _) => k,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, )]
pub struct Offer {
    pub key: OfferEventKey,
    pub value: OfferValue,
}

impl PartialEq for Offer{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Offer {
    pub fn opposite_side(&self) -> Side {
        match self.value.side {
            Side::Sell => Side::Buy,
            Side::Buy => Side::Sell,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
