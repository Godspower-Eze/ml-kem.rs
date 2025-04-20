use std::ops::Add;

use crate::ring::Ring;

#[derive(Clone)]
pub struct Module {
    data: Vec<Vec<Ring>>,
    transpose: bool,
}

impl Module {
    pub fn new(data: &Vec<Vec<Ring>>, transpose: bool) -> Self {
        Self {
            data: data.to_vec(),
            transpose,
        }
    }

    pub fn mat_mul(&self, rhs: &Self) -> Result<Self, String> {
        // TODO: Add checks
        let (m_1, n_1) = self.dim();
        let (m_2, n_2) = rhs.dim();
        if n_1 != m_2 {
            return Err(String::from("Invalid dimensions"));
        }
        for i in 0..m_1 {
            for j in 0..n_2 {
                let mut summation = vec![];
                for k in 0..n_1 {
                    let a = &self.data[i][k];
                    let b = &rhs.data[k][j];
                    let mul = a * b;
                    summation.push(mul);
                }
            }
        }
        todo!()
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
        Module::new(&data, self.transpose)
    }
}

impl Add for &Module {
    type Output = Module;

    fn add(self, rhs: Self) -> Self::Output {
        // TODO: Add checks
        let (m, n) = self.dim();
        let mut new_data = vec![vec![Ring::default(); n]; m];
        for i in 0..m {
            let mut new_row = vec![];
            for j in 0..n {
                let new_element = &self.data[i][j] + &rhs.data[i][j];
                new_row.push(new_element);
            }
            new_data.push(new_row);
        }
        Module::new(&new_data, false)
    }
}
