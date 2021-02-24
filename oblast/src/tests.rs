use crate::*;

#[test]
fn two_times_three_pairing() {
    assert!(verify_pairings(
        Fr::from_u64(2) * P1::generator(),
        Fr::from_u64(3) * P2::generator(),
        Fr::from_u64(3) * P1::generator(),
        Fr::from_u64(2) * P2::generator(),
    ));
}

#[test]
fn invalid_pairing() {
    assert!(!verify_pairings(
        Fr::from_u64(2) * P1::generator(),
        Fr::from_u64(4) * P2::generator(),
        Fr::from_u64(3) * P1::generator(),
        Fr::from_u64(2) * P2::generator(),
    ));
}
