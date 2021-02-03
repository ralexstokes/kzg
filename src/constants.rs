use num_bigint::BigUint;

pub const MODULUS_BIT_SIZE: usize = 255;

pub fn get_modulus() -> BigUint {
    BigUint::parse_bytes(
        b"52435875175126190479447740508185965837690552500527637822603658699938581184513",
        10,
    )
    .unwrap()
}
