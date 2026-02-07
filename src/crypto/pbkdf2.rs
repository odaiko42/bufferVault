// BufferVault - PBKDF2-HMAC-SHA256 et HMAC-SHA256 pure Rust
// Reference : RFC 2104 (HMAC), RFC 8018 (PBKDF2)
//
// Ce module implemente :
// - HMAC-SHA256 (RFC 2104) : code d'authentification de message
// - PBKDF2-HMAC-SHA256 (RFC 8018) : derivation de cle a partir d'un mot de passe
//
// # Utilisation dans BufferVault
// - hmac_sha256 : calcul du HMAC d'integrite du fichier vault
// - pbkdf2_hmac_sha256 : derive les cles de chiffrement si necessaire
//
// # Portabilite
// Ce module est en pur Rust, sans dependance Win32.

use crate::crypto::sha256::{Sha256, sha256};

const SHA256_BLOCK_SIZE: usize = 64;
const SHA256_OUTPUT_SIZE: usize = 32;

/// Calcule HMAC-SHA256(key, message).
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    // Si la cle est plus longue que le bloc, la hasher d'abord
    let key_block = if key.len() > SHA256_BLOCK_SIZE {
        let h = sha256(key);
        let mut kb = [0u8; SHA256_BLOCK_SIZE];
        kb[..SHA256_OUTPUT_SIZE].copy_from_slice(&h);
        kb
    } else {
        let mut kb = [0u8; SHA256_BLOCK_SIZE];
        kb[..key.len()].copy_from_slice(key);
        kb
    };

    // Pads ipad (0x36) et opad (0x5c)
    let mut ipad = [0x36u8; SHA256_BLOCK_SIZE];
    let mut opad = [0x5cu8; SHA256_BLOCK_SIZE];
    for i in 0..SHA256_BLOCK_SIZE {
        ipad[i] ^= key_block[i];
        opad[i] ^= key_block[i];
    }

    // inner = SHA256(ipad || message)
    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(message);
    let inner_hash = inner.finalize();

    // outer = SHA256(opad || inner_hash)
    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    outer.finalize()
}

/// Derive une cle via PBKDF2-HMAC-SHA256.
///
/// * `password` - Le mot de passe ou secret
/// * `salt` - Le sel (idealement 32 octets aleatoires)
/// * `iterations` - Nombre d'iterations (minimum recommande : 100 000)
/// * `key_len` - Longueur de la cle derivee en octets
pub fn pbkdf2_hmac_sha256(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    key_len: usize,
) -> Vec<u8> {
    let mut derived_key = Vec::with_capacity(key_len);
    let blocks_needed = (key_len + SHA256_OUTPUT_SIZE - 1) / SHA256_OUTPUT_SIZE;

    for block_idx in 1..=blocks_needed as u32 {
        // U_1 = PRF(password, salt || INT_32_BE(block_idx))
        let mut salt_block = Vec::with_capacity(salt.len() + 4);
        salt_block.extend_from_slice(salt);
        salt_block.extend_from_slice(&block_idx.to_be_bytes());

        let mut u_prev = hmac_sha256(password, &salt_block);
        let mut result = u_prev;

        // U_2 .. U_iterations
        for _ in 1..iterations {
            let u_next = hmac_sha256(password, &u_prev);
            for j in 0..SHA256_OUTPUT_SIZE {
                result[j] ^= u_next[j];
            }
            u_prev = u_next;
        }

        derived_key.extend_from_slice(&result);
    }

    derived_key.truncate(key_len);
    derived_key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_sha256_empty() {
        // RFC 4231 Test Case 1 (key = 0x0b * 20, data = "Hi There")
        let key = [0x0bu8; 20];
        let data = b"Hi There";
        let mac = hmac_sha256(&key, data);
        let expected = [
            0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53,
            0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b, 0xf1, 0x2b,
            0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7,
            0x26, 0xe9, 0x37, 0x6c, 0x2e, 0x32, 0xcf, 0xf7,
        ];
        assert_eq!(mac, expected);
    }

    #[test]
    fn test_hmac_sha256_rfc4231_case2() {
        // key = "Jefe", data = "what do ya want for nothing?"
        let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        let expected = [
            0x5b, 0xdc, 0xc1, 0x46, 0xbf, 0x60, 0x75, 0x4e,
            0x6a, 0x04, 0x24, 0x26, 0x08, 0x95, 0x75, 0xc7,
            0x5a, 0x00, 0x3f, 0x08, 0x9d, 0x27, 0x39, 0x83,
            0x9d, 0xec, 0x58, 0xb9, 0x64, 0xec, 0x38, 0x43,
        ];
        assert_eq!(mac, expected);
    }

    #[test]
    fn test_pbkdf2_basic() {
        // Verification de coherence : la cle derivee doit etre deterministe
        let key1 = pbkdf2_hmac_sha256(b"password", b"salt", 1, 32);
        let key2 = pbkdf2_hmac_sha256(b"password", b"salt", 1, 32);
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_pbkdf2_rfc6070_case1() {
        // RFC 6070 : P="password", S="salt", c=1, dkLen=20
        let dk = pbkdf2_hmac_sha256(b"password", b"salt", 1, 20);
        let expected = [
            0x12, 0x0f, 0xb6, 0xcf, 0xfc, 0xf8, 0xb3, 0x2c,
            0x43, 0xe7, 0x22, 0x52, 0x56, 0xc4, 0xf8, 0x37,
            0xa8, 0x65, 0x48, 0xc9,
        ];
        assert_eq!(dk, expected);
    }

    #[test]
    fn test_pbkdf2_rfc6070_case2() {
        // RFC 6070 : P="password", S="salt", c=2, dkLen=20
        let dk = pbkdf2_hmac_sha256(b"password", b"salt", 2, 20);
        let expected = [
            0xae, 0x4d, 0x0c, 0x95, 0xaf, 0x6b, 0x46, 0xd3,
            0x2d, 0x0a, 0xdf, 0xf9, 0x28, 0xf0, 0x6d, 0xd0,
            0x2a, 0x30, 0x3f, 0x8e,
        ];
        assert_eq!(dk, expected);
    }

    #[test]
    fn test_pbkdf2_different_passwords() {
        let k1 = pbkdf2_hmac_sha256(b"pass1", b"salt", 100, 32);
        let k2 = pbkdf2_hmac_sha256(b"pass2", b"salt", 100, 32);
        assert_ne!(k1, k2);
    }
}
