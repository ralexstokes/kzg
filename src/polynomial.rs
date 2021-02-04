use crate::point;
use blst;

#[derive(Debug)]
pub struct Polynomial {
    // NOTE: low-order coefficients are first in the vector
    pub coefficients: Vec<point::Point>,
}

impl Polynomial {
    pub fn evaluate_at(self: &Self, point: point::Point) -> point::Point {
        let mut sum = self.coefficients[0].clone();

        let mut powers = point.clone();
        for coefficient in self.coefficients.iter().skip(1) {
            let mut term = point::Point::default();
            unsafe {
                blst::blst_fr_mul(&mut term, coefficient, &powers);
                blst::blst_fr_add(&mut sum, &sum, &term);
                blst::blst_fr_mul(&mut powers, &powers, &point);
            }
        }
        sum
    }
}

pub fn from_coefficients(coefficients: impl Iterator<Item = point::Point>) -> Polynomial {
    Polynomial {
        coefficients: coefficients.collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_eval_polynomial() {
        let coefficients = vec![42, 1, 1, 0, 1]
            .into_iter()
            .map(point::from_u64)
            .collect::<Vec<_>>();
        let polynomial = from_coefficients(coefficients.into_iter());
        let point = point::from_u64(2);
        let result_in_fr = polynomial.evaluate_at(point);
        let mut result: u64 = 0;
        unsafe {
            blst::blst_uint64_from_fr(&mut result, &result_in_fr);
        }
        assert_eq!(result, 64);
    }
}
