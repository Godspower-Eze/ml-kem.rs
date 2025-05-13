use std::{
    fmt::Debug,
    ops::{Add, Index},
};

use crate::ring::Ring;

#[derive(Clone, PartialEq)]
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
        let mut new_data = vec![vec![Ring::zero(); n_2]; m_1];
        for i in 0..m_1 {
            for j in 0..n_2 {
                for k in 0..n_1 {
                    new_data[i][j] += (&self[(i, k)] * &rhs[(k, j)]).unwrap();
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

    pub fn dot(&self, rhs: &Self) -> Result<Ring, String> {
        // TODO: Add checks
        let transposed = Module::new(&self.data, !self.transpose);
        let res = transposed.mat_mul(rhs).unwrap();
        if res.dim() != (1, 1) {
            return Err(String::from("Invalid response"));
        } else {
            return Ok(res[(0, 0)].clone());
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

    pub fn from_ntt(&self) -> Self {
        let mut data = vec![];
        for row in self.data.iter() {
            let mut new_row = vec![];
            for x in row {
                new_row.push(x.from_ntt());
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

    pub fn decode_vector(
        input_bytes: &[u8],
        k: usize,
        d: usize,
        is_ntt: bool,
    ) -> Result<Self, String> {
        if (256 * d * k) != input_bytes.len() * 8 {
            return Err(String::from(
                "Byte length is the wrong length for given k, d values",
            ));
        }
        let n = 32 * d;
        let mut data = vec![];
        for i in (0..input_bytes.len()).step_by(n) {
            let ring = Ring::decode(&input_bytes[i..(i + n)], d, is_ntt).unwrap();
            data.push(ring);
        }
        Ok(Module::new(&vec![data], true))
    }

    pub fn compress(&self, d: u8) -> Self {
        let mut new_data = vec![];
        for row in self.data.iter() {
            let mut new_row = vec![];
            for ele in row {
                let new_ele = ele.compress(d);
                new_row.push(new_ele);
            }
            new_data.push(new_row);
        }
        Module::new(&new_data, self.transpose)
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
    use crate::ring::Ring;

    use super::Module;

    #[test]
    #[ignore]
    fn mat_mul() {
        // Square
        let zero = Ring::zero();
        let one = Ring::one();
        let zero_module = Module::new(
            &vec![
                vec![zero.clone(), zero.clone()],
                vec![zero.clone(), zero.clone()],
            ],
            false,
        );
        let identity_module = Module::new(
            &vec![vec![one.clone(), zero.clone()], vec![zero.clone(), one]],
            false,
        );
        for _ in 0..10 {
            let a = Module::random(2, 2);
            let b = Module::random(2, 2);
            let c = Module::random(2, 2);
            let random_ring = Ring::random();
            let d = Module::new(
                &vec![
                    vec![random_ring.clone(), zero.clone()],
                    vec![zero.clone(), random_ring],
                ],
                false,
            );
            assert_eq!(a.mat_mul(&zero_module).unwrap(), zero_module);
            assert_eq!(a.mat_mul(&identity_module).unwrap(), a);
            assert_eq!(a.mat_mul(&d), d.mat_mul(&a));
            assert_eq!(
                a.mat_mul(&(&b + &c)).unwrap(),
                &(a.mat_mul(&b).unwrap()) + &(a.mat_mul(&c).unwrap())
            )
        }

        // Rectangle
        for _ in 0..10 {
            let a = Module::random(11, 4);
            let b = Module::random(4, 3);
            let c = Module::random(4, 3);

            assert_eq!(
                a.mat_mul(&(&b + &c)).unwrap(),
                &(a.mat_mul(&b).unwrap()) + &(a.mat_mul(&c).unwrap())
            );
        }
    }
}
