use blst;

pub type Point = blst::blst_fr;

pub fn add(x: &Point, y: &Point) -> Point {
    let mut sum = Point::default();

    unsafe {
        blst::blst_fr_add(&mut sum, x, y);
    }

    sum
}

pub fn multiply(x: &Point, y: &Point) -> Point {
    let mut product = Point::default();

    unsafe {
        blst::blst_fr_mul(&mut product, x, y);
    }

    product
}

pub fn subtract(x: &Point, y: &Point) -> Point {
    let negative_y = negate(y);
    add(x, &negative_y)
}

pub fn negate(value: &Point) -> Point {
    let mut point = Point::default();

    unsafe {
        // NOTE: it seems the `flag` in the bindings
        // refers to the top-bit of the encoding of the Fr value
        // indicating if `value` is the additive inverse.
        // We seem to get the desired behavior if we leave this as `true`.
        blst::blst_fr_cneg(&mut point, value, true);
    }

    point
}

// `divide` returns `a/b` via multiplying by
// the multiplicative inverse.
pub fn divide(a: &Point, b: &Point) -> Point {
    let mut point = Point::default();
    let mut b_inverse = Point::default();

    unsafe {
        blst::blst_fr_eucl_inverse(&mut b_inverse, b);
        blst::blst_fr_mul(&mut point, a, &b_inverse);
    }

    point
}

pub fn from_u64(value: u64) -> Point {
    let mut point = Point::default();

    let input = vec![value, 0, 0, 0];

    unsafe {
        blst::blst_fr_from_uint64(&mut point, input.as_ptr());
    }

    point
}

// to_u64 returns low-order u64 from `value`.
pub fn to_u64(value: Point) -> u64 {
    let mut buffer = [0u64; 4];
    unsafe {
        blst::blst_uint64_from_fr(buffer.as_mut_ptr(), &value);
    }
    buffer[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_add() {
        let x = from_u64(1001);
        let y = from_u64(2002);
        let sum = add(&x, &y);
        assert_eq!(to_u64(sum), 3003);
    }

    #[test]
    fn can_multiply() {
        let x = from_u64(200);
        let y = from_u64(3);
        let product = multiply(&x, &y);
        assert_eq!(to_u64(product), 600);
    }

    #[test]
    fn can_subtract() {
        let x = from_u64(200);
        let y = from_u64(10);
        let result = subtract(&x, &y);
        assert_eq!(to_u64(result), 190);
    }

    #[test]
    fn can_negate() {
        let x = from_u64(200);
        let neg_x = negate(&x);
        let sum = add(&x, &neg_x);
        let result = to_u64(sum);
        assert_eq!(result, 0);
    }

    #[test]
    fn can_divide() {
        let x = from_u64(200);
        let y = from_u64(2);
        let result_point = divide(&x, &y);
        let result = to_u64(result_point);
        assert_eq!(result, 100);
    }
}
