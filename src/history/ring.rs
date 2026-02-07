// BufferVault - Ring buffer pour l'historique
// Structure FIFO avec support d'epinglage
//
// Ce module implemente le stockage en memoire des entrees du
// presse-papiers sous forme de liste ordonnee (plus recente en tete).
//
// # Capacite et rotation
// Lorsque la capacite maximale est atteinte, les entrees les plus
// anciennes non epinglees sont supprimees. Les entrees epinglees
// sont protegees de la rotation.
//
// # Dirty flag
// Le flag `dirty` est positionne a chaque modification et remis a
// zero apres une sauvegarde reussie (via `reset_dirty`).
// Cela permet de ne sauvegarder que lorsque necessaire.
//
// # Deduplication
// `push` refuse l'insertion si le contenu est identique a la
// derniere entree en tete.
//
// # Portabilite
// Ce module est en pur Rust, sans dependance Win32.

use crate::history::entry::ClipboardEntry;

/// Historique du presse-papiers en memoire.
/// Les entrees sont stockees dans un Vec, les plus recentes en tete.
pub struct HistoryRing {
    entries: Vec<ClipboardEntry>,
    capacity: usize,
    dirty: bool,
}

impl HistoryRing {
    /// Cree un nouveau ring buffer avec la capacite donnee.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity.min(1024)),
            capacity,
            dirty: false,
        }
    }

    /// Ajoute une entree en tete de l'historique.
    /// Si la capacite est atteinte, supprime la plus ancienne non epinglee.
    /// Retourne false si l'entree est un doublon de la derniere.
    pub fn push(&mut self, entry: ClipboardEntry) -> bool {
        // Deduplication : verifier si identique a la derniere entree
        if let Some(last) = self.entries.first() {
            if last.content_equals(&entry) {
                return false;
            }
        }

        // Inserer en tete
        self.entries.insert(0, entry);
        self.dirty = true;

        // Rotation si necessaire
        self.enforce_capacity();
        true
    }

    /// Supprime les entrees excedentaires (les plus anciennes non epinglees).
    fn enforce_capacity(&mut self) {
        while self.entries.len() > self.capacity {
            // Trouver la derniere entree non epinglee
            let mut removed = false;
            for i in (0..self.entries.len()).rev() {
                if !self.entries[i].flags.pinned {
                    self.entries.remove(i);
                    removed = true;
                    break;
                }
            }
            // Si toutes les entrees sont epinglees, on ne peut plus supprimer
            if !removed { break; }
        }
    }

    /// Retourne l'entree a l'index donne (0 = plus recente).
    pub fn get(&self, index: usize) -> Option<&ClipboardEntry> {
        self.entries.get(index)
    }

    /// Retourne une reference mutable a l'entree a l'index donne.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut ClipboardEntry> {
        self.entries.get_mut(index)
    }

    /// Supprime l'entree a l'index donne.
    pub fn remove(&mut self, index: usize) -> Option<ClipboardEntry> {
        if index < self.entries.len() {
            self.dirty = true;
            Some(self.entries.remove(index))
        } else {
            None
        }
    }

    /// Epingle ou desepingle l'entree a l'index donne.
    pub fn toggle_pin(&mut self, index: usize) -> bool {
        if let Some(entry) = self.entries.get_mut(index) {
            entry.flags.pinned = !entry.flags.pinned;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Purge toutes les entrees non epinglees.
    pub fn clear_unpinned(&mut self) {
        self.entries.retain(|e| e.flags.pinned);
        self.dirty = true;
    }

    /// Purge tout l'historique.
    pub fn clear_all(&mut self) {
        self.entries.clear();
        self.dirty = true;
    }

    /// Supprime les entrees plus anciennes que le nombre de jours donne.
    pub fn apply_retention(&mut self, max_age_days: u32) {
        let max_age_secs = max_age_days as u64 * 86400;
        self.entries.retain(|e| e.flags.pinned || e.age_secs() <= max_age_secs);
        self.dirty = true;
    }

    /// Nombre d'entrees dans l'historique.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Capacite maximale.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Retourne true si l'historique est vide.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Retourne true si l'historique a ete modifie depuis le dernier reset.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marque l'historique comme non modifie.
    pub fn reset_dirty(&mut self) {
        self.dirty = false;
    }

    /// Retourne un iterateur sur les entrees (plus recente en premier).
    pub fn iter(&self) -> std::slice::Iter<'_, ClipboardEntry> {
        self.entries.iter()
    }

    /// Retourne toutes les entrees comme slice.
    pub fn as_slice(&self) -> &[ClipboardEntry] {
        &self.entries
    }

    /// Reconstruit l'historique a partir d'un vecteur d'entrees.
    pub fn load_from(&mut self, entries: Vec<ClipboardEntry>) {
        self.entries = entries;
        self.dirty = false;
    }

    /// Retourne les entrees comme vecteur (pour la serialisation).
    pub fn to_vec(&self) -> Vec<ClipboardEntry> {
        self.entries.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::entry::EntryType;

    fn make_entry(content: &str) -> ClipboardEntry {
        ClipboardEntry::new(EntryType::Text, "test.exe".into(), content.into())
    }

    #[test]
    fn test_push_and_get() {
        let mut ring = HistoryRing::new(10);
        ring.push(make_entry("hello"));
        ring.push(make_entry("world"));
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.get(0).unwrap().content, "world"); // plus recente
        assert_eq!(ring.get(1).unwrap().content, "hello");
    }

    #[test]
    fn test_deduplication() {
        let mut ring = HistoryRing::new(10);
        assert!(ring.push(make_entry("hello")));
        assert!(!ring.push(make_entry("hello"))); // doublon
        assert_eq!(ring.len(), 1);
    }

    #[test]
    fn test_capacity_enforcement() {
        let mut ring = HistoryRing::new(3);
        ring.push(make_entry("a"));
        ring.push(make_entry("b"));
        ring.push(make_entry("c"));
        ring.push(make_entry("d"));
        assert_eq!(ring.len(), 3);
        // "a" (la plus ancienne) doit avoir ete supprimee
        assert_eq!(ring.get(0).unwrap().content, "d");
        assert_eq!(ring.get(2).unwrap().content, "b");
    }

    #[test]
    fn test_pinned_not_removed() {
        let mut ring = HistoryRing::new(2);
        ring.push(make_entry("pinned"));
        ring.toggle_pin(0);
        ring.push(make_entry("second"));
        ring.push(make_entry("third"));
        // "pinned" ne doit pas etre supprime
        let pinned = ring.iter().any(|e| e.content == "pinned" && e.flags.pinned);
        assert!(pinned);
    }

    #[test]
    fn test_remove() {
        let mut ring = HistoryRing::new(10);
        ring.push(make_entry("a"));
        ring.push(make_entry("b"));
        ring.push(make_entry("c"));
        ring.remove(1); // supprime "b"
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.get(0).unwrap().content, "c");
        assert_eq!(ring.get(1).unwrap().content, "a");
    }

    #[test]
    fn test_clear_unpinned() {
        let mut ring = HistoryRing::new(10);
        ring.push(make_entry("a"));
        ring.push(make_entry("b"));
        ring.toggle_pin(0);
        ring.clear_unpinned();
        assert_eq!(ring.len(), 1);
        assert_eq!(ring.get(0).unwrap().content, "b");
    }

    #[test]
    fn test_dirty_flag() {
        let mut ring = HistoryRing::new(10);
        assert!(!ring.is_dirty());
        ring.push(make_entry("x"));
        assert!(ring.is_dirty());
        ring.reset_dirty();
        assert!(!ring.is_dirty());
    }
}
