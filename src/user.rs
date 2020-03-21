use crate::{bincode_des, bincode_ser, derive_key_of, typed_tree::KeyOf};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct UserKey<'a>(pub &'a str);

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct User {
    pub id: String,
    pub password: String,
}

impl<'a> std::convert::AsRef<[u8]> for UserKey<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl User {
    pub fn key<'a>(&'a self) -> UserKey<'a> {
        UserKey(self.id.as_str())
    }
}
derive_key_of!(UserKey<'_>, User, "User", 0);

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct BlackListedKey<'a>(pub &'a str);

impl<'a> std::convert::AsRef<[u8]> for BlackListedKey<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct BlackListed {
    pub exp: u64,
}

derive_key_of!(BlackListedKey<'_>, BlackListed, "BlackListed", 1);
