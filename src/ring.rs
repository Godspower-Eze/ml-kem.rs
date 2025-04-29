use num_bigint::BigUint;
use num_traits::{One, Zero};
use rand::Rng;
use std::{
    fmt::{write, Debug},
    ops::{Add, AddAssign, Mul},
};

#[derive(Default, Clone, PartialEq)]
pub struct Ring {
    q: usize,
    n: usize,
    coefficients: Vec<BigUint>,
    root_of_unity: u8,
    ntt_zetas: Vec<usize>,
    ntt_f: usize,
    is_ntt: bool,
}

impl Ring {
    pub fn new(coefficients: &Vec<BigUint>, is_ntt: bool) -> Self {
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
            is_ntt,
        }
    }

    pub fn zero() -> Self {
        let coefficients = vec![BigUint::zero(); 256];
        Self::new(&coefficients, false)
    }

    pub fn one() -> Self {
        let mut coefficients = vec![BigUint::one()];
        coefficients.resize(256, BigUint::zero());
        Self::new(&coefficients, false)
    }

    pub fn x() -> Self {
        let mut coefficients = vec![BigUint::zero(), BigUint::one()];
        coefficients.resize(256, BigUint::zero());
        Self::new(&coefficients, false)
    }

    fn add_mod_q(&self, x: &BigUint, y: &BigUint) -> BigUint {
        (x + y) % self.q
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let mut coefficients = vec![];
        for _ in 0..256 {
            let random_number: usize = rng.gen_range(0..=255);
            coefficients.push(BigUint::from(random_number));
        }
        Self::new(&coefficients, false)
    }

    pub fn encode(&self, d: usize) -> Vec<u8> {
        let mut t = BigUint::zero();
        for i in 0..255 {
            t |= &self.coefficients[256 - i - 1];
            t <<= d
        }
        t |= &self.coefficients[0];
        let mut encoding = t.to_bytes_le();
        encoding.resize(32 * d, 0);
        encoding
    }

    pub fn ntt_sample(input_bytes: &[u8]) -> Self {
        let mut i = 0;
        let mut j = 0;
        let mut coefficients = vec![BigUint::zero(); 256];
        while j < 256 {
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

            if d_2 < 3329 && j < 256 {
                coefficients[j] = BigUint::from(d_2);
                j = j + 1
            }

            i = i + 3;
        }
        Ring::new(&coefficients, true)
    }

    pub fn cbd(input_bytes: &[u8], eta: u8, is_ntt: bool) -> Result<Self, String> {
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
        Ok(Ring::new(&coefficients, is_ntt))
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
                start += 2 * l;
            }
            l = l >> 1;
        }
        Ring::new(&coefficients, true)
    }

    fn _ntt_base_mul(
        &self,
        a_0: &BigUint,
        a_1: &BigUint,
        b_0: &BigUint,
        b_1: &BigUint,
        zeta: usize,
    ) -> (BigUint, BigUint) {
        let r_0 = (a_0 * b_0 + zeta * a_1 * b_1) % self.q;
        let r_1 = (a_1 * b_0 + a_0 * b_1) % self.q;
        (r_0, r_1)
    }

    fn _ntt_coeff_mul(&self, f_coeffs: &Vec<BigUint>, g_coeffs: &Vec<BigUint>) -> Vec<BigUint> {
        let mut new_coeffs = vec![];
        for i in 0..64 {
            let (r_0, r_1) = self._ntt_base_mul(
                &f_coeffs[4 * i + 0],
                &f_coeffs[4 * i + 1],
                &g_coeffs[4 * i + 0],
                &g_coeffs[4 * i + 1],
                self.ntt_zetas[64 + i],
            );
            let zeta = -(self.ntt_zetas[64 + i] as i128);
            let zeta = zeta + self.q as i128;
            let (r_2, r_3) = self._ntt_base_mul(
                &f_coeffs[4 * i + 2],
                &f_coeffs[4 * i + 3],
                &g_coeffs[4 * i + 2],
                &g_coeffs[4 * i + 3],
                zeta as usize,
            );
            let values = vec![r_0, r_1, r_2, r_3];
            new_coeffs.extend(values);
        }
        new_coeffs
    }

    fn _ntt_mut(&self, rhs: &Self) -> Self {
        let new_coeffs = self._ntt_coeff_mul(&self.coefficients, &rhs.coefficients);
        Self::new(&new_coeffs, true)
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
        Ring::new(&new_coeffs, self.is_ntt)
    }
}

impl AddAssign for Ring {
    fn add_assign(&mut self, rhs: Self) {
        // TODO: Add checks
        let new_ring = &*self + &rhs;
        *self = new_ring;
    }
}

impl Mul for &Ring {
    type Output = Result<Ring, String>;

    fn mul(self, rhs: Self) -> Self::Output {
        // TODO: Add checks
        if self.is_ntt && rhs.is_ntt {
            Ok(self._ntt_mut(&rhs))
        } else if !self.is_ntt && !rhs.is_ntt {
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
                    new_coeffs[i + j - n] -=
                        (&self.coefficients[i] * &rhs.coefficients[j]) % self.q;
                    new_coeffs[i + j - n] %= self.q
                }
            }
            Ok(Ring::new(&new_coeffs, self.is_ntt))
        } else {
            return Err(String::from("Invalid rings"));
        }
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

#[cfg(test)]
mod tests {
    use super::Ring;

    #[test]
    #[ignore]
    fn add() {
        let zero = Ring::zero();

        for _ in 0..30 {
            let f_1 = Ring::random();
            let f_2 = Ring::random();
            let f_3 = Ring::random();

            assert_eq!(&f_1 + &zero, f_1);
            assert_eq!(&f_1 + &f_2, &f_2 + &f_1);
            assert_eq!(&f_1 + &(&f_2 + &f_3), &(&f_1 + &f_2) + &f_3);
            let mut f_4 = f_1.clone();
            f_4 += f_1.clone();
            assert_eq!(&f_1 + &f_1, f_4)
        }
    }

    #[test]
    #[ignore]
    fn multiplication() {
        let zero = Ring::zero();
        let one = Ring::one();

        for _ in 0..20 {
            let f_1 = Ring::random();
            let f_2 = Ring::random();
            let f_3 = Ring::random();

            assert_eq!((&f_1 * &zero).unwrap(), zero);
            assert_eq!((&f_1 * &one).unwrap(), f_1);
            assert_eq!(&f_1 * &f_2, &f_2 * &f_1);
            assert_eq!(
                &f_1 * &((&f_2 * &f_3).unwrap()),
                &((&f_1 * &f_2).unwrap()) * &f_3
            )
        }
    }
}
