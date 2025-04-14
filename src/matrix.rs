use crate::ring::KyberRing;

pub struct Matrix {
    data: Vec<Vec<KyberRing>>,
    transpose: bool,
}

impl Matrix {
    pub fn new(data: &Vec<Vec<KyberRing>>, transpose: bool) -> Self {
        Self {
            data: data.to_vec(),
            transpose,
        }
    }
}
