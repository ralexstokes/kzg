use num_bigint::BigUint;

pub const MODULUS_BIT_SIZE: usize = 255;

/// Return the order of the group(s) defined over elliptic curves in BLS12-381. The `r` in `Fr`.
pub fn curve_order() -> BigUint {
    BigUint::parse_bytes(
        b"52435875175126190479447740508185965837690552500527637822603658699938581184513",
        10,
    )
    .unwrap()
}
