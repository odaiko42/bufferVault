// BufferVault - Structure de configuration et valeurs par defaut
//
// Ce module definit la structure `Settings` qui centralise tous les
// parametres de l'application : historique, hotkey, affichage, theme,
// securite, exclusions et chemins de fichiers.
//
// # Chargement
// `Settings::load(path)` lit le fichier de configuration et applique
// les valeurs parsees. Les valeurs manquantes conservent leur defaut.
// Les valeurs hors bornes sont clampees (ex: max_history 10..10000).
//
// # Fichier par defaut
// `Settings::save_default(path)` genere un fichier de configuration
// commente avec toutes les options disponibles et leurs valeurs.
//
// # Portabilite
// Dependance Windows limitee a `get_env_var("APPDATA")` pour le
// repertoire de donnees. Le reste est en pur Rust.

use crate::config::parser::{self, ParsedConfig};
use crate::constants::*;
use crate::system::win32;
use std::path::{Path, PathBuf};
use std::fs;

/// Mode d'affichage de l'interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Popup,
    Sidebar,
    Permanent,
    Minimal,
}

impl DisplayMode {
    /// Parse depuis une chaine.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sidebar" => Self::Sidebar,
            "permanent" => Self::Permanent,
            "minimal" => Self::Minimal,
            _ => Self::Popup,
        }
    }
    /// Serialise en chaine.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Popup => "popup",
            Self::Sidebar => "sidebar",
            Self::Permanent => "permanent",
            Self::Minimal => "minimal",
        }
    }
}

/// Theme de l'interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    /// Parse depuis une chaine.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Self::Light,
            _ => Self::Dark,
        }
    }
}

/// Position de l'affichage popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupPosition {
    Center,
    Cursor,
}

impl PopupPosition {
    /// Parse depuis une chaine.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "cursor" => Self::Cursor,
            _ => Self::Center,
        }
    }
}

/// Configuration complete de l'application.
#[derive(Debug, Clone)]
pub struct Settings {
    // general
    pub max_history: usize,
    pub max_entry_size: usize,
    pub retention_days: u32,
    pub auto_start: bool,
    // hotkey
    pub hotkey_modifiers: u32,
    pub hotkey_vk: u32,
    // display
    pub display_mode: DisplayMode,
    pub visible_items: usize,
    pub preview_length: usize,
    pub popup_position: PopupPosition,
    pub close_after_select: bool,
    pub show_source: bool,
    pub show_timestamp: bool,
    // theme
    pub theme: ThemeMode,
    pub opacity: f32,
    pub font_size: i32,
    pub accent_color: u32,
    // security
    pub pbkdf2_iterations: u32,
    // exclusions
    pub excluded_apps: Vec<String>,
    // paths
    pub data_dir: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        let data_dir = get_app_data_dir();
        Self {
            max_history: DEFAULT_MAX_HISTORY,
            max_entry_size: DEFAULT_MAX_ENTRY_SIZE,
            retention_days: DEFAULT_RETENTION_DAYS,
            auto_start: false,
            hotkey_modifiers: win32::MOD_CONTROL | win32::MOD_ALT | win32::MOD_NOREPEAT,
            hotkey_vk: win32::VK_V,
            display_mode: DisplayMode::Popup,
            visible_items: DEFAULT_VISIBLE_ITEMS,
            preview_length: DEFAULT_PREVIEW_LENGTH,
            popup_position: PopupPosition::Center,
            close_after_select: true,
            show_source: true,
            show_timestamp: true,
            theme: ThemeMode::Dark,
            opacity: 0.95,
            font_size: 13,
            accent_color: 0xFF9E4A, // #4A9EFF en RGB -> COLORREF inversed
            pbkdf2_iterations: DEFAULT_PBKDF2_ITERATIONS,
            excluded_apps: Vec::new(),
            data_dir,
        }
    }
}

impl Settings {
    /// Charge la configuration depuis un fichier. Utilise les defauts pour les valeurs manquantes.
    pub fn load(path: &Path) -> Self {
        let mut settings = Settings::default();
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => return settings,
        };

        let config = parser::parse_config(&text);
        settings.apply_parsed(&config);
        settings
    }

    /// Applique les valeurs parsees sur les parametres.
    fn apply_parsed(&mut self, config: &ParsedConfig) {
        if let Some(gen) = config.get("general") {
            if let Some(v) = gen.get("max_history").and_then(|v| parser::parse_usize(v)) {
                self.max_history = v.max(10).min(10000);
            }
            if let Some(v) = gen.get("max_entry_size_kb").and_then(|v| parser::parse_usize(v)) {
                self.max_entry_size = v * 1024;
            }
            if let Some(v) = gen.get("retention_days").and_then(|v| parser::parse_u32(v)) {
                self.retention_days = v.max(1).min(365);
            }
            if let Some(v) = gen.get("auto_start").and_then(|v| parser::parse_bool(v)) {
                self.auto_start = v;
            }
        }

        if let Some(hk) = config.get("hotkey") {
            if let Some(mods) = hk.get("modifier") {
                self.hotkey_modifiers = parse_modifiers(mods) | win32::MOD_NOREPEAT;
            }
            if let Some(key) = hk.get("key") {
                if let Some(vk) = parse_vk(key) {
                    self.hotkey_vk = vk;
                }
            }
        }

        if let Some(disp) = config.get("display") {
            if let Some(m) = disp.get("mode") {
                self.display_mode = DisplayMode::from_str(m);
            }
            if let Some(v) = disp.get("visible_items").and_then(|v| parser::parse_usize(v)) {
                self.visible_items = v.max(3).min(30);
            }
            if let Some(v) = disp.get("preview_length").and_then(|v| parser::parse_usize(v)) {
                self.preview_length = v.max(10).min(200);
            }
            if let Some(p) = disp.get("position") {
                self.popup_position = PopupPosition::from_str(p);
            }
            if let Some(v) = disp.get("close_after_select").and_then(|v| parser::parse_bool(v)) {
                self.close_after_select = v;
            }
            if let Some(v) = disp.get("show_source").and_then(|v| parser::parse_bool(v)) {
                self.show_source = v;
            }
            if let Some(v) = disp.get("show_timestamp").and_then(|v| parser::parse_bool(v)) {
                self.show_timestamp = v;
            }
        }

        if let Some(th) = config.get("theme") {
            if let Some(m) = th.get("mode") {
                self.theme = ThemeMode::from_str(m);
            }
            if let Some(v) = th.get("opacity") {
                if let Ok(f) = v.parse::<f32>() {
                    self.opacity = f.clamp(0.3, 1.0);
                }
            }
            if let Some(v) = th.get("font_size").and_then(|v| v.parse::<i32>().ok()) {
                self.font_size = v.clamp(8, 24);
            }
        }

        if let Some(sec) = config.get("security") {
            if let Some(v) = sec.get("pbkdf2_iterations").and_then(|v| parser::parse_u32(v)) {
                self.pbkdf2_iterations = v.max(10_000);
            }
        }

        if let Some(exc) = config.get("exclusions") {
            if let Some(apps) = exc.get("apps") {
                self.excluded_apps = parser::parse_string_list(apps);
            }
        }
    }

    /// Sauvegarde la configuration avec commentaires par defaut.
    pub fn save_default(path: &Path) -> std::io::Result<()> {
        let content = default_config_text();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)
    }

    /// Retourne le chemin du fichier de configuration.
    pub fn config_path(&self) -> PathBuf {
        self.data_dir.join(CONFIG_FILENAME)
    }

    /// Retourne le chemin du fichier vault.
    pub fn vault_path(&self) -> PathBuf {
        self.data_dir.join(VAULT_FILENAME)
    }

    /// Retourne le chemin du fichier keystore.
    pub fn keystore_path(&self) -> PathBuf {
        self.data_dir.join(KEYSTORE_FILENAME)
    }

    /// Verifie si une application est exclue de la capture.
    pub fn is_app_excluded(&self, app_name: &str) -> bool {
        let app_lower = app_name.to_lowercase();
        self.excluded_apps.iter().any(|e| e.to_lowercase() == app_lower)
    }
}

/// Retourne le repertoire de donnees de l'application.
fn get_app_data_dir() -> PathBuf {
    win32::get_env_var("APPDATA")
        .map(|p| PathBuf::from(p).join(APP_DIR_NAME))
        .unwrap_or_else(|| PathBuf::from(".").join(APP_DIR_NAME))
}

/// Parse les modificateurs de raccourci depuis une chaine.
fn parse_modifiers(s: &str) -> u32 {
    let mut mods = 0u32;
    let lower = s.to_lowercase();
    if lower.contains("win") { mods |= win32::MOD_WIN; }
    if lower.contains("ctrl") || lower.contains("control") { mods |= win32::MOD_CONTROL; }
    if lower.contains("alt") { mods |= win32::MOD_ALT; }
    if lower.contains("shift") { mods |= win32::MOD_SHIFT; }
    mods
}

/// Parse une touche virtuelle depuis une chaine.
fn parse_vk(s: &str) -> Option<u32> {
    let lower = s.to_lowercase().trim().to_string();
    match lower.as_str() {
        "a" => Some(0x41), "b" => Some(0x42), "c" => Some(0x43),
        "d" => Some(0x44), "e" => Some(0x45), "f" => Some(0x46),
        "g" => Some(0x47), "h" => Some(0x48), "i" => Some(0x49),
        "j" => Some(0x4A), "k" => Some(0x4B), "l" => Some(0x4C),
        "m" => Some(0x4D), "n" => Some(0x4E), "o" => Some(0x4F),
        "p" => Some(0x50), "q" => Some(0x51), "r" => Some(0x52),
        "s" => Some(0x53), "t" => Some(0x54), "u" => Some(0x55),
        "v" => Some(0x56), "w" => Some(0x57), "x" => Some(0x58),
        "y" => Some(0x59), "z" => Some(0x5A),
        "0" => Some(0x30), "1" => Some(0x31), "2" => Some(0x32),
        "3" => Some(0x33), "4" => Some(0x34), "5" => Some(0x35),
        "6" => Some(0x36), "7" => Some(0x37), "8" => Some(0x38),
        "9" => Some(0x39),
        "f1" => Some(0x70), "f2" => Some(0x71), "f3" => Some(0x72),
        "f4" => Some(0x73), "f5" => Some(0x74), "f6" => Some(0x75),
        "f7" => Some(0x76), "f8" => Some(0x77), "f9" => Some(0x78),
        "f10" => Some(0x79), "f11" => Some(0x7A), "f12" => Some(0x7B),
        _ => None,
    }
}

/// Texte par defaut du fichier de configuration.
fn default_config_text() -> String {
    r#"# BufferVault Configuration
# Emplacement : %APPDATA%\BufferVault\config.txt

[general]
max_history = 500
max_entry_size_kb = 1024
retention_days = 30
auto_start = false

[hotkey]
# Modificateurs : win, ctrl, alt, shift
# Touches : a-z, 0-9, f1-f12
modifier = "win+shift"
key = "v"

[display]
# Mode : popup | sidebar | permanent | minimal
mode = "popup"
visible_items = 8
preview_length = 60
position = "center"
close_after_select = true
show_source = true
show_timestamp = true

[theme]
mode = "dark"
opacity = 0.95
font_size = 13

[security]
pbkdf2_iterations = 100000

[exclusions]
apps = []
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let s = Settings::default();
        assert_eq!(s.max_history, 500);
        assert_eq!(s.display_mode, DisplayMode::Popup);
        assert_eq!(s.theme, ThemeMode::Dark);
        assert!(s.close_after_select);
    }

    #[test]
    fn test_display_mode_parse() {
        assert_eq!(DisplayMode::from_str("popup"), DisplayMode::Popup);
        assert_eq!(DisplayMode::from_str("sidebar"), DisplayMode::Sidebar);
        assert_eq!(DisplayMode::from_str("PERMANENT"), DisplayMode::Permanent);
        assert_eq!(DisplayMode::from_str("unknown"), DisplayMode::Popup);
    }

    #[test]
    fn test_is_app_excluded() {
        let mut s = Settings::default();
        s.excluded_apps = vec!["KeePass.exe".into()];
        assert!(s.is_app_excluded("keepass.exe"));
        assert!(!s.is_app_excluded("notepad.exe"));
    }

    #[test]
    fn test_parse_modifiers() {
        assert_eq!(parse_modifiers("win+shift"), win32::MOD_WIN | win32::MOD_SHIFT);
        assert_eq!(parse_modifiers("ctrl+alt"), win32::MOD_CONTROL | win32::MOD_ALT);
    }

    #[test]
    fn test_parse_vk() {
        assert_eq!(parse_vk("v"), Some(0x56));
        assert_eq!(parse_vk("F1"), Some(0x70));
        assert_eq!(parse_vk("invalid"), None);
    }
}
