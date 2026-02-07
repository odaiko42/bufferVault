// BufferVault - Gestionnaire d'historique
// Fenetre modale avec liste d'entrees, multi-selection, suppression par lot et edition
//
// Ce module implemente la fenetre de gestion de l'historique, accessible
// depuis le menu tray. Elle permet de :
// - Parcourir toutes les entrees avec defilement clavier et souris
// - Selectionner (cocher) des entrees individuellement ou en lot (Ctrl+A)
// - Supprimer les entrees cochees ou l'entree courante (Delete)
// - Editer le contenu d'une entree en mode inline (F2)
// - Copier une entree dans le presse-papiers (Entree)
//
// # Safety
// Tous les appels Win32 (creation fenetre, GDI) sont isoles dans des
// blocs unsafe locaux. Le double buffering est utilise pour le rendu.
//
// # Portabilite
// Ce module est specifique a Windows (Win32 GDI).

use crate::history::entry::ClipboardEntry;
use crate::ui::dpi::DpiContext;
use crate::ui::renderer::{self, RenderContext};
use crate::ui::theme::ThemePalette;
use crate::ui::window;
use crate::system::win32::*;

/// Classe de fenetre pour le gestionnaire.
pub const MANAGER_CLASS: &str = "BufferVaultManager";

/// Hauteur de la barre de boutons en bas (pixels logiques).
const BUTTON_BAR_HEIGHT_BASE: i32 = 48;

/// Largeur de la fenetre (pixels logiques).
const MANAGER_WIDTH_BASE: i32 = 560;

/// Hauteur de la fenetre (pixels logiques).
const MANAGER_HEIGHT_BASE: i32 = 520;

/// Largeur de la case a cocher (pixels logiques).
const CHECKBOX_WIDTH_BASE: i32 = 24;

/// Etat du gestionnaire d'historique.
///
/// Fenetre modale qui affiche la liste complete des entrees avec :
/// - Cases a cocher pour la multi-selection
/// - Mode edition inline pour modifier le contenu
/// - Barre d'actions en bas avec raccourcis clavier
///
/// Le gestionnaire est cree a la demande depuis le menu tray et
/// cache/detruit independamment de la fenetre principale.
pub struct ManagerState {
    /// Handle de la fenetre
    pub hwnd: HWND,
    /// Index de l'element sous le curseur clavier
    pub cursor: usize,
    /// Offset de defilement
    pub scroll_offset: usize,
    /// Indices des elements selectionnes (coches)
    pub checked: Vec<bool>,
    /// Le gestionnaire est-il visible ?
    pub visible: bool,
    /// Contexte de rendu
    pub render_ctx: Option<RenderContext>,
    /// Mode edition actif (-1 = aucun)
    pub editing_index: Option<usize>,
    /// Contenu en cours d'edition
    pub edit_buffer: String,
    /// Position du curseur dans le buffer d'edition
    pub edit_cursor: usize,
}

impl ManagerState {
    /// Cree un nouvel etat de gestionnaire.
    pub fn new() -> Self {
        Self {
            hwnd: NULL_HWND,
            cursor: 0,
            scroll_offset: 0,
            checked: Vec::new(),
            visible: false,
            render_ctx: None,
            editing_index: None,
            edit_buffer: String::new(),
            edit_cursor: 0,
        }
    }

    /// Initialise la fenetre du gestionnaire.
    pub fn create_window(&mut self, dpi: &DpiContext) {
        let width = dpi.scale_i32(MANAGER_WIDTH_BASE);
        let height = dpi.scale_i32(MANAGER_HEIGHT_BASE);

        // Centrer sur l'ecran
        let (sw, sh) = window::screen_size();
        let x = (sw - width) / 2;
        let y = (sh - height) / 2;

        let wclass = to_wstring(MANAGER_CLASS);
        let wtitle = to_wstring("BufferVault - Gestion de l'historique");
        // SAFETY: appels FFI Win32 pour creer la fenetre.
        let hinstance = unsafe { GetModuleHandleW(std::ptr::null()) };
        let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME;
        let ex_style = WS_EX_TOOLWINDOW;

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

        if !hwnd.is_null() {
            self.hwnd = hwnd;
            self.render_ctx = Some(RenderContext::new(dpi));
        }
    }

    /// Affiche le gestionnaire avec le contenu de l'historique.
    pub fn show(&mut self, entry_count: usize, dpi: &DpiContext) {
        if self.hwnd.is_null() {
            self.create_window(dpi);
        }
        self.cursor = 0;
        self.scroll_offset = 0;
        self.editing_index = None;
        self.edit_buffer.clear();
        self.edit_cursor = 0;

        // Initialiser les cases a cocher
        self.checked = vec![false; entry_count];

        self.visible = true;
        // SAFETY: appels FFI Win32.
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
            UpdateWindow(self.hwnd);
            SetForegroundWindow(self.hwnd);
            SetFocus(self.hwnd);
        }
    }

    /// Cache le gestionnaire.
    pub fn hide(&mut self) {
        self.visible = false;
        self.editing_index = None;
        // SAFETY: appel FFI Win32.
        unsafe { ShowWindow(self.hwnd, SW_HIDE) };
    }

    /// Detruit la fenetre du gestionnaire.
    pub fn destroy(&mut self) {
        if !self.hwnd.is_null() {
            window::destroy(self.hwnd);
            self.hwnd = NULL_HWND;
        }
    }

    /// Calcule le nombre d'elements visibles dans la zone de liste.
    fn visible_count(&self, dpi: &DpiContext) -> usize {
        if self.hwnd.is_null() {
            return 0;
        }
        let mut rc = RECT::default();
        // SAFETY: appel FFI Win32.
        unsafe { GetClientRect(self.hwnd, &mut rc) };
        let list_height = rc.bottom - dpi.scale_i32(BUTTON_BAR_HEIGHT_BASE);
        let item_h = dpi.scale_i32(renderer::ITEM_HEIGHT_BASE);
        if item_h <= 0 { return 1; }
        (list_height / item_h).max(1) as usize
    }

    /// Deplace le curseur vers le haut.
    pub fn move_up(&mut self, count: usize, dpi: &DpiContext) {
        if count == 0 { return; }
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        self.ensure_visible(dpi);
        window::invalidate(self.hwnd);
    }

    /// Deplace le curseur vers le bas.
    pub fn move_down(&mut self, count: usize, dpi: &DpiContext) {
        if count == 0 { return; }
        if self.cursor + 1 < count {
            self.cursor += 1;
        }
        self.ensure_visible(dpi);
        window::invalidate(self.hwnd);
    }

    /// S'assure que le curseur est visible dans la zone de defilement.
    fn ensure_visible(&mut self, dpi: &DpiContext) {
        let vis = self.visible_count(dpi);
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + vis {
            self.scroll_offset = self.cursor + 1 - vis;
        }
    }

    /// Bascule la case a cocher de l'element sous le curseur.
    pub fn toggle_check(&mut self) {
        if self.cursor < self.checked.len() {
            self.checked[self.cursor] = !self.checked[self.cursor];
            window::invalidate(self.hwnd);
        }
    }

    /// Selectionne / deselectionne toutes les cases.
    pub fn toggle_all(&mut self) {
        let all_checked = self.checked.iter().all(|&c| c);
        for c in &mut self.checked {
            *c = !all_checked;
        }
        window::invalidate(self.hwnd);
    }

    /// Retourne les indices des elements coches, tries du plus grand au plus petit.
    pub fn checked_indices_desc(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self.checked.iter()
            .enumerate()
            .filter_map(|(i, &c)| if c { Some(i) } else { None })
            .collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        indices
    }

    /// Nombre d'elements coches.
    pub fn checked_count(&self) -> usize {
        self.checked.iter().filter(|&&c| c).count()
    }

    /// Commence l'edition de l'element sous le curseur.
    pub fn start_edit(&mut self, entries: &[ClipboardEntry]) {
        if self.cursor < entries.len() {
            self.editing_index = Some(self.cursor);
            self.edit_buffer = entries[self.cursor].content.clone();
            // Limiter a la premiere ligne pour l'edition inline
            if let Some(pos) = self.edit_buffer.find('\n') {
                self.edit_buffer.truncate(pos);
            }
            self.edit_cursor = self.edit_buffer.len();
            window::invalidate(self.hwnd);
        }
    }

    /// Confirme l'edition en cours et retourne (index, nouveau contenu).
    pub fn confirm_edit(&mut self) -> Option<(usize, String)> {
        if let Some(idx) = self.editing_index.take() {
            let content = self.edit_buffer.clone();
            self.edit_buffer.clear();
            self.edit_cursor = 0;
            window::invalidate(self.hwnd);
            Some((idx, content))
        } else {
            None
        }
    }

    /// Annule l'edition en cours.
    pub fn cancel_edit(&mut self) {
        self.editing_index = None;
        self.edit_buffer.clear();
        self.edit_cursor = 0;
        window::invalidate(self.hwnd);
    }

    /// Defilement a la molette.
    pub fn scroll(&mut self, delta: i32, count: usize, dpi: &DpiContext) {
        let vis = self.visible_count(dpi);
        let max_offset = count.saturating_sub(vis);
        if delta > 0 && self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(3);
        } else if delta < 0 {
            self.scroll_offset = (self.scroll_offset + 3).min(max_offset);
        }
        window::invalidate(self.hwnd);
    }

    /// Dessine le contenu du gestionnaire.
    pub fn paint(&self, entries: &[ClipboardEntry], palette: &ThemePalette, dpi: &DpiContext) {
        let render_ctx = match &self.render_ctx {
            Some(r) => r,
            None => return,
        };
        // SAFETY: appels FFI Win32 GDI pour le rendu.
        unsafe {
            let mut ps = std::mem::zeroed::<PAINTSTRUCT>();
            let hdc = BeginPaint(self.hwnd, &mut ps);
            if hdc.is_null() { return; }

            let mut client_rect = RECT::default();
            GetClientRect(self.hwnd, &mut client_rect);
            let width = client_rect.right;
            let height = client_rect.bottom;

            // Double buffering
            let mem_dc = CreateCompatibleDC(hdc);
            let bmp = CreateCompatibleBitmap(hdc, width, height);
            let old_bmp = SelectObject(mem_dc, bmp as HGDIOBJ);

            // Fond
            let bg_brush = CreateSolidBrush(palette.bg);
            FillRect(mem_dc, &client_rect, bg_brush);
            DeleteObject(bg_brush as HGDIOBJ);

            let item_h = dpi.scale_i32(renderer::ITEM_HEIGHT_BASE);
            let pad_x = dpi.scale_i32(renderer::PADDING_X_BASE);
            let cb_w = dpi.scale_i32(CHECKBOX_WIDTH_BASE);
            let btn_h = dpi.scale_i32(BUTTON_BAR_HEIGHT_BASE);
            let list_height = height - btn_h;
            let vis = (list_height / item_h).max(1) as usize;

            // Dessiner les entrees
            let end = (self.scroll_offset + vis).min(entries.len());
            for idx in self.scroll_offset..end {
                let row = (idx - self.scroll_offset) as i32;
                let y = row * item_h;
                let is_cursor = idx == self.cursor;
                let is_checked = self.checked.get(idx).copied().unwrap_or(false);
                let is_editing = self.editing_index == Some(idx);

                self.draw_manager_entry(
                    mem_dc, render_ctx, &entries[idx], y, width, item_h,
                    pad_x, cb_w, is_cursor, is_checked, is_editing, palette, dpi,
                );
            }

            // Barre de boutons en bas
            self.draw_button_bar(mem_dc, width, height, btn_h, pad_x, palette, dpi, entries.len());

            // Copie vers l'ecran
            BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

            SelectObject(mem_dc, old_bmp);
            DeleteObject(bmp as HGDIOBJ);
            DeleteDC(mem_dc);
            EndPaint(self.hwnd, &ps);
        }
    }

    /// Dessine une entree avec case a cocher dans le gestionnaire.
    unsafe fn draw_manager_entry(
        &self,
        hdc: HDC,
        render_ctx: &RenderContext,
        entry: &ClipboardEntry,
        y: i32,
        width: i32,
        item_h: i32,
        pad_x: i32,
        cb_w: i32,
        is_cursor: bool,
        is_checked: bool,
        is_editing: bool,
        palette: &ThemePalette,
        dpi: &DpiContext,
    ) {
        let item_rect = RECT { left: 0, top: y, right: width, bottom: y + item_h };

        // Fond de l'element
        let bg = if is_cursor { palette.bg_selected } else { palette.bg };
        let bg_brush = CreateSolidBrush(bg);
        FillRect(hdc, &item_rect, bg_brush);
        DeleteObject(bg_brush as HGDIOBJ);

        SetBkMode(hdc, TRANSPARENT);

        // Case a cocher
        let cb_margin = dpi.scale_i32(4);
        let cb_size = cb_w - cb_margin * 2;
        let cb_y = y + (item_h - cb_size) / 2;
        let cb_rect = RECT {
            left: cb_margin,
            top: cb_y,
            right: cb_margin + cb_size,
            bottom: cb_y + cb_size,
        };

        // Bordure de la case
        let border_brush = CreateSolidBrush(palette.border);
        FillRect(hdc, &cb_rect, border_brush);
        DeleteObject(border_brush as HGDIOBJ);

        // Interieur de la case
        let inner = RECT {
            left: cb_rect.left + 1, top: cb_rect.top + 1,
            right: cb_rect.right - 1, bottom: cb_rect.bottom - 1,
        };
        let inner_bg = if is_checked { palette.bg_selected } else { palette.bg };
        let inner_brush = CreateSolidBrush(inner_bg);
        FillRect(hdc, &inner, inner_brush);
        DeleteObject(inner_brush as HGDIOBJ);

        // Coche
        if is_checked {
            SetTextColor(hdc, palette.text_selected);
            let old_font = SelectObject(hdc, render_ctx.font_main() as HGDIOBJ);
            let check_mark = to_wstring("\u{2713}"); // Symbole coche Unicode
            let mut cr = RECT {
                left: cb_rect.left, top: cb_rect.top,
                right: cb_rect.right, bottom: cb_rect.bottom,
            };
            DrawTextW(hdc, check_mark.as_ptr(), -1, &mut cr,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER | DT_NOPREFIX);
            SelectObject(hdc, old_font);
        }

        // Texte de l'entree
        let text_left = cb_w + pad_x;
        let text_color = if is_cursor { palette.text_selected } else { palette.text };
        SetTextColor(hdc, text_color);

        if is_editing {
            // Afficher le buffer d'edition avec curseur
            let old_font = SelectObject(hdc, render_ctx.font_main() as HGDIOBJ);
            let display = format!("{}|{}", &self.edit_buffer[..self.edit_cursor],
                &self.edit_buffer[self.edit_cursor..]);
            let wtext = to_wstring(&display);
            let pad_y = dpi.scale_i32(renderer::PADDING_Y_BASE);
            let mut text_rect = RECT {
                left: text_left, top: y + pad_y,
                right: width - pad_x, bottom: y + item_h - pad_y,
            };
            // Fond d'edition
            let edit_bg = CreateSolidBrush(palette.search_bg);
            FillRect(hdc, &text_rect, edit_bg);
            DeleteObject(edit_bg as HGDIOBJ);
            SetTextColor(hdc, palette.text);
            DrawTextW(hdc, wtext.as_ptr(), -1, &mut text_rect,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX);
            SelectObject(hdc, old_font);
        } else {
            // Affichage normal : preview
            let old_font = SelectObject(hdc, render_ctx.font_main() as HGDIOBJ);
            let pad_y = dpi.scale_i32(renderer::PADDING_Y_BASE);

            // Indicateur epingle
            let mut tl = text_left;
            if entry.flags.pinned {
                let pin = to_wstring("[*] ");
                SetTextColor(hdc, palette.pin_indicator);
                let mut pr = RECT { left: tl, top: y + pad_y, right: tl + dpi.scale_i32(24), bottom: y + item_h / 2 + pad_y };
                DrawTextW(hdc, pin.as_ptr(), -1, &mut pr, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);
                tl += dpi.scale_i32(24);
                SetTextColor(hdc, text_color);
            }

            let preview = entry.preview(80);
            let wtext = to_wstring(&preview);
            let mut text_rect = RECT {
                left: tl, top: y + pad_y,
                right: width - pad_x, bottom: y + item_h / 2 + pad_y,
            };
            DrawTextW(hdc, wtext.as_ptr(), -1, &mut text_rect,
                DT_LEFT | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX);

            // Ligne secondaire (source + age)
            let sec_color = if is_cursor { palette.text_selected } else { palette.text_secondary };
            SetTextColor(hdc, sec_color);
            SelectObject(hdc, render_ctx.font_small() as HGDIOBJ);
            let info = format!("{} - {}", entry.source_app, entry.age_display());
            let winfo = to_wstring(&info);
            let mut info_rect = RECT {
                left: text_left, top: y + item_h / 2 + 2,
                right: width - pad_x, bottom: y + item_h - 2,
            };
            DrawTextW(hdc, winfo.as_ptr(), -1, &mut info_rect,
                DT_LEFT | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX);
            SelectObject(hdc, old_font);
        }

        // Separateur
        if !is_cursor {
            let sep_brush = CreateSolidBrush(palette.border);
            let sep_rect = RECT { left: pad_x, top: y + item_h - 1, right: width - pad_x, bottom: y + item_h };
            FillRect(hdc, &sep_rect, sep_brush);
            DeleteObject(sep_brush as HGDIOBJ);
        }
    }

    /// Dessine la barre de boutons en bas de la fenetre.
    unsafe fn draw_button_bar(
        &self,
        hdc: HDC,
        width: i32,
        height: i32,
        btn_h: i32,
        pad_x: i32,
        palette: &ThemePalette,
        dpi: &DpiContext,
        entry_count: usize,
    ) {
        let bar_y = height - btn_h;

        // Fond de la barre
        let bar_rect = RECT { left: 0, top: bar_y, right: width, bottom: height };
        let bar_bg = CreateSolidBrush(palette.search_bg);
        FillRect(hdc, &bar_rect, bar_bg);
        DeleteObject(bar_bg as HGDIOBJ);

        // Separateur superieur
        let sep_brush = CreateSolidBrush(palette.border);
        let sep_rect = RECT { left: 0, top: bar_y, right: width, bottom: bar_y + 1 };
        FillRect(hdc, &sep_rect, sep_brush);
        DeleteObject(sep_brush as HGDIOBJ);

        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, palette.text_secondary);

        // Obtenir la police (on reutilise la police interne du render_ctx)
        let font = self.render_ctx.as_ref().map(|r| r.font_small()).unwrap_or(std::ptr::null_mut());
        let old_font = SelectObject(hdc, font as HGDIOBJ);

        // Info a gauche : "X/Y selectionnes"
        let checked = self.checked_count();
        let info = format!("{}/{} selectionnes", checked, entry_count);
        let winfo = to_wstring(&info);
        let mut info_rect = RECT {
            left: pad_x, top: bar_y + 4, right: width / 3, bottom: height - 4,
        };
        DrawTextW(hdc, winfo.as_ptr(), -1, &mut info_rect,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);

        // Libelles des actions au centre/droite
        let actions = [
            ("Espace: Cocher", width / 3),
            ("Ctrl+A: Tout", width / 3 + dpi.scale_i32(110)),
            ("F2: Modifier", width / 3 + dpi.scale_i32(210)),
            ("Suppr: Supprimer", width / 3 + dpi.scale_i32(310)),
        ];

        let font_sm = self.render_ctx.as_ref().map(|r| r.font_small()).unwrap_or(std::ptr::null_mut());
        SelectObject(hdc, font_sm as HGDIOBJ);
        SetTextColor(hdc, palette.text_secondary);

        for (label, x_pos) in &actions {
            let wlabel = to_wstring(label);
            let mut lr = RECT {
                left: *x_pos, top: bar_y + 4,
                right: *x_pos + dpi.scale_i32(120), bottom: height - 4,
            };
            DrawTextW(hdc, wlabel.as_ptr(), -1, &mut lr,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);
        }

        SelectObject(hdc, old_font);
    }

    /// Gere un clic souris dans la zone de liste.
    pub fn on_click(&mut self, y: i32, dpi: &DpiContext, entry_count: usize) {
        let item_h = dpi.scale_i32(renderer::ITEM_HEIGHT_BASE);
        let btn_h = dpi.scale_i32(BUTTON_BAR_HEIGHT_BASE);
        let mut rc = RECT::default();
        // SAFETY: appel FFI Win32.
        unsafe { GetClientRect(self.hwnd, &mut rc) };
        let list_bottom = rc.bottom - btn_h;

        if y >= list_bottom || item_h <= 0 {
            return;
        }

        let row = (y / item_h) as usize;
        let idx = self.scroll_offset + row;
        if idx < entry_count {
            self.cursor = idx;
            // Clic sur la zone checkbox (x < cb_w) => toggle check
            // Sinon juste deplacer le curseur
            window::invalidate(self.hwnd);
        }
    }

    /// Gere un clic dans la zone de case a cocher.
    pub fn on_checkbox_click(&mut self, x: i32, y: i32, dpi: &DpiContext, entry_count: usize) {
        let item_h = dpi.scale_i32(renderer::ITEM_HEIGHT_BASE);
        let cb_w = dpi.scale_i32(CHECKBOX_WIDTH_BASE);
        let btn_h = dpi.scale_i32(BUTTON_BAR_HEIGHT_BASE);
        let mut rc = RECT::default();
        // SAFETY: appel FFI Win32.
        unsafe { GetClientRect(self.hwnd, &mut rc) };

        if y >= rc.bottom - btn_h || item_h <= 0 {
            return;
        }

        let row = (y / item_h) as usize;
        let idx = self.scroll_offset + row;

        if idx < entry_count {
            self.cursor = idx;
            if x < cb_w {
                self.toggle_check();
            }
            window::invalidate(self.hwnd);
        }
    }
}
