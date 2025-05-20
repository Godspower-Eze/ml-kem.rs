![Project Status](https://img.shields.io/badge/status-in--development-yellow)

## pqc-ml-kem.rs

A rust library for ML-KEM(Module-Lattice-Based Key-Encapsulation Mechanism)

### 🚧 Project Status

**This project is currently under active development.**  Expect breaking changes and incomplete features.

### Features

- [x] `keygen()` - generate a key pair `(ek, dk)`
- [x] `encaps(ek)` - generate a key and ciphertext pair `(key, ciphertext)`
- [x] `decaps(dk, ciphertext)`- generate the shared key `key`

### How to Use

To install, use:

```bash
cargo add pqc-ml-kem
```

To use

```rust
use pqc_ml_kem::{ML_KEM_512, ML_KEM_768, ML_KEM_1024};

let (ek, dk) = ML_KEM_512.keygen();

let (key_1, ct) = ML_KEM_512.encaps(&ek);

let key_2 = ML_KEM_512.decaps(&dk, &ct);

assert_eq(key_1, key_2)
```

### Acknowledgements

- [kyber.py](https://github.com/GiacomoPope/kyber-py) for serving as an initial reference.

### License

This project is licensed under the MIT license.

See [LICENSE](/LICENSE) for more information.
