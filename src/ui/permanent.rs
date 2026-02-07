// BufferVault - Mode fenetre permanente
// Fenetre classique avec barre de titre, redimensionnable
//
// Ce module implemente le mode permanent : une fenetre classique Win32
// avec barre de titre et bordure redimensionnable qui affiche l'historique.
// Contrairement au popup, elle reste visible independamment du focus.
//
// # Fonctionnalites
// - Fenetre avec barre de titre et bouton systeme
// - Redimensionnement libre (WS_THICKFRAME)
// - Navigation clavier haut/bas
// - Bascule via le raccourci clavier global
//
// # Safety
// Tous les appels Win32 sont isoles dans des blocs unsafe locaux.
//
// # Portabilite
// Ce module est specifique a Windows (Win32 GDI).

use crate::history::entry::ClipboardEntry;
use crate::ui::dpi::DpiContext;
use crate::ui::renderer::{self, RenderContext};
use crate::ui::theme::ThemePalette;
use crate::ui::window;
use crate::system::win32::*;

/// Etat de la fenetre permanente.
///
/// Contient le handle de fenetre, la position de selection, le defilement
/// et le contexte de rendu GDI. La fenetre permanente est une fenetre
/// classique Win32 avec barre de titre et redimensionnement libre.
pub struct PermanentState {
    /// Handle de la fenetre
    pub hwnd: HWND,
    /// Index de l'element selectionne
    pub selected: usize,
    /// Offset de defilement
    pub scroll_offset: usize,
    /// Contexte de rendu
    pub render_ctx: Option<RenderContext>,
    /// Est-ce que la fenetre est visible ?
    pub visible: bool,
}

impl PermanentState {
    /// Cree un nouvel etat de fenetre permanente.
    pub fn new() -> Self {
        Self {
            hwnd: NULL_HWND,
            selected: 0,
            scroll_offset: 0,
            render_ctx: None,
            visible: false,
        }
    }

    /// Initialise la fenetre permanente (style classique avec titre).
    pub fn create_window(&mut self, dpi: &DpiContext) {
        let width = dpi.scale_i32(400);
        let height = dpi.scale_i32(600);
        let (sw, _sh) = window::screen_size();
        let x = (sw - width) / 2;
        let y = 100;

        let wclass = to_wstring("BufferVaultPermanent");
        let wtitle = to_wstring("BufferVault - Historique");
        // SAFETY: appels FFI Win32 pour creer la fenetre.
        let hinstance = unsafe { GetModuleHandleW(std::ptr::null()) };

        let style = WS_CAPTION | WS_SYSMENU | WS_THICKFRAME;
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW,
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

        if hwnd.is_null() {
            eprintln!("Failed to create permanent window");
            return;
        }

        self.hwnd = hwnd;
        self.render_ctx = Some(RenderContext::new(dpi));
    }

    /// Affiche ou cache la fenetre.
    pub fn toggle(&mut self) {
        if self.hwnd.is_null() {
            return;
        }
        if self.visible {
            window::hide_window(self.hwnd);
            self.visible = false;
        } else {
            window::show_window(self.hwnd);
            self.visible = true;
            window::invalidate(self.hwnd);
        }
    }

    /// Deplace la selection vers le haut.
    pub fn move_up(&mut self, entries_len: usize) {
        if entries_len == 0 || self.selected == 0 {
            return;
        }
        self.selected -= 1;
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
        window::invalidate(self.hwnd);
    }

    /// Deplace la selection vers le bas.
    pub fn move_down(&mut self, entries_len: usize) {
        if entries_len == 0 || self.selected + 1 >= entries_len {
            return;
        }
        self.selected += 1;
        let visible = self.visible_count();
        if self.selected >= self.scroll_offset + visible {
            self.scroll_offset = self.selected - visible + 1;
        }
        window::invalidate(self.hwnd);
    }

    /// Calcule le nombre d'elements visibles.
    fn visible_count(&self) -> usize {
        if self.hwnd.is_null() {
            return 10;
        }
        let mut rect = RECT::default();
        // SAFETY: appel FFI Win32.
        unsafe { GetClientRect(self.hwnd, &mut rect) };
        let height = rect.bottom - rect.top;
        let item_h = renderer::ITEM_HEIGHT_BASE;
        (height / item_h).max(1) as usize
    }

    /// Dessine la fenetre.
    pub fn paint(&self, entries: &[ClipboardEntry], palette: &ThemePalette) {
        if let Some(ref ctx) = self.render_ctx {
            let visible = self.visible_count();
            ctx.paint(
                self.hwnd,
                entries,
                self.selected,
                self.scroll_offset,
                visible,
                palette,
                "",
            );
        }
    }

    /// Detruit la fenetre permanente.
    pub fn destroy(&mut self) {
        window::destroy(self.hwnd);
        self.hwnd = NULL_HWND;
        self.render_ctx = None;
    }
}
