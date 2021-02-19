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

    let mut dividend_pos = dividend.len() - 1;
    let divisor_pos = divisor.coefficients.len() - 1;
    let mut difference = dividend_pos as isize - divisor_pos as isize;

    while difference >= 0 {
        let term_quotient =
            point::divide(&dividend[dividend_pos], &divisor.coefficients[divisor_pos]);
        coefficients.push(term_quotient);

        for i in (0..=divisor_pos).rev() {
            let difference = difference as usize;
            let x = divisor.coefficients[i];
            let y = point::multiply(&x, &term_quotient);
            let z = dividend[difference + i];
            dividend[difference + i] = point::subtract(&z, &y);
        }

        dividend_pos -= 1;
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
                11 ,
                "80fd75ebcc0a21649e3177bcce15426da0e4f25d6828fbf4038d4d7ed3bd4421de3ef61d70f794687b12b2d571971a55",
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                vec![0, 1],
                15 ,
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                vec![1, 12],
                181 ,
                "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                "8345dd80ffef0eaec8920e39ebb7f5e9ae9c1d6179e9129b705923df7830c67f3690cbc48649d4079eadf5397339580c",
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                vec![1, 2, 2],
                481 ,
                "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                "a72841987e4f219d54f2b6a9eac5fe6e78704644753c3579e776a3691bc123743f8c63770ed0f72a71e9e964dbf58f43",
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13],
                6099236329206434206 ,
                "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
                "95c2663b029a933ca94f346061b52dfc85da11386c9aaffe2b604a00589299c10b0855f90c5f7db31cc1cc45353dc948",
            ),
            (
                "f90b6bfdb2f26a3d8ca62b71bb1cb4db6690d5cbc6de88c4ba11ff1fc00c3876",
                vec![0],
                0 ,
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                "f90b6bfdb2f26a3d8ca62b71bb1cb4db6690d5cbc6de88c4ba11ff1fc00c3876",
                vec![11],
                11 ,
                "80fd75ebcc0a21649e3177bcce15426da0e4f25d6828fbf4038d4d7ed3bd4421de3ef61d70f794687b12b2d571971a55",
                "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                "f90b6bfdb2f26a3d8ca62b71bb1cb4db6690d5cbc6de88c4ba11ff1fc00c3876",
                vec![0, 1],
                15 ,
                "b43e5c8916759f302ce05430a147cbac51ce1ed763a732b09f48b4abd0f291a5e82c254a532d6d1e6f3eeb41a37cefbb",
                "97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb",
            ),
            (
                "f90b6bfdb2f26a3d8ca62b71bb1cb4db6690d5cbc6de88c4ba11ff1fc00c3876",
                vec![1, 12],
                181 ,
                "a9290ebea935358a73dccb511c24f4bd81d7451b834bc56eae49257b0cab5ac36c10525910bbde664cb39de7c8aaf995",
                "8345dd80ffef0eaec8920e39ebb7f5e9ae9c1d6179e9129b705923df7830c67f3690cbc48649d4079eadf5397339580c",
            ),
            (
                "f90b6bfdb2f26a3d8ca62b71bb1cb4db6690d5cbc6de88c4ba11ff1fc00c3876",
                vec![1, 2, 2],
                481 ,
                "a9754fe6de9d2e8bc2702501f8ee86adf2dd83bad7150c794ee1382716a47d78e34e2acf6f01f92e4a4af7ec28154503",
                "95892cd75d24e865d739c63e7874a9e1810c44e1ffa4eeb197ba70717daad2b4b4a4ee7742a28754f1cb35a178417b00",
            ),
            (
                "f90b6bfdb2f26a3d8ca62b71bb1cb4db6690d5cbc6de88c4ba11ff1fc00c3876",
                vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13],
                6099236329206434206 ,
                "8dcb3189ff1b845a2e4dbb5859a95b4f3fa0a63aa86e41619ec0616c70cf45869b1ae4e1e9e387947e43242827a6642d",
                "97a532e00dc2504f060506580d450b5293ac7e17a358018e01242b39357b45e1b2527c4b28ca0a3ccf0149da4bc69292",
            ),
        ];

        let point = point::from_u64(15);

        for (secret_hex, polynomial, value, expected_commitment_hex, expected_proof_hex) in
            test_cases
        {
            let secret = hex::decode(secret_hex).unwrap();
            let coefficients = polynomial
                .into_iter()
                .map(point::from_u64)
                .collect::<Vec<_>>();

            let degree = coefficients.len();

            let secret = secret.as_slice().try_into().unwrap();
            let setup = setup::generate(secret, degree);

            let polynomial = polynomial::from_coefficients(coefficients.into_iter());

            let commitment = create(&polynomial, &setup);

            let opening = commitment.open_at(point);

            // does evaluation match?
            assert_eq!(point::to_u64(opening.value), value);

            // does commitment match?
            let mut commitment_serialization = vec![0u8; 48];
            unsafe {
                blst::blst_p1_compress(commitment_serialization.as_mut_ptr(), &commitment.element);
            }
            let expected_commitment_serialization = hex::decode(expected_commitment_hex).unwrap();
            assert_eq!(commitment_serialization, expected_commitment_serialization);

            // does proof match?
            let mut proof_serialization = vec![0u8; 48];
            unsafe {
                blst::blst_p1_compress(proof_serialization.as_mut_ptr(), &opening.proof);
            }
            let expected_proof_serialization = hex::decode(expected_proof_hex).unwrap();
            assert_eq!(proof_serialization, expected_proof_serialization);
        }
    }
}
