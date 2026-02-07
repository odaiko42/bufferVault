// BufferVault - Implementation AES-256-GCM pure Rust
// Reference : FIPS 197 (AES), NIST SP 800-38D (GCM)
//
// Ce module implemente le chiffrement/dechiffrement AES-256 en mode
// Galois/Counter (GCM) avec authentification integree.
//
// # Architecture
// - Key expansion AES-256 (14 rounds, 240 octets de cles)
// - Chiffrement CTR (compteur 32 bits, blocs de 16 octets)
// - Authentification GHASH (multiplication GF(2^128))
// - Tag de 128 bits pour verification d'integrite
//
// # Securite
// - Comparaison du tag en temps constant (constant_time_compare)
// - Le nonce DOIT etre unique pour chaque chiffrement avec la meme cle
// - Utilise la S-Box standard AES (pas de table T pour eviter les
//   attaques par cache timing, au prix de performances reduites)
//
// # Portabilite
// Ce module est en pur Rust, sans dependance Win32.

use crate::constants::{AES_BLOCK_SIZE, AES_GCM_NONCE_SIZE, AES_GCM_TAG_SIZE, AES_KEY_SIZE};
use crate::crypto::ghash::{GfElement, ghash};
use crate::error::{BvError, BvResult};

// --- AES S-Box ---
const SBOX: [u8; 256] = [
    0x63,0x7c,0x77,0x7b,0xf2,0x6b,0x6f,0xc5,0x30,0x01,0x67,0x2b,0xfe,0xd7,0xab,0x76,
    0xca,0x82,0xc9,0x7d,0xfa,0x59,0x47,0xf0,0xad,0xd4,0xa2,0xaf,0x9c,0xa4,0x72,0xc0,
    0xb7,0xfd,0x93,0x26,0x36,0x3f,0xf7,0xcc,0x34,0xa5,0xe5,0xf1,0x71,0xd8,0x31,0x15,
    0x04,0xc7,0x23,0xc3,0x18,0x96,0x05,0x9a,0x07,0x12,0x80,0xe2,0xeb,0x27,0xb2,0x75,
    0x09,0x83,0x2c,0x1a,0x1b,0x6e,0x5a,0xa0,0x52,0x3b,0xd6,0xb3,0x29,0xe3,0x2f,0x84,
    0x53,0xd1,0x00,0xed,0x20,0xfc,0xb1,0x5b,0x6a,0xcb,0xbe,0x39,0x4a,0x4c,0x58,0xcf,
    0xd0,0xef,0xaa,0xfb,0x43,0x4d,0x33,0x85,0x45,0xf9,0x02,0x7f,0x50,0x3c,0x9f,0xa8,
    0x51,0xa3,0x40,0x8f,0x92,0x9d,0x38,0xf5,0xbc,0xb6,0xda,0x21,0x10,0xff,0xf3,0xd2,
    0xcd,0x0c,0x13,0xec,0x5f,0x97,0x44,0x17,0xc4,0xa7,0x7e,0x3d,0x64,0x5d,0x19,0x73,
    0x60,0x81,0x4f,0xdc,0x22,0x2a,0x90,0x88,0x46,0xee,0xb8,0x14,0xde,0x5e,0x0b,0xdb,
    0xe0,0x32,0x3a,0x0a,0x49,0x06,0x24,0x5c,0xc2,0xd3,0xac,0x62,0x91,0x95,0xe4,0x79,
    0xe7,0xc8,0x37,0x6d,0x8d,0xd5,0x4e,0xa9,0x6c,0x56,0xf4,0xea,0x65,0x7a,0xae,0x08,
    0xba,0x78,0x25,0x2e,0x1c,0xa6,0xb4,0xc6,0xe8,0xdd,0x74,0x1f,0x4b,0xbd,0x8b,0x8a,
    0x70,0x3e,0xb5,0x66,0x48,0x03,0xf6,0x0e,0x61,0x35,0x57,0xb9,0x86,0xc1,0x1d,0x9e,
    0xe1,0xf8,0x98,0x11,0x69,0xd9,0x8e,0x94,0x9b,0x1e,0x87,0xe9,0xce,0x55,0x28,0xdf,
    0x8c,0xa1,0x89,0x0d,0xbf,0xe6,0x42,0x68,0x41,0x99,0x2d,0x0f,0xb0,0x54,0xbb,0x16,
];

/// Constantes de round Rcon pour AES key expansion.
const RCON: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

/// Nombre de rounds AES-256.
const NR: usize = 14;

/// Nombre de mots de 32 bits dans la cle AES-256.
const NK: usize = 8;

/// Cles de round expandues : 15 blocs de 16 octets = 240 octets.
type RoundKeys = [[u8; 16]; NR + 1];

/// Expansion de la cle AES-256 en cles de round.
fn key_expansion(key: &[u8; AES_KEY_SIZE]) -> RoundKeys {
    let mut w = [0u32; 4 * (NR + 1)];

    // Copier la cle dans les premiers NK mots
    for i in 0..NK {
        w[i] = u32::from_be_bytes([key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3]]);
    }

    for i in NK..w.len() {
        let mut temp = w[i - 1];
        if i % NK == 0 {
            // RotWord + SubWord + Rcon
            temp = sub_word(rot_word(temp)) ^ ((RCON[i / NK - 1] as u32) << 24);
        } else if i % NK == 4 {
            temp = sub_word(temp);
        }
        w[i] = w[i - NK] ^ temp;
    }

    let mut round_keys: RoundKeys = [[0u8; 16]; NR + 1];
    for i in 0..=NR {
        for j in 0..4 {
            let bytes = w[i * 4 + j].to_be_bytes();
            round_keys[i][j * 4..j * 4 + 4].copy_from_slice(&bytes);
        }
    }
    round_keys
}

/// Rotation d'un mot de 32 bits vers la gauche de 8 bits.
const fn rot_word(w: u32) -> u32 {
    (w << 8) | (w >> 24)
}

/// Substitution S-box sur chaque octet d'un mot de 32 bits.
fn sub_word(w: u32) -> u32 {
    let b = w.to_be_bytes();
    u32::from_be_bytes([SBOX[b[0] as usize], SBOX[b[1] as usize], SBOX[b[2] as usize], SBOX[b[3] as usize]])
}

/// Chiffre un seul bloc AES de 16 octets.
fn aes_encrypt_block(block: &[u8; 16], round_keys: &RoundKeys) -> [u8; 16] {
    let mut state = *block;

    // AddRoundKey initial
    xor_block(&mut state, &round_keys[0]);

    // Rounds 1 .. NR-1
    for round in 1..NR {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        xor_block(&mut state, &round_keys[round]);
    }

    // Dernier round (sans MixColumns)
    sub_bytes(&mut state);
    shift_rows(&mut state);
    xor_block(&mut state, &round_keys[NR]);

    state
}

/// SubBytes : substitution S-box sur chaque octet.
fn sub_bytes(state: &mut [u8; 16]) {
    for byte in state.iter_mut() {
        *byte = SBOX[*byte as usize];
    }
}

/// ShiftRows : decalage cyclique des lignes de la matrice d'etat.
fn shift_rows(s: &mut [u8; 16]) {
    // Ligne 1 : decalage de 1
    let t = s[1];
    s[1] = s[5]; s[5] = s[9]; s[9] = s[13]; s[13] = t;
    // Ligne 2 : decalage de 2
    let (t0, t1) = (s[2], s[6]);
    s[2] = s[10]; s[6] = s[14]; s[10] = t0; s[14] = t1;
    // Ligne 3 : decalage de 3
    let t = s[15];
    s[15] = s[11]; s[11] = s[7]; s[7] = s[3]; s[3] = t;
}

/// Multiplication par x dans GF(2^8) avec reduction par le polynome AES.
const fn xtime(a: u8) -> u8 {
    let shifted = (a as u16) << 1;
    let reduced = shifted ^ (((a >> 7) as u16) * 0x1b);
    reduced as u8
}

/// Multiplication dans GF(2^8).
fn gmul(mut a: u8, mut b: u8) -> u8 {
    let mut p = 0u8;
    for _ in 0..8 {
        if b & 1 != 0 { p ^= a; }
        let hi = a & 0x80;
        a <<= 1;
        if hi != 0 { a ^= 0x1b; }
        b >>= 1;
    }
    p
}

/// MixColumns : melange les colonnes de la matrice d'etat.
fn mix_columns(s: &mut [u8; 16]) {
    for i in 0..4 {
        let c = i * 4;
        let (a0, a1, a2, a3) = (s[c], s[c + 1], s[c + 2], s[c + 3]);
        s[c]     = gmul(a0, 2) ^ gmul(a1, 3) ^ a2 ^ a3;
        s[c + 1] = a0 ^ gmul(a1, 2) ^ gmul(a2, 3) ^ a3;
        s[c + 2] = a0 ^ a1 ^ gmul(a2, 2) ^ gmul(a3, 3);
        s[c + 3] = gmul(a0, 3) ^ a1 ^ a2 ^ gmul(a3, 2);
    }
}

/// XOR de deux blocs de 16 octets.
fn xor_block(dst: &mut [u8; 16], src: &[u8; 16]) {
    for i in 0..16 {
        dst[i] ^= src[i];
    }
}

/// Incremente le compteur GCM (4 derniers octets, big-endian).
fn inc_counter(ctr: &mut [u8; 16]) {
    let mut c = u32::from_be_bytes([ctr[12], ctr[13], ctr[14], ctr[15]]);
    c = c.wrapping_add(1);
    ctr[12..16].copy_from_slice(&c.to_be_bytes());
}

/// Prepare les donnees pour GHASH : padding a un multiple de 16 octets.
fn ghash_pad(data: &[u8]) -> Vec<u8> {
    let mut padded = data.to_vec();
    let rem = data.len() % 16;
    if rem != 0 {
        padded.resize(data.len() + (16 - rem), 0);
    }
    padded
}

/// Chiffre en AES-256-GCM.
///
/// * `key` - Cle AES-256 (32 octets)
/// * `nonce` - Nonce (12 octets)
/// * `plaintext` - Donnees a chiffrer
/// * `aad` - Donnees additionnelles authentifiees (non chiffrees)
///
/// Retourne (ciphertext, tag de 16 octets).
pub fn aes_gcm_encrypt(
    key: &[u8; AES_KEY_SIZE],
    nonce: &[u8; AES_GCM_NONCE_SIZE],
    plaintext: &[u8],
    aad: &[u8],
) -> (Vec<u8>, [u8; AES_GCM_TAG_SIZE]) {
    let round_keys = key_expansion(key);

    // H = AES_K(0^128)
    let h_block = aes_encrypt_block(&[0u8; 16], &round_keys);
    let h = GfElement::from_bytes(&h_block);

    // J0 = nonce || 0x00000001
    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(nonce);
    j0[15] = 1;

    // Chiffrer J0 pour le tag final
    let s0 = aes_encrypt_block(&j0, &round_keys);

    // Compteur pour le chiffrement : commence a J0 + 1
    let mut ctr = j0;
    inc_counter(&mut ctr);

    // Chiffrement CTR
    let mut ciphertext = Vec::with_capacity(plaintext.len());
    let mut offset = 0;
    while offset < plaintext.len() {
        let keystream = aes_encrypt_block(&ctr, &round_keys);
        inc_counter(&mut ctr);

        let block_len = (plaintext.len() - offset).min(AES_BLOCK_SIZE);
        for i in 0..block_len {
            ciphertext.push(plaintext[offset + i] ^ keystream[i]);
        }
        offset += block_len;
    }

    // GHASH pour le tag
    let tag = compute_tag(&h, aad, &ciphertext, &s0);

    (ciphertext, tag)
}

/// Dechiffre en AES-256-GCM.
///
/// Retourne les donnees dechiffrees ou une erreur si le tag est invalide.
pub fn aes_gcm_decrypt(
    key: &[u8; AES_KEY_SIZE],
    nonce: &[u8; AES_GCM_NONCE_SIZE],
    ciphertext: &[u8],
    tag: &[u8; AES_GCM_TAG_SIZE],
    aad: &[u8],
) -> BvResult<Vec<u8>> {
    let round_keys = key_expansion(key);

    // H = AES_K(0^128)
    let h_block = aes_encrypt_block(&[0u8; 16], &round_keys);
    let h = GfElement::from_bytes(&h_block);

    // J0
    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(nonce);
    j0[15] = 1;

    let s0 = aes_encrypt_block(&j0, &round_keys);

    // Verifier le tag d'abord (avant de dechiffrer)
    let computed_tag = compute_tag(&h, aad, ciphertext, &s0);
    if !constant_time_eq(&computed_tag, tag) {
        return Err(BvError::Crypto("AES-GCM tag verification failed".into()));
    }

    // Dechiffrement CTR (identique au chiffrement)
    let mut ctr = j0;
    inc_counter(&mut ctr);

    let mut plaintext = Vec::with_capacity(ciphertext.len());
    let mut offset = 0;
    while offset < ciphertext.len() {
        let keystream = aes_encrypt_block(&ctr, &round_keys);
        inc_counter(&mut ctr);

        let block_len = (ciphertext.len() - offset).min(AES_BLOCK_SIZE);
        for i in 0..block_len {
            plaintext.push(ciphertext[offset + i] ^ keystream[i]);
        }
        offset += block_len;
    }

    Ok(plaintext)
}

/// Calcule le tag GCM via GHASH.
fn compute_tag(h: &GfElement, aad: &[u8], ciphertext: &[u8], s0: &[u8; 16]) -> [u8; AES_GCM_TAG_SIZE] {
    // Construire l'input GHASH : pad(AAD) || pad(C) || len(AAD) || len(C)
    let mut ghash_input = ghash_pad(aad);
    ghash_input.extend_from_slice(&ghash_pad(ciphertext));

    // Longueurs en bits, big-endian, 8 octets chacune
    let aad_bits = (aad.len() as u64) * 8;
    let ct_bits = (ciphertext.len() as u64) * 8;
    ghash_input.extend_from_slice(&aad_bits.to_be_bytes());
    ghash_input.extend_from_slice(&ct_bits.to_be_bytes());

    let ghash_result = ghash(h, &ghash_input);
    let mut tag = ghash_result.to_bytes();

    // XOR avec S0 = AES_K(J0)
    for i in 0..16 {
        tag[i] ^= s0[i];
    }

    tag
}

/// Comparaison en temps constant pour eviter les attaques timing.
fn constant_time_eq(a: &[u8; 16], b: &[u8; 16]) -> bool {
    let mut diff = 0u8;
    for i in 0..16 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encrypt_block_known() {
        // NIST AES-256 test vector
        let key: [u8; 32] = [
            0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07,
            0x08,0x09,0x0a,0x0b,0x0c,0x0d,0x0e,0x0f,
            0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,
            0x18,0x19,0x1a,0x1b,0x1c,0x1d,0x1e,0x1f,
        ];
        let plain: [u8; 16] = [
            0x00,0x11,0x22,0x33,0x44,0x55,0x66,0x77,
            0x88,0x99,0xaa,0xbb,0xcc,0xdd,0xee,0xff,
        ];
        let expected: [u8; 16] = [
            0x8e,0xa2,0xb7,0xca,0x51,0x67,0x45,0xbf,
            0xea,0xfc,0x49,0x90,0x4b,0x49,0x60,0x89,
        ];
        let rk = key_expansion(&key);
        let ct = aes_encrypt_block(&plain, &rk);
        assert_eq!(ct, expected);
    }

    #[test]
    fn test_gcm_roundtrip_empty() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];
        let (ct, tag) = aes_gcm_encrypt(&key, &nonce, &[], &[]);
        assert!(ct.is_empty());
        let pt = aes_gcm_decrypt(&key, &nonce, &ct, &tag, &[]).unwrap();
        assert!(pt.is_empty());
    }

    #[test]
    fn test_gcm_roundtrip_data() {
        let key = [0xABu8; 32];
        let nonce = [0xCDu8; 12];
        let plaintext = b"Hello, BufferVault secure clipboard!";
        let aad = b"metadata";
        let (ct, tag) = aes_gcm_encrypt(&key, &nonce, plaintext, aad);
        assert_ne!(&ct[..], plaintext);
        let pt = aes_gcm_decrypt(&key, &nonce, &ct, &tag, aad).unwrap();
        assert_eq!(&pt, plaintext);
    }

    #[test]
    fn test_gcm_tampered_ciphertext() {
        let key = [0x11u8; 32];
        let nonce = [0x22u8; 12];
        let (mut ct, tag) = aes_gcm_encrypt(&key, &nonce, b"secret data", &[]);
        ct[0] ^= 0xFF; // alterer le ciphertext
        let result = aes_gcm_decrypt(&key, &nonce, &ct, &tag, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_gcm_tampered_tag() {
        let key = [0x33u8; 32];
        let nonce = [0x44u8; 12];
        let (ct, mut tag) = aes_gcm_encrypt(&key, &nonce, b"important", &[]);
        tag[0] ^= 1; // alterer le tag
        let result = aes_gcm_decrypt(&key, &nonce, &ct, &tag, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_gcm_wrong_aad() {
        let key = [0x55u8; 32];
        let nonce = [0x66u8; 12];
        let (ct, tag) = aes_gcm_encrypt(&key, &nonce, b"data", b"correct_aad");
        let result = aes_gcm_decrypt(&key, &nonce, &ct, &tag, b"wrong_aad");
        assert!(result.is_err());
    }

    #[test]
    fn test_gcm_large_data() {
        let key = [0x77u8; 32];
        let nonce = [0x88u8; 12];
        let plaintext = vec![0xAA; 1024]; // 1KB
        let (ct, tag) = aes_gcm_encrypt(&key, &nonce, &plaintext, &[]);
        let pt = aes_gcm_decrypt(&key, &nonce, &ct, &tag, &[]).unwrap();
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn test_constant_time_eq() {
        let a = [1u8; 16];
        let b = [1u8; 16];
        let c = [2u8; 16];
        assert!(constant_time_eq(&a, &b));
        assert!(!constant_time_eq(&a, &c));
    }
}
