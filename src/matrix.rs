use std::ops::Add;

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

    fn dim(&self) -> (usize, usize) {
        if self.transpose {
            return (self.data[0].len(), self.data.len());
        } else {
            return (self.data.len(), self.data[0].len());
        }
    }

    pub fn to_ntt(&self) -> Self {
        let mut data = vec![];
        for row in self.data.iter() {
            let mut new_row = vec![];
            for element in row {
                new_row.push(element.to_ntt());
            }
            data.push(new_row);
        }
        Matrix::new(&data, self.transpose)
    }
}

impl Add for &Matrix {
    type Output = Matrix;

    fn add(self, rhs: Self) -> Self::Output {
        // TODO: Add checks
        let (m, n) = self.dim();
        let new_data = vec![vec![KyberRing::default(); n]; m];
        todo!()
    }
}
