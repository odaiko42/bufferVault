// BufferVault - Module system
// Bindings Win32, gestion hotkey, icone de notification
//
// Ce module regroupe tous les composants systeme specifiques a Windows.
// Chaque sous-module isole ses appels FFI dans des blocs unsafe locaux
// et expose une API safe en Rust.
//
// # Sous-modules
// - `win32`      : declarations FFI (types, constantes, fonctions extern)
//                  et helpers de conversion (to_wstring, from_wstring, csprng_fill)
// - `hotkey`     : enregistrement/desenregistrement de raccourcis clavier globaux
// - `tray`       : gestion de l'icone de notification systeme et menu contextuel
// - `autostart`  : lecture/ecriture de la cle registre HKCU\Run pour le demarrage auto
// - `process`    : detection du processus au premier plan (source de la copie)
//
// # Portabilite
// Ce module est specifique a Windows 10/11 (cfg(target_os = "windows")).
// Les bindings Win32 sont declares manuellement pour eviter toute dependance
// externe (pas de crate windows ou winapi).

/// Gestion du demarrage automatique via la cle registre HKCU\Run.
pub mod autostart;
/// Enregistrement et reception des raccourcis clavier globaux.
pub mod hotkey;
/// Detection du processus au premier plan pour identifier la source.
pub mod process;
/// Icone de notification systeme (tray icon) et menu contextuel.
pub mod tray;
/// Declarations FFI Win32 (types, constantes, fonctions) et helpers.
pub mod win32;
