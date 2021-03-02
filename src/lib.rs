mod commitment;
mod constants;
mod polynomial;
mod setup;

#[cfg(test)]
mod tests {
    use super::*;
    use oblast;

    #[test]
    fn end_to_end() {
        let coefficients = vec![1, 2, 3, 1, 1, 17, 32]
            .into_iter()
            .map(oblast::Fr::from_u64)
            .collect::<Vec<_>>();
        let degree = coefficients.len();

        let setup = setup::generate_with_random_secret(degree);

        let polynomial = polynomial::from_coefficients(coefficients.into_iter());

        // prover sends commitment
        let commitment = commitment::create(&polynomial, &setup);

        // verifier sends over a point
        let point = oblast::Fr::from_u64(1234);

        // prover "opens" at that point
        let opening = commitment.open_at(point);

        // verifier can verify the opening
        let valid = opening.verify(&point, &commitment);
        assert!(valid);
    }
}
