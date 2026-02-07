// BufferVault - Module history
// Gestion de l'historique du presse-papiers en memoire
//
// Ce module gere l'historique des entrees de presse-papiers en memoire.
// Il est independant de la plateforme (pas d'appels Win32 directs).
//
// # Sous-modules
// - `entry`  : structure ClipboardEntry avec type, flags, contenu et metadonnees
// - `ring`   : buffer circulaire HistoryRing avec capacite configurable,
//              support du pinning, deduplication et retention temporelle
// - `search` : recherche incrementale insensible a la casse dans les entrees
//
// # Architecture
// L'historique utilise un Vec<ClipboardEntry> avec gestion FIFO : les entrees
// les plus anciennes (non epinglees) sont supprimees quand la capacite max
// est atteinte. Un flag `dirty` permet de ne sauvegarder que si l'historique
// a ete modifie depuis la derniere sauvegarde.

/// Structure de donnees d'une entree de presse-papiers.
pub mod entry;
/// Buffer circulaire FIFO pour l'historique avec pinning et retention.
pub mod ring;
/// Recherche incrementale insensible a la casse dans les entrees.
pub mod search;
