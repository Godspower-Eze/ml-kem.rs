use crate::matrix::Matrix;
use crate::ring::KyberRing;

pub struct Module {
    ring: KyberRing,
    matrix: Matrix,
}

impl Module {
    pub fn new(ring: KyberRing, matrix: Matrix) -> Self {
        Self { ring, matrix }
    }
}
