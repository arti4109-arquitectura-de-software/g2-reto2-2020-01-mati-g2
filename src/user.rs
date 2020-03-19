use crate::{bincode_des, bincode_ser, derive_key_of, derive_simple_struct, typed_tree::KeyOf};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct UserKey(pub String);

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct User {
    pub id: String,
    pub password: String,
}
derive_key_of!(UserKey, User, "User", 0);

impl std::convert::AsRef<[u8]> for UserKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl User {
    pub fn key(&self) -> UserKey {
        UserKey(self.id.clone())
    }
}

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
