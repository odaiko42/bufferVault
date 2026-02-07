// BufferVault - Demarrage automatique Windows
// Gestion de la cle registre HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run
//
// Ce module permet d'activer/desactiver le demarrage automatique de
// BufferVault au lancement de Windows en manipulant la cle registre
// HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run.
//
// # Safety
// Tous les appels FFI Win32 (advapi32, kernel32) sont isoles dans des
// blocs unsafe locaux. Les handles de cle registre sont fermes dans
// le meme scope que leur ouverture pour eviter les fuites.
//
// # Portabilite
// Ce module est specifique a Windows (registre HKCU).

use crate::system::win32::*;

/// Nom de la valeur registre pour le demarrage automatique.
const REG_VALUE_NAME: &str = "BufferVault";

/// Chemin de la cle Run dans le registre Windows.
const REG_RUN_PATH: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

// --- Types et constantes registre ---
type HKEY = *mut std::ffi::c_void;
/// Handle predifini pour HKEY_CURRENT_USER.
const HKEY_CURRENT_USER: HKEY = 0x80000001u32 as isize as HKEY;
/// Droit d'acces en lecture au registre.
const KEY_READ: u32 = 0x20019;
/// Droit d'acces en ecriture au registre.
const KEY_WRITE: u32 = 0x20006;
/// Type de valeur registre : chaine de caracteres.
const REG_SZ: u32 = 1;
/// Code de retour : operation reussie.
const ERROR_SUCCESS: u32 = 0;
/// Code de retour : fichier/valeur non trouve.
const ERROR_FILE_NOT_FOUND: u32 = 2;

// --- FFI advapi32 ---
#[link(name = "advapi32")]
extern "system" {
    fn RegOpenKeyExW(key: HKEY, sub: LPCWSTR, opt: u32, sam: u32, out: *mut HKEY) -> u32;
    fn RegCloseKey(key: HKEY) -> u32;
    fn RegSetValueExW(
        key: HKEY, name: LPCWSTR, reserved: u32, typ: u32,
        data: *const u8, cb: u32,
    ) -> u32;
    fn RegDeleteValueW(key: HKEY, name: LPCWSTR) -> u32;
    fn RegQueryValueExW(
        key: HKEY, name: LPCWSTR, reserved: *mut u32, typ: *mut u32,
        data: *mut u8, cb: *mut u32,
    ) -> u32;
}

// --- FFI kernel32 (GetModuleFileNameW) ---
extern "system" {
    fn GetModuleFileNameW(module: HMODULE, buf: LPWSTR, size: u32) -> u32;
}

/// Recupere le chemin complet de l'executable courant.
///
/// Utilise `GetModuleFileNameW` pour obtenir le chemin absolu du binaire.
/// Retourne None si l'appel echoue ou si le chemin depasse le buffer.
fn get_exe_path() -> Option<String> {
    let mut buf = [0u16; 512];
    // SAFETY: appel FFI Win32, buffer de taille fixe.
    let len = unsafe { GetModuleFileNameW(std::ptr::null_mut(), buf.as_mut_ptr(), buf.len() as u32) };
    if len == 0 || len >= buf.len() as u32 {
        return None;
    }
    Some(from_wstring(&buf[..len as usize]))
}

/// Verifie si le demarrage automatique est active dans le registre.
///
/// Ouvre la cle `HKCU\...\Run` en lecture et verifie l'existence
/// de la valeur "BufferVault". Retourne false en cas d'erreur d'acces.
pub fn is_autostart_enabled() -> bool {
    let wpath = to_wstring(REG_RUN_PATH);
    let wname = to_wstring(REG_VALUE_NAME);
    let mut hkey: HKEY = std::ptr::null_mut();

    // SAFETY: appels FFI Win32 pour lire le registre.
    unsafe {
        let res = RegOpenKeyExW(HKEY_CURRENT_USER, wpath.as_ptr(), 0, KEY_READ, &mut hkey);
        if res != ERROR_SUCCESS {
            return false;
        }

        let res = RegQueryValueExW(
            hkey,
            wname.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        RegCloseKey(hkey);

        res == ERROR_SUCCESS
    }
}

/// Active le demarrage automatique en ajoutant l'executable dans la cle Run.
///
/// Ecrit le chemin complet de l'executable (entre guillemets pour supporter
/// les espaces) comme valeur REG_SZ dans la cle Run de HKCU.
///
/// # Returns
/// `true` si l'ecriture a reussi, `false` sinon.
pub fn enable_autostart() -> bool {
    let exe_path = match get_exe_path() {
        Some(p) => p,
        None => return false,
    };

    // Encadrer le chemin entre guillemets pour supporter les espaces
    let quoted = format!("\"{}\"", exe_path);
    let wpath = to_wstring(REG_RUN_PATH);
    let wname = to_wstring(REG_VALUE_NAME);
    let wvalue = to_wstring(&quoted);
    let mut hkey: HKEY = std::ptr::null_mut();

    // SAFETY: appels FFI Win32 pour ecrire dans le registre.
    unsafe {
        let res = RegOpenKeyExW(HKEY_CURRENT_USER, wpath.as_ptr(), 0, KEY_WRITE, &mut hkey);
        if res != ERROR_SUCCESS {
            return false;
        }

        let data_bytes = wvalue.len() * 2; // taille en octets, null inclus
        let res = RegSetValueExW(
            hkey,
            wname.as_ptr(),
            0,
            REG_SZ,
            wvalue.as_ptr() as *const u8,
            data_bytes as u32,
        );
        RegCloseKey(hkey);

        res == ERROR_SUCCESS
    }
}

/// Desactive le demarrage automatique en supprimant la valeur du registre.
pub fn disable_autostart() -> bool {
    let wpath = to_wstring(REG_RUN_PATH);
    let wname = to_wstring(REG_VALUE_NAME);
    let mut hkey: HKEY = std::ptr::null_mut();

    // SAFETY: appels FFI Win32 pour supprimer une valeur du registre.
    unsafe {
        let res = RegOpenKeyExW(HKEY_CURRENT_USER, wpath.as_ptr(), 0, KEY_WRITE, &mut hkey);
        if res != ERROR_SUCCESS {
            return false;
        }

        let res = RegDeleteValueW(hkey, wname.as_ptr());
        RegCloseKey(hkey);

        // Succes ou valeur deja absente
        res == ERROR_SUCCESS || res == ERROR_FILE_NOT_FOUND
    }
}

/// Bascule l'etat du demarrage automatique.
///
/// Si actuellement active, le desactive ; sinon, l'active.
///
/// # Returns
/// Le nouvel etat : `true` = active, `false` = desactive.
pub fn toggle_autostart() -> bool {
    if is_autostart_enabled() {
        disable_autostart();
        false
    } else {
        enable_autostart();
        true
    }
}
