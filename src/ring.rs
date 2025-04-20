use num_bigint::BigUint;
use std::ops::{Add, Mul};

#[derive(Default, Clone)]
pub struct Ring {
    q: usize,
    n: usize,
    coefficients: Vec<usize>,
    root_of_unity: u8,
    ntt_zetas: Vec<usize>,
    ntt_f: usize,
}

impl Ring {
    pub fn new(coefficients: &Vec<usize>) -> Self {
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

    fn is_zero(&self) -> bool {
        // Return if the polynomial ring is zero: f = 0
        self.coefficients.iter().all(|x| *x == 0)
    }

    fn add_mod_q(&self, x: usize, y: usize) -> usize {
        (x + y) % self.q
    }

    fn is_constant(&self) -> bool {
        // Return if the polynomial ring is constant: f = c
        //
        // Note: For simplicity, we don't consider zero polynomial a constant. That is [0] or [0, 0] is not a constant polynomial ring
        if self.coefficients.is_empty() {
            return false;
        }
        self.coefficients[1..].iter().all(|x| *x == 0) && !self.coefficients[0] == 0
    }

    pub fn ntt_sample(&self, input_bytes: &[u8]) -> Self {
        let mut i = 0;
        let mut j = 0;
        let mut coefficients = vec![0; self.n];
        while j < self.n {
            let a: usize = input_bytes[i].into();
            let b: usize = input_bytes[i + 1].into();
            let c: usize = 256 * (b % 16);
            let d_1: usize = a + c;
            let d: usize = input_bytes[i + 2].into();
            let d_2 = (b / 16) + (16 * d);

            if d_1 < 3329 {
                coefficients[j] = d_1;
                j = j + 1
            }

            if d_2 < 3329 && j < self.n {
                coefficients[j] = d_2;
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
        let mut coefficients = vec![0_usize; 256];
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
            let value = (one_bits_in_a - one_bits_in_b) % 3329;
            coefficients[i] = value.try_into().unwrap();
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
                    let t = zeta * coefficients[j + l];
                    coefficients[j + l] = coefficients[j] - t;
                    coefficients[j] = coefficients[j] + t;
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
            new_coeffs.push(self.add_mod_q(*x, *y));
        }
        Ring::new(&new_coeffs)
    }
}

impl Mul for &Ring {
    type Output = Ring;

    fn mul(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn new() {
//         let q = BigInt::from(7);
//         let n = BigInt::from(256);
//         let coefficients = vec![BigInt::from(1), BigInt::from(2)];
//         let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
//         assert_eq!(poly_ring.q, q);
//         assert_eq!(poly_ring.n, n);
//         assert_eq!(poly_ring.coefficients, coefficients);
//     }

//     #[test]
//     fn is_zero() {
//         let q = BigInt::from(7);
//         let n = BigInt::from(256);
//         let coefficients = vec![BigInt::from(1), BigInt::from(2)];
//         let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
//         assert!(!poly_ring.is_zero());
//         let coefficients = vec![BigInt::zero(), BigInt::from(2)];
//         let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
//         assert!(!poly_ring.is_zero());
//         let coefficients = vec![BigInt::zero(), BigInt::zero()];
//         let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
//         assert!(poly_ring.is_zero());
//         let poly_ring = PolynomialRing::new(q, n, &vec![]);
//         assert!(poly_ring.is_zero());
//     }

//     #[test]
//     fn is_constant() {
//         let q = BigInt::from(7);
//         let n = BigInt::from(256);
//         let coefficients = vec![BigInt::from(1), BigInt::from(2)];
//         let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
//         assert!(!poly_ring.is_constant());
//         let coefficients = vec![BigInt::zero(), BigInt::from(2)];
//         let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
//         assert!(!poly_ring.is_constant());
//         let coefficients = vec![BigInt::zero(), BigInt::zero()];
//         let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
//         assert!(!poly_ring.is_constant());
//         let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &vec![]);
//         assert!(!poly_ring.is_constant());
//         let coefficients = vec![BigInt::one(), BigInt::zero()];
//         let poly_ring = PolynomialRing::new(q, n, &coefficients);
//         assert!(poly_ring.is_constant());
//     }
// }
