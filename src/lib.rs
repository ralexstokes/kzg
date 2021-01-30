mod setup;

#[cfg(test)]
mod tests {
    use blst::min_pk::*;
    use blst::BLST_ERROR;
    use rand::prelude::*;

    const DST: &[u8; 43] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

    #[test]
    fn it_works() {
        let mut rng = thread_rng();

        let mut ikm = [0u8; 32];
        rng.fill_bytes(&mut ikm);
        let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
        let pk = sk.sk_to_pk();

        let msg = b"blst is such a blast!";
        let sig = sk.sign(msg, DST, &[]);

        // no need to check data we get directly from the library...
        let perform_checks = false;
        let err = sig.verify(perform_checks, msg, DST, &[], &pk, perform_checks);
        assert_eq!(err, BLST_ERROR::BLST_SUCCESS);
    }
}
