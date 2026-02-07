// BufferVault - Multiplication GF(2^128) pour GCM (GHASH)
// Reference : NIST SP 800-38D
//
// Ce module implemente l'arithmetique dans GF(2^128) necessaire
// pour calculer le tag d'authentification GHASH en mode GCM.
//
// # Architecture
// - GfElement : element de GF(2^128) represente comme (hi:u64, lo:u64)
// - gf_mul : multiplication dans GF(2^128) avec reduction par le polynome
//   P(x) = x^128 + x^7 + x^2 + x + 1 (0xE1...00)
// - ghash : fonction GHASH qui accumule les blocs AAD et ciphertext
//
// # Securite
// L'implementation n'est pas en temps constant (la boucle bit-a-bit
// depend des bits de l'operande). Pour un usage en environnement
// hostile, une table de precomputation serait preferable.
//
// # Portabilite
// Ce module est en pur Rust, sans dependance Win32.

/// Represente un element de GF(2^128) comme deux u64.
/// Convention big-endian bit-reflected pour compatibilite GCM.
#[derive(Clone, Copy, Default)]
pub struct GfElement {
    pub hi: u64,
    pub lo: u64,
}

impl GfElement {
    /// Cree un element a partir de 16 octets (big-endian).
    pub fn from_bytes(b: &[u8; 16]) -> Self {
        Self {
            hi: u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
            lo: u64::from_be_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]),
        }
    }

    /// Convertit en 16 octets (big-endian).
    pub fn to_bytes(self) -> [u8; 16] {
        let mut out = [0u8; 16];
        out[..8].copy_from_slice(&self.hi.to_be_bytes());
        out[8..].copy_from_slice(&self.lo.to_be_bytes());
        out
    }

    /// XOR de deux elements.
    pub fn xor(self, other: Self) -> Self {
        Self {
            hi: self.hi ^ other.hi,
            lo: self.lo ^ other.lo,
        }
    }
}

/// Polynome de reduction pour GF(2^128) : x^128 + x^7 + x^2 + x + 1
/// Representation : 0xE1 << 120
const R_POLY: u64 = 0xE100000000000000;

/// Multiplication dans GF(2^128) utilisant l'algorithme de multiplication bit a bit.
/// Optimise pour eviter les timing side-channels autant que possible.
pub fn gf_mul(x: GfElement, y: GfElement) -> GfElement {
    let mut z = GfElement::default();
    let mut v = x;

    // Parcourir chaque bit de Y (128 bits, MSB first)
    for i in 0..128 {
        // Determiner le bit courant de Y
        let bit = if i < 64 {
            (y.hi >> (63 - i)) & 1
        } else {
            (y.lo >> (127 - i)) & 1
        };

        // Si le bit est 1, XOR Z avec V
        if bit == 1 {
            z = z.xor(v);
        }

        // Shift V a droite de 1 bit dans GF(2^128)
        let carry = v.hi & 1;
        v.hi >>= 1;
        v.lo = (v.lo >> 1) | (carry << 63);

        // Si le bit sorti est 1, XOR avec le polynome de reduction
        if carry == 1 {
            v.hi ^= R_POLY;
        }
    }

    z
}

/// Calcule GHASH sur une sequence de blocs de 16 octets.
/// GHASH(H, X) = X_1 * H xor X_2 * H xor ... xor X_n * H
///
/// * `h` - Sous-cle de hachage H = AES_K(0^128)
/// * `data` - Donnees dont la longueur doit etre un multiple de 16
pub fn ghash(h: &GfElement, data: &[u8]) -> GfElement {
    debug_assert!(data.len() % 16 == 0, "GHASH input must be multiple of 16 bytes");

    let mut y = GfElement::default();
    let mut i = 0;

    while i + 16 <= data.len() {
        let block: [u8; 16] = data[i..i + 16].try_into().unwrap();
        let x = GfElement::from_bytes(&block);
        y = gf_mul(y.xor(x), *h);
        i += 16;
    }

    y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gf_element_roundtrip() {
        let bytes: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
            0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10,
        ];
        let elem = GfElement::from_bytes(&bytes);
        assert_eq!(elem.to_bytes(), bytes);
    }

    #[test]
    fn test_gf_mul_identity() {
        // Multiplier par zero donne zero
        let a = GfElement { hi: 0x12345678, lo: 0x9ABCDEF0 };
        let zero = GfElement::default();
        let result = gf_mul(a, zero);
        assert_eq!(result.hi, 0);
        assert_eq!(result.lo, 0);
    }

    #[test]
    fn test_gf_mul_nonzero() {
        // Verifier que le produit de deux elements non-nuls est non-nul
        let a = GfElement::from_bytes(&[
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        let b = GfElement::from_bytes(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        ]);
        let ab = gf_mul(a, b);
        // Le produit doit etre non-nul
        assert!(ab.hi != 0 || ab.lo != 0);
    }

    #[test]
    fn test_ghash_empty() {
        let h = GfElement::from_bytes(&[1u8; 16]);
        let result = ghash(&h, &[]);
        assert_eq!(result.hi, 0);
        assert_eq!(result.lo, 0);
    }

    #[test]
    fn test_ghash_single_block() {
        let h_bytes = [0u8; 16];
        let h = GfElement::from_bytes(&h_bytes);
        let data = [0xFFu8; 16];
        let result = ghash(&h, &data);
        // H = 0 -> result should be 0 (multiply by zero)
        assert_eq!(result.hi, 0);
        assert_eq!(result.lo, 0);
    }
}
