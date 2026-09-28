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

/// `hash(text)` for `PYTHONHASHSEED=0` (SipHash-1-3, zero key).
///
/// CPython 3.12 uses this for `str`. The empty string hashes to 0. A digest
/// of `-1` becomes `-2`.
pub fn py_str_hash(text: &str) -> i64 {
    if text.is_empty() {
        return 0;
    }
    let mut digest = siphash13(text.as_bytes());
    if digest == u64::MAX {
        digest = u64::MAX - 1;
    }
    digest as i64
}

/// `list(set(items))` for CPython 3.12.3 with `PYTHONHASHSEED=0`.
///
/// Insertion follows the iterator. Duplicates are ignored. The returned order
/// is the set table's probe order, which is what `rng.choice(list(a_set))` sees.
pub fn py_set_order<I, S>(items: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    const LINEAR: usize = 9;
    let mut mask = 7usize;
    let mut table: Vec<Option<(String, i64)>> = vec![None; 8];
    let mut fill = 0usize;
    let mut used = 0usize;

    for item in items {
        let key = item.as_ref();
        let hash = py_str_hash(key);
        let mut perturb = hash as u64;
        let mut index = (perturb as usize) & mask;
        let mut placed = false;
        while !placed {
            let mut probes = if index + LINEAR <= mask { LINEAR } else { 0 };
            let mut entry = index;
            loop {
                match &table[entry] {
                    None => {
                        table[entry] = Some((key.to_string(), hash));
                        fill += 1;
                        used += 1;
                        placed = true;
                        break;
                    }
                    Some((existing, existing_hash))
                        if *existing_hash == hash && existing == key =>
                    {
                        placed = true;
                        break;
                    }
                    Some(_) => {}
                }
                entry += 1;
                if probes == 0 {
                    break;
                }
                probes -= 1;
            }
            if placed {
                break;
            }
            perturb >>= 5;
            index = index
                .wrapping_mul(5)
                .wrapping_add(1)
                .wrapping_add(perturb as usize)
                & mask;
        }
        if fill * 5 >= mask * 3 {
            let minused = if used > 50_000 { used * 2 } else { used * 4 };
            let old: Vec<(String, i64)> = table.into_iter().flatten().collect();
            let mut newsize = 8usize;
            while newsize <= minused {
                newsize <<= 1;
            }
            mask = newsize - 1;
            table = vec![None; newsize];
            for (key, hash) in old {
                insert_clean(&mut table, mask, key, hash);
            }
        }
    }
    table.into_iter().flatten().map(|(key, _)| key).collect()
}

fn insert_clean(table: &mut [Option<(String, i64)>], mask: usize, key: String, hash: i64) {
    const LINEAR: usize = 9;
    let mut perturb = hash as u64;
    let mut index = (perturb as usize) & mask;
    loop {
        if table[index].is_none() {
            table[index] = Some((key, hash));
            return;
        }
        if index + LINEAR <= mask {
            for step in 1..=LINEAR {
                if table[index + step].is_none() {
                    table[index + step] = Some((key, hash));
                    return;
                }
            }
        }
        perturb >>= 5;
        index = index
            .wrapping_mul(5)
            .wrapping_add(1)
            .wrapping_add(perturb as usize)
            & mask;
    }
}

fn siphash13(data: &[u8]) -> u64 {
    let mut v0 = 0x736f6d6570736575u64;
    let mut v1 = 0x646f72616e646f6du64;
    let mut v2 = 0x6c7967656e657261u64;
    let mut v3 = 0x7465646279746573u64;
    let sipround = |v0: &mut u64, v1: &mut u64, v2: &mut u64, v3: &mut u64| {
        *v0 = v0.wrapping_add(*v1);
        *v1 = v1.rotate_left(13) ^ *v0;
        *v0 = v0.rotate_left(32);
        *v2 = v2.wrapping_add(*v3);
        *v3 = v3.rotate_left(16) ^ *v2;
        *v0 = v0.wrapping_add(*v3);
        *v3 = v3.rotate_left(21) ^ *v0;
        *v2 = v2.wrapping_add(*v1);
        *v1 = v1.rotate_left(17) ^ *v2;
        *v2 = v2.rotate_left(32);
    };
    let mut offset = 0;
    while offset + 8 <= data.len() {
        let mut block = 0u64;
        for (shift, byte) in data[offset..offset + 8].iter().enumerate() {
            block |= u64::from(*byte) << (8 * shift);
        }
        v3 ^= block;
        sipround(&mut v0, &mut v1, &mut v2, &mut v3);
        v0 ^= block;
        offset += 8;
    }
    let mut tail = (data.len() as u64 & 0xff) << 56;
    for (shift, byte) in data[offset..].iter().enumerate() {
        tail |= u64::from(*byte) << (8 * shift);
    }
    v3 ^= tail;
    sipround(&mut v0, &mut v1, &mut v2, &mut v3);
    v0 ^= tail;
    v2 ^= 0xff;
    sipround(&mut v0, &mut v1, &mut v2, &mut v3);
    sipround(&mut v0, &mut v1, &mut v2, &mut v3);
    sipround(&mut v0, &mut v1, &mut v2, &mut v3);
    v0 ^ v1 ^ v2 ^ v3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_set_order_match_cpython_312_seed_zero() {
        assert_eq!(py_str_hash("porto_novo"), -5275048633881927893);
        assert_eq!(py_str_hash(""), 0);
        assert_eq!(
            py_set_order(["porto_novo", "silva_bay", "al_manar", "ironhaven"]),
            vec![
                "ironhaven".to_string(),
                "porto_novo".to_string(),
                "al_manar".to_string(),
                "silva_bay".to_string(),
            ]
        );
        assert_eq!(
            py_set_order(["a", "b", "c", "d", "e", "f", "g", "h", "i"]),
            ["d", "f", "g", "h", "b", "i", "c", "a", "e"]
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            py_set_order(["porto_novo", "silva_bay", "porto_novo"]),
            py_set_order(["porto_novo", "silva_bay"])
        );
    }

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
