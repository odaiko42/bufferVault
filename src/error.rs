// BufferVault - Types d'erreur centralises
//
// Ce module definit l'enumeration `BvError` et le type alias
// `BvResult<T>` utilises dans toute l'application.
//
// # Categories d'erreurs
// - `Clipboard` : echec d'acces au presse-papiers (OpenClipboard, etc.)
// - `Crypto` : erreur de chiffrement/dechiffrement (cle invalide, etc.)
// - `Storage` : erreur d'I/O disque (lecture/ecriture vault)
// - `Config` : erreur de parsing de la configuration
// - `Win32` : erreur API Windows generique (avec code GetLastError)
// - `Integrity` : corruption detectee (HMAC invalide, magic incorrect)
//
// L'implementation de `Display` formate chaque variante avec un
// prefixe entre crochets pour faciliter le diagnostic dans les logs.

use std::fmt;

/// Enumeration de toutes les erreurs possibles dans BufferVault.
#[derive(Debug)]
pub enum BvError {
    /// Erreur d'acces au presse-papiers Windows
    Clipboard(String),
    /// Erreur de chiffrement ou dechiffrement
    Crypto(String),
    /// Erreur de lecture/ecriture disque
    Storage(String),
    /// Erreur de configuration
    Config(String),
    /// Erreur Win32 API avec code d'erreur
    Win32(String, u32),
    /// Erreur d'integrite (HMAC invalide, donnees corrompues)
    Integrity(String),
}

impl fmt::Display for BvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BvError::Clipboard(m) => write!(f, "[Clipboard] {}", m),
            BvError::Crypto(m) => write!(f, "[Crypto] {}", m),
            BvError::Storage(m) => write!(f, "[Storage] {}", m),
            BvError::Config(m) => write!(f, "[Config] {}", m),
            BvError::Win32(m, c) => write!(f, "[Win32] {} (code={})", m, c),
            BvError::Integrity(m) => write!(f, "[Integrity] {}", m),
        }
    }
}

impl From<std::io::Error> for BvError {
    fn from(e: std::io::Error) -> Self {
        BvError::Storage(e.to_string())
    }
}

/// Type Result specialise pour BufferVault.
pub type BvResult<T> = Result<T, BvError>;
