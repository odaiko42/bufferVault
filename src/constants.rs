// BufferVault - Constantes globales
//
// Ce module centralise toutes les constantes de l'application :
// - Format de fichier vault (magic, version, tailles)
// - Valeurs par defaut de la configuration
// - Tailles cryptographiques (AES, HMAC, PBKDF2)
// - Identifiants systeme (hotkey, tray, messages Windows)
// - Noms de fichiers et repertoires
//
// Les constantes sont utilisees par les modules storage, crypto,
// config, system et ui pour garantir la coherence des valeurs.

/// Magic number du fichier vault : "BVAULT01"
pub const VAULT_MAGIC: &[u8; 8] = b"BVAULT01";

/// Version du format de fichier vault
pub const VAULT_FORMAT_VERSION: u32 = 1;

/// Nombre max d'entrees par defaut
pub const DEFAULT_MAX_HISTORY: usize = 500;

/// Taille max par entree par defaut (1 Mo)
pub const DEFAULT_MAX_ENTRY_SIZE: usize = 1_048_576;

/// Retention par defaut (jours)
pub const DEFAULT_RETENTION_DAYS: u32 = 30;

/// Iterations PBKDF2 par defaut
pub const DEFAULT_PBKDF2_ITERATIONS: u32 = 100_000;

/// Taille cle AES-256 (octets)
pub const AES_KEY_SIZE: usize = 32;

/// Taille nonce AES-GCM (octets)
pub const AES_GCM_NONCE_SIZE: usize = 12;

/// Taille tag AES-GCM (octets)
pub const AES_GCM_TAG_SIZE: usize = 16;

/// Taille du salt PBKDF2 (octets)
pub const PBKDF2_SALT_SIZE: usize = 32;

/// Taille HMAC-SHA256 (octets)
pub const HMAC_SIZE: usize = 32;

/// Intervalle de sauvegarde auto (ms)
pub const AUTO_SAVE_INTERVAL_MS: u32 = 30_000;

/// Elements visibles par defaut dans le popup
pub const DEFAULT_VISIBLE_ITEMS: usize = 8;

/// Longueur d'apercu par defaut (caracteres)
pub const DEFAULT_PREVIEW_LENGTH: usize = 60;

/// Nom du dossier application dans %APPDATA%
pub const APP_DIR_NAME: &str = "BufferVault";

/// Nom du fichier vault
pub const VAULT_FILENAME: &str = "vault.dat";

/// Nom du fichier de configuration
pub const CONFIG_FILENAME: &str = "config.txt";

/// Nom du fichier keystore (blob DPAPI)
pub const KEYSTORE_FILENAME: &str = "keystore.bin";

/// ID du hotkey global
pub const HOTKEY_ID: i32 = 1;

/// ID de l'icone de notification
pub const TRAY_ICON_ID: u32 = 1;

/// Message custom pour l'icone tray
pub const WM_TRAY_CALLBACK: u32 = 0x0400 + 100;

/// Taille d'un bloc AES (octets)
pub const AES_BLOCK_SIZE: usize = 16;
