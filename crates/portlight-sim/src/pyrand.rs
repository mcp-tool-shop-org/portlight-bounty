//! CPython `random.Random` for the subset the voyage and market engines use.
//!
//! This is the MT19937 generator from CPython's `_random` module (init_by_array
//! seeding, `random()`, `getrandbits`, `randrange`/`randint`, `choice`, and
//! weighted `choices`). Same seed and call sequence as CPython 3.12 produces
//! the same floats and integers, so scripted parity does not need a shared process.

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908_b0df;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;

#[derive(Clone, Debug)]
pub struct PyRandom {
    mt: [u32; N],
    index: usize,
}

impl PyRandom {
    /// Seed from any signed 128-bit integer.
    ///
    /// CPython's `Random.seed(int)` takes the absolute value and splits it into
    /// little-endian 32-bit words. Every `i128` follows that rule, including
    /// the full `u64` range and `2**63`. `i128::MIN` uses magnitude `2**127`.
    /// Python also accepts integers outside `i128`; those are rejected at the
    /// script and CLI parsers (`Invalid number`).
    pub fn from_seed(seed: i128) -> Self {
        let mut rng = Self {
            mt: [0; N],
            index: N,
        };
        rng.seed(seed);
        rng
    }

    pub fn seed(&mut self, seed: i128) {
        let magnitude = if seed == i128::MIN {
            1u128 << 127
        } else {
            seed.unsigned_abs()
        };
        self.seed_u128(magnitude);
    }

    /// Seed from a non-negative integer, split into little-endian 32-bit words
    /// the way `_random.Random.seed` does.
    pub fn seed_u128(&mut self, n: u128) {
        if n == 0 {
            self.init_by_array(&[0]);
            return;
        }
        let mut key = Vec::new();
        let mut x = n;
        while x > 0 {
            key.push(x as u32);
            x >>= 32;
        }
        self.init_by_array(&key);
    }

    fn init_genrand(&mut self, seed: u32) {
        self.mt[0] = seed;
        for mti in 1..N {
            let prev = self.mt[mti - 1];
            self.mt[mti] = 1_812_433_253u32
                .wrapping_mul(prev ^ (prev >> 30))
                .wrapping_add(mti as u32);
        }
        self.index = N;
    }

    fn init_by_array(&mut self, key: &[u32]) {
        self.init_genrand(19_650_218);
        let mut i = 1usize;
        let mut j = 0usize;
        let mut k = if N > key.len() { N } else { key.len() };
        while k > 0 {
            let prev = self.mt[i - 1];
            self.mt[i] = (self.mt[i] ^ ((prev ^ (prev >> 30)).wrapping_mul(1_664_525)))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                self.mt[0] = self.mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
            k -= 1;
        }
        k = N - 1;
        while k > 0 {
            let prev = self.mt[i - 1];
            self.mt[i] = (self.mt[i] ^ ((prev ^ (prev >> 30)).wrapping_mul(1_566_083_941)))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                self.mt[0] = self.mt[N - 1];
                i = 1;
            }
            k -= 1;
        }
        self.mt[0] = 0x8000_0000;
    }

    fn genrand_uint32(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }
        let mut y = self.mt[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }

    fn twist(&mut self) {
        for kk in 0..(N - M) {
            let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
            let mag = if y & 1 == 0 { 0 } else { MATRIX_A };
            self.mt[kk] = self.mt[kk + M] ^ (y >> 1) ^ mag;
        }
        for kk in (N - M)..(N - 1) {
            let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
            let mag = if y & 1 == 0 { 0 } else { MATRIX_A };
            self.mt[kk] = self.mt[kk + M - N] ^ (y >> 1) ^ mag;
        }
        let y = (self.mt[N - 1] & UPPER_MASK) | (self.mt[0] & LOWER_MASK);
        let mag = if y & 1 == 0 { 0 } else { MATRIX_A };
        self.mt[N - 1] = self.mt[M - 1] ^ (y >> 1) ^ mag;
        self.index = 0;
    }

    /// `random.Random.random`: float in `[0.0, 1.0)`.
    pub fn random(&mut self) -> f64 {
        let a = (self.genrand_uint32() >> 5) as f64;
        let b = (self.genrand_uint32() >> 6) as f64;
        (a * 67_108_864.0 + b) * (1.0 / 9_007_199_254_740_992.0)
    }

    pub fn getrandbits(&mut self, k: u32) -> u64 {
        if k == 0 {
            return 0;
        }
        if k <= 32 {
            return u64::from(self.genrand_uint32() >> (32 - k));
        }
        let mut result = 0u128;
        let mut remaining = k;
        let mut shift = 0u32;
        while remaining > 0 {
            let take = remaining.min(32);
            let bits = u128::from(self.genrand_uint32() >> (32 - take));
            result |= bits << shift;
            shift += take;
            remaining -= take;
        }
        result as u64
    }

    /// `Random._randbelow(n)` for `n >= 1`.
    pub fn randbelow(&mut self, n: u64) -> u64 {
        debug_assert!(n >= 1);
        let k = 64 - n.leading_zeros();
        loop {
            let r = self.getrandbits(k);
            if r < n {
                return r;
            }
        }
    }

    /// Inclusive `randint(a, b)`, matching `randrange(a, b + 1)`.
    pub fn randint(&mut self, a: i64, b: i64) -> i64 {
        let width = (b - a + 1) as u64;
        a + self.randbelow(width) as i64
    }

    pub fn choice_index(&mut self, n: usize) -> usize {
        self.randbelow(n as u64) as usize
    }

    /// `Random.choices(population, weights=..., k=1)[0]` index.
    ///
    /// Uses cumulative weights and `bisect_right` with `hi = n - 1`, which is
    /// what CPython's `random.choices` does for non-integer totals.
    pub fn choices_weighted(&mut self, weights: &[f64]) -> usize {
        let n = weights.len();
        debug_assert!(n >= 1);
        let mut cum = Vec::with_capacity(n);
        let mut acc = 0.0;
        for w in weights {
            acc += *w;
            cum.push(acc);
        }
        let total = cum[n - 1];
        let x = self.random() * total;
        bisect_right(&cum, x, 0, n - 1)
    }
}

fn bisect_right(a: &[f64], x: f64, mut lo: usize, mut hi: usize) -> usize {
    while lo < hi {
        let mid = (lo + hi) / 2;
        if x < a[mid] {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_matches_cpython_seeds() {
        // First values from CPython 3.12 `random.Random(seed).random()`.
        let first = |seed| PyRandom::from_seed(seed).random();
        assert_eq!(first(0), 0.844_421_851_525_048_1);
        assert_eq!(first(1), 0.134_364_244_112_401_22);
        assert_eq!(first(42), 0.639_426_798_457_883_7);

        let mut rng = PyRandom::from_seed(42);
        let seq = [
            0.639_426_798_457_883_7,
            0.025_010_755_222_666_936,
            0.275_029_318_369_119_26,
            0.223_210_738_148_822_75,
            0.736_471_214_164_012_4,
        ];
        for expected in seq {
            assert_eq!(rng.random(), expected);
        }
    }

    #[test]
    fn randint_and_choices_match_cpython() {
        let mut rng = PyRandom::from_seed(42);
        let got: Vec<_> = (0..8).map(|_| rng.randint(0, 10)).collect();
        assert_eq!(got, vec![10, 1, 0, 4, 3, 3, 2, 1]);

        let mut rng = PyRandom::from_seed(42);
        assert_eq!(rng.choice_index(4), 0); // choice(["a","b","c","d"]) == "a"

        let mut rng = PyRandom::from_seed(42);
        let weights = [0.2, 0.5, 0.3];
        let labels = ["a", "b", "c"];
        let got: Vec<_> = (0..6)
            .map(|_| labels[rng.choices_weighted(&weights)])
            .collect();
        assert_eq!(got, vec!["b", "a", "b", "b", "c", "b"]);
    }

    #[test]
    fn big_seed_matches_cpython() {
        // random.Random(2**70 + 12345).random()
        let mut rng = PyRandom::from_seed((1i128 << 70) + 12_345);
        assert_eq!(rng.random(), 0.960_280_354_602_301_1);
    }

    #[test]
    fn signed_64_bit_boundary_matches_cpython() {
        // 2**63 and -(2**63) share an absolute value, so the sequences match.
        // 2**63 does not fit in i64; i128 accepts it. Integers outside i128
        // are not representable here.
        let mut pos = PyRandom::from_seed(1i128 << 63);
        let mut neg = PyRandom::from_seed(-(1i128 << 63));
        let expected = [
            0.553_463_998_391_419_9,
            0.940_614_470_909_046_2,
            0.241_994_321_116_908_33,
        ];
        for value in expected {
            assert_eq!(pos.random(), value);
            assert_eq!(neg.random(), value);
        }
        assert_eq!(pos.randint(1, 6), 5);
        assert_eq!(neg.randint(1, 6), 5);
        assert_eq!(pos.choice_index(3), 0);
        assert_eq!(neg.choice_index(3), 0);
    }
}
