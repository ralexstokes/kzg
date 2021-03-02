use crate::constants;
use num_bigint::BigUint;
use oblast;
use rand::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Setup {
    pub in_g1: Vec<oblast::P1>,
    pub in_g2: oblast::P2,
}

/// Generate a `Setup` with randomness supplied by the `rand` crate.
/// Ensures the secret is properly constructed.
pub fn generate_with_random_secret(degree: usize) -> Setup {
    let mut rng = thread_rng();

    let mut secret = [0u8; 32];
    rng.fill_bytes(&mut secret);

    let mut s = BigUint::from_bytes_be(&secret);

    let modulus = constants::get_modulus();
    while s >= modulus {
        rng.fill_bytes(&mut secret);
        s = BigUint::from_bytes_be(&secret);
    }

    generate(&secret, degree)
}

pub fn generate(secret: &[u8; 32], degree: usize) -> Setup {
    let modulus = constants::get_modulus();
    let s = BigUint::from_bytes_be(secret);

    assert!(s < modulus, "secret must be less than size of group r");

    let mut points_in_g1 = vec![];

    let g1 = oblast::P1::generator();
    for i in 0..=degree {
        let i_as_bigint = BigUint::from_slice(&[i as u32]);
        let s_i_as_bigint = s.modpow(&i_as_bigint, &modulus);

        let mut s_i_bytes = vec![0u8; 32];
        let raw_bytes = s_i_as_bigint.to_bytes_be();
        s_i_bytes[32 - raw_bytes.len()..].copy_from_slice(&raw_bytes);
        let s_i_scalar = oblast::Scalar::from_fr_bytes(&s_i_bytes);

        let result = s_i_scalar * g1;
        points_in_g1.push(result);
    }

    // NOTE: `secret` in Fr via prior `assert`.
    let scalar = oblast::Scalar::from_fr_bytes(secret);
    let result_in_g2 = scalar * oblast::P2::generator();

    Setup {
        in_g1: points_in_g1,
        in_g2: result_in_g2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let secret = [11u8; 32];
        let degree = 16;

        let setup = generate(&secret, degree);
        let second_setup = generate(&secret, degree);
        // NOTE: had an earlier bug w/ non-deterministic setups...
        assert_eq!(setup, second_setup);
        assert_eq!(setup.in_g1.len(), degree + 1);
    }
}
