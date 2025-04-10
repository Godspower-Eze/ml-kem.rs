mod ring;

use rand::rngs::OsRng;
use rand::RngCore;
use sha3::{Digest, Sha3_256, Sha3_512};

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
}

impl MLKEM {
    fn new(type_of: TYPE) -> Self {
        match type_of {
            TYPE::ML_KEM_512 => MLKEM {
                k: 2,
                eta_1: 3,
                eta_2: 2,
                du: 10,
                dv: 4,
            },
            TYPE::ML_KEM_768 => MLKEM {
                k: 3,
                eta_1: 2,
                eta_2: 2,
                du: 10,
                dv: 4,
            },
            TYPE::ML_KEM_1024 => MLKEM {
                k: 2,
                eta_1: 2,
                eta_2: 2,
                du: 11,
                dv: 5,
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

        todo!()
    }

    fn random_bytes(length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        bytes
    }

    fn _G(s: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut hasher = Sha3_512::new();
        hasher.update(s);
        let result = hasher.finalize();
        return (result[..32].to_vec(), result[32..].to_vec());
    }

    fn _H(s: &[u8]) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(s);
        let result = hasher.finalize();
        return result.to_vec();
    }

    fn _generate_matrix_from_seed(&self, rho: &[u8], transpose: bool) {}
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
