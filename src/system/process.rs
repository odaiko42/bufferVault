// BufferVault - Detection du processus actif
// Identifie l'application au premier plan pour le champ source_app
//
// Ce module detecte le processus qui possede la fenetre au premier plan
// au moment d'une copie, afin de renseigner le champ `source_app` dans
// chaque ClipboardEntry.
//
// # Safety
// Les appels FFI Win32 (GetForegroundWindow, OpenProcess, etc.) sont
// isoles dans des blocs unsafe locaux. Le handle de processus est
// ferme dans le meme scope que son ouverture.
//
// # Portabilite
// Ce module est specifique a Windows (Win32 process API).

use crate::system::win32::*;

/// Retourne le nom de l'executable de la fenetre au premier plan.
///
/// Utilise la sequence GetForegroundWindow -> GetWindowThreadProcessId
/// -> OpenProcess -> QueryFullProcessImageNameW pour obtenir le chemin
/// complet, puis extrait le nom de fichier.
///
/// # Returns
/// Le nom du fichier executable en minuscules (ex: "notepad.exe").
/// Retourne "unknown" si la detection echoue a n'importe quelle etape.
pub fn get_foreground_process_name() -> String {
    // SAFETY: appels FFI Win32 pour identifier le processus actif.
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return "unknown".into();
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return "unknown".into();
        }

        let proc_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
        if proc_handle.is_null() {
            return "unknown".into();
        }

        let mut buf = [0u16; 512];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(proc_handle, 0, buf.as_mut_ptr(), &mut size);
        CloseHandle(proc_handle);

        if ok == FALSE || size == 0 {
            return "unknown".into();
        }

        let full_path = from_wstring(&buf[..size as usize]);
        extract_filename(&full_path)
    }
}

/// Extrait le nom de fichier d'un chemin complet Windows.
///
/// Recherche le dernier separateur '\\' et retourne tout ce qui suit,
/// converti en minuscules pour une comparaison insensible a la casse.
///
/// # Examples
/// ```ignore
/// assert_eq!(extract_filename("C:\\Windows\\notepad.exe"), "notepad.exe");
/// ```
fn extract_filename(path: &str) -> String {
    path.rsplit('\\')
        .next()
        .unwrap_or("unknown")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename() {
        assert_eq!(extract_filename("C:\\Windows\\System32\\notepad.exe"), "notepad.exe");
        assert_eq!(extract_filename("notepad.exe"), "notepad.exe");
        assert_eq!(extract_filename(""), "");
    }

    #[test]
    fn test_foreground_process_no_panic() {
        // Verifie que la fonction ne panique pas meme sans contexte Win32 complet
        let _name = get_foreground_process_name();
    }
}
