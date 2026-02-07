// BufferVault - Arbre de modules (crate library)
//
// Ce fichier constitue le point d'entree de la crate library de BufferVault.
// Il re-exporte tous les modules pour permettre l'acces depuis le binaire
// et faciliter les tests d'integration.
//
// # Modules
// - `app`       : orchestrateur principal, boucle de messages Win32
// - `clipboard` : surveillance et injection dans le presse-papiers
// - `config`    : lecture/ecriture de la configuration utilisateur
// - `constants` : constantes globales (tailles, delais, identifiants)
// - `crypto`    : primitives cryptographiques (AES-256-GCM, PBKDF2, DPAPI)
// - `error`     : types d'erreur centralises (BvError, BvResult)
// - `history`   : gestion de l'historique en memoire (ring buffer, recherche)
// - `storage`   : persistance chiffree sur disque (vault.dat)
// - `system`    : bindings Win32, hotkey, tray icon, autostart
// - `ui`        : interface graphique Win32 GDI (popup, sidebar, themes)

#![allow(non_snake_case, non_camel_case_types, dead_code)]
#![cfg(target_os = "windows")]

/// Orchestrateur principal de l'application.
pub mod app;
/// Surveillance et injection du presse-papiers Windows.
pub mod clipboard;
/// Configuration utilisateur et parseur de fichiers.
pub mod config;
/// Constantes globales de l'application.
pub mod constants;
/// Primitives cryptographiques (AES-256-GCM, SHA-256, PBKDF2, DPAPI).
pub mod crypto;
/// Types d'erreur centralises.
pub mod error;
/// Gestion de l'historique en memoire.
pub mod history;
/// Persistance chiffree sur disque.
pub mod storage;
/// Bindings Win32 et composants systeme.
pub mod system;
/// Interface graphique Win32 GDI.
pub mod ui;
