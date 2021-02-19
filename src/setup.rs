use crate::constants;
use blst;
use num_bigint::BigUint;

#[derive(Debug, PartialEq, Eq)]
pub struct Setup {
    pub in_g1: Vec<blst::blst_p1>,
    pub in_g2: blst::blst_p2,
}

pub fn generate(secret: &[u8; 32], degree: usize) -> Setup {
    let modulus = constants::get_modulus();
    let s = BigUint::from_bytes_be(secret);
    let mut points_in_g1 = vec![];

    unsafe {
        let g1 = blst::blst_p1_generator();

        for i in 0..=degree {
            let i_as_bigint = BigUint::from_slice(&[i as u32]);
            let s_i_as_bigint = s.modpow(&i_as_bigint, &modulus);

            let mut scalar = blst::blst_scalar::default();
            let mut s_i_bytes = vec![0u8; 32];
            let raw_bytes = s_i_as_bigint.to_bytes_be();
            s_i_bytes[32 - raw_bytes.len()..].copy_from_slice(&raw_bytes);
            blst::blst_scalar_from_bendian(&mut scalar, s_i_bytes.as_ptr());

            let mut result = blst::blst_p1::default();
            blst::blst_p1_mult(&mut result, g1, &scalar, constants::MODULUS_BIT_SIZE);
            points_in_g1.push(result);
        }

        let g2 = blst::blst_p2_generator();
        let mut result_in_g2 = blst::blst_p2::default();
        let mut scalar = blst::blst_scalar::default();
        blst::blst_scalar_from_bendian(&mut scalar, secret.as_ptr());
        blst::blst_p2_mult(&mut result_in_g2, g2, &scalar, constants::MODULUS_BIT_SIZE);

        Setup {
            in_g1: points_in_g1,
            in_g2: result_in_g2,
        }
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
