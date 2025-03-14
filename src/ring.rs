use num_bigint::BigInt;
use num_traits::{One, Zero};

struct PolynomialRing {
    q: BigInt,
    n: BigInt,
    coefficients: Vec<BigInt>,
}

impl PolynomialRing {
    fn new(q: BigInt, n: BigInt, coefficients: &Vec<BigInt>) -> Self {
        PolynomialRing {
            q,
            n,
            coefficients: coefficients.to_vec(),
        }
    }

    fn is_zero(&self) -> bool {
        // Return if the polynomial ring is zero: f = 0
        self.coefficients.iter().all(|x| x.is_zero())
    }

    fn is_constant(&self) -> bool {
        // Return if the polynomial ring is constant: f = c
        //
        // Note: For simplicity, we don't consider zero polynomial a constant. That is [0] or [0, 0] is not a constant polynomial ring
        if self.coefficients.is_empty() {
            return false;
        }
        self.coefficients[1..].iter().all(|x| x.is_zero()) && !self.coefficients[0].is_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let q = BigInt::from(7);
        let n = BigInt::from(256);
        let coefficients = vec![BigInt::from(1), BigInt::from(2)];
        let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
        assert_eq!(poly_ring.q, q);
        assert_eq!(poly_ring.n, n);
        assert_eq!(poly_ring.coefficients, coefficients);
    }

    #[test]
    fn is_zero() {
        let q = BigInt::from(7);
        let n = BigInt::from(256);
        let coefficients = vec![BigInt::from(1), BigInt::from(2)];
        let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
        assert!(!poly_ring.is_zero());
        let coefficients = vec![BigInt::zero(), BigInt::from(2)];
        let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
        assert!(!poly_ring.is_zero());
        let coefficients = vec![BigInt::zero(), BigInt::zero()];
        let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
        assert!(poly_ring.is_zero());
        let poly_ring = PolynomialRing::new(q, n, &vec![]);
        assert!(poly_ring.is_zero());
    }

    #[test]
    fn is_constant() {
        let q = BigInt::from(7);
        let n = BigInt::from(256);
        let coefficients = vec![BigInt::from(1), BigInt::from(2)];
        let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
        assert!(!poly_ring.is_constant());
        let coefficients = vec![BigInt::zero(), BigInt::from(2)];
        let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
        assert!(!poly_ring.is_constant());
        let coefficients = vec![BigInt::zero(), BigInt::zero()];
        let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &coefficients);
        assert!(!poly_ring.is_constant());
        let poly_ring = PolynomialRing::new(q.clone(), n.clone(), &vec![]);
        assert!(!poly_ring.is_constant());
        let coefficients = vec![BigInt::one(), BigInt::zero()];
        let poly_ring = PolynomialRing::new(q, n, &coefficients);
        assert!(poly_ring.is_constant());
    }
}
