use serde::{Deserialize, Serialize};
use sled::IVec;
use std::convert::TryFrom;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Deserialize, Serialize)]
pub struct KeyVal<K>
where
    K: AsRef<[u8]> + KeyOf,
    <ValOf<K> as TryFrom<IVec>>::Error: Into<sled::Error>,
{
    pub key: K,
    pub val: ValOf<K>,
}

pub trait KeyOf: AsRef<[u8]>
where
    <Self::T as TryFrom<IVec>>::Error: Into<sled::Error>,
{
    const NAME: &'static str;
    const PREFIX: [u8; 1];
    type T: ?Sized + TryFrom<IVec> + Into<IVec> + for<'a> Deserialize<'a> + Serialize;
}

type ValOf<K> = <K as KeyOf>::T;

pub trait TypedTree<K>
where
    K: AsRef<[u8]> + KeyOf,
    <ValOf<K> as TryFrom<IVec>>::Error: Into<sled::Error>,
    ValOf<K>: TryFrom<IVec>,
    IVec: From<ValOf<K>>,
{
    fn get_typed(&self, key: &K) -> sled::Result<Option<ValOf<K>>>;
    fn insert_typed(&self, key: &K, value: ValOf<K>) -> sled::Result<Option<IVec>>;
}

pub trait MonotonicTypedTree<K>: TypedTree<K>
where
    K: AsRef<[u8]> + KeyOf + From<u64>,
    <ValOf<K> as TryFrom<IVec>>::Error: Into<sled::Error>,
    ValOf<K>: TryFrom<IVec>,
    IVec: From<ValOf<K>>,
{
    // fn insert_monotonic(&self, value: ValOf<K>) -> sled::Result<(K, Option<IVec>)>;
    fn get_max_key(&mut self) -> sled::Result<u64>;
    fn insert_monotonic_atomic(
        &self,
        atomic: &AtomicU64,
        value: ValOf<K>,
    ) -> sled::Result<(K, Option<IVec>)> {
        let key = K::from(atomic.fetch_add(1, Ordering::SeqCst));
        self.insert_typed(&key, value).and_then(|v| Ok((key, v)))
    }
}

use std::convert::TryInto;

fn read_be_u64(input: &mut &[u8]) -> u64 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u64>());
    *input = rest;
    u64::from_be_bytes(int_bytes.try_into().unwrap())
}

impl<K> MonotonicTypedTree<K> for sled::Tree
where
    K: AsRef<[u8]> + KeyOf + From<u64>,
    <ValOf<K> as TryFrom<IVec>>::Error: Into<sled::Error>,
    ValOf<K>: TryFrom<IVec>,
    IVec: From<ValOf<K>>,
{
    // fn insert_monotonic(&self, value: ValOf<K>) -> sled::Result<(K, Option<IVec>)> {
    //     let key = K::from(self.generate_id()?);
    //     self.insert(&key, value).and_then(|v| Ok((key, v)))
    // }
    fn get_max_key(&mut self) -> sled::Result<u64> {
        if let Some((k, v)) = self.pop_max()? {
            let count = {
                let mut b = k.as_ref();
                if b.len() == 8 {
                    read_be_u64(&mut b)
                } else {
                    panic!()
                }
            };
            self.insert(k.clone(), <ValOf<K>>::try_from(v).map_err(|e| e.into())?)?;

            Ok(count + 1)
        } else {
            Ok(1)
        }
    }
}

impl<K> TypedTree<K> for sled::Tree
where
    <ValOf<K> as TryFrom<IVec>>::Error: Into<sled::Error>,
    ValOf<K>: TryFrom<IVec>,
    K: AsRef<[u8]> + KeyOf,
    IVec: From<ValOf<K>>,
{
    fn get_typed(&self, key: &K) -> sled::Result<Option<ValOf<K>>>
    where
        K: AsRef<[u8]>,
    {
        self.get(key).and_then(|v| match v {
            Some(v) => sled::Result::Ok(Some(<ValOf<K>>::try_from(v).map_err(|e| e.into())?)),
            None => sled::Result::Ok(None),
        })
    }

    fn insert_typed(&self, key: &K, value: ValOf<K>) -> sled::Result<Option<IVec>> {
        self.insert(key, value)
    }
}

#[macro_export]
macro_rules! derive_key_of {
    ($key: ty, $value: ty, $NAME: literal, $PREFIX: literal) => {
        impl KeyOf for $key {
            const NAME: &'static str = $NAME;
            const PREFIX: [u8; 1] = [$PREFIX];
            type T = $value;
        }

        impl From<$value> for sled::IVec {
            fn from(data: $value) -> Self {
                sled::IVec::from(bincode_ser!(&data).unwrap())
            }
        }

        impl<'a> std::convert::TryFrom<sled::IVec> for $value {
            type Error = sled::Error;

            fn try_from(data: sled::IVec) -> Result<$value, sled::Error> {
                bincode_des!(data.as_ref())
                    .map_err(|_| sled::Error::Unsupported("Error Deserializing".into()))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::User;

    #[test]
    fn typed() {
        let t: sled::Db = sled::Config::default().temporary(true).open().unwrap();

        let val = User {
            id: "dw".to_string(),
            password: "pass".to_string(),
        };
        let key = val.key();
        t.insert_typed(&key, val.clone()).unwrap();
        let v = t.get_typed(&key).unwrap();
        assert_eq!(v, Some(val));
    }

    // #[test]
    // fn atomics() {
    //     let t: sled::Db = sled::Config::default().temporary(true).open().unwrap();
    //     let atomic = Arc::new(AtomicU64::new(0));

    //     let val = User {
    //         id: "dw".into(),
    //         password: "pass".into(),
    //     };
    //     let (k, _val_bytes): (UserKey, Option<_>) =
    //         t.insert_monotonic_atomic(&atomic, val).unwrap();
    //     assert_eq!(u64::from(k), 0);
    //     assert_eq!(atomic.load(Ordering::SeqCst), 1);
    // }
}

// pub trait ValOf: TryFrom<IVec> + Into<IVec> + for<'a> Deserialize<'a> + Serialize
// where
//     <Self as TryFrom<IVec>>::Error: Into<sled::Error>,
// {
//     type K:  AsRef<[u8]> + KeyOf;
// }

// pub trait KeyValT: TryFrom<IVec> + Into<IVec> + for<'a> Deserialize<'a> + Serialize
// where
//     <Self as TryFrom<IVec>>::Error: Into<sled::Error>,
// {
//     const NAME: &'static str;
//     const PREFIX: u8;
//     type V: ?Sized + TryFrom<IVec> + Into<IVec> + for<'a> Deserialize<'a> + Serialize;
//     type K: AsRef<[u8]>;
// }
