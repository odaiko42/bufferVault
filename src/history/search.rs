// BufferVault - Recherche dans l'historique
// Filtrage par sous-chaine, insensible a la casse
//
// Ce module fournit la recherche incrementale dans l'historique :
// l'utilisateur tape un texte et les entrees sont filtrees en temps
// reel sur le contenu et le nom de l'application source.
//
// # Algorithme
// Recherche naive par `contains` en O(n*m) sur chaque entree.
// La recherche est insensible a la casse (to_lowercase).
// Si la requete est vide, tous les indices sont retournes.
//
// # Portabilite
// Ce module est en pur Rust, sans dependance Win32.

use crate::history::entry::ClipboardEntry;

/// Filtre les entrees dont le contenu ou la source contiennent la requete.
/// Recherche insensible a la casse.
/// Retourne les indices des entrees correspondantes.
pub fn search_entries(entries: &[ClipboardEntry], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..entries.len()).collect();
    }
    let query_lower = query.to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            e.content.to_lowercase().contains(&query_lower)
                || e.source_app.to_lowercase().contains(&query_lower)
        })
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::entry::EntryType;

    fn make(content: &str, source: &str) -> ClipboardEntry {
        ClipboardEntry::new(EntryType::Text, source.into(), content.into())
    }

    #[test]
    fn test_search_empty_query() {
        let entries = vec![make("hello", "app"), make("world", "app")];
        assert_eq!(search_entries(&entries, ""), vec![0, 1]);
    }

    #[test]
    fn test_search_content_match() {
        let entries = vec![make("Hello World", "app"), make("foo bar", "app")];
        assert_eq!(search_entries(&entries, "hello"), vec![0]);
    }

    #[test]
    fn test_search_source_match() {
        let entries = vec![make("data", "notepad.exe"), make("data", "chrome.exe")];
        assert_eq!(search_entries(&entries, "notepad"), vec![0]);
    }

    #[test]
    fn test_search_no_match() {
        let entries = vec![make("hello", "app")];
        assert_eq!(search_entries(&entries, "xyz"), Vec::<usize>::new());
    }

    #[test]
    fn test_search_case_insensitive() {
        let entries = vec![make("HELLO", "APP")];
        assert_eq!(search_entries(&entries, "hello"), vec![0]);
    }
}
