// BufferVault - Bindings et constantes Win32
// Declarations FFI pour les APIs Windows utilisees
//
// Ce fichier contient toutes les declarations FFI necessaires pour
// interagir avec les APIs Win32 depuis Rust, sans dependance externe.
//
// # Organisation
// 1. Types de base Win32 (HWND, HDC, HFONT, etc.)
// 2. Constantes de messages, styles, virtual keys
// 3. Structures (WNDCLASSEXW, MSG, RECT, PAINTSTRUCT, etc.)
// 4. Declarations FFI extern "system" par DLL (user32, kernel32, gdi32, etc.)
// 5. Fonctions helpers Rust (to_wstring, from_wstring, csprng_fill, etc.)
//
// # Safety
// Toutes les fonctions FFI sont marquees unsafe. Les wrappers safe
// sont fournis dans les modules de plus haut niveau (clipboard, system, ui).
//
// # Portabilite
// Ce module est specifique a Windows. Les types et constantes suivent
// les conventions Win32 (HWND, LPARAM, etc.).

#![allow(non_snake_case, non_camel_case_types, dead_code)]

use std::ffi::c_void;

// --- Types de base Win32 ---
pub type HANDLE = *mut c_void;
pub type HWND = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HMODULE = *mut c_void;
pub type HICON = *mut c_void;
pub type HCURSOR = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type HMENU = *mut c_void;
pub type HDC = *mut c_void;
pub type HFONT = *mut c_void;
pub type HGDIOBJ = *mut c_void;
pub type HBITMAP = *mut c_void;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type ATOM = u16;
pub type BOOL = i32;
pub type DWORD = u32;
pub type UINT = u32;
pub type COLORREF = u32;
pub type LPCWSTR = *const u16;
pub type LPWSTR = *mut u16;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
pub const NULL_HWND: HWND = std::ptr::null_mut();
pub const NULL_HANDLE: HANDLE = std::ptr::null_mut();

/// Cree un COLORREF a partir de composantes RGB.
pub const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

// --- Window Messages ---
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_HOTKEY: u32 = 0x0312;
pub const WM_CLIPBOARDUPDATE: u32 = 0x031D;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_CHAR: u32 = 0x0102;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
pub const WM_RBUTTONDOWN: u32 = 0x0204;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_MOUSEWHEEL: u32 = 0x020A;
pub const WM_KILLFOCUS: u32 = 0x0008;
pub const WM_ACTIVATE: u32 = 0x0006;
pub const WM_USER: u32 = 0x0400;
pub const WM_ENDSESSION: u32 = 0x0016;

// --- Window Styles ---
pub const WS_POPUP: u32 = 0x80000000;
pub const WS_VISIBLE: u32 = 0x10000000;
pub const WS_BORDER: u32 = 0x00800000;
pub const WS_THICKFRAME: u32 = 0x00040000;
pub const WS_CAPTION: u32 = 0x00C00000;
pub const WS_SYSMENU: u32 = 0x00080000;
pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;
pub const WS_EX_TOPMOST: u32 = 0x00000008;
pub const WS_EX_LAYERED: u32 = 0x00080000;
pub const WS_EX_NOACTIVATE: u32 = 0x08000000;
pub const WS_OVERLAPPED: u32 = 0x00000000;

// --- ShowWindow ---
pub const SW_HIDE: i32 = 0;
pub const SW_SHOW: i32 = 5;

// --- Layered Window ---
pub const LWA_ALPHA: u32 = 0x00000002;

// --- Virtual Keys ---
pub const VK_RETURN: u32 = 0x0D;
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_UP: u32 = 0x26;
pub const VK_DOWN: u32 = 0x28;
pub const VK_DELETE: u32 = 0x2E;
pub const VK_CONTROL: u32 = 0x11;
pub const VK_V: u32 = 0x56;
pub const VK_SPACE: u32 = 0x20;
pub const VK_F2: u32 = 0x71;
pub const VK_A: u32 = 0x41;

// --- Hotkey Modifiers ---
pub const MOD_ALT: u32 = 0x0001;
pub const MOD_CONTROL: u32 = 0x0002;
pub const MOD_SHIFT: u32 = 0x0004;
pub const MOD_WIN: u32 = 0x0008;
pub const MOD_NOREPEAT: u32 = 0x4000;

// --- Clipboard Formats ---
pub const CF_TEXT: u32 = 1;
pub const CF_UNICODETEXT: u32 = 13;
pub const CF_HDROP: u32 = 15;

// --- Cursor / Icon ---
pub const IDC_ARROW: LPCWSTR = 32512 as LPCWSTR;
pub const IDI_APPLICATION: LPCWSTR = 32512 as LPCWSTR;

// --- Class Styles ---
pub const CS_HREDRAW: u32 = 0x0002;
pub const CS_VREDRAW: u32 = 0x0001;
pub const CS_DBLCLKS: u32 = 0x0008;

// --- SetWindowPos ---
pub const HWND_TOPMOST: HWND = -1isize as HWND;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SWP_SHOWWINDOW: u32 = 0x0040;

// --- GDI ---
pub const TRANSPARENT: i32 = 1;
pub const DT_LEFT: u32 = 0x00000000;
pub const DT_SINGLELINE: u32 = 0x00000020;
pub const DT_VCENTER: u32 = 0x00000004;
pub const DT_END_ELLIPSIS: u32 = 0x00008000;
pub const DT_NOPREFIX: u32 = 0x00000800;
pub const DT_CENTER: u32 = 0x00000001;
pub const FW_NORMAL: i32 = 400;
pub const FW_BOLD: i32 = 700;
pub const DEFAULT_CHARSET: u32 = 1;
pub const CLEARTYPE_QUALITY: u32 = 5;

// --- Notify Icon ---
pub const NIM_ADD: u32 = 0x00000000;
pub const NIM_MODIFY: u32 = 0x00000001;
pub const NIM_DELETE: u32 = 0x00000002;
pub const NIF_MESSAGE: u32 = 0x00000001;
pub const NIF_ICON: u32 = 0x00000002;
pub const NIF_TIP: u32 = 0x00000004;

// --- TrackPopupMenu ---
pub const TPM_LEFTALIGN: u32 = 0x0000;
pub const TPM_BOTTOMALIGN: u32 = 0x0020;
pub const TPM_RETURNCMD: u32 = 0x0100;
pub const TPM_NONOTIFY: u32 = 0x0080;

// --- Menu ---
pub const MF_STRING: u32 = 0x00000000;
pub const MF_SEPARATOR: u32 = 0x00000800;
pub const MF_CHECKED: u32 = 0x00000008;

// --- SendInput ---
pub const INPUT_KEYBOARD: u32 = 1;
pub const KEYEVENTF_KEYUP: u32 = 0x0002;

// --- Process ---
pub const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

// --- System metrics ---
pub const SM_CXSCREEN: i32 = 0;
pub const SM_CYSCREEN: i32 = 1;

// --- DPAPI ---
pub const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x1;

// --- BCrypt ---
pub const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x00000002;

// --- Memory ---
pub const GMEM_MOVEABLE: u32 = 0x0002;
pub const GMEM_ZEROINIT: u32 = 0x0040;
pub const GHND: u32 = GMEM_MOVEABLE | GMEM_ZEROINIT;
pub const GWLP_USERDATA: i32 = -21;

pub const SRCCOPY: u32 = 0x00CC0020;

// --- MessageBox ---
pub const MB_OK: u32 = 0x00000000;
pub const MB_ICONINFORMATION: u32 = 0x00000040;

// --- LoadImage ---
pub const IMAGE_ICON: u32 = 1;
pub const LR_DEFAULTSIZE: u32 = 0x00000040;

// --- DrawIcon flags ---
pub const DI_NORMAL: u32 = 0x0003;

// --- MAKEINTRESOURCE ---
/// Convertit un ID de ressource entier en pointeur LPCWSTR.
pub const fn makeintresource(id: u16) -> LPCWSTR {
    id as usize as LPCWSTR
}

// --- Timer IDs ---
pub const TIMER_AUTOSAVE: usize = 1;

// --- Structures ---

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: LPCWSTR,
    pub lpszClassName: LPCWSTR,
    pub hIconSm: HICON,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt: POINT,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: BOOL,
    pub rcPaint: RECT,
    pub fRestore: BOOL,
    pub fIncUpdate: BOOL,
    pub rgbReserved: [u8; 32],
}

#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: HWND,
    pub uID: u32,
    pub uFlags: u32,
    pub uCallbackMessage: u32,
    pub hIcon: HICON,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub uVersion: u32,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: [u8; 16],
    pub hBalloonIcon: HICON,
}

#[repr(C)]
pub struct INPUT {
    pub r#type: u32,
    pub ki: KEYBDINPUT,
    pub padding: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KEYBDINPUT {
    pub wVk: u16,
    pub wScan: u16,
    pub dwFlags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}

#[repr(C)]
pub struct DATA_BLOB {
    pub cbData: u32,
    pub pbData: *mut u8,
}

#[repr(C)]
#[derive(Default)]
pub struct LOGFONTW {
    pub lfHeight: i32,
    pub lfWidth: i32,
    pub lfEscapement: i32,
    pub lfOrientation: i32,
    pub lfWeight: i32,
    pub lfItalic: u8,
    pub lfUnderline: u8,
    pub lfStrikeOut: u8,
    pub lfCharSet: u8,
    pub lfOutPrecision: u8,
    pub lfClipPrecision: u8,
    pub lfQuality: u8,
    pub lfPitchAndFamily: u8,
    pub lfFaceName: [u16; 32],
}

#[repr(C)]
#[derive(Default)]
pub struct SIZE {
    pub cx: i32,
    pub cy: i32,
}

// --- FFI user32 ---
#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> ATOM;
    pub fn CreateWindowExW(
        exStyle: u32, cls: LPCWSTR, name: LPCWSTR, style: u32,
        x: i32, y: i32, w: i32, h: i32,
        parent: HWND, menu: HMENU, inst: HINSTANCE, param: *mut c_void,
    ) -> HWND;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn ShowWindow(hWnd: HWND, cmd: i32) -> BOOL;
    pub fn UpdateWindow(hWnd: HWND) -> BOOL;
    pub fn SetWindowPos(
        hWnd: HWND, after: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32,
    ) -> BOOL;
    pub fn GetMessageW(msg: *mut MSG, hWnd: HWND, min: u32, max: u32) -> BOOL;
    pub fn TranslateMessage(msg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(msg: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(code: i32);
    pub fn PostMessageW(hWnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> BOOL;
    pub fn DefWindowProcW(hWnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn BeginPaint(hWnd: HWND, ps: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(hWnd: HWND, ps: *const PAINTSTRUCT) -> BOOL;
    pub fn InvalidateRect(hWnd: HWND, r: *const RECT, erase: BOOL) -> BOOL;
    pub fn GetClientRect(hWnd: HWND, r: *mut RECT) -> BOOL;
    pub fn SetLayeredWindowAttributes(hWnd: HWND, key: COLORREF, a: u8, f: u32) -> BOOL;
    pub fn RegisterHotKey(hWnd: HWND, id: i32, mods: u32, vk: u32) -> BOOL;
    pub fn UnregisterHotKey(hWnd: HWND, id: i32) -> BOOL;
    pub fn AddClipboardFormatListener(hWnd: HWND) -> BOOL;
    pub fn RemoveClipboardFormatListener(hWnd: HWND) -> BOOL;
    pub fn OpenClipboard(hWnd: HWND) -> BOOL;
    pub fn CloseClipboard() -> BOOL;
    pub fn EmptyClipboard() -> BOOL;
    pub fn GetClipboardData(fmt: u32) -> HANDLE;
    pub fn SetClipboardData(fmt: u32, hMem: HANDLE) -> HANDLE;
    pub fn IsClipboardFormatAvailable(fmt: u32) -> BOOL;
    pub fn GetForegroundWindow() -> HWND;
    pub fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    pub fn GetWindowThreadProcessId(hWnd: HWND, pid: *mut u32) -> u32;
    pub fn GetCursorPos(pt: *mut POINT) -> BOOL;
    pub fn SetTimer(hWnd: HWND, id: usize, ms: u32, func: *const c_void) -> usize;
    pub fn KillTimer(hWnd: HWND, id: usize) -> BOOL;
    pub fn SendInput(cnt: u32, inputs: *const INPUT, sz: i32) -> u32;
    pub fn LoadCursorW(inst: HINSTANCE, name: LPCWSTR) -> HCURSOR;
    pub fn LoadIconW(inst: HINSTANCE, name: LPCWSTR) -> HICON;
    pub fn GetSystemMetrics(idx: i32) -> i32;
    pub fn CreatePopupMenu() -> HMENU;
    pub fn DestroyMenu(hMenu: HMENU) -> BOOL;
    pub fn AppendMenuW(m: HMENU, f: u32, id: usize, s: LPCWSTR) -> BOOL;
    pub fn TrackPopupMenu(m: HMENU, f: u32, x: i32, y: i32, r: i32, hWnd: HWND, rc: *const RECT) -> BOOL;
    pub fn SetWindowLongPtrW(hWnd: HWND, idx: i32, val: isize) -> isize;
    pub fn GetWindowLongPtrW(hWnd: HWND, idx: i32) -> isize;
    pub fn GetDpiForWindow(hWnd: HWND) -> u32;
    pub fn SetFocus(hWnd: HWND) -> HWND;
    pub fn GetKeyState(vk: i32) -> i16;
    pub fn MessageBoxW(hWnd: HWND, text: LPCWSTR, caption: LPCWSTR, mtype: u32) -> i32;
    pub fn LoadImageW(
        hInst: HINSTANCE, name: LPCWSTR, r#type: u32,
        cx: i32, cy: i32, load: u32,
    ) -> HANDLE;
    pub fn DrawIconEx(
        hdc: HDC, xLeft: i32, yTop: i32, hIcon: HICON,
        cxWidth: i32, cyWidth: i32, istepIfAniCur: u32,
        hbrFlickerFreeDraw: HBRUSH, diFlags: u32,
    ) -> BOOL;
}

// --- FFI kernel32 ---
#[link(name = "kernel32")]
extern "system" {
    pub fn GetModuleHandleW(name: LPCWSTR) -> HMODULE;
    pub fn GetLastError() -> u32;
    pub fn GlobalAlloc(flags: u32, bytes: usize) -> HANDLE;
    pub fn GlobalLock(hMem: HANDLE) -> *mut c_void;
    pub fn GlobalUnlock(hMem: HANDLE) -> BOOL;
    pub fn GlobalSize(hMem: HANDLE) -> usize;
    pub fn GlobalFree(hMem: HANDLE) -> HANDLE;
    pub fn OpenProcess(access: u32, inherit: BOOL, pid: u32) -> HANDLE;
    pub fn CloseHandle(h: HANDLE) -> BOOL;
    pub fn QueryFullProcessImageNameW(h: HANDLE, f: u32, buf: LPWSTR, sz: *mut u32) -> BOOL;
    pub fn GetEnvironmentVariableW(name: LPCWSTR, buf: LPWSTR, sz: u32) -> u32;
    pub fn LocalFree(hMem: *mut c_void) -> *mut c_void;
    pub fn Sleep(ms: u32);
}

// --- FFI gdi32 ---
#[link(name = "gdi32")]
extern "system" {
    pub fn CreateFontIndirectW(lf: *const LOGFONTW) -> HFONT;
    pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteObject(h: HGDIOBJ) -> BOOL;
    pub fn SetTextColor(hdc: HDC, c: COLORREF) -> COLORREF;
    pub fn SetBkColor(hdc: HDC, c: COLORREF) -> COLORREF;
    pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    pub fn DrawTextW(hdc: HDC, s: LPCWSTR, n: i32, r: *mut RECT, f: u32) -> i32;
    pub fn CreateSolidBrush(c: COLORREF) -> HBRUSH;
    pub fn GetStockObject(i: i32) -> HGDIOBJ;
    pub fn FillRect(hdc: HDC, r: *const RECT, br: HBRUSH) -> i32;
    pub fn RoundRect(hdc: HDC, l: i32, t: i32, r: i32, b: i32, w: i32, h: i32) -> BOOL;
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn CreateCompatibleBitmap(hdc: HDC, w: i32, h: i32) -> HBITMAP;
    pub fn BitBlt(d: HDC, x: i32, y: i32, w: i32, h: i32, s: HDC, sx: i32, sy: i32, r: u32) -> BOOL;
    pub fn DeleteDC(hdc: HDC) -> BOOL;
    pub fn GetTextExtentPoint32W(hdc: HDC, s: LPCWSTR, n: i32, sz: *mut SIZE) -> BOOL;
    pub fn CreatePen(style: i32, width: i32, color: COLORREF) -> HGDIOBJ;
}

// --- FFI shell32 ---
#[link(name = "shell32")]
extern "system" {
    pub fn Shell_NotifyIconW(msg: u32, data: *mut NOTIFYICONDATAW) -> BOOL;
    pub fn DragQueryFileW(hDrop: HANDLE, idx: u32, buf: LPWSTR, sz: u32) -> u32;
}

// --- FFI crypt32 ---
#[link(name = "crypt32")]
extern "system" {
    pub fn CryptProtectData(
        din: *const DATA_BLOB, desc: LPCWSTR, ent: *const DATA_BLOB,
        res: *mut c_void, prompt: *mut c_void, flags: u32, dout: *mut DATA_BLOB,
    ) -> BOOL;
    pub fn CryptUnprotectData(
        din: *const DATA_BLOB, desc: *mut LPWSTR, ent: *const DATA_BLOB,
        res: *mut c_void, prompt: *mut c_void, flags: u32, dout: *mut DATA_BLOB,
    ) -> BOOL;
}

// --- FFI bcrypt ---
#[link(name = "bcrypt")]
extern "system" {
    pub fn BCryptGenRandom(alg: HANDLE, buf: *mut u8, sz: u32, flags: u32) -> i32;
}

// --- Helpers ---

/// Convertit un &str en Vec<u16> UTF-16 null-termine.
pub fn to_wstring(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Convertit un slice UTF-16 (possiblement null-termine) en String.
pub fn from_wstring(s: &[u16]) -> String {
    let len = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    String::from_utf16_lossy(&s[..len])
}

/// Recupere le dernier code d'erreur Win32.
pub fn last_error() -> u32 {
    // SAFETY: Fonction Win32 sans effet de bord dangereux.
    unsafe { GetLastError() }
}

/// Genere des octets aleatoires cryptographiquement surs.
pub fn csprng_fill(buf: &mut [u8]) -> bool {
    // SAFETY: BCryptGenRandom avec BCRYPT_USE_SYSTEM_PREFERRED_RNG est thread-safe.
    unsafe {
        BCryptGenRandom(NULL_HANDLE, buf.as_mut_ptr(), buf.len() as u32, BCRYPT_USE_SYSTEM_PREFERRED_RNG) == 0
    }
}

/// Recupere une variable d'environnement Windows.
pub fn get_env_var(name: &str) -> Option<String> {
    let wname = to_wstring(name);
    let mut buf = [0u16; 512];
    // SAFETY: Lecture seule de l'environnement.
    let len = unsafe { GetEnvironmentVariableW(wname.as_ptr(), buf.as_mut_ptr(), buf.len() as u32) };
    if len == 0 || len >= buf.len() as u32 { return None; }
    Some(from_wstring(&buf[..len as usize]))
}

/// Extrait le mot bas d'un LPARAM.
pub const fn loword_l(l: LPARAM) -> i16 { (l & 0xFFFF) as i16 }

/// Extrait le mot haut d'un LPARAM.
pub const fn hiword_l(l: LPARAM) -> i16 { ((l >> 16) & 0xFFFF) as i16 }

/// Extrait le mot haut d'un WPARAM (pour WM_MOUSEWHEEL).
pub const fn hiword_w(w: WPARAM) -> i16 { ((w >> 16) & 0xFFFF) as i16 }
