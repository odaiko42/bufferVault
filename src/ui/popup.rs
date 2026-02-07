// BufferVault - Mode popup (affichage principal)
// Fenetre flottante apparaissant au hotkey, pres du curseur
//
// Ce module implemente le mode d'affichage principal de BufferVault :
// une fenetre popup flottante qui apparait a l'appui du raccourci clavier
// global et affiche les entrees les plus recentes de l'historique.
//
// # Fonctionnalites
// - Navigation clavier : haut/bas pour parcourir, Entree pour selectionner
// - Recherche incrementale : taper du texte filtre les entrees en temps reel
// - Defilement a la molette de souris
// - Selection par clic souris, epinglage par double-clic
// - Fermeture automatique a la perte de focus ou touche Echap
//
// # Safety
// Tous les appels Win32 sont isoles dans des blocs unsafe locaux.
//
// # Portabilite
// Ce module est specifique a Windows (Win32 GDI).

use crate::history::entry::ClipboardEntry;
use crate::history::search::search_entries;
use crate::ui::dpi::DpiContext;
use crate::ui::renderer::{self, RenderContext};
use crate::ui::theme::ThemePalette;
use crate::ui::window;
use crate::system::win32::*;

/// Etat du popup de l'historique.
///
/// Contient le handle de fenetre, la position de selection, le defilement,
/// le texte de recherche et le contexte de rendu GDI. Cree une seule fois
/// au demarrage de l'application et reutilise a chaque affichage.
pub struct PopupState {
    /// Handle de la fenetre popup
    pub hwnd: HWND,
    /// Index de l'element selectionne
    pub selected: usize,
    /// Offset de defilement
    pub scroll_offset: usize,
    /// Nombre d'elements visibles
    pub visible_count: usize,
    /// Texte de recherche actif
    pub search_text: String,
    /// Le popup est-il visible ?
    pub visible: bool,
    /// Contexte de rendu
    pub render_ctx: Option<RenderContext>,
}

impl PopupState {
    /// Cree un nouvel etat de popup.
    pub fn new(visible_count: usize) -> Self {
        Self {
            hwnd: NULL_HWND,
            selected: 0,
            scroll_offset: 0,
            visible_count,
            search_text: String::new(),
            visible: false,
            render_ctx: None,
        }
    }

    /// Initialise la fenetre popup (appelee apres l'enregistrement de la classe).
    pub fn create_window(&mut self, dpi: &DpiContext) {
        let item_h = dpi.scale_i32(renderer::ITEM_HEIGHT_BASE);
        let width = dpi.scale_i32(380);
        let height = item_h * self.visible_count as i32;

        match window::create_popup_window(
            window::POPUP_CLASS,
            0, 0, width, height,
            std::ptr::null_mut(),
        ) {
            Ok(h) => {
                self.hwnd = h;
                self.render_ctx = Some(RenderContext::new(dpi));
            }
            Err(e) => {
                eprintln!("Failed to create popup window: {}", e);
            }
        }
    }

    /// Affiche le popup pres du curseur.
    pub fn show(&mut self, entries: &[ClipboardEntry], dpi: &DpiContext) {
        if self.hwnd.is_null() {
            return;
        }

        self.selected = 0;
        self.scroll_offset = 0;
        self.search_text.clear();
        self.visible = true;

        let (cx, cy) = window::cursor_pos();
        let (sw, sh) = window::screen_size();

        let item_h = dpi.scale_i32(renderer::ITEM_HEIGHT_BASE);
        let width = dpi.scale_i32(380);
        let count = entries.len().min(self.visible_count);
        let height = item_h * count.max(1) as i32;

        // Ajuster la position pour ne pas sortir de l'ecran
        let x = if cx + width > sw { sw - width } else { cx };
        let y = if cy + height > sh { cy - height } else { cy };

        window::set_topmost(self.hwnd, x, y, width, height);
        // Donner le focus clavier au popup
        // SAFETY: appels FFI Win32.
        unsafe {
            SetForegroundWindow(self.hwnd);
            SetFocus(self.hwnd);
        }
        window::invalidate(self.hwnd);
    }

    /// Cache le popup.
    pub fn hide(&mut self) {
        if !self.hwnd.is_null() {
            window::hide_window(self.hwnd);
        }
        self.visible = false;
        self.search_text.clear();
    }

    /// Deplace la selection vers le haut.
    pub fn move_up(&mut self, entries_len: usize) {
        if entries_len == 0 {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
        window::invalidate(self.hwnd);
    }

    /// Deplace la selection vers le bas.
    pub fn move_down(&mut self, entries_len: usize) {
        if entries_len == 0 {
            return;
        }
        if self.selected + 1 < entries_len {
            self.selected += 1;
        }
        if self.selected >= self.scroll_offset + self.visible_count {
            self.scroll_offset = self.selected - self.visible_count + 1;
        }
        window::invalidate(self.hwnd);
    }

    /// Gere le defilement a la molette.
    pub fn scroll(&mut self, delta: i32, entries_len: usize) {
        if entries_len == 0 {
            return;
        }
        if delta > 0 && self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(3);
        } else if delta < 0 {
            let max = entries_len.saturating_sub(self.visible_count);
            self.scroll_offset = (self.scroll_offset + 3).min(max);
        }
        window::invalidate(self.hwnd);
    }

    /// Ajoute un caractere au texte de recherche.
    pub fn search_push(&mut self, c: char) {
        self.search_text.push(c);
        self.selected = 0;
        self.scroll_offset = 0;
        window::invalidate(self.hwnd);
    }

    /// Supprime le dernier caractere du texte de recherche.
    pub fn search_pop(&mut self) {
        self.search_text.pop();
        self.selected = 0;
        self.scroll_offset = 0;
        window::invalidate(self.hwnd);
    }

    /// Retourne l'index dans l'historique original pour l'element selectionne.
    /// Prend en compte le filtre de recherche.
    pub fn resolve_selected_index(&self, entries: &[ClipboardEntry]) -> Option<usize> {
        if entries.is_empty() {
            return None;
        }

        if self.search_text.is_empty() {
            if self.selected < entries.len() {
                Some(self.selected)
            } else {
                None
            }
        } else {
            let results = search_entries(entries, &self.search_text);
            results.get(self.selected).copied()
        }
    }

    /// Dessine le popup.
    pub fn paint(&self, entries: &[ClipboardEntry], palette: &ThemePalette) {
        if let Some(ref ctx) = self.render_ctx {
            let display_entries: Vec<&ClipboardEntry>;
            let display_slice: &[ClipboardEntry];

            if self.search_text.is_empty() {
                display_slice = entries;
            } else {
                let indices = search_entries(entries, &self.search_text);
                display_entries = indices.iter().filter_map(|&i| entries.get(i)).collect();
                // On doit creer un vecteur temporaire pour le rendu
                let temp: Vec<ClipboardEntry> = display_entries.into_iter().cloned().collect();
                ctx.paint(
                    self.hwnd,
                    &temp,
                    self.selected,
                    self.scroll_offset,
                    self.visible_count,
                    palette,
                    &self.search_text,
                );
                return;
            }

            ctx.paint(
                self.hwnd,
                display_slice,
                self.selected,
                self.scroll_offset,
                self.visible_count,
                palette,
                &self.search_text,
            );
        }
    }

    /// Detruit la fenetre popup.
    pub fn destroy(&mut self) {
        window::destroy(self.hwnd);
        self.hwnd = NULL_HWND;
        self.render_ctx = None;
    }
}
