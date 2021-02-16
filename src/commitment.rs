use crate::constants;
use crate::point;
use crate::polynomial;
use crate::setup;
use blst;

#[derive(Debug)]
pub struct Opening {
    pub value: point::Point,
    pub proof: blst::blst_p1,
}

#[derive(Debug)]
pub struct Commitment<'a> {
    element: blst::blst_p1,
    polynomial: &'a polynomial::Polynomial,
    setup: &'a setup::Setup,
}

fn compute_quotient(
    dividend: &polynomial::Polynomial,
    divisor: &polynomial::Polynomial,
) -> polynomial::Polynomial {
    let mut dividend = dividend.coefficients.clone();
    let mut coefficients = vec![];

    let mut len_dividend = dividend.len() - 1;
    let len_divisor = divisor.coefficients.len() - 1;
    let mut difference = len_dividend - len_divisor;

    while difference > 0 {
        let term_quotient =
            point::divide(&dividend[len_dividend], &divisor.coefficients[len_divisor]);
        coefficients.push(term_quotient);

        for i in (0..=len_divisor).rev() {
            let x = divisor.coefficients[i];
            let y = point::multiply(&x, &term_quotient);
            let z = dividend[difference + i];
            dividend[difference + i] = point::subtract(&z, &y);
        }

        len_dividend -= 1;
        if difference == 0 {
            break;
        }
        difference -= 1;
    }

    coefficients.reverse();
    polynomial::Polynomial { coefficients }
}

impl<'a> Commitment<'a> {
    pub fn open_at(self: &Self, point: point::Point) -> Opening {
        let result = self.polynomial.evaluate_at(point);

        // divisor `s - x` for `f(x) = y`
        let divisor_coefficients = vec![point::negate(&point), point::from_u64(1)];
        let divisor = polynomial::from_coefficients(divisor_coefficients.into_iter());

        let quotient_polynomial = compute_quotient(self.polynomial, &divisor);

        let commitment = create(&quotient_polynomial, self.setup);

        Opening {
            value: result,
            proof: commitment.element,
        }
    }
}

pub fn create<'a>(
    polynomial: &'a polynomial::Polynomial,
    setup: &'a setup::Setup,
) -> Commitment<'a> {
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
            setup,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup;

    #[test]
    fn test_opening() {
        let secret = [0u8; 32];
        let coefficients = vec![1, 2, 3]
            .into_iter()
            .map(point::from_u64)
            .collect::<Vec<_>>();
        let degree = coefficients.len();

        let setup = setup::generate(&secret, degree);

        let polynomial = polynomial::from_coefficients(coefficients.into_iter());

        let commitment = create(&polynomial, &setup);

        let point = point::from_u64(3);
        let opening = commitment.open_at(point);
        assert_eq!(point::to_u64(opening.value), 34);
    }
}
