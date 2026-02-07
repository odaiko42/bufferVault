// BufferVault - Injection dans le presse-papiers
// Ecrit du texte dans le presse-papiers Windows via les APIs Win32.
//
// Ce module fournit deux fonctionnalites :
// - `set_clipboard_text` : place du texte dans le presse-papiers (CF_UNICODETEXT)
// - `simulate_paste` : simule la combinaison Ctrl+V via SendInput
//
// # Safety
// Tous les appels FFI Win32 sont isoles dans des blocs unsafe locaux.
// La sequence OpenClipboard/EmptyClipboard/SetClipboardData/CloseClipboard
// est executee dans un seul scope pour garantir la coherence.
// En cas d'erreur, les ressources (GlobalAlloc) sont liberees avant retour.
//
// # Portabilite
// Ce module est specifique a Windows (Win32 API).
// Sur d'autres plateformes, il faudrait une implementation alternative.

use crate::error::{BvError, BvResult};
use crate::system::win32::*;

/// Ecrit du texte dans le presse-papiers Windows au format CF_UNICODETEXT.
///
/// Ouvre le presse-papiers, le vide, alloue un bloc de memoire globale,
/// y copie le texte au format UTF-16, puis le transmet au presse-papiers.
///
/// # Arguments
/// * `hwnd` - Handle de la fenetre proprietaire du presse-papiers
/// * `text` - Texte a placer dans le presse-papiers
///
/// # Errors
/// Retourne `BvError::Clipboard` si une des operations Win32 echoue
/// (ouverture, vidage, allocation memoire, verrouillage, ecriture).
pub fn set_clipboard_text(hwnd: HWND, text: &str) -> BvResult<()> {
    let wtext = to_wstring(text);
    let bytes_needed = wtext.len() * 2;

    // SAFETY: sequence d'appels FFI Win32 pour le clipboard.
    unsafe {
        if OpenClipboard(hwnd) == FALSE {
            return Err(BvError::Clipboard("OpenClipboard failed".into()));
        }

        if EmptyClipboard() == FALSE {
            CloseClipboard();
            return Err(BvError::Clipboard("EmptyClipboard failed".into()));
        }

        let hmem = GlobalAlloc(GHND, bytes_needed);
        if hmem.is_null() {
            CloseClipboard();
            return Err(BvError::Clipboard("GlobalAlloc failed".into()));
        }

        let ptr = GlobalLock(hmem);
        if ptr.is_null() {
            GlobalFree(hmem);
            CloseClipboard();
            return Err(BvError::Clipboard("GlobalLock failed".into()));
        }

        std::ptr::copy_nonoverlapping(
            wtext.as_ptr() as *const u8,
            ptr as *mut u8,
            bytes_needed,
        );
        GlobalUnlock(hmem);

        if SetClipboardData(CF_UNICODETEXT, hmem).is_null() {
            GlobalFree(hmem);
            CloseClipboard();
            return Err(BvError::Clipboard("SetClipboardData failed".into()));
        }

        CloseClipboard();
    }
    Ok(())
}

/// Simule l'appui Ctrl+V pour coller le contenu du presse-papiers.
///
/// Utilise SendInput pour generer des evenements clavier synthetiques.
/// Attend 50ms avant la simulation pour laisser le temps a l'application
/// cible de se preparer (retour de focus apres fermeture du popup).
///
/// # Safety
/// Les appels a SendInput sont isoles dans un bloc unsafe local.
/// Les structures INPUT sont initialisees sur la pile (pas d'allocation).
pub fn simulate_paste() {
    // SAFETY: appels FFI Win32 pour SendInput.
    unsafe {
        // Petit delai pour que l'application cible soit prete
        Sleep(50);

        let inputs = [
            // Ctrl press
            INPUT {
                r#type: INPUT_KEYBOARD,
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL as u16,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                },
                padding: [0u8; 8],
            },
            // V press
            INPUT {
                r#type: INPUT_KEYBOARD,
                ki: KEYBDINPUT {
                    wVk: VK_V as u16,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                },
                padding: [0u8; 8],
            },
            // V release
            INPUT {
                r#type: INPUT_KEYBOARD,
                ki: KEYBDINPUT {
                    wVk: VK_V as u16,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
                padding: [0u8; 8],
            },
            // Ctrl release
            INPUT {
                r#type: INPUT_KEYBOARD,
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL as u16,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
                padding: [0u8; 8],
            },
        ];

        SendInput(
            4,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}

/// Ecrit du texte dans le presse-papiers puis simule Ctrl+V.
///
/// Combine `set_clipboard_text` et `simulate_paste` pour une
/// operation copier-coller complete en une seule etape.
///
/// # Arguments
/// * `hwnd` - Handle de la fenetre proprietaire du presse-papiers
/// * `text` - Texte a coller dans l'application cible
///
/// # Errors
/// Retourne `BvError::Clipboard` si l'ecriture dans le presse-papiers echoue.
pub fn paste_text(hwnd: HWND, text: &str) -> BvResult<()> {
    set_clipboard_text(hwnd, text)?;
    simulate_paste();
    Ok(())
}

#[cfg(test)]
mod tests {
    // Les tests d'injection clipboard necessitent un contexte Win32 complet
    // et ne peuvent pas etre executes en CI headless.
    // Les tests manuels sont decrits dans docs/TESTING.md.
}
