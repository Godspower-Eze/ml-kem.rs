mod module;
mod ring;

use std::vec;

use module::Module;
use rand::rngs::OsRng;
use rand::RngCore;
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Digest, Sha3_256, Sha3_512, Shake128, Shake256,
};

use ring::Ring;

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
    ring: Ring,
}

impl MLKEM {
    fn new(type_of: TYPE) -> Self {
        let ring = Ring::new(&vec![]);
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

    fn keygen(&self) -> (Vec<u8>, Vec<u8>) {
        let d = Self::random_bytes(32);
        let z = Self::random_bytes(32);

        let (ek, dk) = self._keygen_internal(&d, &z);

        (ek, dk)
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

    fn _keygen_internal(&self, d: &[u8], z: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let (ek_pke, dk_pke) = self._k_pke_keygen(d);

        let ek = ek_pke;
        let dk = [dk_pke, ek.clone(), Self::_H(&ek), z.to_vec()].concat();

        (ek, dk)
    }

    fn _k_pke_keygen(&self, d: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let pre_image: Vec<u8> = [d, &[self.k]].concat();

        let (rho, sigma) = Self::_G(&pre_image);

        let a_hat = self._generate_matrix_from_seed(&rho, false);

        let n = 0;

        let (s, N) = self._generate_error_vector(&sigma, self.eta_1, n);

        let (e, N) = self._generate_error_vector(&sigma, self.eta_1, N);

        let s_hat = s.to_ntt();

        let e_hat = e.to_ntt();

        let se_hat = &s_hat + &e_hat;

        let t_hat = a_hat.mat_mul(&se_hat).unwrap();

        let ek_pke = [t_hat.encode(12), rho].concat();

        let dk_pke = s_hat.encode(12);

        (ek_pke, dk_pke)
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

    fn _generate_matrix_from_seed(&self, rho: &[u8], transpose: bool) -> Module {
        let k: usize = self.k.into();
        let mut a_data = vec![vec![Ring::default(); k]; k];
        for i in 0..k {
            for j in 0..k {
                let xof_bytes = Self::_xof(rho, i.try_into().unwrap(), j.try_into().unwrap());
                a_data[i][j] = self.ring.ntt_sample(&xof_bytes);
            }
        }
        Module::new(&a_data, transpose)
    }

    fn _generate_error_vector(&self, sigma: &[u8], eta: u8, n: u8) -> (Module, u8) {
        let k: usize = self.k.into();
        let mut elements = vec![Ring::default(); k];
        let mut n = n;
        for i in 0..k {
            let prf_output = Self::_prf(eta, sigma, n);
            elements[i] = self.ring.cbd(&prf_output, eta).unwrap();
            n += 1;
        }
        let data = vec![elements];
        (Module::new(&data, true), n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::fs;

    fn keygen_kat(_type: TYPE) {
        let data =
            fs::read_to_string("assets/ML-KEM-keyGen-FIPS203/internalProjection.json").unwrap();
        let json: Value = serde_json::from_str(&data).unwrap();
        match _type {
            TYPE::ML_KEM_512 => {
                let tests = json["testGroups"][0]["tests"].as_array().unwrap();
                for value in tests {
                    let z = &value["z"];
                    let d = &value["d"];
                    let ek = &value["ek"];
                    let dk = &value["dk"];

                    let z_as_bytes = hex::decode(z.as_str().unwrap()).unwrap();
                    let d_as_bytes = hex::decode(d.as_str().unwrap()).unwrap();

                    let ml_kem_512 = MLKEM::new(TYPE::ML_KEM_512);

                    let (ek, dk) = ml_kem_512._keygen_internal(&d_as_bytes, &z_as_bytes);

                    println!("{:?} {:?}", ek, dk);
                }
            }
            TYPE::ML_KEM_768 => {
                let tests = &json["testGroups"][1]["tests"];
            }
            TYPE::ML_KEM_1024 => {
                let tests = &json["testGroups"][2]["tests"];
            }
        }
        todo!()
    }

    #[test]
    fn test_keygen_using_kat() {
        keygen_kat(TYPE::ML_KEM_512)
    }
}
