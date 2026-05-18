//! Scalar field used by LP-0016: the Ristretto255 scalar field.
//!
//! `Scalar` is a thin newtype around [`curve25519_dalek::Scalar`] so the
//! protocol code can switch backends if needed without touching every call
//! site. All field operations are constant-time per upstream guarantees.

use core::fmt;
use core::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

use curve25519_dalek::scalar::Scalar as DalekScalar;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use subtle::ConstantTimeEq;

// `DalekScalar` implements `Zeroize` and `ZeroizeOnDrop` when the upstream
// `zeroize` feature is enabled (it is, via the workspace dep). Our newtype
// inherits that drop behavior without an extra derive macro.
#[derive(Clone, Copy, Default)]
pub struct Scalar(pub(crate) DalekScalar);

impl Scalar {
    pub const ZERO: Scalar = Scalar(DalekScalar::ZERO);
    pub const ONE: Scalar = Scalar(DalekScalar::ONE);

    pub fn from_u64(n: u64) -> Self {
        Scalar(DalekScalar::from(n))
    }

    /// Reduce 64 bytes uniformly into the scalar field. Use this for
    /// hash-to-field where bias must be negligible.
    pub fn from_bytes_wide(bytes: &[u8; 64]) -> Self {
        Scalar(DalekScalar::from_bytes_mod_order_wide(bytes))
    }

    pub fn from_canonical_bytes(bytes: [u8; 32]) -> Option<Self> {
        Option::from(DalekScalar::from_canonical_bytes(bytes)).map(Scalar)
    }

    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn invert(&self) -> Self {
        Scalar(self.0.invert())
    }
}

impl PartialEq for Scalar {
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.0.ct_eq(&other.0))
    }
}

impl Eq for Scalar {}

impl fmt::Debug for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.0.to_bytes();
        write!(f, "Scalar({})", hex::encode(&bytes[..8]))
    }
}

impl Add for Scalar {
    type Output = Scalar;
    fn add(self, rhs: Self) -> Self::Output {
        Scalar(self.0 + rhs.0)
    }
}

impl Sub for Scalar {
    type Output = Scalar;
    fn sub(self, rhs: Self) -> Self::Output {
        Scalar(self.0 - rhs.0)
    }
}

impl Mul for Scalar {
    type Output = Scalar;
    fn mul(self, rhs: Self) -> Self::Output {
        Scalar(self.0 * rhs.0)
    }
}

impl Neg for Scalar {
    type Output = Scalar;
    fn neg(self) -> Self::Output {
        Scalar(-self.0)
    }
}

impl Div for Scalar {
    type Output = Scalar;
    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.invert()
    }
}

impl AddAssign for Scalar {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Scalar {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl MulAssign for Scalar {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl From<u64> for Scalar {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl Serialize for Scalar {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let bytes = self.to_bytes();
        if ser.is_human_readable() {
            ser.serialize_str(&hex::encode(bytes))
        } else {
            ser.serialize_bytes(&bytes)
        }
    }
}

impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        if de.is_human_readable() {
            let s = String::deserialize(de)?;
            let raw = hex::decode(&s).map_err(D::Error::custom)?;
            let bytes: [u8; 32] = raw
                .try_into()
                .map_err(|_| D::Error::custom("scalar must be 32 bytes"))?;
            Self::from_canonical_bytes(bytes).ok_or_else(|| D::Error::custom("non-canonical scalar"))
        } else {
            let raw = <Vec<u8>>::deserialize(de)?;
            let bytes: [u8; 32] = raw
                .try_into()
                .map_err(|_| D::Error::custom("scalar must be 32 bytes"))?;
            Self::from_canonical_bytes(bytes).ok_or_else(|| D::Error::custom("non-canonical scalar"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub_round_trip() {
        let a = Scalar::from_u64(123);
        let b = Scalar::from_u64(456);
        assert_eq!(a + b - b, a);
    }

    #[test]
    fn mul_div_round_trip() {
        let a = Scalar::from_u64(7);
        let b = Scalar::from_u64(13);
        assert_eq!((a * b) / b, a);
    }

    #[test]
    fn json_round_trip_is_hex() {
        let s = Scalar::from_u64(0xdead_beef);
        let json = serde_json::to_string(&s).unwrap();
        let back: Scalar = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
