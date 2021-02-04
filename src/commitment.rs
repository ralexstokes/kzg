use crate::constants;
use crate::point;
use crate::polynomial;
use crate::setup;
use blst;

pub struct Proof {}

pub struct Opening {
    pub value: point::Point,
    pub proof: Proof,
}

#[derive(Debug)]
pub struct Commitment<'a> {
    element: blst::blst_p1,
    polynomial: &'a polynomial::Polynomial,
}

impl<'a> Commitment<'a> {
    pub fn open_at(self: &Self, point: point::Point) -> Opening {
        let result = self.polynomial.evaluate_at(point);

        Opening {
            value: result,
            proof: Proof {},
        }
    }
}

pub fn create<'a>(polynomial: &'a polynomial::Polynomial, setup: &setup::Setup) -> Commitment<'a> {
    let basis = &setup.in_g1;
    let coefficients = &polynomial.coefficients;

    unsafe {
        let mut result = blst::blst_p1::default();
        for (coefficient, element) in coefficients.iter().zip(basis.iter()) {
            let mut term = blst::blst_p1::default();

            let mut coefficient_scalar = blst::blst_scalar::default();
            blst::blst_scalar_from_fr(&mut coefficient_scalar, coefficient);
            blst::blst_p1_mult(
                &mut term,
                element,
                &coefficient_scalar,
                constants::MODULUS_BIT_SIZE,
            );

            blst::blst_p1_add(&mut result, &result, &term);
        }

        Commitment {
            element: result,
            polynomial,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup;

    #[test]
    fn test_create() {
        let secret = [0u8; 32];
        let coefficients = vec![1, 2, 3, 1, 1, 17, 32]
            .into_iter()
            .map(point::from_u64)
            .collect::<Vec<_>>();
        let degree = coefficients.len();

        let setup = setup::generate(&secret, degree);

        let polynomial = polynomial::from_coefficients(coefficients.into_iter());

        create(&polynomial, &setup);
    }
}
