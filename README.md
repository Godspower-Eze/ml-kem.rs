![Project Status](https://img.shields.io/badge/status-in--development-yellow)

## ml-kem.rs

A rust library for ML-KEM(Module-Lattice-Based Key-Encryption Mechanism)

### 🚧 Project Status

**This project is currently under active development.**  Expect breaking changes and incomplete features.

### Features

- [x] `keygen()` - generate a key pair `(ek, dk)`
- [ ] `key_derive(seed)` - generate a key pair from a provided seed
- [ ] `encaps(ek)` - generate a key and ciphertext pair `(key, ciphertext)`
- [ ] `decaps(dk, ciphertext)`- generate the shared key `key`

### License

This project is licensed under the MIT license.

See [LICENSE](/LICENSE) for more information.
