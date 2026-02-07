// BufferVault - Module crypto
// Chiffrement AES-256-GCM, PBKDF2, DPAPI, buffer securise
//
// Ce module regroupe toutes les primitives cryptographiques de BufferVault,
// implementees en pur Rust (aucune dependance externe).
//
// # Sous-modules
// - `aes_gcm`    : chiffrement/dechiffrement AES-256-GCM avec authentification
// - `ghash`      : multiplication GF(2^128) pour le mode GCM
// - `sha256`     : implementation complete de SHA-256 (FIPS 180-4)
// - `pbkdf2`     : derivation de cle PBKDF2-HMAC-SHA256 (RFC 8018)
// - `dpapi`      : protection de la cle maitre via Windows DPAPI
// - `secure_buf` : buffer memoire securise avec effacement a la liberation
//
// # Securite
// - Les buffers sensibles sont zeroes via ecriture volatile avant liberation
// - Les comparaisons de tags/HMAC sont en temps constant
// - Le CSPRNG utilise BCryptGenRandom (Windows CSPRNG)
// - La cle maitre est protegee par DPAPI (credential store Windows)

/// Chiffrement et dechiffrement AES-256-GCM avec authentification.
pub mod aes_gcm;
/// Protection de la cle maitre via Windows DPAPI (CryptProtectData).
pub mod dpapi;
/// Multiplication GF(2^128) pour le mode Galois/Counter (GCM).
pub mod ghash;
/// Derivation de cle PBKDF2-HMAC-SHA256 conforme RFC 8018.
pub mod pbkdf2;
/// Buffer memoire securise avec effacement automatique (zeroing).
pub mod secure_buf;
/// Implementation SHA-256 conforme FIPS 180-4.
pub mod sha256;
