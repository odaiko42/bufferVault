// BufferVault - Module storage
// Persistance chiffree de l'historique sur disque
//
// Ce module gere la sauvegarde et le chargement de l'historique
// dans un fichier vault.dat chiffre avec AES-256-GCM.
//
// # Sous-modules
// - `format` : serialisation/deserialisation binaire des ClipboardEntry
//              Format proprietaire compact avec prefixe de taille par entree
// - `vault`  : lecture/ecriture du fichier vault.dat avec chiffrement,
//              verification HMAC d'integrite et ecriture atomique (temp+rename)
//
// # Format du fichier vault.dat
// ```text
// [MAGIC 8B][VERSION 4B][NONCE 12B][TAG 16B][CT_LEN 4B][CIPHERTEXT][HMAC 32B]
// ```
//
// # Securite
// - Chiffrement AES-256-GCM avec nonce aleatoire unique par sauvegarde
// - HMAC-SHA256 sur le fichier entier pour verification d'integrite
// - Ecriture atomique pour eviter la corruption en cas de crash

/// Serialisation/deserialisation binaire des entrees de l'historique.
pub mod format;
/// Lecture/ecriture du fichier vault.dat chiffre avec AES-256-GCM.
pub mod vault;
