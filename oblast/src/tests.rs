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

#[test]
fn can_add() {
    let x = Fr::from_u64(1001);
    let y = Fr::from_u64(2002);
    let sum = x + y;
    assert_eq!(sum.as_u64(), 3003);
}

#[test]
fn can_multiply() {
    let x = Fr::from_u64(200);
    let y = Fr::from_u64(3);
    let product = x * y;
    assert_eq!(product.as_u64(), 600);
}

#[test]
fn can_subtract() {
    let x = Fr::from_u64(200);
    let y = Fr::from_u64(10);
    let result = x - y;
    assert_eq!(result.as_u64(), 190);
}

#[test]
fn can_negate() {
    let x = Fr::from_u64(200);
    let sum = x + -x;
    assert_eq!(sum.as_u64(), 0);
}

#[test]
fn can_divide() {
    let x = Fr::from_u64(200);
    let y = Fr::from_u64(2);
    let result = x / y;
    assert_eq!(result.as_u64(), 100);
}
