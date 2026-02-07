// BufferVault - Gestion du DPI et mise a l'echelle
// Support des ecrans haute resolution (HiDPI / Per-Monitor DPI)
//
// Ce module fournit un contexte DPI pour la mise a l'echelle automatique
// des elements d'interface. Il interroge le DPI de chaque fenetre via
// GetDpiForWindow (Windows 10 1607+) et calcule un facteur d'echelle.
//
// # Utilisation
// ```rust
// let dpi = DpiContext::from_hwnd(hwnd);
// let scaled = dpi.scale_i32(100); // 150 a 150% DPI
// ```
//
// # Portabilite
// Specifique a Windows (GetDpiForWindow). Sur d'autres plateformes,
// le DPI par defaut (96) est utilise.

use crate::system::win32::*;

/// DPI de reference Windows (100% scaling = 96 DPI).
pub const BASE_DPI: u32 = 96;

/// Contexte DPI pour la mise a l'echelle de l'interface.
///
/// Stocke le DPI actuel et le facteur d'echelle pour convertir
/// les dimensions logiques (base 96 DPI) en pixels physiques.
///
/// # Facteurs d'echelle courants
/// - 100% : DPI=96, scale=1.0
/// - 125% : DPI=120, scale=1.25
/// - 150% : DPI=144, scale=1.5
/// - 200% : DPI=192, scale=2.0
#[derive(Debug, Clone, Copy)]
pub struct DpiContext {
    /// DPI actuel
    pub dpi: u32,
    /// Facteur d'echelle (ex: 1.0, 1.25, 1.5, 2.0)
    pub scale: f32,
}

impl DpiContext {
    /// Cree un contexte avec le DPI par defaut (96).
    pub fn new() -> Self {
        Self {
            dpi: BASE_DPI,
            scale: 1.0,
        }
    }

    /// Cree un contexte depuis le DPI d'une fenetre.
    pub fn from_hwnd(hwnd: HWND) -> Self {
        // SAFETY: appel FFI Win32. hwnd doit etre un handle valide.
        let dpi = unsafe { GetDpiForWindow(hwnd) };
        let dpi = if dpi == 0 { BASE_DPI } else { dpi };
        Self {
            dpi,
            scale: dpi as f32 / BASE_DPI as f32,
        }
    }

    /// Met a jour le DPI depuis la fenetre.
    pub fn update(&mut self, hwnd: HWND) {
        // SAFETY: appel FFI Win32.
        let dpi = unsafe { GetDpiForWindow(hwnd) };
        if dpi > 0 {
            self.dpi = dpi;
            self.scale = dpi as f32 / BASE_DPI as f32;
        }
    }

    /// Convertit une valeur en pixels logiques vers des pixels physiques.
    pub fn scale_i32(&self, value: i32) -> i32 {
        ((value as f32) * self.scale + 0.5) as i32
    }

    /// Convertit une valeur en pixels logiques vers des pixels physiques (unsigned).
    pub fn scale_u32(&self, value: u32) -> u32 {
        ((value as f32) * self.scale + 0.5) as u32
    }
}

impl Default for DpiContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpi_default() {
        let dpi = DpiContext::new();
        assert_eq!(dpi.dpi, 96);
        assert!((dpi.scale - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_dpi_scaling() {
        let dpi = DpiContext {
            dpi: 144,
            scale: 1.5,
        };
        assert_eq!(dpi.scale_i32(100), 150);
        assert_eq!(dpi.scale_i32(10), 15);
    }

    #[test]
    fn test_dpi_scaling_200() {
        let dpi = DpiContext {
            dpi: 192,
            scale: 2.0,
        };
        assert_eq!(dpi.scale_i32(50), 100);
        assert_eq!(dpi.scale_u32(50), 100);
    }
}
