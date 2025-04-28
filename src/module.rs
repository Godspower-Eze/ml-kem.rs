use std::{
    fmt::Debug,
    ops::{Add, Index},
};

use num_bigint::BigUint;
use num_traits::Zero;

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

    pub fn random(m: usize, n: usize) -> Self {
        let mut data = vec![];
        for _ in 0..m {
            let mut row = vec![];
            for _ in 0..n {
                row.push(Ring::random());
            }
            data.push(row);
        }
        Self::new(&data, false)
    }

    pub fn mat_mul(&self, rhs: &Self) -> Result<Self, String> {
        // TODO: Add checks
        let (m_1, n_1) = self.dim();
        let (m_2, n_2) = rhs.dim();
        if n_1 != m_2 {
            return Err(String::from("Invalid dimensions"));
        }
        let mut new_data = vec![vec![Ring::new(&vec![BigUint::zero(); 256]); n_2]; m_1];
        for i in 0..m_1 {
            for j in 0..n_2 {
                for k in 0..n_1 {
                    new_data[i][j] += &self[(i, k)] * &rhs[(k, j)];
                }
            }
        }
        Ok(Module::new(&new_data, false))
    }

    pub fn dim(&self) -> (usize, usize) {
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

    pub fn encode(&self, d: usize) -> Vec<u8> {
        let mut output = vec![];
        for row in self.data.iter() {
            for element in row {
                let bytes = element.encode(d);
                output = [output, bytes].concat()
            }
        }
        output
    }

    pub fn transpose(&self) -> bool {
        self.transpose
    }
}

impl Add for &Module {
    type Output = Module;

    fn add(self, rhs: Self) -> Self::Output {
        // TODO: Add checks
        let (m, n) = self.dim();
        let mut new_data = vec![];
        for i in 0..m {
            let mut new_row = vec![];
            for j in 0..n {
                let new_element = &self[(i, j)] + &rhs[(i, j)];
                new_row.push(new_element);
            }
            new_data.push(new_row);
        }
        Module::new(&new_data, false)
    }
}

impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for row in self.data.iter() {
            write!(f, "{:?}", row)?;
        }
        write!(f, "]")
    }
}

impl Index<(usize, usize)> for Module {
    type Output = Ring;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        if self.transpose {
            &self.data[index.1][index.0]
        } else {
            &self.data[index.0][index.1]
        }
    }
}

mod tests {

    #[test]
    fn multiplication() {}
}
