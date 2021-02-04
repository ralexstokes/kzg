use blst;

pub type Point = blst::blst_fr;

pub fn from_u64(input: u64) -> Point {
    let mut point = Point::default();

    unsafe {
        blst::blst_fr_from_uint64(&mut point, &input);
    }

    point
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_add() {
        let x = from_u64(1001);
        let y = from_u64(2002);
        let mut sum = Point::default();
        let mut result: u64 = 0;
        unsafe {
            blst::blst_fr_add(&mut sum, &x, &y);
            blst::blst_uint64_from_fr(&mut result, &sum);
        }
        assert_eq!(result, 3003);
    }

    #[test]
    fn can_mult() {
        let x = from_u64(1);
        let y = from_u64(2);
        let mut product = Point::default();
        let mut result: u64 = 0;

        unsafe {
            blst::blst_fr_mul(&mut product, &x, &y);
            blst::blst_uint64_from_fr(&mut result, &product);
        }
        assert_eq!(result, 2);
    }
}
