// BufferVault - Module clipboard
// Surveillance et injection dans le presse-papiers Windows
//
// Ce module fournit deux sous-modules complementaires :
// - `monitor` : ecoute les changements du presse-papiers via AddClipboardFormatListener
//   et capture automatiquement le contenu (texte, fichiers) en ClipboardEntry.
// - `injector` : ecrit du texte dans le presse-papiers et peut simuler Ctrl+V
//   pour coller le contenu dans l'application cible.
//
// Architecture :
// Le module est specifique a Windows (Win32 API) et utilise des blocs unsafe
// isoles pour chaque appel FFI. Toutes les fonctions publiques retournent
// des BvResult pour une gestion d'erreur explicite.
//
// # Securite
// - Le presse-papiers est ouvert/ferme dans le meme scope (RAII-like)
// - Les donnees sensibles ne sont jamais loguees
// - Les applications exclues sont filtrees avant capture

/// Injection de texte dans le presse-papiers Windows.
pub mod injector;
/// Surveillance des changements du presse-papiers via Win32 API.
pub mod monitor;
