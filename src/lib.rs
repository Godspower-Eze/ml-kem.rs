mod matrix;
mod modules;
mod ring;

use std::vec;

use matrix::Matrix;
use modules::Module;
use rand::rngs::OsRng;
use rand::RngCore;
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Digest, Sha3_256, Sha3_512, Shake128, Shake256,
};

use ring::KyberRing;

enum TYPE {
    ML_KEM_512,
    ML_KEM_768,
    ML_KEM_1024,
}

struct MLKEM {
    k: u8,
    eta_1: u8,
    eta_2: u8,
    du: u8,
    dv: u8,
    ring: KyberRing,
}

impl MLKEM {
    fn new(type_of: TYPE) -> Self {
        let ring = KyberRing::new(3329, 256);
        match type_of {
            TYPE::ML_KEM_512 => MLKEM {
                k: 2,
                eta_1: 3,
                eta_2: 2,
                du: 10,
                dv: 4,
                ring,
            },
            TYPE::ML_KEM_768 => MLKEM {
                k: 3,
                eta_1: 2,
                eta_2: 2,
                du: 10,
                dv: 4,
                ring,
            },
            TYPE::ML_KEM_1024 => MLKEM {
                k: 2,
                eta_1: 2,
                eta_2: 2,
                du: 11,
                dv: 5,
                ring,
            },
        }
    }

    fn keygen(&self) -> (String, String) {
        let d = Self::random_bytes(32);
        let z = Self::random_bytes(32);

        // let (ek, dk) = self._keygen_internal(&d, &z);
        todo!()
    }

    fn key_derive(&self, seed: String) -> (String, String) {
        todo!()
    }

    fn encaps(&self, ek: String) -> (String, String) {
        todo!()
    }

    fn decaps(&self, dk: String, c: String) -> String {
        todo!()
    }

    // fn _keygen_internal(&self, d: &[u8], z: &[u8]) -> (&[u8], &[u8]) {}

    fn _k_pke_keygen(&self, d: &[u8]) -> (String, String) {
        let pre_image: Vec<u8> = [d, &[self.k]].concat();

        let (rho, sigma) = Self::_G(&pre_image);

        let a_hat = self._generate_matrix_from_seed(&rho, false);

        let n = 0;

        todo!()
    }

    fn random_bytes(length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        bytes
    }

    fn _G(s: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut hasher = Sha3_512::new();
        Update::update(&mut hasher, s);
        let result = hasher.finalize();
        return (result[..32].to_vec(), result[32..].to_vec());
    }

    fn _H(s: &[u8]) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        Update::update(&mut hasher, s);
        let result = hasher.finalize();
        return result.to_vec();
    }

    fn _xof(b: &[u8], i: u8, j: u8) -> Vec<u8> {
        // TODO: Add checks
        let mut hasher = Shake128::default();
        let pre_image: Vec<u8> = [b, &[i], &[j]].concat();
        hasher.update(&pre_image);

        let mut reader = hasher.finalize_xof();
        let mut buf = [0u8; 840];
        reader.read(&mut buf);

        buf.to_vec()
    }

    fn _prf(eta: u8, s: &[u8], b: u8) -> Vec<u8> {
        // TODO: Add checks
        let mut hasher = Shake256::default();
        let pre_image: Vec<u8> = [s, &[b]].concat();
        hasher.update(&pre_image);

        let mut reader = hasher.finalize_xof();
        let mut buf: Vec<u8> = vec![0u8; (eta * 64).into()];
        reader.read(&mut buf);

        buf.to_vec()
    }

    fn _generate_matrix_from_seed(&self, rho: &[u8], transpose: bool) -> Matrix {
        let k: usize = self.k.into();
        let mut a_data = vec![vec![KyberRing::default(); k]; k];
        for i in 0..k {
            for j in 0..k {
                let xof_bytes = Self::_xof(rho, i.try_into().unwrap(), j.try_into().unwrap());
                a_data[i][j] = self.ring.ntt_sample(&xof_bytes);
            }
        }
        Matrix::new(&a_data, transpose)
    }

    fn _generate_error_vector(&self, sigma: &[u8], eta: u8, n: u8) -> (Matrix, u8) {
        let k: usize = self.k.into();
        let elements = vec![0; k];
        for i in 0..k {
            let prf_output = Self::_prf(eta, sigma, n);
        }
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_bytes() {
        let bytes = MLKEM::random_bytes(32);
    }

    #[test]
    fn _G() {
        let bytes = MLKEM::random_bytes(32);
        let hash = MLKEM::_H(&bytes);
        println!("{:?}", hash);
    }
}
