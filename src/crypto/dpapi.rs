// BufferVault - Wrappers DPAPI Windows
// Protection de la cle maitre via CryptProtectData / CryptUnprotectData
//
// Ce module utilise la Windows Data Protection API (DPAPI) pour
// proteger la cle maitre AES-256 de BufferVault. La cle est chiffree
// avec les credentials Windows de l'utilisateur courant.
//
// # Fonctionnement
// - `dpapi_protect`  : chiffre un blob avec les credentials utilisateur
// - `dpapi_unprotect` : dechiffre un blob DPAPI
// - `load_or_create_master_key` : charge la cle depuis le keystore ou
//   en genere une nouvelle (32 octets aleatoires via BCryptGenRandom)
//
// # Securite
// - La cle maitre est liee a la session Windows de l'utilisateur
// - CRYPTPROTECT_UI_FORBIDDEN empeche l'affichage de boites de dialogue
// - La memoire allouee par DPAPI est liberee via LocalFree
//
// # Portabilite
// Ce module est specifique a Windows (crypt32.dll).

use crate::error::{BvError, BvResult};
use crate::system::win32;
use std::path::Path;
use std::fs;

/// Protege des donnees via DPAPI (liees a la session Windows de l'utilisateur).
/// Retourne le blob chiffre.
pub fn dpapi_protect(data: &[u8]) -> BvResult<Vec<u8>> {
    let data_in = win32::DATA_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut data_out = win32::DATA_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    // SAFETY: CryptProtectData est une API Windows documentee.
    // data_in pointe vers des donnees valides, data_out sera alloue par Windows.
    let result = unsafe {
        win32::CryptProtectData(
            &data_in as *const _,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            win32::CRYPTPROTECT_UI_FORBIDDEN,
            &mut data_out as *mut _,
        )
    };

    if result == 0 {
        return Err(BvError::Crypto(format!(
            "DPAPI CryptProtectData failed (code={})", win32::last_error()
        )));
    }

    // SAFETY: data_out.pbData a ete alloue par Windows et contient cbData octets valides.
    let protected = unsafe {
        std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec()
    };

    // SAFETY: Liberer la memoire allouee par Windows.
    unsafe { win32::LocalFree(data_out.pbData as *mut _); }

    Ok(protected)
}

/// Dechiffre des donnees protegees par DPAPI.
/// Retourne les donnees en clair.
pub fn dpapi_unprotect(blob: &[u8]) -> BvResult<Vec<u8>> {
    let data_in = win32::DATA_BLOB {
        cbData: blob.len() as u32,
        pbData: blob.as_ptr() as *mut u8,
    };
    let mut data_out = win32::DATA_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    // SAFETY: CryptUnprotectData est une API Windows documentee.
    let result = unsafe {
        win32::CryptUnprotectData(
            &data_in as *const _,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            win32::CRYPTPROTECT_UI_FORBIDDEN,
            &mut data_out as *mut _,
        )
    };

    if result == 0 {
        return Err(BvError::Crypto(format!(
            "DPAPI CryptUnprotectData failed (code={})", win32::last_error()
        )));
    }

    // SAFETY: data_out.pbData a ete alloue par Windows et contient cbData octets valides.
    let unprotected = unsafe {
        std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec()
    };

    unsafe { win32::LocalFree(data_out.pbData as *mut _); }

    Ok(unprotected)
}

/// Charge ou genere la cle maitre protegee par DPAPI.
/// Au premier lancement, genere un secret aleatoire de 32 octets,
/// le protege via DPAPI et le sauvegarde dans keystore_path.
/// Aux lancements suivants, lit le blob et le dechiffre.
pub fn load_or_create_master_key(keystore_path: &Path) -> BvResult<Vec<u8>> {
    if keystore_path.exists() {
        let blob = fs::read(keystore_path)
            .map_err(|e| BvError::Storage(format!("Cannot read keystore: {}", e)))?;
        dpapi_unprotect(&blob)
    } else {
        // Generer un secret aleatoire
        let mut secret = vec![0u8; 32];
        if !win32::csprng_fill(&mut secret) {
            return Err(BvError::Crypto("CSPRNG failed to generate master key".into()));
        }

        // Proteger via DPAPI
        let blob = dpapi_protect(&secret)?;

        // Creer le repertoire parent si necessaire
        if let Some(parent) = keystore_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| BvError::Storage(format!("Cannot create keystore dir: {}", e)))?;
        }

        // Sauvegarder le blob
        fs::write(keystore_path, &blob)
            .map_err(|e| BvError::Storage(format!("Cannot write keystore: {}", e)))?;

        Ok(secret)
    }
}
