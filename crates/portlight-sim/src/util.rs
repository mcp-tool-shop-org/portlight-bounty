//! Numeric helpers that match CPython 3 rounding and truncation.

/// Python 3 `round(x)` with no `ndigits` (returns an int).
///
/// CPython rounds half away from zero, then if the input was exactly halfway
/// it corrects toward even via `2 * round(x / 2)`.
pub fn py_round(x: f64) -> i64 {
    if !x.is_finite() {
        return 0;
    }
    let rounded = x.round();
    if (x - rounded).abs() == 0.5 {
        let even = (x / 2.0).round() * 2.0;
        return even as i64;
    }
    rounded as i64
}

/// Python 3 `round(x, ndigits)` for `ndigits >= 0`.
///
/// Scales by `10**ndigits`, applies [`py_round`], and scales back. That matches
/// CPython for the magnitudes the victory-path scores use.
pub fn py_round_places(x: f64, places: u32) -> f64 {
    if !x.is_finite() {
        return 0.0;
    }
    let scale = 10f64.powi(places as i32);
    py_round(x * scale) as f64 / scale
}

/// Python `int(x)` for a finite float: truncate toward zero.
pub fn py_trunc(x: f64) -> i64 {
    if !x.is_finite() {
        return 0;
    }
    x.trunc() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_matches_cpython() {
        // Values checked against CPython 3.12 `round`.
        let cases = [
            (0.5, 0),
            (1.5, 2),
            (2.5, 2),
            (3.5, 4),
            (-0.5, 0),
            (-1.5, -2),
            (-2.5, -2),
            (1.4, 1),
            (1.6, 2),
            (-1.4, -1),
            (-1.6, -2),
        ];
        for (input, expected) in cases {
            assert_eq!(py_round(input), expected, "round({input})");
        }
    }

    #[test]
    fn round_places_matches_cpython() {
        // Checked against CPython 3.12 `round(x, 1)`.
        assert_eq!(py_round_places(26.666666666666668, 1), 26.7);
        assert_eq!(py_round_places(16.666666666666668, 1), 16.7);
        assert_eq!(py_round_places(-3.333333333333332, 1), -3.3);
        assert_eq!(py_round_places(1.25, 1), 1.2);
        assert_eq!(py_round_places(0.0, 1), 0.0);
    }

    #[test]
    fn trunc_matches_cpython_int() {
        assert_eq!(py_trunc(3.5), 3);
        assert_eq!(py_trunc(-3.9), -3);
        assert_eq!(py_trunc(7.15), 7);
    }
}
