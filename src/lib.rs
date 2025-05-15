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

enum Type {
    MlKem512,
    MlKem768,
    MlKem1024,
}

struct MLKem {
    k: u8,
    eta_1: u8,
    eta_2: u8,
    du: u8,
    dv: u8,
}

impl MLKem {
    pub fn new(type_of: Type) -> Self {
        match type_of {
            Type::MlKem512 => MLKem {
                k: 2,
                eta_1: 3,
                eta_2: 2,
                du: 10,
                dv: 4,
            },
            Type::MlKem768 => MLKem {
                k: 3,
                eta_1: 2,
                eta_2: 2,
                du: 10,
                dv: 4,
            },
            Type::MlKem1024 => MLKem {
                k: 4,
                eta_1: 2,
                eta_2: 2,
                du: 11,
                dv: 5,
            },
        }
    }

    pub fn keygen(&self) -> (Vec<u8>, Vec<u8>) {
        let d = Self::random_bytes(32);
        let z = Self::random_bytes(32);

        let (ek, dk) = self._keygen_internal(&d, &z);

        (ek, dk)
    }

    pub fn encaps(&self, ek: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let m = Self::random_bytes(32);
        let (k, c) = self._encaps_internal(ek, &m);
        (k, c)
    }

    pub fn decaps(&self, dk: &[u8], c: &[u8]) -> Vec<u8> {
        self._decaps_internal(dk, c).unwrap()
    }

    fn _decaps_internal(&self, dk: &[u8], c: &[u8]) -> Result<Vec<u8>, String> {
        if c.len() != 32 * (self.du * self.k + self.dv) as usize {
            return Err(String::from("ciphertext type check failed"));
        }
        if dk.len() != (768_usize * self.k as usize + 96) {
            return Err(String::from("decapsulation key type check failed"));
        }

        let dk_pke = &dk[0..(384_usize * self.k as usize)];
        let ek_pke = &dk[(384_usize * self.k as usize)..(768_usize * self.k as usize + 32)];
        let h = &dk[(768_usize * self.k as usize + 32)..(768_usize * self.k as usize + 64)];
        let z = &dk[(768_usize * self.k as usize + 64)..];

        if Self::_h(ek_pke) != h {
            return Err(String::from("hash check failed"));
        }

        let m_prime = self._k_pke_decrypt(dk_pke, c);

        let pre_image = [m_prime.clone(), h.to_vec()].concat();
        let (k_prime, r_prime) = Self::_g(&pre_image);
        let pre_image = [z, c].concat();
        let k_bar = Self::_j(&pre_image);

        let c_prime = self._k_pke_encrypt(ek_pke, &m_prime, &r_prime).unwrap();

        Ok(select_bytes(&k_bar, &k_prime, c == c_prime))
    }

    fn _encaps_internal(&self, ek: &[u8], m: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let pre_image = [m, &Self::_h(ek)].concat();
        let (k, r) = Self::_g(&pre_image);
        let c = self._k_pke_encrypt(ek, m, &r).unwrap();
        (k, c)
    }

    fn _k_pke_encrypt(&self, ek_pke: &[u8], m: &[u8], r: &[u8]) -> Result<Vec<u8>, String> {
        if ek_pke.len() != 384 * (self.k as usize) + 32 {
            return Err(String::from(
                "Type check failed, ek_pke has the wrong length",
            ));
        }
        let t_hat_bytes = &ek_pke[..ek_pke.len() - 32];
        let rho = &ek_pke[ek_pke.len() - 32..];
        let t_hat = Module::decode_vector(t_hat_bytes, self.k as usize, 12, true)?;

        if t_hat.encode(12) != t_hat_bytes {
            return Err(String::from(
                "Modulus check failed, t_hat does not encode correctly",
            ));
        }
        let a_hat_t = self._generate_matrix_from_seed(rho, true);
        let n = 0;
        let (y, n) = self._generate_error_vector(r, self.eta_1, n);
        let (e_1, n) = self._generate_error_vector(r, self.eta_2, n);
        let (e_2, _) = self._generate_polynomial(r, self.eta_2, n);

        let y_hat = y.to_ntt();

        let u = &((a_hat_t.mat_mul(&y_hat)?).from_ntt()) + &e_1;

        let mu = Ring::decode(m, 1, false)?.decompress(1);

        let v = &(t_hat.dot(&y_hat)?.from_ntt()) + &(&e_2 + &mu);

        let c_1 = u.compress(self.du).encode(self.du as usize);
        let c_2 = v.compress(self.dv).encode(self.dv as usize);

        Ok([c_1, c_2].concat())
    }

    fn _k_pke_decrypt(&self, dk_pke: &[u8], c: &[u8]) -> Vec<u8> {
        let n = self.k as usize * self.du as usize * 32;
        let c_1 = &c[..n];
        let c_2 = &c[n..];
        let u = Module::decode_vector(c_1, self.k as usize, self.du as usize, false)
            .unwrap()
            .decompress(self.du);
        let v = Ring::decode(c_2, self.dv as usize, false)
            .unwrap()
            .decompress(self.dv);
        let s_hat = Module::decode_vector(dk_pke, self.k as usize, 12, true).unwrap();

        let u_hat = u.to_ntt();
        let w = &v - &(s_hat.dot(&u_hat).unwrap()).from_ntt();

        w.compress(1).encode(1)
    }

    fn _keygen_internal(&self, d: &[u8], z: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let (ek_pke, dk_pke) = self._k_pke_keygen(d);

        let ek = ek_pke;
        let dk = [dk_pke, ek.clone(), Self::_h(&ek), z.to_vec()].concat();

        (ek, dk)
    }

    fn _k_pke_keygen(&self, d: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let pre_image: Vec<u8> = [d, &[self.k]].concat();

        let (rho, sigma) = Self::_g(&pre_image);

        let a_hat = self._generate_matrix_from_seed(&rho, false);

        let n = 0;

        let (s, n) = self._generate_error_vector(&sigma, self.eta_1, n);

        let (e, _) = self._generate_error_vector(&sigma, self.eta_1, n);

        let s_hat = s.to_ntt();

        let e_hat = e.to_ntt();

        let sa_hat = a_hat.mat_mul(&s_hat).unwrap();

        let t_hat = &sa_hat + &e_hat;

        let ek_pke = [t_hat.encode(12), rho].concat();

        let dk_pke = s_hat.encode(12);

        (ek_pke, dk_pke)
    }

    fn random_bytes(length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        bytes
    }

    fn _g(s: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut hasher = Sha3_512::new();
        Update::update(&mut hasher, s);
        let result = hasher.finalize();
        (result[..32].to_vec(), result[32..].to_vec())
    }

    fn _h(s: &[u8]) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        Update::update(&mut hasher, s);
        let result = hasher.finalize();
        result.to_vec()
    }

    fn _j(s: &[u8]) -> Vec<u8> {
        let mut hasher = Shake256::default();
        hasher.update(s);

        let mut reader = hasher.finalize_xof();
        let mut buf = [0u8; 32];
        reader.read(&mut buf);

        buf.to_vec()
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
                let xof_bytes = Self::_xof(rho, j.try_into().unwrap(), i.try_into().unwrap());
                a_data[i][j] = Ring::ntt_sample(&xof_bytes);
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
            elements[i] = Ring::cbd(&prf_output, eta, false).unwrap();
            n += 1;
        }
        let data = vec![elements];
        (Module::new(&data, true), n)
    }

    fn _generate_polynomial(&self, sigma: &[u8], eta: u8, n: u8) -> (Ring, u8) {
        let prf_output = Self::_prf(eta, sigma, n);
        let p = Ring::cbd(&prf_output, eta, false).unwrap();
        (p, n + 1)
    }
}

fn select_bytes(a: &[u8], b: &[u8], cond: bool) -> Vec<u8> {
    // TODO: Add checks
    let mut out = vec![0_u8; a.len()];
    let cw = if !cond { 0 } else { 255 };
    for i in 0..(a.len()) {
        out[i] = a[i] ^ (cw & (a[i] ^ b[i]))
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::fs;

    fn keygen_kat(_type: Type, index: usize) {
        let data =
            fs::read_to_string("assets/ML-KEM-keyGen-FIPS203/internalProjection.json").unwrap();
        let json: Value = serde_json::from_str(&data).unwrap();
        let tests = json["testGroups"][index]["tests"].as_array().unwrap();
        let ml_kem = MLKem::new(_type);
        for value in tests.iter() {
            let z = &value["z"];
            let d = &value["d"];
            let ek = &value["ek"];
            let dk = &value["dk"];

            let z_as_bytes = hex::decode(z.as_str().unwrap()).unwrap();
            let d_as_bytes = hex::decode(d.as_str().unwrap()).unwrap();

            let (actual_ek, actual_dk) = ml_kem._keygen_internal(&d_as_bytes, &z_as_bytes);

            let ek_as_bytes = hex::decode(ek.as_str().unwrap()).unwrap();
            let dk_as_bytes = hex::decode(dk.as_str().unwrap()).unwrap();

            assert_eq!(actual_ek, ek_as_bytes);
            assert_eq!(actual_dk, dk_as_bytes);
        }
    }

    fn encaps_kat(_type: Type, index: usize) {
        let data =
            fs::read_to_string("assets/ML-KEM-encapDecap-FIPS203/internalProjection.json").unwrap();
        let json: Value = serde_json::from_str(&data).unwrap();
        let tests = json["testGroups"][index]["tests"].as_array().unwrap();
        let ml_kem = MLKem::new(_type);
        for value in tests.iter() {
            let c = &value["c"];
            let k = &value["k"];
            let m = &value["m"];
            let ek = &value["ek"];
            let dk = &value["dk"];

            let ek_as_bytes = hex::decode(ek.as_str().unwrap()).unwrap();
            let m_as_bytes = hex::decode(m.as_str().unwrap()).unwrap();

            let (actual_k, actual_c) = ml_kem._encaps_internal(&ek_as_bytes, &m_as_bytes);

            let k_as_bytes = hex::decode(k.as_str().unwrap()).unwrap();
            let c_as_bytes = hex::decode(c.as_str().unwrap()).unwrap();

            assert_eq!(actual_k, k_as_bytes);
            assert_eq!(actual_c, c_as_bytes);

            let dk_as_bytes = hex::decode(dk.as_str().unwrap()).unwrap();

            let k_prime = ml_kem.decaps(&dk_as_bytes, &c_as_bytes);
            assert_eq!(k_prime, k_as_bytes);
        }
    }

    fn decaps_kat(_type: Type, index: usize) {
        let data =
            fs::read_to_string("assets/ML-KEM-encapDecap-FIPS203/internalProjection.json").unwrap();
        let json: Value = serde_json::from_str(&data).unwrap();
        let kat_data = json["testGroups"][3 + index]["tests"].as_array().unwrap();
        let dk = json["testGroups"][3 + index]["dk"].as_str().unwrap();
        let dk_as_bytes = hex::decode(dk).unwrap();
        let ml_kem = MLKem::new(_type);
        for value in kat_data.iter() {
            let c = &value["c"];
            let c_as_bytes = hex::decode(c.as_str().unwrap()).unwrap();
            let k = &value["k"];
            let k_as_bytes = hex::decode(k.as_str().unwrap()).unwrap();
            let k = ml_kem.decaps(&dk_as_bytes, &c_as_bytes);
            assert_eq!(k, k_as_bytes)
        }
    }

    #[test]
    fn test_keygen_using_kat() {
        keygen_kat(Type::MlKem512, 0);
        keygen_kat(Type::MlKem768, 1);
        keygen_kat(Type::MlKem1024, 2);
    }

    #[test]
    fn test_encaps_using_kat() {
        encaps_kat(Type::MlKem512, 0);
        encaps_kat(Type::MlKem768, 1);
        encaps_kat(Type::MlKem1024, 2);
    }

    #[test]
    fn test_decaps_using_kat() {
        decaps_kat(Type::MlKem512, 0);
        decaps_kat(Type::MlKem768, 1);
        decaps_kat(Type::MlKem1024, 2);
    }
}
