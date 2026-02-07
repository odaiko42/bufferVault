// BufferVault - Icone de notification systeme (tray icon)
// Gestion de la zone de notification Windows
//
// Ce module gere l'icone de BufferVault dans la zone de notification
// (system tray) : ajout, mise a jour du tooltip, retrait et affichage
// du menu contextuel.
//
// # Safety
// Tous les appels Win32 (Shell_NotifyIconW, CreatePopupMenu, etc.)
// sont isoles dans des blocs unsafe locaux. Les handles de menu sont
// detruits dans le meme scope que leur creation.
//
// # Portabilite
// Ce module est specifique a Windows (Shell_NotifyIconW, TrackPopupMenu).

use crate::constants::{TRAY_ICON_ID, WM_TRAY_CALLBACK};
use crate::error::{BvError, BvResult};
use crate::system::win32::*;

/// Ajoute l'icone de notification dans la zone de notification.
///
/// Charge l'icone depuis les ressources embarquees du binaire (ID=1).
/// En cas d'echec, utilise l'icone systeme par defaut (IDI_APPLICATION).
///
/// # Arguments
/// * `hwnd` - Handle de la fenetre receptrice des messages tray
/// * `tooltip` - Texte du tooltip affiche au survol de l'icone
///
/// # Errors
/// Retourne `BvError::Win32` si Shell_NotifyIconW echoue.
pub fn add_tray_icon(hwnd: HWND, tooltip: &str) -> BvResult<()> {
    let mut nid = create_nid(hwnd);
    set_tooltip(&mut nid, tooltip);

    // Charger l'icone custom depuis les ressources embarquees (ID=1)
    // SAFETY: appel FFI Win32 pour charger l'icone depuis les ressources du binaire.
    let icon = unsafe {
        let hinst = GetModuleHandleW(std::ptr::null());
        LoadImageW(
            hinst,
            makeintresource(1),
            IMAGE_ICON,
            0, 0,
            LR_DEFAULTSIZE,
        ) as HICON
    };

    // Fallback sur l'icone systeme si la ressource n'est pas trouvee
    nid.hIcon = if icon.is_null() {
        unsafe { LoadIconW(std::ptr::null_mut(), IDI_APPLICATION) }
    } else {
        icon
    };

    // SAFETY: appel FFI Win32 pour ajouter l'icone tray.
    let ok = unsafe { Shell_NotifyIconW(NIM_ADD, &mut nid) };
    if ok == FALSE {
        return Err(BvError::Win32("Shell_NotifyIconW NIM_ADD failed".into(), last_error()));
    }
    Ok(())
}

/// Met a jour le tooltip de l'icone de notification.
pub fn update_tray_tooltip(hwnd: HWND, tooltip: &str) -> BvResult<()> {
    let mut nid = create_nid(hwnd);
    set_tooltip(&mut nid, tooltip);
    nid.uFlags = NIF_TIP;

    // SAFETY: appel FFI Win32.
    let ok = unsafe { Shell_NotifyIconW(NIM_MODIFY, &mut nid) };
    if ok == FALSE {
        return Err(BvError::Win32("Shell_NotifyIconW NIM_MODIFY failed".into(), last_error()));
    }
    Ok(())
}

/// Retire l'icone de notification.
pub fn remove_tray_icon(hwnd: HWND) {
    let mut nid = create_nid(hwnd);
    // SAFETY: appel FFI Win32.
    unsafe { Shell_NotifyIconW(NIM_DELETE, &mut nid) };
}

/// Affiche le menu contextuel de l'icone tray.
///
/// Cree un menu popup Win32, y ajoute les elements specifies, puis
/// l'affiche a la position du curseur. Le menu est modal (bloquant).
///
/// # Arguments
/// * `hwnd` - Handle de la fenetre proprietaire du menu
/// * `items` - Tableau de tuples (label, id, checked).
///   Un label vide insere un separateur. Le flag `checked` ajoute
///   une coche visuelle devant l'element.
///
/// # Returns
/// L'ID de la commande selectionnee (0 si l'utilisateur annule).
pub fn show_tray_menu(hwnd: HWND, items: &[(&str, u16, bool)]) -> u16 {
    // SAFETY: appels FFI Win32 pour le menu popup.
    unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return 0;
        }

        for (label, id, checked) in items {
            if label.is_empty() {
                AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
            } else {
                let flags = if *checked { MF_STRING | MF_CHECKED } else { MF_STRING };
                let wlabel = to_wstring(label);
                AppendMenuW(menu, flags, *id as usize, wlabel.as_ptr());
            }
        }

        // Position du curseur
        let mut pt = POINT::default();
        GetCursorPos(&mut pt);

        // Forcer la fenetre au premier plan pour que le menu se ferme correctement
        SetForegroundWindow(hwnd);

        let cmd = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_NONOTIFY | TPM_LEFTALIGN | TPM_BOTTOMALIGN,
            pt.x,
            pt.y,
            0,
            hwnd,
            std::ptr::null(),
        );

        DestroyMenu(menu);

        // Forcer la fermeture du menu en postant WM_NULL
        PostMessageW(hwnd, 0, 0, 0);

        cmd as u16
    }
}

/// Cree une structure NOTIFYICONDATAW initialisee.
fn create_nid(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAY_CALLBACK,
        hIcon: std::ptr::null_mut(),
        szTip: [0u16; 128],
        dwState: 0,
        dwStateMask: 0,
        szInfo: [0u16; 256],
        uVersion: 0,
        szInfoTitle: [0u16; 64],
        dwInfoFlags: 0,
        guidItem: [0u8; 16],
        hBalloonIcon: std::ptr::null_mut(),
    }
}

/// Ecrit le tooltip dans la structure NOTIFYICONDATAW.
fn set_tooltip(nid: &mut NOTIFYICONDATAW, tooltip: &str) {
    let wtext = to_wstring(tooltip);
    let max = nid.szTip.len() - 1;
    let copy_len = wtext.len().min(max);
    nid.szTip[..copy_len].copy_from_slice(&wtext[..copy_len]);
    nid.szTip[copy_len] = 0;
}
