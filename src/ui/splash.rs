// BufferVault - Splash screen au demarrage
// Affiche l'icone et le nom de l'application au centre de l'ecran
// avec une legere transparence, puis disparait progressivement.
//
// Ce module implemente l'ecran de demarrage de BufferVault :
// 1. Creation d'une fenetre layered (transparente) au centre de l'ecran
// 2. Affichage de l'icone, du titre, de la version et de l'auteur
// 3. Attente de 2 secondes (timer TIMER_SPLASH_WAIT)
// 4. Fade-out progressif sur ~500ms (timer TIMER_SPLASH_FADE)
// 5. Destruction automatique quand l'opacite atteint 0
//
// # Safety
// Tous les appels Win32 sont isoles dans des blocs unsafe locaux.
// Les objets GDI (polices, bitmaps) sont crees et detruits dans le scope.
//
// # Portabilite
// Ce module est specifique a Windows (Win32 Layered Windows, GDI).

use crate::system::win32::*;
use crate::ui::dpi::DpiContext;
use crate::ui::window;

/// Classe de fenetre pour le splash screen.
pub const SPLASH_CLASS: &str = "BufferVaultSplash";

/// Timer ID pour le delai d'affichage initial (2 secondes).
const TIMER_SPLASH_WAIT: usize = 100;

/// Timer ID pour le fade-out progressif.
const TIMER_SPLASH_FADE: usize = 101;

/// Duree d'affichage initial (ms).
const SPLASH_DISPLAY_MS: u32 = 2000;

/// Intervalle du timer de fade-out (ms).
const FADE_INTERVAL_MS: u32 = 30;

/// Opacite initiale (0-255). Legerement transparent.
const INITIAL_ALPHA: u8 = 220;

/// Decrement d'opacite a chaque pas du fade (~500ms / 30ms = ~17 pas).
const FADE_STEP: u8 = 14;

/// Largeur du splash (pixels logiques).
const SPLASH_WIDTH_BASE: i32 = 320;

/// Hauteur du splash (pixels logiques).
const SPLASH_HEIGHT_BASE: i32 = 200;

/// Taille de l'icone (pixels logiques).
const ICON_SIZE_BASE: i32 = 64;

/// Etat du splash screen.
///
/// Gere la fenetre layered avec transparence progressive.
/// Le cycle de vie est : affichage -> attente 2s -> fade-out -> destruction.
/// Le champ `alpha` diminue progressivement de INITIAL_ALPHA (220) a 0.
pub struct SplashState {
    /// Handle de la fenetre splash
    hwnd: HWND,
    /// Opacite courante
    alpha: u8,
    /// Icone chargee
    icon: HICON,
}

impl SplashState {
    /// Cree et affiche le splash screen.
    /// Retourne le handle pour la gestion des messages dans la boucle principale.
    pub fn show(dpi: &DpiContext) -> Self {
        let width = dpi.scale_i32(SPLASH_WIDTH_BASE);
        let height = dpi.scale_i32(SPLASH_HEIGHT_BASE);

        let (sw, sh) = window::screen_size();
        let x = (sw - width) / 2;
        let y = (sh - height) / 2;

        // Charger l'icone depuis les ressources
        // SAFETY: appels FFI Win32.
        let (icon, hinstance) = unsafe {
            let hinst = GetModuleHandleW(std::ptr::null());
            let icon_size = dpi.scale_i32(ICON_SIZE_BASE);
            let ico = LoadImageW(
                hinst,
                makeintresource(1),
                IMAGE_ICON,
                icon_size, icon_size,
                0,
            ) as HICON;
            (ico, hinst)
        };

        let wclass = to_wstring(SPLASH_CLASS);
        let wtitle = to_wstring("BufferVault");
        let ex_style = WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_LAYERED;
        let style = WS_POPUP;

        // SAFETY: appels FFI Win32 pour creer la fenetre splash.
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
                std::ptr::null_mut(),
            )
        };

        let state = Self {
            hwnd,
            alpha: INITIAL_ALPHA,
            icon,
        };

        if !hwnd.is_null() {
            // SAFETY: appels FFI Win32 pour configurer la transparence et afficher.
            unsafe {
                SetLayeredWindowAttributes(hwnd, 0, INITIAL_ALPHA, LWA_ALPHA);
                ShowWindow(hwnd, SW_SHOW);
                UpdateWindow(hwnd);
                // Timer : attendre 2 secondes avant de commencer le fade-out
                SetTimer(hwnd, TIMER_SPLASH_WAIT, SPLASH_DISPLAY_MS, std::ptr::null());
            }
        }

        state
    }

    /// Retourne le handle de la fenetre.
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Retourne l'icone chargee.
    pub fn icon(&self) -> HICON {
        self.icon
    }

    /// Gere le timer du splash.
    /// Retourne true si le splash doit etre detruit.
    pub fn on_timer(&mut self, timer_id: usize) -> bool {
        match timer_id {
            TIMER_SPLASH_WAIT => {
                // Fin de l'attente initiale, demarrer le fade-out
                // SAFETY: appels FFI Win32.
                unsafe {
                    KillTimer(self.hwnd, TIMER_SPLASH_WAIT);
                    SetTimer(self.hwnd, TIMER_SPLASH_FADE, FADE_INTERVAL_MS, std::ptr::null());
                }
                false
            }
            TIMER_SPLASH_FADE => {
                // Reduire progressivement l'opacite
                if self.alpha <= FADE_STEP {
                    self.alpha = 0;
                    // SAFETY: appels FFI Win32.
                    unsafe {
                        KillTimer(self.hwnd, TIMER_SPLASH_FADE);
                        ShowWindow(self.hwnd, SW_HIDE);
                        DestroyWindow(self.hwnd);
                    }
                    self.hwnd = NULL_HWND;
                    return true;
                }
                self.alpha -= FADE_STEP;
                // SAFETY: appel FFI Win32.
                unsafe {
                    SetLayeredWindowAttributes(self.hwnd, 0, self.alpha, LWA_ALPHA);
                }
                false
            }
            _ => false,
        }
    }

    /// Dessine le contenu du splash screen.
    pub fn paint(&self) {
        if self.hwnd.is_null() {
            return;
        }
        // SAFETY: appels FFI Win32 GDI.
        unsafe {
            let mut ps = std::mem::zeroed::<PAINTSTRUCT>();
            let hdc = BeginPaint(self.hwnd, &mut ps);
            if hdc.is_null() { return; }

            let mut rc = RECT::default();
            GetClientRect(self.hwnd, &mut rc);
            let width = rc.right;
            let height = rc.bottom;

            // Double buffering
            let mem_dc = CreateCompatibleDC(hdc);
            let bmp = CreateCompatibleBitmap(hdc, width, height);
            let old_bmp = SelectObject(mem_dc, bmp as HGDIOBJ);

            // Fond sombre avec bordure
            let bg = rgb(28, 28, 32);
            let bg_brush = CreateSolidBrush(bg);
            FillRect(mem_dc, &rc, bg_brush);
            DeleteObject(bg_brush as HGDIOBJ);

            // Bordure exterieure subtile
            let border_color = rgb(0, 100, 180);
            let border_brush = CreateSolidBrush(border_color);
            // Haut
            let r = RECT { left: 0, top: 0, right: width, bottom: 2 };
            FillRect(mem_dc, &r, border_brush);
            // Bas
            let r = RECT { left: 0, top: height - 2, right: width, bottom: height };
            FillRect(mem_dc, &r, border_brush);
            // Gauche
            let r = RECT { left: 0, top: 0, right: 2, bottom: height };
            FillRect(mem_dc, &r, border_brush);
            // Droite
            let r = RECT { left: width - 2, top: 0, right: width, bottom: height };
            FillRect(mem_dc, &r, border_brush);
            DeleteObject(border_brush as HGDIOBJ);

            // Dessiner l'icone au centre-haut
            let icon_size = 64; // Taille fixe pour le rendu
            let icon_x = (width - icon_size) / 2;
            let icon_y = 24;
            if !self.icon.is_null() {
                DrawIconEx(
                    mem_dc, icon_x, icon_y,
                    self.icon,
                    icon_size, icon_size,
                    0, std::ptr::null_mut(), DI_NORMAL,
                );
            }

            SetBkMode(mem_dc, TRANSPARENT);

            // Titre "BufferVault"
            let title_font = create_splash_font(-24, FW_BOLD);
            let old_font = SelectObject(mem_dc, title_font as HGDIOBJ);
            SetTextColor(mem_dc, rgb(230, 230, 240));
            let wtitle = to_wstring("BufferVault");
            let mut title_rect = RECT {
                left: 0, top: icon_y + icon_size + 12,
                right: width, bottom: icon_y + icon_size + 48,
            };
            DrawTextW(mem_dc, wtitle.as_ptr(), -1, &mut title_rect,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);
            SelectObject(mem_dc, old_font);
            DeleteObject(title_font as HGDIOBJ);

            // Sous-titre
            let sub_font = create_splash_font(-13, FW_NORMAL);
            let old_font = SelectObject(mem_dc, sub_font as HGDIOBJ);
            SetTextColor(mem_dc, rgb(140, 150, 170));
            let version = env!("CARGO_PKG_VERSION");
            let subtitle = format!("Clipboard History Manager v{}", version);
            let wsub = to_wstring(&subtitle);
            let mut sub_rect = RECT {
                left: 0, top: title_rect.bottom + 4,
                right: width, bottom: title_rect.bottom + 24,
            };
            DrawTextW(mem_dc, wsub.as_ptr(), -1, &mut sub_rect,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);

            // Auteur
            SetTextColor(mem_dc, rgb(110, 120, 140));
            let wauthor = to_wstring("par Emmanuel Forgues");
            let mut author_rect = RECT {
                left: 0, top: sub_rect.bottom + 8,
                right: width, bottom: sub_rect.bottom + 28,
            };
            DrawTextW(mem_dc, wauthor.as_ptr(), -1, &mut author_rect,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);

            SelectObject(mem_dc, old_font);
            DeleteObject(sub_font as HGDIOBJ);

            // Copie vers l'ecran
            BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

            SelectObject(mem_dc, old_bmp);
            DeleteObject(bmp as HGDIOBJ);
            DeleteDC(mem_dc);

            EndPaint(self.hwnd, &ps);
        }
    }
}

/// Cree une police pour le splash screen.
fn create_splash_font(height: i32, weight: i32) -> HFONT {
    let face = to_wstring("Segoe UI");
    let mut lf = LOGFONTW::default();
    lf.lfHeight = height;
    lf.lfWeight = weight;
    lf.lfCharSet = DEFAULT_CHARSET as u8;
    lf.lfQuality = CLEARTYPE_QUALITY as u8;
    let copy_len = face.len().min(lf.lfFaceName.len());
    lf.lfFaceName[..copy_len].copy_from_slice(&face[..copy_len]);
    // SAFETY: structure correctement initialisee.
    unsafe { CreateFontIndirectW(&lf) }
}
