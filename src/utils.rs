const F64_TO_U64_FACTOR: f64 = 10_000.0;
#[allow(dead_code)]
pub fn f64_to_u64(price: f64) -> u64 {
    (price * F64_TO_U64_FACTOR) as u64
}
#[allow(dead_code)]
pub fn f64_to_i64(price: f64) -> i64 {
    (price * F64_TO_U64_FACTOR) as i64
}
#[allow(dead_code)]
pub fn u64_to_f64(price: u64) -> f64 {
    price as f64 / F64_TO_U64_FACTOR
}
#[allow(dead_code)]
pub fn i64_to_f64(price: i64) -> f64 {
    price as f64 / F64_TO_U64_FACTOR
}

/// Returns the mantissa, exponent and sign as integers.
#[allow(dead_code)]
pub fn integer_decode(float: f64) -> (u64, i16, i8) {
    let bits: u64 = unsafe { ::std::mem::transmute(float) };
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };
    // Exponent bias + mantissa shift
    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

#[macro_export]
macro_rules! bincode_ser {
    ($val: expr ) => {
        bincode::config().big_endian().serialize($val)
    };
}
#[macro_export]
macro_rules! bincode_des {
    ($val: expr) => {
        bincode::config().big_endian().deserialize($val)
    };
}

#[macro_export]
macro_rules! derive_simple_struct {
    ($name: ty, $type: ty) => {
        impl std::convert::AsRef<$type> for $name {
            fn as_ref(&self) -> &$type {
                &self.0
            }
        }
        impl From<$type> for $name {
            fn from(v: $type) -> Self {
                $name(v)
            }
        }
        impl From<$name> for $type {
            fn from(v: $name) -> Self {
                v.0
            }
        }
    };
}

#[macro_export]
macro_rules! derive_monotonic_key {
    ($name: ident) => {
        derive_simple_struct!($name, [u8; 8]);

        impl std::convert::AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }
        impl From<u64> for $name {
            fn from(v: u64) -> Self {
                $name(u64::to_be_bytes(v))
            }
        }
        impl From<$name> for u64 {
            fn from(v: $name) -> Self {
                u64::from_be_bytes(v.0)
            }
        }
    };
}
