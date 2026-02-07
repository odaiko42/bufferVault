// BufferVault - Creation et gestion des fenetres Win32
// Fenetre cachee pour la boucle de messages + fenetres d'affichage
//
// Ce module fournit les abstractions de haut niveau pour la creation
// et la gestion des fenetres Win32 :
// - Enregistrement de classes de fenetres (RegisterClassExW)
// - Creation de fenetres cachees, popup et avec decoration
// - Positionnement, affichage/masquage, destruction
// - Recuperation des dimensions ecran et position du curseur
//
// # Safety
// Tous les appels Win32 sont isoles dans des blocs unsafe locaux.
// Les fonctions publiques retournent des BvResult pour signaler les erreurs.
//
// # Portabilite
// Ce module est specifique a Windows (Win32 API).

use crate::error::{BvError, BvResult};
use crate::system::win32::*;
use std::ffi::c_void;

/// Classe de fenetre pour la fenetre principale cachee.
pub const MAIN_CLASS: &str = "BufferVaultMain";

/// Classe de fenetre pour le popup de l'historique.
pub const POPUP_CLASS: &str = "BufferVaultPopup";

/// Classe de fenetre pour la sidebar.
pub const SIDEBAR_CLASS: &str = "BufferVaultSidebar";

/// Enregistre une classe de fenetre Win32.
///
/// # Arguments
/// * `class_name` - Nom de la classe
/// * `wndproc` - Callback de la fenetre
/// * `style` - Style de la classe (CS_HREDRAW | CS_VREDRAW par defaut)
pub fn register_class(
    class_name: &str,
    wndproc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
    style: u32,
) -> BvResult<ATOM> {
    let wclass = to_wstring(class_name);
    // SAFETY: appels FFI Win32 pour enregistrer la classe.
    let hinstance = unsafe { GetModuleHandleW(std::ptr::null()) };

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style,
        lpfnWndProc: Some(wndproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: unsafe { LoadIconW(std::ptr::null_mut(), IDI_APPLICATION) },
        hCursor: unsafe { LoadCursorW(std::ptr::null_mut(), IDC_ARROW) },
        hbrBackground: std::ptr::null_mut(),
        lpszMenuName: std::ptr::null(),
        lpszClassName: wclass.as_ptr(),
        hIconSm: std::ptr::null_mut(),
    };

    // SAFETY: la structure est correctement initialisee ci-dessus.
    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        return Err(BvError::Win32("RegisterClassExW failed".into(), last_error()));
    }
    Ok(atom)
}

/// Cree une fenetre cachee (message-only).
pub fn create_hidden_window(class_name: &str) -> BvResult<HWND> {
    let wclass = to_wstring(class_name);
    let wtitle = to_wstring("BufferVault");
    // SAFETY: appels FFI Win32.
    let hinstance = unsafe { GetModuleHandleW(std::ptr::null()) };

    let hwnd = unsafe {
        CreateWindowExW(
            0,
            wclass.as_ptr(),
            wtitle.as_ptr(),
            0, // Pas de style visible
            0, 0, 0, 0,
            NULL_HWND,
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null_mut(),
        )
    };

    if hwnd.is_null() {
        return Err(BvError::Win32("CreateWindowExW hidden failed".into(), last_error()));
    }
    Ok(hwnd)
}

/// Cree une fenetre popup pour l'affichage de l'historique.
///
/// # Arguments
/// * `class_name` - Nom de la classe enregistree
/// * `x`, `y` - Position en pixels
/// * `width`, `height` - Dimensions en pixels
/// * `user_data` - Pointeur optionnel stocke dans GWLP_USERDATA
pub fn create_popup_window(
    class_name: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    user_data: *mut c_void,
) -> BvResult<HWND> {
    let wclass = to_wstring(class_name);
    let wtitle = to_wstring("BufferVault");
    // SAFETY: appels FFI Win32.
    let hinstance = unsafe { GetModuleHandleW(std::ptr::null()) };

    let ex_style = WS_EX_TOOLWINDOW | WS_EX_TOPMOST;
    let style = WS_POPUP | WS_BORDER;

    let hwnd = unsafe {
        CreateWindowExW(
            ex_style,
            wclass.as_ptr(),
            wtitle.as_ptr(),
            style,
            x, y, width, height,
            NULL_HWND,
            std::ptr::null_mut(),
            hinstance,
            user_data,
        )
    };

    if hwnd.is_null() {
        return Err(BvError::Win32("CreateWindowExW popup failed".into(), last_error()));
    }

    // Stocker le user_data
    if !user_data.is_null() {
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, user_data as isize) };
    }

    Ok(hwnd)
}

/// Affiche une fenetre.
pub fn show_window(hwnd: HWND) {
    // SAFETY: appel FFI Win32.
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    };
}

/// Cache une fenetre.
pub fn hide_window(hwnd: HWND) {
    // SAFETY: appel FFI Win32.
    unsafe { ShowWindow(hwnd, SW_HIDE) };
}

/// Positionne une fenetre au premier plan.
pub fn set_topmost(hwnd: HWND, x: i32, y: i32, w: i32, h: i32) {
    // SAFETY: appel FFI Win32.
    unsafe {
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            x, y, w, h,
            SWP_SHOWWINDOW,
        );
    }
}

/// Detruit une fenetre.
pub fn destroy(hwnd: HWND) {
    if !hwnd.is_null() {
        // SAFETY: appel FFI Win32.
        unsafe { DestroyWindow(hwnd) };
    }
}

/// Recupere les dimensions de l'ecran principal.
pub fn screen_size() -> (i32, i32) {
    // SAFETY: appel FFI Win32 sans effet de bord dangereux.
    unsafe {
        (
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
        )
    }
}

/// Recupere la position du curseur.
pub fn cursor_pos() -> (i32, i32) {
    let mut pt = POINT::default();
    // SAFETY: appel FFI Win32.
    unsafe { GetCursorPos(&mut pt) };
    (pt.x, pt.y)
}

/// Recupere le pointeur user_data associe a une fenetre.
pub fn get_user_data<T>(hwnd: HWND) -> *mut T {
    // SAFETY: appel FFI Win32.
    unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut T }
}

/// Force le repaint de la fenetre.
pub fn invalidate(hwnd: HWND) {
    // SAFETY: appel FFI Win32.
    unsafe { InvalidateRect(hwnd, std::ptr::null(), FALSE) };
}
