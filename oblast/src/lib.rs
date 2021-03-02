//! High-level wrapper for BLS12-381 arithmetic using `blst`.

mod constants;
#[cfg(test)]
mod tests;

use blst::{blst_fp12, blst_fr, blst_scalar};
use paste::paste;

/// Field sub-group element.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Fr {
    element: blst_fr,
}

impl Fr {
    pub fn from_u64(value: u64) -> Self {
        let mut point = Self::default();
        let input = vec![value, 0, 0, 0];
        unsafe {
            blst::blst_fr_from_uint64(&mut point.element, input.as_ptr());
        }
        point
    }

    pub fn from_raw(element: blst_fr) -> Self {
        Self { element }
    }

    pub fn as_u64(&self) -> u64 {
        let mut buffer = [0u64; 4];
        unsafe {
            blst::blst_uint64_from_fr(buffer.as_mut_ptr(), &self.element);
        }
        buffer[0]
    }
}

impl std::ops::Add for Fr {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut sum = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_add(&mut sum, &self.element, &other.element);
        }
        Self { element: sum }
    }
}

impl std::ops::AddAssign for Fr {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl std::ops::Neg for Fr {
    type Output = Self;

    fn neg(self) -> Self {
        let mut result = blst::blst_fr::default();
        unsafe {
            // NOTE: boolean is conditional operation, always set `true` here.
            blst::blst_fr_cneg(&mut result, &self.element, true);
        }
        Self { element: result }
    }
}

impl std::ops::Sub for Fr {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + -other
    }
}

impl std::ops::Mul for Fr {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut result = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_mul(&mut result, &self.element, &other.element);
        }
        Self { element: result }
    }
}

impl std::ops::MulAssign for Fr {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other
    }
}

impl std::ops::Div for Fr {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        let mut result = blst::blst_fr::default();
        let mut other_inverse = blst::blst_fr::default();

        unsafe {
            blst::blst_fr_eucl_inverse(&mut other_inverse, &other.element);
            blst::blst_fr_mul(&mut result, &self.element, &other_inverse);
        }
        Self { element: result }
    }
}

/// Element of the degree-12 field extension.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Fp12 {
    element: blst_fp12,
}

impl std::ops::Mul for Fp12 {
    type Output = Fp12;

    fn mul(mut self, rhs: Fp12) -> Fp12 {
        unsafe {
            blst::blst_fp12_mul(&mut self.element, &self.element, &rhs.element);
        }
        self
    }
}

impl Fp12 {
    pub fn from_raw(element: blst_fp12) -> Self {
        Self { element }
    }

    pub fn final_exp(mut self) -> Self {
        unsafe {
            blst::blst_final_exp(&mut self.element, &self.element);
        }
        self
    }

    pub fn is_one(&self) -> bool {
        unsafe { blst::blst_fp12_is_one(&self.element) }
    }
}

/// Scalar for multiplying curve points.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Scalar {
    value: blst_scalar,
}

impl Scalar {
    /// Construct a `blst_scalar` instance of a `Fr` value from bytes in big-endian order.
    /// Panics if the value is not in `Fr`.
    pub fn from_fr_bytes(value: &[u8]) -> Self {
        dbg!(value.len());
        assert!(value.len() == 32);
        let mut scalar = blst::blst_scalar::default();
        unsafe {
            blst::blst_scalar_from_bendian(&mut scalar, value.as_ptr());
            assert!(blst::blst_scalar_fr_check(&scalar));
        }
        Self { value: scalar }
    }
}

impl From<Fr> for Scalar {
    fn from(x: Fr) -> Self {
        let mut scalar = Self::default();
        unsafe {
            blst::blst_scalar_from_fr(&mut scalar.value, &x.element);
        }
        scalar
    }
}

macro_rules! define_curve_struct {
    ($struct_name:ident, $blst_name:ident, $group_name:ident, $compressed_bytes:expr) => {
        paste! {
            #[doc = "Point on the curve sub-group " $group_name "."]
            #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
            pub struct $struct_name {
                point: blst::[<blst_ $blst_name>],
            }
        }

        paste! {
            #[doc = "Affine encoding of a point on " $group_name "."]
            #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
            pub struct [<$struct_name Affine>] {
                point: blst::[<blst_ $blst_name _affine>],
            }

            impl From<&$struct_name> for [<$struct_name Affine>] {
                fn from(point: &$struct_name) -> Self {
                    let mut affine = Self::default();
                    unsafe {
                        blst::[<blst_ $blst_name _to_affine>](&mut affine.point, &point.point);
                    }
                    affine
                }
            }
        }

        impl $struct_name {
            /// Return the distinguished generator point.
            pub fn generator() -> Self {
                unsafe {
                    Self {
                        point: *paste! { blst::[<blst_ $blst_name _generator>]() },
                    }
                }
            }

            pub fn from_raw(point: paste! { blst::[<blst_ $blst_name>] }) -> Self {
                Self { point }
            }

            pub fn compress(&self) -> Vec<u8> {
                let mut compressed_point = vec![0; $compressed_bytes];
                let compress = paste! { blst::[<blst_ $blst_name _compress>] };
                unsafe {
                    compress(compressed_point.as_mut_ptr(), &self.point);
                }
                compressed_point
            }
        }

        impl std::fmt::Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "0x{}", hex::encode(self.compress()))
            }
        }

        impl $struct_name {
            /// Negate this point in-place.
            pub fn negate(&mut self) {
                paste! {
                    unsafe {
                        blst::[<blst_ $blst_name _cneg>](&mut self.point, true);
                    }
                }
            }
        }

        /// Unary negation.
        impl std::ops::Neg for $struct_name {
            type Output = Self;

            fn neg(mut self) -> Self {
                self.negate();
                self
            }
        }

        /// Point addition.
        impl std::ops::Add for $struct_name {
            type Output = Self;

            fn add(mut self, rhs: $struct_name) -> $struct_name {
                let add = paste! { blst::[<blst_ $blst_name _add>] };
                unsafe {
                    add(&mut self.point, &self.point, &rhs.point);
                }
                self
            }
        }

        /// Scalar multiplication.
        impl std::ops::Mul<$struct_name> for Scalar {
            type Output = $struct_name;

            fn mul(self, mut rhs: $struct_name) -> Self::Output {
                let mult = paste! { blst::[<blst_ $blst_name _mult>] };
                unsafe {
                    mult(
                        &mut rhs.point,
                        &rhs.point,
                        &self.value,
                        constants::MODULUS_BIT_SIZE,
                    );
                }
                rhs
            }
        }

        /// Scalar multiplication for [<$struct_name>].
        impl std::ops::Mul<$struct_name> for Fr {
            type Output = $struct_name;

            fn mul(self, rhs: $struct_name) -> Self::Output {
                Scalar::from(self) * rhs
            }
        }
    };
}

define_curve_struct!(P1, p1, G1, 48);
define_curve_struct!(P2, p2, G2, 96);

/// Check that `e(x1, x2) = e(y1, y2)`.
pub fn verify_pairings(mut x1: P1, x2: P2, y1: P1, y2: P2) -> bool {
    // Negate one of the inputs to avoid an exponentiation.
    x1.negate();

    let x1_affine = P1Affine::from(&x1);
    let x2_affine = P2Affine::from(&x2);
    let y1_affine = P1Affine::from(&y1);
    let y2_affine = P2Affine::from(&y2);

    let mut lhs = Fp12::default();
    let mut rhs = Fp12::default();

    unsafe {
        blst::blst_miller_loop(&mut lhs.element, &x2_affine.point, &x1_affine.point);
        blst::blst_miller_loop(&mut rhs.element, &y2_affine.point, &y1_affine.point);
    }
    (lhs * rhs).final_exp().is_one()
}
