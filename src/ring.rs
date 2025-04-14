#[derive(Default, Clone)]
pub struct KyberRing {
    q: usize,
    n: usize,
    coefficients: Vec<usize>,
}

impl KyberRing {
    pub fn new(q: usize, n: usize) -> Self {
        KyberRing {
            q,
            n,
            coefficients: vec![],
        }
    }

    fn is_zero(&self) -> bool {
        // Return if the polynomial ring is zero: f = 0
        self.coefficients.iter().all(|x| *x == 0)
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
        Self {
            q: self.q,
            n: self.n,
            coefficients: coefficients,
        }
    }

    pub fn cbd(&self, input_bytes: &[u8], eta: u8, is_ntt: bool) -> Result<Self, String> {
        let valid_byte_len: usize = (eta * 64).into();
        if valid_byte_len != input_bytes.len() {
            return Err(String::from("Invalid byte length"));
        }
        let coefficients = [0; 256];
        let b_int = usize::from_le_bytes(input_bytes);
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
