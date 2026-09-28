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
    fn trunc_matches_cpython_int() {
        assert_eq!(py_trunc(3.5), 3);
        assert_eq!(py_trunc(-3.9), -3);
        assert_eq!(py_trunc(7.15), 7);
    }
}
