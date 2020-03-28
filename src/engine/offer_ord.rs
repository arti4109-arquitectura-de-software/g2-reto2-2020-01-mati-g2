
use crate::offers::{OfferValue, Offer, Security, Side};
use std::cmp::Ordering::{self, Greater, Less}; 

pub trait OfferOrd {
    fn key(&self) -> u64;
    fn price(&self) -> Option<u64>;

    fn cmp_min_buy(&self, other: &Self) -> Ordering {
        match self.price() {
            Some(price) => match other.price() {
                Some(price_other) => price_other
                    .cmp(&price)
                    .then_with(|| self.key().cmp(&other.key())),
                None => Greater,
            },
            None => match other.price() {
                Some(_price_other) => Less,
                None => self.key().cmp(&other.key()),
            },
        }
    }
    fn cmp_min_sell(&self, other: &Self) -> Ordering {
        match self.price() {
            Some(price) => match other.price() {
                Some(price_other) => price
                    .cmp(&price_other)
                    .then_with(|| self.key().cmp(&other.key())),
                None => Greater,
            },
            None => match other.price() {
                Some(_price_other) => Less,
                None => self.key().cmp(&other.key()),
            },
        }
    }
    fn cmp_max_buy(&self, other: &Self) -> Ordering {
        match self.price() {
            Some(price) => match other.price() {
                Some(price_other) => price.cmp(&price_other).then(other.key().cmp(&self.key())),
                None => Less,
            },
            None => match other.price() {
                Some(_price_other) => Greater,
                None => other.key().cmp(&self.key()),
            },
        }
    }
    fn cmp_max_sell(&self, other: &Self) -> Ordering {
        match self.price() {
            Some(price) => match other.price() {
                Some(price_other) => price_other.cmp(&price).then(other.key().cmp(&self.key())),
                None => Less,
            },
            None => match other.price() {
                Some(_price_other) => Greater,
                None => other.key().cmp(&self.key()),
            },
        }
    }
}

pub trait OfferOrdSigned: std::marker::Sized {
    fn key(&self) -> [u8; 8];
    fn amount(&self) -> u64;
    fn price(&self) -> Option<i64>;

    fn cmp_min(&self, other: &Self) -> Ordering {
        match self.price() {
            Some(price) => match other.price() {
                Some(price_other) => price
                    .cmp(&price_other)
                    .then_with(|| self.key().cmp(&other.key())),
                None => Greater,
            },
            None => match other.price() {
                Some(_price_other) => Less,
                None => self.key().cmp(&other.key()),
            },
        }
    }

    fn cmp_max(&self, other: &Self) -> Ordering {
        match self.price() {
            Some(price) => match other.price() {
                Some(price_other) => price_other.cmp(&price).then(other.key().cmp(&self.key())),
                None => Less,
            },
            None => match other.price() {
                Some(_price_other) => Greater,
                None => other.key().cmp(&self.key()),
            },
        }
    }

    fn into_offer(&self, side: Side, security: Security) -> Offer {
        Offer {
            key: self.key().into(),
            value: OfferValue {
                amount: self.amount(),
                price: self.price().and_then(|v| Some(v.abs() as u64)),
                side,
                security,
            },
        }
    }

    fn price_from_offer(offer: &Offer) -> Option<i64> {
        match offer.value.price {
            Some(price) => Some(match offer.value.side {
                Side::Buy => -(price as i64),
                Side::Sell => price as i64,
            }),
            None => None,
        }
    }
}

#[macro_export]
macro_rules! derive_offer_ord {
    ($traitr: ty, $name: ty, $f: ident) => {
        impl $traitr for $name {
            fn key(&self) -> [u8; 8] {
                self.key
            }
            fn price(&self) -> Option<i64> {
                self.price
            }
            fn amount(&self) -> u64 {
                self.amount
            }
        }
        impl std::cmp::PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.key == other.key
            }
        }
        impl std::cmp::Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.$f(other)
            }
        }
    };
    ($traitr: ty, $name: ty, $f: ident, $key: tt, $value:tt) => {
      impl $traitr for $name { 
          fn key(&self) -> [u8; 8] {
              self.$key
          }
          fn price(&self) -> Option<i64> {
              self.$value.price
          }
          fn amount(&self) -> u64 {
              self.$value.amount
          }
      }
      impl std::cmp::PartialEq for $name {
          fn eq(&self, other: &Self) -> bool {
              self.$key == other.$key
          }
      }
      impl std::cmp::Ord for $name {
          fn cmp(&self, other: &Self) -> std::cmp::Ordering {
              self.$f(other)
          }
      }
  };
}

pub trait OfferOrdSome {
    fn key(&self) -> u64;
    fn price(&self) -> u64;

    fn cmp_min_buy(&self, other: &Self) -> Ordering {
        other
            .price()
            .cmp(&self.price())
            .then_with(|| self.key().cmp(&other.key()))
    }
}
