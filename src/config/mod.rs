// BufferVault - Module config
// Lecture/ecriture de la configuration utilisateur
//
// Ce module gere la configuration de BufferVault via un fichier texte
// au format cle-valeur avec sections, situe dans %APPDATA%\BufferVault\config.txt.
//
// # Sous-modules
// - `parser`   : parseur generique de fichiers cle-valeur avec sections,
//                commentaires, guillemets et listes. Supporte la serialisation
//                et la deserialisation bidirectionnelle.
// - `settings` : structure Settings contenant tous les parametres de l'application
//                (hotkey, affichage, theme, securite, exclusions) avec valeurs
//                par defaut robustes et validation des plages.
//
// # Utilisation
// ```rust
// let settings = Settings::load(&config_path);
// // Les valeurs manquantes utilisent les defauts
// ```

/// Parseur de fichiers de configuration au format cle-valeur avec sections.
pub mod parser;
/// Structure de configuration et valeurs par defaut de l'application.
pub mod settings;
