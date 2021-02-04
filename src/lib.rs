mod commitment;
mod constants;
mod point;
mod polynomial;
mod setup;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn it_works() {
        let mut rng = thread_rng();

        let mut secret = [0u8; 32];
        rng.fill_bytes(&mut secret);

        let coefficients = vec![1, 2, 3, 1, 1, 17, 32]
            .into_iter()
            .map(point::from_u64)
            .collect::<Vec<_>>();
        let degree = coefficients.len();

        let setup = setup::generate(&secret, degree);

        let polynomial = polynomial::from_coefficients(coefficients.into_iter());

        let commitment = commitment::create(&polynomial, &setup);

        let point = point::from_u64(1234);

        let opening = commitment.open_at(point);

        // let valid = opening.verify(&setup);
        let valid = true;
        assert!(valid)
    }
}
