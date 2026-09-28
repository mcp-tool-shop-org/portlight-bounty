//! Merchant price markup from `engine/merchant.py`.
//!
//! Buying an item also needs the weapon and armor catalogs, which are not part
//! of this area. The markup itself is `max(1, round(base * markup))`.

use crate::util::py_round;

pub fn marked_price(base_cost: i64, markup: f64) -> i64 {
    1.max(py_round(base_cost as f64 * markup))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content;

    #[test]
    fn marco_marks_a_blade_up_by_ten_percent() {
        let marco = content::content().merchant("marco_the_blade").unwrap();
        assert!((marco.price_markup - 1.10).abs() < 1e-9);
        assert_eq!(marked_price(15, marco.price_markup), py_round_expected());
        assert_eq!(marked_price(0, 1.1), 1);
    }

    fn py_round_expected() -> i64 {
        // 15 * 1.10 = 16.5, and Python 3 rounds halves to even → 16.
        16
    }
}
