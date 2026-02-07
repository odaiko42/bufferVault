// BufferVault - Lecture/ecriture du fichier vault.dat
// Sauvegarde et chargement chiffre de l'historique
//
// Ce module gere la persistance chiffree de l'historique sur disque.
//
// # Format du fichier vault.dat
// ```text
// [BVAULT01]  magic number (8 octets)
// [version]   format version u32 LE (4 octets)
// [nonce]     AES-GCM nonce (12 octets)
// [tag]       AES-GCM tag (16 octets)
// [ct_len]    taille ciphertext u32 LE (4 octets)
// [ciphertext] donnees chiffrees (ct_len octets)
// [hmac]      HMAC-SHA256 de tout ce qui precede (32 octets)
// ```
//
// # Securite
// - Le nonce est genere aleatoirement via BCryptGenRandom (CSPRNG)
// - Le magic number est utilise comme AAD (additional authenticated data)
// - Le HMAC couvre l'integralite du fichier avant lui-meme
// - L'ecriture est atomique (fichier temporaire + rename)
//
// # Portabilite
// Dependance Windows limitee a BCryptGenRandom pour le nonce.

use crate::constants::*;
use crate::crypto::aes_gcm::{aes_gcm_encrypt, aes_gcm_decrypt};
use crate::crypto::pbkdf2::hmac_sha256;
use crate::error::{BvError, BvResult};
use crate::history::entry::ClipboardEntry;
use crate::storage::format;
use crate::system::win32;
use std::path::Path;
use std::fs;

/// Header du fichier vault.
struct VaultHeader {
    salt: [u8; PBKDF2_SALT_SIZE],
    iterations: u32,
}

/// Sauvegarde l'historique chiffre sur disque.
/// Utilise une ecriture atomique (fichier temporaire + rename).
pub fn save_vault(
    path: &Path,
    entries: &[ClipboardEntry],
    key: &[u8],
) -> BvResult<()> {
    // Serialiser les entrees
    let plaintext = format::serialize_entries(entries);

    // Generer un nonce aleatoire
    let mut nonce = [0u8; AES_GCM_NONCE_SIZE];
    if !win32::csprng_fill(&mut nonce) {
        return Err(BvError::Crypto("Failed to generate nonce".into()));
    }

    // Construire la cle AES a partir du slice
    let aes_key: [u8; AES_KEY_SIZE] = key[..AES_KEY_SIZE]
        .try_into()
        .map_err(|_| BvError::Crypto("Invalid key length".into()))?;

    // Chiffrer
    let aad = VAULT_MAGIC;
    let (ciphertext, tag) = aes_gcm_encrypt(&aes_key, &nonce, &plaintext, aad);

    // Construire le fichier vault
    let mut data = Vec::new();
    data.extend_from_slice(VAULT_MAGIC);
    data.extend_from_slice(&VAULT_FORMAT_VERSION.to_le_bytes());
    data.extend_from_slice(&nonce);
    data.extend_from_slice(&tag);
    data.extend_from_slice(&(ciphertext.len() as u32).to_le_bytes());
    data.extend_from_slice(&ciphertext);

    // HMAC du fichier entier pour verification d'integrite
    let file_hmac = hmac_sha256(key, &data);
    data.extend_from_slice(&file_hmac);

    // Ecriture atomique : temp file + rename
    let tmp_path = path.with_extension("tmp");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&tmp_path, &data)?;
    fs::rename(&tmp_path, path)?;

    Ok(())
}

/// Charge l'historique depuis le fichier vault chiffre.
pub fn load_vault(
    path: &Path,
    key: &[u8],
) -> BvResult<Vec<ClipboardEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let data = fs::read(path)?;

    // Verifier la taille minimale : magic(8) + version(4) + nonce(12) + tag(16) + len(4) + hmac(32)
    let min_size = 8 + 4 + AES_GCM_NONCE_SIZE + AES_GCM_TAG_SIZE + 4 + HMAC_SIZE;
    if data.len() < min_size {
        return Err(BvError::Integrity("Vault file too small".into()));
    }

    // Verifier le magic number
    if &data[0..8] != VAULT_MAGIC {
        return Err(BvError::Integrity("Invalid vault magic number".into()));
    }

    // Verifier la version (octets 8-11, juste apres le magic)
    let mut pos = 8;
    let version = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
    pos += 4;
    if version != VAULT_FORMAT_VERSION {
        return Err(BvError::Integrity(format!("Unsupported vault version: {}", version)));
    }

    // Verifier le HMAC d'integrite
    let hmac_offset = data.len() - HMAC_SIZE;
    let file_hmac = hmac_sha256(key, &data[..hmac_offset]);
    if file_hmac != data[hmac_offset..] {
        return Err(BvError::Integrity("Vault file integrity check failed (HMAC)".into()));
    }

    // Lire le nonce
    let nonce: [u8; AES_GCM_NONCE_SIZE] = data[pos..pos + AES_GCM_NONCE_SIZE]
        .try_into()
        .map_err(|_| BvError::Integrity("Invalid nonce".into()))?;
    pos += AES_GCM_NONCE_SIZE;

    // Lire le tag
    let tag: [u8; AES_GCM_TAG_SIZE] = data[pos..pos + AES_GCM_TAG_SIZE]
        .try_into()
        .map_err(|_| BvError::Integrity("Invalid tag".into()))?;
    pos += AES_GCM_TAG_SIZE;

    // Lire la taille du ciphertext
    let ct_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;

    if pos + ct_len > hmac_offset {
        return Err(BvError::Integrity("Ciphertext size exceeds file".into()));
    }
    let ciphertext = &data[pos..pos + ct_len];

    // Dechiffrer
    let aes_key: [u8; AES_KEY_SIZE] = key[..AES_KEY_SIZE]
        .try_into()
        .map_err(|_| BvError::Crypto("Invalid key length".into()))?;

    let plaintext = aes_gcm_decrypt(&aes_key, &nonce, ciphertext, &tag, VAULT_MAGIC)?;

    // Deserialiser
    format::deserialize_entries(&plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::entry::EntryType;
    use std::path::PathBuf;

    fn test_key() -> Vec<u8> {
        vec![0xAA; AES_KEY_SIZE]
    }

    fn make_entries() -> Vec<ClipboardEntry> {
        vec![
            ClipboardEntry::new(EntryType::Text, "test.exe".into(), "hello world".into()),
            ClipboardEntry::new(EntryType::Text, "app.exe".into(), "second entry".into()),
        ]
    }

    #[test]
    fn test_vault_roundtrip() {
        let dir = std::env::temp_dir().join("buffervault_test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_vault.dat");

        let key = test_key();
        let entries = make_entries();

        save_vault(&path, &entries, &key).unwrap();
        let loaded = load_vault(&path, &key).unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].content, "hello world");
        assert_eq!(loaded[1].content, "second entry");

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_vault_empty() {
        let dir = std::env::temp_dir().join("buffervault_test_empty");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_empty.dat");

        let key = test_key();
        save_vault(&path, &[], &key).unwrap();
        let loaded = load_vault(&path, &key).unwrap();
        assert!(loaded.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_vault_nonexistent() {
        let path = PathBuf::from("nonexistent_vault_file.dat");
        let key = test_key();
        let loaded = load_vault(&path, &key).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_vault_wrong_key() {
        let dir = std::env::temp_dir().join("buffervault_test_wrongkey");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_wrongkey.dat");

        let key = test_key();
        let entries = make_entries();
        save_vault(&path, &entries, &key).unwrap();

        let wrong_key = vec![0xBB; AES_KEY_SIZE];
        let result = load_vault(&path, &wrong_key);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }
}
