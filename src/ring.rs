use num_bigint::BigUint;
use num_traits::Zero;
use std::{
    fmt::{write, Debug},
    ops::{Add, AddAssign, Mul},
};

#[derive(Default, Clone)]
pub struct Ring {
    q: usize,
    n: usize,
    coefficients: Vec<BigUint>,
    root_of_unity: u8,
    ntt_zetas: Vec<usize>,
    ntt_f: usize,
}

impl Ring {
    pub fn new(coefficients: &Vec<BigUint>) -> Self {
        Ring {
            q: 3329,
            n: 256,
            coefficients: coefficients.clone(),
            root_of_unity: 17,
            ntt_f: 3303, // pow(128, -1, 3329)
            ntt_zetas: vec![
                1, 1729, 2580, 3289, 2642, 630, 1897, 848, 1062, 1919, 193, 797, 2786, 3260, 569,
                1746, 296, 2447, 1339, 1476, 3046, 56, 2240, 1333, 1426, 2094, 535, 2882, 2393,
                2879, 1974, 821, 289, 331, 3253, 1756, 1197, 2304, 2277, 2055, 650, 1977, 2513,
                632, 2865, 33, 1320, 1915, 2319, 1435, 807, 452, 1438, 2868, 1534, 2402, 2647,
                2617, 1481, 648, 2474, 3110, 1227, 910, 17, 2761, 583, 2649, 1637, 723, 2288, 1100,
                1409, 2662, 3281, 233, 756, 2156, 3015, 3050, 1703, 1651, 2789, 1789, 1847, 952,
                1461, 2687, 939, 2308, 2437, 2388, 733, 2337, 268, 641, 1584, 2298, 2037, 3220,
                375, 2549, 2090, 1645, 1063, 319, 2773, 757, 2099, 561, 2466, 2594, 2804, 1092,
                403, 1026, 1143, 2150, 2775, 886, 1722, 1212, 1874, 1029, 2110, 2935, 885, 2154,
            ],
        }
    }

    fn add_mod_q(&self, x: &BigUint, y: &BigUint) -> BigUint {
        (x + y) % self.q
    }

    pub fn encode(&self, d: usize) -> Vec<u8> {
        let mut t = BigUint::zero();
        for i in 0..255 {
            t |= &self.coefficients[256 - i - 1];
            t <<= d
        }
        t |= &self.coefficients[0];
        t.to_bytes_le()[0..(32 * d)].to_vec()
    }

    pub fn ntt_sample(&self, input_bytes: &[u8]) -> Self {
        let mut i = 0;
        let mut j = 0;
        let mut coefficients = vec![BigUint::zero(); self.n];
        while j < self.n {
            let a: usize = input_bytes[i].into();
            let b: usize = input_bytes[i + 1].into();
            let c: usize = 256 * (b % 16);
            let d_1: usize = a + c;
            let d: usize = input_bytes[i + 2].into();
            let d_2 = (b / 16) + (16 * d);

            if d_1 < 3329 {
                coefficients[j] = BigUint::from(d_1);
                j = j + 1
            }

            if d_2 < 3329 && j < self.n {
                coefficients[j] = BigUint::from(d_2);
                j = j + 1
            }

            i = i + 3;
        }
        Ring::new(&coefficients)
    }

    pub fn cbd(&self, input_bytes: &[u8], eta: u8) -> Result<Self, String> {
        let valid_byte_len: usize = (eta * 64).into();
        if valid_byte_len != input_bytes.len() {
            return Err(String::from("Invalid byte length"));
        }
        let mut coefficients = vec![BigUint::zero(); 256];
        let mut b_int = BigUint::from_bytes_le(input_bytes);
        let mask_1: usize = (1 << eta) - 1;
        let mask_2: usize = (1 << 2 * eta) - 1;
        for i in 0..256 {
            let x = b_int.clone() & BigUint::from(mask_2);
            let a = x.clone() & BigUint::from(mask_1);
            let one_bits_in_a: i128 = a.count_ones().into();
            let b = (x >> eta) & BigUint::from(mask_1);
            let one_bits_in_b: i128 = b.count_ones().into();
            b_int >>= 2 * eta;
            let value = (((one_bits_in_a - one_bits_in_b) % 3329) + 3329) % 3329;
            coefficients[i] = BigUint::from(value as u128);
        }
        Ok(Ring::new(&coefficients))
    }

    pub fn to_ntt(&self) -> Self {
        let mut k = 1;
        let mut l = 128;
        let mut coefficients = self.coefficients.clone();
        let zetas = &self.ntt_zetas;
        while l >= 2 {
            let mut start = 0;
            while start < 256 {
                let zeta = zetas[k];
                k = k + 1;
                for j in start..(start + l) {
                    let t = zeta * &coefficients[j + l];
                    let first = &coefficients[j] + self.q;
                    let second = t.clone() % self.q;
                    coefficients[j + l] = (first - second) % self.q;
                    coefficients[j] = (&coefficients[j] + t) % self.q;
                }
                start = start + (2 * l);
            }
            l = l >> 1;
        }
        Ring::new(&coefficients)
    }
}

impl Add for &Ring {
    type Output = Ring;

    fn add(self, rhs: Self) -> Self::Output {
        // TODO: Add checks
        let mut new_coeffs = vec![];
        for (x, y) in self.coefficients.iter().zip(rhs.coefficients.iter()) {
            new_coeffs.push(self.add_mod_q(x, y));
        }
        Ring::new(&new_coeffs)
    }
}

impl AddAssign for Ring {
    fn add_assign(&mut self, rhs: Self) {
        let new_ring = &*self + &rhs;
        *self = new_ring;
    }
}

impl Mul for &Ring {
    type Output = Ring;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut new_coeffs = vec![BigUint::zero(); self.n];
        let n = self.n;
        for i in 0..n {
            for j in 0..(n - i) {
                new_coeffs[i + j] += &self.coefficients[i] * &rhs.coefficients[j];
                new_coeffs[i + j] %= self.q
            }
        }
        for j in 1..n {
            for i in (n - j)..n {
                new_coeffs[i + j - n] += self.q;
                new_coeffs[i + j - n] -= (&self.coefficients[i] * &rhs.coefficients[j]) % self.q;
                new_coeffs[i + j - n] %= self.q
            }
        }
        Ring::new(&new_coeffs)
    }
}

impl Debug for Ring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, value) in self.coefficients.iter().enumerate() {
            if !value.is_zero() {
                if i == 255 {
                    write!(f, "{}x^{}", value, i)?;
                } else if i == 0 {
                    write!(f, "{} + ", value)?;
                } else if i == 1 {
                    write!(f, "{}x + ", value)?;
                } else {
                    write!(f, "{}x^{} + ", value, i)?;
                }
            }
        }
        write!(f, "")
    }
}
