use crate::polynomial;
use crate::setup;
use oblast::{verify_pairings, Fr, P1, P2};

#[derive(Debug)]
pub struct Opening {
    pub value: Fr,
    pub proof: P1,
}

#[derive(Debug)]
pub struct Commitment<'a> {
    element: P1,
    polynomial: &'a polynomial::Polynomial,
    setup: &'a setup::Setup,
}

fn compute_quotient(
    dividend: &polynomial::Polynomial,
    divisor: &polynomial::Polynomial,
) -> polynomial::Polynomial {
    let mut dividend = dividend.coefficients.clone();
    let mut coefficients = vec![];

    let mut dividend_pos = dividend.len() - 1;
    let divisor_pos = divisor.coefficients.len() - 1;
    let mut difference = dividend_pos as isize - divisor_pos as isize;

    while difference >= 0 {
        let term_quotient = dividend[dividend_pos] / divisor.coefficients[divisor_pos];
        coefficients.push(term_quotient);

        for i in (0..=divisor_pos).rev() {
            let difference = difference as usize;
            let x = divisor.coefficients[i];
            let y = x * term_quotient;
            let z = dividend[difference + i];
            dividend[difference + i] = z - y;
        }

        dividend_pos -= 1;
        difference -= 1;
    }

    coefficients.reverse();
    polynomial::Polynomial { coefficients }
}

impl<'a> Commitment<'a> {
    pub fn open_at(self: &Self, point: Fr) -> Opening {
        let result = self.polynomial.evaluate_at(point);

        // divisor `s - x` for `f(x) = y`
        let divisor_coefficients = vec![-point, Fr::from_u64(1)];
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

    let mut result = P1::default();
    for (coefficient, element) in coefficients.iter().zip(basis.iter()) {
        let term = *coefficient * *element;
        result = result + term;
    }

    Commitment {
        element: result,
        polynomial,
        setup,
    }
}

impl Opening {
    pub fn verify(&self, input: &Fr, commitment: &Commitment) -> bool {
        // Compute [f(s) - y]_1 for LHS
        let y_p1 = self.value * P1::generator();
        let commitment_minus_y = commitment.element + -y_p1;

        // Compute [s - z]_2 for RHS
        let z_p2 = *input * P2::generator();
        let s_minus_z = commitment.setup.in_g2 + -z_p2;

        verify_pairings(commitment_minus_y, P2::generator(), self.proof, s_minus_z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup;
    use hex;
    use std::convert::TryInto;

    #[test]
    fn test_opening() {
        // computed from python reference: https://github.com/ethereum/research/blob/master/kzg_data_availability/kzg_proofs.py
        let test_cases = vec![
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                vec![0],
                0,
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                vec![11],
                11,
                "80fd75ebcc0a21649e3177bcce15426da0e4f25d6828fbf4038d4d7ed3bd4421de3ef61d70f794687b12b2d571971a55",
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                vec![0, 1],
                15,
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                vec![1, 12],
                181,
                "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                "8345dd80ffef0eaec8920e39ebb7f5e9ae9c1d6179e9129b705923df7830c67f3690cbc48649d4079eadf5397339580c",
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                vec![1, 2, 2],
                481,
                "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                "a72841987e4f219d54f2b6a9eac5fe6e78704644753c3579e776a3691bc123743f8c63770ed0f72a71e9e964dbf58f43",
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13],
                6099236329206434206,
                "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                "95c2663b029a933ca94f346061b52dfc85da11386c9aaffe2b604a00589299c10b0855f90c5f7db31cc1cc45353dc948",
            ),
            (
                "0b598c0727a94e556b8c1dcb64af40daea6971901b5dcb8b49da2fe2b533a52e",
                vec![0],
                0,
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                "0b598c0727a94e556b8c1dcb64af40daea6971901b5dcb8b49da2fe2b533a52e",
                vec![11],
                11,
                "80fd75ebcc0a21649e3177bcce15426da0e4f25d6828fbf4038d4d7ed3bd4421de3ef61d70f794687b12b2d571971a55",
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                "0b598c0727a94e556b8c1dcb64af40daea6971901b5dcb8b49da2fe2b533a52e",
                vec![0, 1],
                15,
                "b6464852dee959d00049ce3630a863d5226309fc9cdcb50d991b571a4e8b2f55c61955045918ab4bd6c0460a01fedfe0",
                "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
            ),
            (
                "0b598c0727a94e556b8c1dcb64af40daea6971901b5dcb8b49da2fe2b533a52e",
                vec![1, 12],
                181,
                "adea87ebbba6c937d96ea9bac45a5de282b17bce08e40ab6ed358e2eedda5a0e667a9a744d1369b6e7ffe049686261de",
                "8345dd80ffef0eaec8920e39ebb7f5e9ae9c1d6179e9129b705923df7830c67f3690cbc48649d4079eadf5397339580c",
            ),
            (
                "0b598c0727a94e556b8c1dcb64af40daea6971901b5dcb8b49da2fe2b533a52e",
                vec![1, 2, 2],
                481,
                "b3e43da9f207cb9d717f85d40b967a28254b22bb6269b551aed50444eb1aed7f93a2b519acd7076e56451dc084389323",
                "b8cea544c0d68bf429533df6126a3f9a3ce9027595df4e7fc1e00a368f8b92690251434e51a9b53b35e8e9677960e0b1",
            ),
            (
                "0b598c0727a94e556b8c1dcb64af40daea6971901b5dcb8b49da2fe2b533a52e",
                vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13],
                6099236329206434206,
                "970d3aa5cad4492adb0c87c1f9ee4a82e48a59777d66868827080c145e4562995348af9a486b59f7bdf62a7c25c7159f",
                "b37b9247ff4965586a6e6bb0c5634e34865c233c5c2efc123410fa9f536da2d258c816d3b2db7a3c9c54311837fea7ac",
            ),
            (
                "57a29351ad759e70ac84de21c4a5a54780b46b1a7cfc5bfa033e3b9321562bce",
                vec![0],
                0,
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                "57a29351ad759e70ac84de21c4a5a54780b46b1a7cfc5bfa033e3b9321562bce",
                vec![11],
                11,
                "80fd75ebcc0a21649e3177bcce15426da0e4f25d6828fbf4038d4d7ed3bd4421de3ef61d70f794687b12b2d571971a55",
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                "57a29351ad759e70ac84de21c4a5a54780b46b1a7cfc5bfa033e3b9321562bce",
                vec![0, 1],
                15,
                "94976e86763f440d1338d7c17d181c027630dc39a1d648068683d228300b1085d0c4fbfd9f6f308cda71fdd641834a36",
                "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
            ),
            (
                "57a29351ad759e70ac84de21c4a5a54780b46b1a7cfc5bfa033e3b9321562bce",
                vec![1, 12],
                181,
                "a2dffe3cfef260770472215a66689c0ad35d2fd5868ea369e1a65c47c1cabdb1786a8e5763021b0cac33f458650e80ce",
                "8345dd80ffef0eaec8920e39ebb7f5e9ae9c1d6179e9129b705923df7830c67f3690cbc48649d4079eadf5397339580c",
            ),
            (
                "57a29351ad759e70ac84de21c4a5a54780b46b1a7cfc5bfa033e3b9321562bce",
                vec![1, 2, 2],
                481,
                "a8372e96e8db620e5a5a359f884aea597f358ba9b54d3bf36c712e241dc612e2a7fa81efe3159b2eff19c84b0b7f31f5",
                "acb40f1a984eba565dc9025284fc32f58e01f4bc1af92edbe8114151057998c45da684e50563a2a0a2660d374d851a2f",
            ),
            (
                "57a29351ad759e70ac84de21c4a5a54780b46b1a7cfc5bfa033e3b9321562bce",
                vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13],
                6099236329206434206,
                "81cdc95341621862ebf968daf2760c5412beecb06d272d276a007e1a9c0355f2b053c7bb3e1569366ab7e1b414c5af2e",
                "89e2eb1c44cc5ad3337562570c9940737a1e006a0148f7982c8f3c99bf6484cba0b86edc082b5b90da4190b588c3a3bb",
            ),
        ];

        let point = Fr::from_u64(15);

        for (secret_hex, polynomial, value, expected_commitment_hex, expected_proof_hex) in
            test_cases
        {
            let secret = hex::decode(secret_hex).unwrap();
            let coefficients = polynomial.into_iter().map(Fr::from_u64).collect::<Vec<_>>();

            let degree = coefficients.len();

            let secret = secret.as_slice().try_into().unwrap();
            let setup = setup::generate(secret, degree);

            let polynomial = polynomial::from_coefficients(coefficients.into_iter());

            let commitment = create(&polynomial, &setup);

            let opening = commitment.open_at(point);

            // does evaluation match?
            assert_eq!(opening.value.as_u64(), value);

            // does commitment match?
            let commitment_serialization = commitment.element.compress();
            let expected_commitment_serialization = hex::decode(expected_commitment_hex).unwrap();
            assert_eq!(commitment_serialization, expected_commitment_serialization);

            // does proof match?
            let proof_serialization = opening.proof.compress();
            let expected_proof_serialization = hex::decode(expected_proof_hex).unwrap();
            assert_eq!(proof_serialization, expected_proof_serialization);

            // does the proof verify?
            assert!(opening.verify(&point, &commitment));
        }
    }

    #[test]
    fn test_verify_opening_identity_polynomial() {
        let secret = [11u8; 32];
        let degree = 1;
        let setup = setup::generate(&secret, degree);

        // Using f(x) = x, so [f(s)] = [s]
        let commitment_element = &setup.in_g1[1];
        // Use the same point for input & output
        let point = Fr::from_u64(2);

        let polynomial =
            &polynomial::from_coefficients(vec![Fr::from_u64(0), Fr::from_u64(1)].into_iter());

        let commitment = Commitment {
            element: *commitment_element,
            polynomial: polynomial,
            setup: &setup,
        };

        // Therefore the quotient polynomial is q(x) = 1
        let proof = P1::generator();

        let opening = Opening {
            value: point,
            proof,
        };

        assert!(opening.verify(&point, &commitment));
    }
}
