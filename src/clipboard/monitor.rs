// BufferVault - Surveillance du presse-papiers
// Utilise AddClipboardFormatListener pour detecter les changements.
//
// Ce module fournit les fonctions de surveillance du presse-papiers :
// - Enregistrement/desenregistrement du listener Win32
// - Lecture du contenu texte (CF_UNICODETEXT, CF_TEXT) et fichiers (CF_HDROP)
// - Detection du format disponible et creation de ClipboardEntry
//
// # Safety
// Tous les appels Win32 sont isoles dans des blocs unsafe locaux.
// La sequence OpenClipboard/lecture/CloseClipboard est garantie dans
// chaque fonction de lecture pour eviter les fuites de ressources.
//
// # Portabilite
// Ce module est specifique a Windows (Win32 API).

use crate::error::{BvError, BvResult};
use crate::history::entry::{ClipboardEntry, EntryType};
use crate::system::win32::*;

/// Enregistre la fenetre comme ecouteur du presse-papiers.
///
/// Apres enregistrement, la fenetre recevra le message WM_CLIPBOARDUPDATE
/// a chaque modification du presse-papiers par une application quelconque.
///
/// # Arguments
/// * `hwnd` - Handle de la fenetre qui recevra les notifications
///
/// # Errors
/// Retourne `BvError::Clipboard` si l'enregistrement echoue.
pub fn register_listener(hwnd: HWND) -> BvResult<()> {
    // SAFETY: appel FFI Win32. hwnd doit etre un handle de fenetre valide.
    let ok = unsafe { AddClipboardFormatListener(hwnd) };
    if ok == FALSE {
        return Err(BvError::Clipboard("AddClipboardFormatListener failed".into()));
    }
    Ok(())
}

/// Desenregistre l'ecouteur du presse-papiers.
pub fn unregister_listener(hwnd: HWND) {
    // SAFETY: appel FFI Win32.
    unsafe { RemoveClipboardFormatListener(hwnd) };
}

/// Lit le contenu texte du presse-papiers.
///
/// Tente d'abord CF_UNICODETEXT (UTF-16), puis CF_TEXT (ANSI) en fallback.
/// Retourne None si le presse-papiers ne contient pas de texte ou si
/// le contenu est vide.
///
/// # Arguments
/// * `hwnd` - Handle de la fenetre proprietaire pour OpenClipboard
///
/// # Safety
/// La sequence OpenClipboard/lecture/CloseClipboard est garantie.
pub fn read_clipboard_text(hwnd: HWND) -> Option<String> {
    // SAFETY: sequence d'appels FFI Win32 pour le clipboard.
    unsafe {
        if OpenClipboard(hwnd) == FALSE {
            return None;
        }
        let result = read_text_inner();
        CloseClipboard();
        result
    }
}

/// Lecture interne du texte (doit etre appelee entre Open/CloseClipboard).
unsafe fn read_text_inner() -> Option<String> {
    // Privilegier CF_UNICODETEXT
    if IsClipboardFormatAvailable(CF_UNICODETEXT) != FALSE {
        let hdata = GetClipboardData(CF_UNICODETEXT);
        if hdata.is_null() {
            return None;
        }
        let ptr = GlobalLock(hdata);
        if ptr.is_null() {
            return None;
        }
        let size_bytes = GlobalSize(hdata);
        let len_u16 = size_bytes / 2;
        let slice = std::slice::from_raw_parts(ptr as *const u16, len_u16);
        let text = from_wstring(slice);
        GlobalUnlock(hdata);
        if text.is_empty() { None } else { Some(text) }
    } else if IsClipboardFormatAvailable(CF_TEXT) != FALSE {
        let hdata = GetClipboardData(CF_TEXT);
        if hdata.is_null() {
            return None;
        }
        let ptr = GlobalLock(hdata);
        if ptr.is_null() {
            return None;
        }
        let size_bytes = GlobalSize(hdata);
        let slice = std::slice::from_raw_parts(ptr as *const u8, size_bytes);
        let end = slice.iter().position(|&b| b == 0).unwrap_or(size_bytes);
        let text = String::from_utf8_lossy(&slice[..end]).to_string();
        GlobalUnlock(hdata);
        if text.is_empty() { None } else { Some(text) }
    } else {
        None
    }
}

/// Lit les fichiers deposes (CF_HDROP) depuis le presse-papiers.
pub fn read_clipboard_files(hwnd: HWND) -> Option<String> {
    // SAFETY: sequence d'appels FFI Win32 pour le clipboard.
    unsafe {
        if OpenClipboard(hwnd) == FALSE {
            return None;
        }
        let result = read_files_inner();
        CloseClipboard();
        result
    }
}

/// Lecture interne des fichiers (doit etre appelee entre Open/CloseClipboard).
unsafe fn read_files_inner() -> Option<String> {
    if IsClipboardFormatAvailable(CF_HDROP) == FALSE {
        return None;
    }
    let hdata = GetClipboardData(CF_HDROP);
    if hdata.is_null() {
        return None;
    }
    // Nombre de fichiers
    let count = DragQueryFileW(hdata, 0xFFFFFFFF, std::ptr::null_mut(), 0);
    if count == 0 {
        return None;
    }
    let mut lines = Vec::with_capacity(count as usize);
    for i in 0..count {
        let mut buf = [0u16; 512];
        let len = DragQueryFileW(hdata, i, buf.as_mut_ptr(), buf.len() as u32);
        if len > 0 {
            lines.push(from_wstring(&buf[..len as usize]));
        }
    }
    if lines.is_empty() { None } else { Some(lines.join("\n")) }
}

/// Detecte le type de contenu disponible sur le presse-papiers.
pub fn detect_clipboard_format() -> Option<EntryType> {
    // SAFETY: appels FFI Win32 sans effet de bord dangereux.
    unsafe {
        if IsClipboardFormatAvailable(CF_HDROP) != FALSE {
            Some(EntryType::FileDrop)
        } else if IsClipboardFormatAvailable(CF_UNICODETEXT) != FALSE {
            Some(EntryType::Text)
        } else if IsClipboardFormatAvailable(CF_TEXT) != FALSE {
            Some(EntryType::PlainText)
        } else {
            None
        }
    }
}

/// Lit le contenu du presse-papiers et cree une ClipboardEntry.
///
/// Detecte automatiquement le format disponible (fichiers, texte Unicode,
/// texte ANSI) et lit le contenu correspondant. Refuse les entrees
/// depassant la taille maximale configuree.
///
/// # Arguments
/// * `hwnd` - Handle de la fenetre pour l'acces au presse-papiers
/// * `source_app` - Nom de l'application qui a modifie le presse-papiers
///
/// # Returns
/// `Some(ClipboardEntry)` si le contenu a ete capture, `None` sinon.
pub fn capture_clipboard(hwnd: HWND, source_app: String) -> Option<ClipboardEntry> {
    let format = detect_clipboard_format()?;

    let content = match format {
        EntryType::FileDrop => read_clipboard_files(hwnd)?,
        EntryType::Text | EntryType::PlainText => read_clipboard_text(hwnd)?,
    };

    // Limiter la taille du contenu
    if content.len() > crate::constants::DEFAULT_MAX_ENTRY_SIZE {
        return None;
    }

    Some(ClipboardEntry::new(format, source_app, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_returns_option() {
        // Ce test verifie uniquement que la fonction ne panique pas
        let _ = detect_clipboard_format();
    }
}
