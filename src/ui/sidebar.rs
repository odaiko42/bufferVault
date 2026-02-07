// BufferVault - Mode sidebar (barre laterale ancree)
// Fenetre ancree au bord droit de l'ecran, toujours visible
//
// Ce module implemente le mode sidebar : une fenetre sans decoration
// ancree au bord droit de l'ecran qui affiche en permanence l'historique.
// La sidebar se bascule avec le raccourci clavier global.
//
// # Fonctionnalites
// - Affichage permanent ancre au bord droit
// - Navigation clavier haut/bas
// - Defilement automatique avec le curseur
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

/// Largeur par defaut de la sidebar en pixels logiques.
pub const SIDEBAR_WIDTH_BASE: i32 = 320;

/// Etat de la sidebar ancree.
///
/// Contient le handle de fenetre, la position de selection, le defilement
/// et le contexte de rendu GDI. La sidebar occupe toute la hauteur de
/// l'ecran et est ancree au bord droit.
pub struct SidebarState {
    /// Handle de la fenetre sidebar
    pub hwnd: HWND,
    /// Index de l'element selectionne
    pub selected: usize,
    /// Offset de defilement
    pub scroll_offset: usize,
    /// Contexte de rendu
    pub render_ctx: Option<RenderContext>,
    /// Est-ce que la sidebar est visible ?
    pub visible: bool,
}

impl SidebarState {
    /// Cree un nouvel etat de sidebar.
    pub fn new() -> Self {
        Self {
            hwnd: NULL_HWND,
            selected: 0,
            scroll_offset: 0,
            render_ctx: None,
            visible: false,
        }
    }

    /// Initialise la fenetre sidebar ancree a droite de l'ecran.
    pub fn create_window(&mut self, dpi: &DpiContext) {
        let (sw, sh) = window::screen_size();
        let width = dpi.scale_i32(SIDEBAR_WIDTH_BASE);
        let x = sw - width;
        let y = 0;

        match window::create_popup_window(
            window::SIDEBAR_CLASS,
            x, y, width, sh,
            std::ptr::null_mut(),
        ) {
            Ok(h) => {
                self.hwnd = h;
                self.render_ctx = Some(RenderContext::new(dpi));
            }
            Err(e) => {
                eprintln!("Failed to create sidebar window: {}", e);
            }
        }
    }

    /// Affiche ou cache la sidebar.
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

    /// Calcule le nombre d'elements visibles en fonction de la hauteur.
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

    /// Dessine la sidebar.
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

    /// Detruit la fenetre sidebar.
    pub fn destroy(&mut self) {
        window::destroy(self.hwnd);
        self.hwnd = NULL_HWND;
        self.render_ctx = None;
    }
}
