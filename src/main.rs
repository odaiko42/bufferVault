// BufferVault - Point d'entree
// Gestionnaire d'historique de presse-papiers pour Windows
//
// Ce binaire lance l'application BufferVault qui s'installe dans la zone
// de notification systeme et intercepte toutes les copies pour maintenir
// un historique chiffre accessible via un raccourci clavier global.
//
// # Prerequis
// - Windows 10 ou 11 (x86_64)
// - Aucune dependance externe requise
//
// # Configuration
// Le fichier %APPDATA%\BufferVault\config.txt est cree automatiquement
// au premier lancement avec les valeurs par defaut.

#![cfg_attr(not(test), windows_subsystem = "windows")]
#![allow(non_snake_case, non_camel_case_types, dead_code)]
#![cfg(target_os = "windows")]

mod app;
mod clipboard;
mod config;
mod constants;
mod crypto;
mod error;
mod history;
mod storage;
mod system;
mod ui;

use app::App;

/// Point d'entree principal de BufferVault.
///
/// Initialise l'application puis demarre la boucle de messages Win32.
/// En cas d'erreur fatale, affiche un message sur stderr et termine
/// le processus avec un code de sortie non nul.
fn main() {
    match App::new() {
        Ok(mut app) => {
            if let Err(e) = app.run() {
                eprintln!("BufferVault fatal error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("BufferVault init error: {}", e);
            std::process::exit(1);
        }
    }
}
