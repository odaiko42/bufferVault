// BufferVault - Module UI
// Interface utilisateur Win32 GDI avec themes et modes d'affichage
//
// Ce module implemente l'interface graphique de BufferVault en utilisant
// exclusivement les APIs Win32 GDI (pas de framework UI externe).
//
// # Sous-modules
// - `window`    : creation et gestion des fenetres Win32 (cachee, popup, etc.)
// - `renderer`  : moteur de rendu GDI avec double buffering pour eviter le scintillement
// - `popup`     : mode d'affichage principal (fenetre flottante au hotkey)
// - `sidebar`   : mode barre laterale ancree au bord droit de l'ecran
// - `permanent` : mode fenetre classique avec barre de titre, redimensionnable
// - `manager`   : gestionnaire d'historique avec multi-selection et edition inline
// - `splash`    : ecran de demarrage avec fade-out progressif
// - `theme`     : palettes de couleurs (clair/sombre/systeme)
// - `dpi`       : gestion du DPI et mise a l'echelle pour ecrans haute resolution
//
// # Architecture
// Chaque mode d'affichage (popup, sidebar, permanent) possede sa propre struct
// d'etat et reutilise le RenderContext commun pour le dessin. Le double
// buffering est utilise dans tous les modes pour un rendu sans scintillement.

/// Gestion du DPI et mise a l'echelle pour ecrans haute resolution.
pub mod dpi;
/// Gestionnaire d'historique avec multi-selection, suppression et edition.
pub mod manager;
/// Mode fenetre permanente avec barre de titre et redimensionnement.
pub mod permanent;
/// Mode popup principal (fenetre flottante au hotkey).
pub mod popup;
/// Moteur de rendu GDI avec double buffering.
pub mod renderer;
/// Mode barre laterale ancree au bord droit de l'ecran.
pub mod sidebar;
/// Ecran de demarrage avec animation de fade-out.
pub mod splash;
/// Palettes de couleurs pour les themes clair, sombre et systeme.
pub mod theme;
/// Creation et gestion des fenetres Win32 (classes, positionnement, helpers).
pub mod window;
