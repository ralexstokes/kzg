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
    fn can_mul() {
        let x = from_u64(200);
        let y = from_u64(3);
        let mut product = Point::default();
        let mut result: u64 = 0;

        unsafe {
            blst::blst_fr_mul(&mut product, &x, &y);
            blst::blst_uint64_from_fr(&mut result, &product);
        }
        dbg!(result);
        assert_eq!(result, 600);
    }
}
