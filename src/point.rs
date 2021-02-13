use blst;

pub type Point = blst::blst_fr;

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
        let mut sum = Point::default();
        unsafe {
            blst::blst_fr_add(&mut sum, &x, &y);
        }
        assert_eq!(to_u64(sum), 3003);
    }

    #[test]
    fn can_mul() {
        let x = from_u64(200);
        let y = from_u64(3);
        let mut product = Point::default();
        unsafe {
            blst::blst_fr_mul(&mut product, &x, &y);
        }
        assert_eq!(to_u64(product), 600);
    }
}
