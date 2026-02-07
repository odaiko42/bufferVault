// BufferVault - Gestion des hotkeys globaux
// Enregistrement et reception des raccourcis clavier systeme
//
// Ce module gere l'enregistrement/desenregistrement de hotkeys globaux
// via l'API Win32 RegisterHotKey/UnregisterHotKey. Un seul hotkey est
// utilise pour ouvrir/fermer la fenetre BufferVault.
//
// # Fonctionnement
// - `register_global_hotkey` : enregistre un raccourci systeme
// - `unregister_global_hotkey` : libere le raccourci
// - `parse_vk_code` : convertit un nom de touche en code VK_*
//
// Le flag MOD_NOREPEAT est toujours ajoute pour eviter les messages
// WM_HOTKEY repetes si l'utilisateur maintient les touches.
//
// # SAFETY
// Tous les appels Win32 sont isoles dans des blocs unsafe locaux.
// hwnd doit etre un handle de fenetre valide.
//
// # Portabilite
// Ce module est specifique a Windows (user32.dll).

use crate::constants::HOTKEY_ID;
use crate::error::{BvError, BvResult};
use crate::system::win32::*;

/// Enregistre un hotkey global (par defaut : Ctrl+Shift+V).
///
/// # Arguments
/// * `hwnd` - Handle de la fenetre receptrice du message WM_HOTKEY
/// * `modifiers` - Combinaison de MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN
/// * `vk` - Code de touche virtuelle (ex: VK_V)
pub fn register_global_hotkey(hwnd: HWND, modifiers: u32, vk: u32) -> BvResult<()> {
    let mods = modifiers | MOD_NOREPEAT;
    // SAFETY: appel FFI Win32. hwnd doit etre un handle de fenetre valide.
    let ok = unsafe { RegisterHotKey(hwnd, HOTKEY_ID, mods, vk) };
    if ok == FALSE {
        let err = last_error();
        return Err(BvError::Win32("RegisterHotKey failed".into(), err));
    }
    Ok(())
}

/// Desenregistre le hotkey global.
pub fn unregister_global_hotkey(hwnd: HWND) {
    // SAFETY: appel FFI Win32.
    unsafe { UnregisterHotKey(hwnd, HOTKEY_ID) };
}

/// Parse un code de touche virtuelle depuis un nom de touche.
/// Supporte : A-Z, 0-9, F1-F12, V, SPACE, RETURN, etc.
pub fn parse_vk_code(key_name: &str) -> Option<u32> {
    let upper = key_name.to_uppercase();
    match upper.as_str() {
        "V" => Some(VK_V),
        "RETURN" | "ENTER" => Some(VK_RETURN),
        "ESCAPE" | "ESC" => Some(VK_ESCAPE),
        "UP" => Some(VK_UP),
        "DOWN" => Some(VK_DOWN),
        "DELETE" | "DEL" => Some(VK_DELETE),
        "SPACE" => Some(0x20),
        "TAB" => Some(0x09),
        s if s.len() == 1 => {
            let ch = s.chars().next()?;
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_uppercase() as u32)
            } else {
                None
            }
        }
        s if s.starts_with('F') && s.len() <= 3 => {
            let num: u32 = s[1..].parse().ok()?;
            if (1..=12).contains(&num) {
                // VK_F1 = 0x70
                Some(0x70 + num - 1)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vk_v() {
        assert_eq!(parse_vk_code("V"), Some(VK_V));
        assert_eq!(parse_vk_code("v"), Some(VK_V));
    }

    #[test]
    fn test_parse_vk_f_keys() {
        assert_eq!(parse_vk_code("F1"), Some(0x70));
        assert_eq!(parse_vk_code("F12"), Some(0x7B));
        assert_eq!(parse_vk_code("F13"), None);
    }

    #[test]
    fn test_parse_vk_letter() {
        assert_eq!(parse_vk_code("A"), Some(0x41));
        assert_eq!(parse_vk_code("z"), Some(0x5A));
    }

    #[test]
    fn test_parse_vk_special() {
        assert_eq!(parse_vk_code("RETURN"), Some(VK_RETURN));
        assert_eq!(parse_vk_code("SPACE"), Some(0x20));
        assert_eq!(parse_vk_code("ESCAPE"), Some(VK_ESCAPE));
    }
}
