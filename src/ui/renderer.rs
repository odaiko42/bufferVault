// BufferVault - Rendu GDI
// Dessin des entrees de l'historique avec double buffering
//
// Ce module contient le moteur de rendu GDI utilise par tous les modes
// d'affichage (popup, sidebar, permanent). Il gere :
// - La creation et destruction des polices GDI (Segoe UI)
// - Le rendu double-buffered pour eviter le scintillement
// - Le dessin des entrees (texte principal, source, age, separateurs)
// - La barre de recherche avec curseur anime
//
// # Architecture
// Le RenderContext possede les polices GDI et implemente Drop pour
// garantir leur liberation. Le dessin est effectue dans un bitmap
// memoire (CreateCompatibleBitmap) puis copie a l'ecran (BitBlt).
//
// # Safety
// Tous les appels Win32 GDI sont isoles dans des blocs unsafe locaux.
// Les objets GDI (polices, brushes, bitmaps) sont crees et detruits
// dans le meme scope ou via le trait Drop.
//
// # Portabilite
// Ce module est specifique a Windows (Win32 GDI).

use crate::history::entry::ClipboardEntry;
use crate::ui::dpi::DpiContext;
use crate::ui::theme::ThemePalette;
use crate::system::win32::*;

/// Hauteur d'un element en pixels logiques (base DPI).
pub const ITEM_HEIGHT_BASE: i32 = 44;

/// Padding horizontal en pixels logiques.
pub const PADDING_X_BASE: i32 = 12;

/// Padding vertical en pixels logiques.
pub const PADDING_Y_BASE: i32 = 6;

/// Hauteur de la barre de recherche en pixels logiques.
pub const SEARCH_BAR_HEIGHT_BASE: i32 = 32;

/// Taille de police principale en pixels logiques.
pub const FONT_SIZE_BASE: i32 = -14;

/// Taille de police secondaire en pixels logiques.
pub const FONT_SIZE_SMALL_BASE: i32 = -11;

/// Contexte de rendu GDI avec double buffering.
///
/// Possede les polices GDI (principale et petite) et le contexte DPI.
/// Utilise par tous les modes d'affichage pour dessiner les entrees
/// de l'historique avec un rendu sans scintillement.
///
/// # Lifecycle
/// Les polices GDI sont creees dans `new()` et detruites automatiquement
/// via l'implementation de Drop. Appeler `update_dpi()` apres un
/// changement de DPI pour recreer les polices a la bonne taille.
pub struct RenderContext {
    /// Police principale
    font: HFONT,
    /// Police secondaire (petite)
    font_small: HFONT,
    /// Contexte DPI
    dpi: DpiContext,
}

impl RenderContext {
    /// Cree un contexte de rendu avec les polices.
    pub fn new(dpi: &DpiContext) -> Self {
        let font = create_font(dpi.scale_i32(FONT_SIZE_BASE), FW_NORMAL, dpi);
        let font_small = create_font(dpi.scale_i32(FONT_SIZE_SMALL_BASE), FW_NORMAL, dpi);
        Self {
            font,
            font_small,
            dpi: *dpi,
        }
    }

    /// Retourne le handle de la police principale.
    pub fn font_main(&self) -> HFONT {
        self.font
    }

    /// Retourne le handle de la petite police.
    pub fn font_small(&self) -> HFONT {
        self.font_small
    }

    /// Met a jour les polices apres un changement de DPI.
    pub fn update_dpi(&mut self, dpi: &DpiContext) {
        self.cleanup();
        self.dpi = *dpi;
        self.font = create_font(dpi.scale_i32(FONT_SIZE_BASE), FW_NORMAL, dpi);
        self.font_small = create_font(dpi.scale_i32(FONT_SIZE_SMALL_BASE), FW_NORMAL, dpi);
    }

    /// Dessine la liste des entrees dans un HDC avec double buffering.
    pub fn paint(
        &self,
        hwnd: HWND,
        entries: &[ClipboardEntry],
        selected: usize,
        scroll_offset: usize,
        visible_count: usize,
        palette: &ThemePalette,
        search_text: &str,
    ) {
        // SAFETY: appels FFI Win32 GDI.
        unsafe {
            let mut ps = std::mem::zeroed::<PAINTSTRUCT>();
            let hdc = BeginPaint(hwnd, &mut ps);
            if hdc.is_null() {
                return;
            }

            let mut client_rect = RECT::default();
            GetClientRect(hwnd, &mut client_rect);
            let width = client_rect.right - client_rect.left;
            let height = client_rect.bottom - client_rect.top;

            // Double buffering
            let mem_dc = CreateCompatibleDC(hdc);
            let bmp = CreateCompatibleBitmap(hdc, width, height);
            let old_bmp = SelectObject(mem_dc, bmp as HGDIOBJ);

            // Fond
            let bg_brush = CreateSolidBrush(palette.bg);
            FillRect(mem_dc, &client_rect, bg_brush);
            DeleteObject(bg_brush as HGDIOBJ);

            let item_h = self.dpi.scale_i32(ITEM_HEIGHT_BASE);
            let pad_x = self.dpi.scale_i32(PADDING_X_BASE);
            let search_h = self.dpi.scale_i32(SEARCH_BAR_HEIGHT_BASE);

            // Barre de recherche
            if !search_text.is_empty() {
                self.draw_search_bar(mem_dc, width, search_h, pad_x, palette, search_text);
            }

            let y_start = if search_text.is_empty() { 0 } else { search_h };

            // Entrees visibles
            let end = (scroll_offset + visible_count).min(entries.len());
            for idx in scroll_offset..end {
                let row = (idx - scroll_offset) as i32;
                let y = y_start + row * item_h;
                let is_selected = idx == selected;

                self.draw_entry(
                    mem_dc, &entries[idx], y, width, item_h, pad_x,
                    is_selected, palette,
                );
            }

            // Copie vers l'ecran
            BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

            // Nettoyage
            SelectObject(mem_dc, old_bmp);
            DeleteObject(bmp as HGDIOBJ);
            DeleteDC(mem_dc);

            EndPaint(hwnd, &ps);
        }
    }

    /// Dessine une entree de l'historique.
    unsafe fn draw_entry(
        &self,
        hdc: HDC,
        entry: &ClipboardEntry,
        y: i32,
        width: i32,
        item_h: i32,
        pad_x: i32,
        is_selected: bool,
        palette: &ThemePalette,
    ) {
        let item_rect = RECT {
            left: 0,
            top: y,
            right: width,
            bottom: y + item_h,
        };

        // Fond de l'element
        let bg_color = if is_selected { palette.bg_selected } else { palette.bg };
        let bg_brush = CreateSolidBrush(bg_color);
        FillRect(hdc, &item_rect, bg_brush);
        DeleteObject(bg_brush as HGDIOBJ);

        SetBkMode(hdc, TRANSPARENT);

        // Texte principal (premiere ligne tronquee)
        let text_color = if is_selected { palette.text_selected } else { palette.text };
        SetTextColor(hdc, text_color);
        let old_font = SelectObject(hdc, self.font as HGDIOBJ);

        let preview = entry.preview(80);
        let wtext = to_wstring(&preview);
        let pad_y = self.dpi.scale_i32(PADDING_Y_BASE);
        let mut text_rect = RECT {
            left: pad_x,
            top: y + pad_y,
            right: width - pad_x,
            bottom: y + item_h / 2 + pad_y,
        };

        // Indicateur d'element epingle
        if entry.flags.pinned {
            let pin_text = to_wstring("[*] ");
            SetTextColor(hdc, palette.pin_indicator);
            DrawTextW(hdc, pin_text.as_ptr(), -1, &mut text_rect, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);
            text_rect.left += self.dpi.scale_i32(24);
            SetTextColor(hdc, text_color);
        }

        DrawTextW(hdc, wtext.as_ptr(), -1, &mut text_rect, DT_LEFT | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX);

        // Texte secondaire (source + age)
        let sec_color = if is_selected { palette.text_selected } else { palette.text_secondary };
        SetTextColor(hdc, sec_color);
        SelectObject(hdc, self.font_small as HGDIOBJ);

        let info = format!("{} - {}", entry.source_app, entry.age_display());
        let winfo = to_wstring(&info);
        let mut info_rect = RECT {
            left: pad_x,
            top: y + item_h / 2 + 2,
            right: width - pad_x,
            bottom: y + item_h - 2,
        };
        DrawTextW(hdc, winfo.as_ptr(), -1, &mut info_rect, DT_LEFT | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX);

        // Separateur
        if !is_selected {
            let sep_brush = CreateSolidBrush(palette.border);
            let sep_rect = RECT {
                left: pad_x,
                top: y + item_h - 1,
                right: width - pad_x,
                bottom: y + item_h,
            };
            FillRect(hdc, &sep_rect, sep_brush);
            DeleteObject(sep_brush as HGDIOBJ);
        }

        SelectObject(hdc, old_font);
    }

    /// Dessine la barre de recherche.
    unsafe fn draw_search_bar(
        &self,
        hdc: HDC,
        width: i32,
        height: i32,
        pad_x: i32,
        palette: &ThemePalette,
        search_text: &str,
    ) {
        let bar_rect = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        let bg_brush = CreateSolidBrush(palette.search_bg);
        FillRect(hdc, &bar_rect, bg_brush);
        DeleteObject(bg_brush as HGDIOBJ);

        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, palette.text);
        let old_font = SelectObject(hdc, self.font as HGDIOBJ);

        let display = format!("> {}_", search_text);
        let wtext = to_wstring(&display);
        let mut text_rect = RECT {
            left: pad_x,
            top: 0,
            right: width - pad_x,
            bottom: height,
        };
        DrawTextW(hdc, wtext.as_ptr(), -1, &mut text_rect, DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);

        SelectObject(hdc, old_font);

        // Bordure inferieure
        let sep_brush = CreateSolidBrush(palette.border);
        let sep_rect = RECT {
            left: 0,
            top: height - 1,
            right: width,
            bottom: height,
        };
        FillRect(hdc, &sep_rect, sep_brush);
        DeleteObject(sep_brush as HGDIOBJ);
    }

    /// Nettoie les ressources GDI.
    fn cleanup(&mut self) {
        // SAFETY: appels FFI Win32 pour liberer les objets GDI.
        unsafe {
            if !self.font.is_null() {
                DeleteObject(self.font as HGDIOBJ);
                self.font = std::ptr::null_mut();
            }
            if !self.font_small.is_null() {
                DeleteObject(self.font_small as HGDIOBJ);
                self.font_small = std::ptr::null_mut();
            }
        }
    }
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Cree une police GDI avec les parametres specifies.
fn create_font(height: i32, weight: i32, _dpi: &DpiContext) -> HFONT {
    let face = to_wstring("Segoe UI");
    let mut lf = LOGFONTW::default();
    lf.lfHeight = height;
    lf.lfWeight = weight;
    lf.lfCharSet = DEFAULT_CHARSET as u8;
    lf.lfQuality = CLEARTYPE_QUALITY as u8;
    // Copier le nom de la police
    let copy_len = face.len().min(lf.lfFaceName.len());
    lf.lfFaceName[..copy_len].copy_from_slice(&face[..copy_len]);
    // SAFETY: la structure est correctement initialisee ci-dessus.
    unsafe { CreateFontIndirectW(&lf) }
}
