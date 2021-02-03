use crate::constants;
use blst;
use num_bigint::BigUint;

pub struct Setup {
    pub in_g1: Vec<blst::blst_p1>,
    pub in_g2: Vec<blst::blst_p2>,
}

pub fn generate(secret: &[u8; 32], degree: usize) -> Setup {
    let modulus = constants::get_modulus();
    let s = BigUint::from_bytes_be(secret);
    let mut points_in_g1 = vec![];
    let mut points_in_g2 = vec![];

    unsafe {
        let g1 = blst::blst_p1_generator();
        let g2 = blst::blst_p2_generator();

        for i in 0..=degree {
            let i_as_bigint = BigUint::from_slice(&[i as u32]);
            let s_i_as_bigint = s.modpow(&i_as_bigint, &modulus);

            let mut scalar = blst::blst_scalar::default();
            blst::blst_scalar_from_bendian(&mut scalar, s_i_as_bigint.to_bytes_be().as_ptr());

            let mut result = blst::blst_p1::default();
            blst::blst_p1_mult(&mut result, g1, &scalar, constants::MODULUS_BIT_SIZE);
            points_in_g1.push(result);

            let mut result = blst::blst_p2::default();
            blst::blst_p2_mult(&mut result, g2, &scalar, constants::MODULUS_BIT_SIZE);
            points_in_g2.push(result);
        }
    }

    Setup {
        in_g1: points_in_g1,
        in_g2: points_in_g2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup() {
        let secret = [0u8; 32];
        let degree = 16;

        let setup = generate(&secret, degree);
        assert_eq!(setup.in_g1.len(), (degree + 1));
    }
}
