use core::fmt;
use serde::{Deserialize, Serialize};

/// Development field for protocol simulation: p = 2^61 - 1.
///
/// Production should switch to `curve25519_dalek::Scalar` or another field
/// agreed in `SPEC.md`. This type exists so the state machine is executable
/// without crypto dependencies.
pub const FIELD_PRIME: u64 = (1u64 << 61) - 1;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct F(pub u64);

impl F {
    pub const ZERO: F = F(0);
    pub const ONE: F = F(1);

    pub fn new(n: u64) -> Self {
        F(n % FIELD_PRIME)
    }

    pub fn from_u64(n: u64) -> Self {
        Self::new(n)
    }

    pub fn to_be_bytes(self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    pub fn inv(self) -> Option<Self> {
        if self.0 == 0 {
            return None;
        }
        Some(self.pow(FIELD_PRIME - 2))
    }

    pub fn pow(self, mut e: u64) -> Self {
        let mut base = self;
        let mut acc = F::ONE;
        while e > 0 {
            if e & 1 == 1 {
                acc *= base;
            }
            base *= base;
            e >>= 1;
        }
        acc
    }
}

impl fmt::Debug for F {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "F({})", self.0)
    }
}

impl core::ops::Add for F {
    type Output = F;
    fn add(self, rhs: Self) -> Self::Output {
        F::new(((self.0 as u128 + rhs.0 as u128) % FIELD_PRIME as u128) as u64)
    }
}

impl core::ops::Sub for F {
    type Output = F;
    fn sub(self, rhs: Self) -> Self::Output {
        F::new(((FIELD_PRIME as u128 + self.0 as u128 - rhs.0 as u128) % FIELD_PRIME as u128) as u64)
    }
}

impl core::ops::Mul for F {
    type Output = F;
    fn mul(self, rhs: Self) -> Self::Output {
        F::new(((self.0 as u128 * rhs.0 as u128) % FIELD_PRIME as u128) as u64)
    }
}

impl core::ops::Div for F {
    type Output = F;
    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.inv().expect("division by zero in F")
    }
}

impl core::ops::AddAssign for F {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl core::ops::SubAssign for F {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl core::ops::MulAssign for F {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl From<u64> for F {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}
