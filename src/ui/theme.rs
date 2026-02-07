// BufferVault - Gestion des themes (clair/sombre/systeme)
// Palettes de couleurs pour le rendu GDI
//
// Ce module definit les palettes de couleurs utilisees par le moteur
// de rendu GDI. Deux themes sont disponibles (clair et sombre), et
// le mode systeme tente de detecter le theme Windows actif.
//
// # Palettes
// Chaque palette contient les couleurs pour : fond, selection, survol,
// texte principal/secondaire/selectionne, bordure, indicateur epingle
// et barre de recherche.
//
// # Portabilite
// Les couleurs sont au format COLORREF Win32 (BGR). La detection du
// theme systeme est actuellement simplifiee (fallback sur light).

use crate::system::win32::*;

/// Mode de theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl ThemeMode {
    /// Parse depuis une chaine de configuration.
    pub fn from_str_config(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dark" => Self::Dark,
            "light" => Self::Light,
            _ => Self::System,
        }
    }
}

/// Palette de couleurs pour le rendu GDI.
///
/// Contient toutes les couleurs necessaires au dessin des elements
/// d'interface : fonds, textes, bordures, indicateurs. Les couleurs
/// sont au format COLORREF Win32 (0x00BBGGRR).
#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    /// Couleur de fond de la fenetre
    pub bg: COLORREF,
    /// Couleur de fond de l'element selectionne
    pub bg_selected: COLORREF,
    /// Couleur de fond au survol
    pub bg_hover: COLORREF,
    /// Couleur du texte principal
    pub text: COLORREF,
    /// Couleur du texte secondaire (age, source)
    pub text_secondary: COLORREF,
    /// Couleur du texte selectionne
    pub text_selected: COLORREF,
    /// Couleur de la bordure
    pub border: COLORREF,
    /// Couleur de l'indicateur d'element epingle
    pub pin_indicator: COLORREF,
    /// Couleur de la barre de recherche
    pub search_bg: COLORREF,
}

/// Palette du theme clair.
pub const LIGHT_PALETTE: ThemePalette = ThemePalette {
    bg: rgb(255, 255, 255),
    bg_selected: rgb(0, 120, 212),
    bg_hover: rgb(240, 240, 240),
    text: rgb(30, 30, 30),
    text_secondary: rgb(130, 130, 130),
    text_selected: rgb(255, 255, 255),
    border: rgb(200, 200, 200),
    pin_indicator: rgb(255, 185, 0),
    search_bg: rgb(245, 245, 245),
};

/// Palette du theme sombre.
pub const DARK_PALETTE: ThemePalette = ThemePalette {
    bg: rgb(32, 32, 32),
    bg_selected: rgb(0, 120, 212),
    bg_hover: rgb(50, 50, 50),
    text: rgb(230, 230, 230),
    text_secondary: rgb(150, 150, 150),
    text_selected: rgb(255, 255, 255),
    border: rgb(60, 60, 60),
    pin_indicator: rgb(255, 185, 0),
    search_bg: rgb(45, 45, 45),
};

/// Detecte si le systeme utilise le theme sombre.
/// Lit la valeur du registre AppsUseLightTheme.
pub fn is_system_dark_mode() -> bool {
    // Heuristique : verifier la variable d'environnement
    // ou detecter via le fond de la fenetre.
    // Simplification : lire la cle de registre via une commande
    // Pour eviter d'ajouter des FFI registry, on utilise une approche simple
    false
}

/// Retourne la palette active en fonction du mode.
pub fn get_palette(mode: ThemeMode) -> &'static ThemePalette {
    match mode {
        ThemeMode::Light => &LIGHT_PALETTE,
        ThemeMode::Dark => &DARK_PALETTE,
        ThemeMode::System => {
            if is_system_dark_mode() {
                &DARK_PALETTE
            } else {
                &LIGHT_PALETTE
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_from_str() {
        assert_eq!(ThemeMode::from_str_config("dark"), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_str_config("light"), ThemeMode::Light);
        assert_eq!(ThemeMode::from_str_config("system"), ThemeMode::System);
        assert_eq!(ThemeMode::from_str_config("DARK"), ThemeMode::Dark);
    }

    #[test]
    fn test_palette_colors() {
        let p = get_palette(ThemeMode::Light);
        assert_ne!(p.bg, p.text);
        let p = get_palette(ThemeMode::Dark);
        assert_ne!(p.bg, p.text);
    }
}
