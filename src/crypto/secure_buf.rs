// BufferVault - Buffer securise avec zeroing automatique
// Garantit que les donnees sensibles sont effacees de la memoire.
//
// Ce module fournit un wrapper securise autour de Vec<u8> qui garantit
// que le contenu du buffer est ecrase avec des zeros lorsque le buffer
// est libere (via Drop), meme si le compilateur tente d'optimiser
// l'ecriture.
//
// # Implementation
// Le zeroing utilise `write_volatile` octet par octet pour empecher
// le compilateur de supprimer l'ecriture comme "morte". C'est la
// technique standard recommandee avant l'arrivee de `zeroize` en crate.
//
// # Utilisation
// ```rust
// let buf = SecureBuf::from_slice(secret_data);
// // ... utilisation via Deref/DerefMut ...
// // A la sortie du scope, le contenu est efface
// ```
//
// # Portabilite
// Ce module est en pur Rust, sans dependance Win32.

use std::ops::{Deref, DerefMut};

/// Buffer securise qui efface son contenu a la destruction.
/// Utilise `write_volatile` pour empecher l'optimiseur de supprimer le zeroing.
pub struct SecureBuf {
    data: Vec<u8>,
}

impl SecureBuf {
    /// Cree un buffer securise a partir de donnees existantes.
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Cree un buffer securise rempli de zeros de la taille donnee.
    pub fn zeroed(len: usize) -> Self {
        Self { data: vec![0u8; len] }
    }

    /// Cree un buffer securise a partir d'un slice (copie).
    pub fn from_slice(s: &[u8]) -> Self {
        Self { data: s.to_vec() }
    }

    /// Retourne la longueur du buffer.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Retourne true si le buffer est vide.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Efface le contenu du buffer avec des zeros de maniere non optimisable.
    fn secure_zero(&mut self) {
        for byte in self.data.iter_mut() {
            // SAFETY: Ecriture volatile pour empecher l'optimiseur
            // de supprimer le zeroing. Le pointeur est valide car il provient
            // d'une reference mutable vers un Vec alloue.
            unsafe {
                std::ptr::write_volatile(byte as *mut u8, 0);
            }
        }
        // Barriere memoire pour s'assurer que les ecritures sont visibles
        std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
    }
}

impl Deref for SecureBuf {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.data
    }
}

impl DerefMut for SecureBuf {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl Drop for SecureBuf {
    /// Efface le contenu du buffer avant liberation.
    fn drop(&mut self) {
        self.secure_zero();
    }
}

impl Clone for SecureBuf {
    fn clone(&self) -> Self {
        Self { data: self.data.clone() }
    }
}

/// Efface un slice de maniere non optimisable (pour les tableaux sur la stack).
pub fn secure_zero_slice(s: &mut [u8]) {
    for byte in s.iter_mut() {
        // SAFETY: Meme justification que SecureBuf::secure_zero.
        unsafe {
            std::ptr::write_volatile(byte as *mut u8, 0);
        }
    }
    std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_buf_new() {
        let buf = SecureBuf::new(vec![1, 2, 3, 4]);
        assert_eq!(&*buf, &[1, 2, 3, 4]);
        assert_eq!(buf.len(), 4);
    }

    #[test]
    fn test_secure_buf_zeroed() {
        let buf = SecureBuf::zeroed(16);
        assert_eq!(buf.len(), 16);
        assert!(buf.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_secure_zero_slice() {
        let mut data = [0xFFu8; 32];
        secure_zero_slice(&mut data);
        assert!(data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_secure_buf_deref_mut() {
        let mut buf = SecureBuf::zeroed(4);
        buf[0] = 0xAA;
        buf[3] = 0xBB;
        assert_eq!(buf[0], 0xAA);
        assert_eq!(buf[3], 0xBB);
    }
}
