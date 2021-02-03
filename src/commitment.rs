use crate::constants;
use crate::setup;
use blst;

#[derive(Debug)]
pub struct Commitment {
    element: blst::blst_p1,
}

pub struct Polynomial {
    // NOTE: coefficients are elements in Fr
    coefficients: Vec<blst::blst_scalar>,
}

impl Polynomial {
    fn from_u64(coefficients: impl Iterator<Item = u64>) -> Self {
        let scalars = coefficients.into_iter().map(|coeff| unsafe {
            let mut scalar = blst::blst_scalar::default();

            blst::blst_scalar_from_uint64(&mut scalar, &coeff);

            scalar
        });
        Self::from_scalars(scalars)
    }
    fn from_scalars(scalars: impl Iterator<Item = blst::blst_scalar>) -> Self {
        let mut coefficients = vec![];

        for scalar in scalars {
            unsafe {
                // TODO error handling here...
                assert!(blst::blst_scalar_fr_check(&scalar));
            }
            coefficients.push(scalar);
        }

        Self { coefficients }
    }
}

pub fn create(polynomial: Polynomial, setup: setup::Setup) -> Commitment {
    let basis = setup.in_g1;
    let coefficients = polynomial.coefficients;

    unsafe {
        let mut result = blst::blst_p1::default();
        for (coefficient, element) in coefficients.iter().zip(basis.iter()) {
            let mut term = blst::blst_p1::default();

            blst::blst_p1_mult(&mut term, element, coefficient, constants::MODULUS_BIT_SIZE);

            blst::blst_p1_add(&mut result, &result, &term);
        }

        Commitment { element: result }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::setup;

    #[test]
    fn it_works() {
        let secret = [0u8; 32];
        let coefficients = vec![1, 2, 3, 1, 1, 17, 32];
        let degree = coefficients.len();

        let setup = setup::generate(&secret, degree);

        let polynomial = Polynomial::from_u64(coefficients.into_iter());

        let commitment = create(polynomial, setup);
        dbg!(commitment);
    }
}
