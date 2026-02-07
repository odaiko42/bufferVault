// BufferVault - Implementation SHA-256 pure Rust
// Reference : FIPS 180-4 (Secure Hash Standard)
//
// Ce module implemente l'algorithme de hashage SHA-256 en pur Rust.
// Il fournit deux interfaces :
// - `sha256(data)` : hashage en une seule etape
// - `Sha256::new() -> update() -> finalize()` : hashage incremental
//
// # Tests
// Les vecteurs de test NIST sont inclus pour valider l'implementation.
//
// # Portabilite
// Ce module est en pur Rust, sans dependance Win32.

/// Constantes K pour SHA-256 (64 mots de 32 bits).
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Valeurs initiales du hash SHA-256.
const H_INIT: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Rotation droite sur 32 bits.
const fn rotr(x: u32, n: u32) -> u32 {
    (x >> n) | (x << (32 - n))
}

/// Fonction Sigma majuscule 0.
const fn big_sigma0(x: u32) -> u32 {
    rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22)
}

/// Fonction Sigma majuscule 1.
const fn big_sigma1(x: u32) -> u32 {
    rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25)
}

/// Fonction sigma minuscule 0.
const fn small_sigma0(x: u32) -> u32 {
    rotr(x, 7) ^ rotr(x, 18) ^ (x >> 3)
}

/// Fonction sigma minuscule 1.
const fn small_sigma1(x: u32) -> u32 {
    rotr(x, 17) ^ rotr(x, 19) ^ (x >> 10)
}

/// Fonction Ch(x, y, z).
const fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ ((!x) & z)
}

/// Fonction Maj(x, y, z).
const fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// Etat interne du hasher SHA-256.
pub struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buf_len: usize,
    total_len: u64,
}

impl Sha256 {
    /// Cree un nouveau hasher SHA-256.
    pub fn new() -> Self {
        Self {
            state: H_INIT,
            buffer: [0u8; 64],
            buf_len: 0,
            total_len: 0,
        }
    }

    /// Ajoute des donnees au hash.
    pub fn update(&mut self, data: &[u8]) {
        self.total_len += data.len() as u64;
        let mut offset = 0;

        // Completer le buffer existant si necessaire
        if self.buf_len > 0 {
            let space = 64 - self.buf_len;
            let to_copy = data.len().min(space);
            self.buffer[self.buf_len..self.buf_len + to_copy].copy_from_slice(&data[..to_copy]);
            self.buf_len += to_copy;
            offset = to_copy;

            if self.buf_len == 64 {
                let block = self.buffer;
                Self::compress(&mut self.state, &block);
                self.buf_len = 0;
            }
        }

        // Traiter les blocs complets directement
        while offset + 64 <= data.len() {
            let block: [u8; 64] = data[offset..offset + 64].try_into().unwrap();
            Self::compress(&mut self.state, &block);
            offset += 64;
        }

        // Stocker le reste dans le buffer
        let remaining = data.len() - offset;
        if remaining > 0 {
            self.buffer[..remaining].copy_from_slice(&data[offset..]);
            self.buf_len = remaining;
        }
    }

    /// Finalise le hash et retourne le digest de 32 octets.
    pub fn finalize(mut self) -> [u8; 32] {
        let bit_len = self.total_len * 8;

        // Padding : ajouter 0x80
        self.buffer[self.buf_len] = 0x80;
        self.buf_len += 1;

        // Si pas assez de place pour la longueur (8 octets), remplir et compresser
        if self.buf_len > 56 {
            for i in self.buf_len..64 {
                self.buffer[i] = 0;
            }
            let block = self.buffer;
            Self::compress(&mut self.state, &block);
            self.buf_len = 0;
        }

        // Remplir de zeros jusqu'a l'offset 56
        for i in self.buf_len..56 {
            self.buffer[i] = 0;
        }

        // Ecrire la longueur en bits (big-endian) dans les 8 derniers octets
        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());

        let block = self.buffer;
        Self::compress(&mut self.state, &block);

        // Construire le digest
        let mut digest = [0u8; 32];
        for i in 0..8 {
            digest[i * 4..(i + 1) * 4].copy_from_slice(&self.state[i].to_be_bytes());
        }
        digest
    }

    /// Compresse un bloc de 64 octets dans l'etat.
    fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
        // Preparer le message schedule (W)
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            w[i] = small_sigma1(w[i - 2])
                .wrapping_add(w[i - 7])
                .wrapping_add(small_sigma0(w[i - 15]))
                .wrapping_add(w[i - 16]);
        }

        // Variables de travail
        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        // 64 rounds
        for i in 0..64 {
            let t1 = h
                .wrapping_add(big_sigma1(e))
                .wrapping_add(ch(e, f, g))
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let t2 = big_sigma0(a).wrapping_add(maj(a, b, c));

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        // Ajouter au state
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }
}

/// Calcule le SHA-256 d'un bloc de donnees en une seule passe.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let hash = sha256(b"");
        let expected = [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
            0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
            0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
            0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_abc() {
        let hash = sha256(b"abc");
        let expected = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea,
            0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
            0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
            0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_long() {
        // "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
        let input = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let hash = sha256(input);
        let expected = [
            0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, 0xb8,
            0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60, 0x39,
            0xa3, 0x3c, 0xe4, 0x59, 0x64, 0xff, 0x21, 0x67,
            0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb, 0x06, 0xc1,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_incremental() {
        // Le meme resultat qu'en une passe
        let mut h = Sha256::new();
        h.update(b"ab");
        h.update(b"c");
        let hash = h.finalize();
        assert_eq!(hash, sha256(b"abc"));
    }
}
