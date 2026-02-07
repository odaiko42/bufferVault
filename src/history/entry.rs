// BufferVault - Structure ClipboardEntry
// Represente une entree dans l'historique du presse-papiers
//
// Ce module definit les types de donnees fondamentaux de l'historique :
// - `EntryType` : type de contenu (texte, fichier, etc.)
// - `EntryFlags` : drapeaux (epingle, etc.) serialises sur 1 octet
// - `ClipboardEntry` : entree complete avec timestamp, source, contenu
//
// # Serialisation
// Chaque entree est serialisable en format binaire compact
// (voir storage/format.rs). Le timestamp est en secondes UTC
// depuis l'epoch Unix.
//
// # Deduplication
// `content_equals` compare uniquement le contenu et le type,
// pas la source ni le timestamp, pour la deduplication en push.
//
// # Portabilite
// Ce module est en pur Rust, sans dependance Win32.

/// Type de contenu de l'entree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EntryType {
    /// Texte brut (CF_UNICODETEXT)
    Text = 0,
    /// Texte brut (CF_TEXT)
    PlainText = 1,
    /// Chemins de fichiers (CF_HDROP)
    FileDrop = 2,
}

impl EntryType {
    /// Convertit un octet en EntryType.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Text),
            1 => Some(Self::PlainText),
            2 => Some(Self::FileDrop),
            _ => None,
        }
    }
}

/// Flags d'une entree.
#[derive(Debug, Clone, Copy, Default)]
pub struct EntryFlags {
    /// L'entree est epinglee (protegee de la rotation)
    pub pinned: bool,
}

impl EntryFlags {
    /// Serialise les flags en un octet.
    pub fn to_byte(self) -> u8 {
        if self.pinned { 1 } else { 0 }
    }

    /// Deserialise les flags depuis un octet.
    pub fn from_byte(b: u8) -> Self {
        Self { pinned: (b & 1) != 0 }
    }
}

/// Une entree dans l'historique du presse-papiers.
#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    /// Horodatage UTC (secondes depuis epoch Unix)
    pub timestamp: i64,
    /// Type de contenu
    pub entry_type: EntryType,
    /// Flags (epingle, etc.)
    pub flags: EntryFlags,
    /// Nom de l'application source
    pub source_app: String,
    /// Contenu en clair (texte UTF-8 ou chemins)
    pub content: String,
}

impl ClipboardEntry {
    /// Cree une nouvelle entree avec le timestamp courant.
    pub fn new(entry_type: EntryType, source_app: String, content: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        Self {
            timestamp,
            entry_type,
            flags: EntryFlags::default(),
            source_app,
            content,
        }
    }

    /// Retourne un apercu tronque du contenu.
    pub fn preview(&self, max_len: usize) -> String {
        let first_line = self.content.lines().next().unwrap_or("");
        if first_line.len() <= max_len {
            first_line.to_string()
        } else {
            let mut s: String = first_line.chars().take(max_len - 3).collect();
            s.push_str("...");
            s
        }
    }

    /// Retourne l'age de l'entree en secondes.
    pub fn age_secs(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        (now - self.timestamp).max(0) as u64
    }

    /// Retourne une description relative de l'age.
    pub fn age_display(&self) -> String {
        let secs = self.age_secs();
        if secs < 60 {
            "A l'instant".to_string()
        } else if secs < 3600 {
            let m = secs / 60;
            format!("Il y a {} min", m)
        } else if secs < 86400 {
            let h = secs / 3600;
            format!("Il y a {}h", h)
        } else {
            let d = secs / 86400;
            format!("Il y a {}j", d)
        }
    }

    /// Verifie si le contenu est identique a un autre (deduplication).
    pub fn content_equals(&self, other: &ClipboardEntry) -> bool {
        self.content == other.content && self.entry_type == other.entry_type
    }

    /// Taille du contenu en octets.
    pub fn content_size(&self) -> usize {
        self.content.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_preview_short() {
        let e = ClipboardEntry::new(EntryType::Text, "test.exe".into(), "hello".into());
        assert_eq!(e.preview(50), "hello");
    }

    #[test]
    fn test_entry_preview_long() {
        let e = ClipboardEntry::new(EntryType::Text, "test.exe".into(), "a".repeat(100));
        let p = e.preview(20);
        assert!(p.len() <= 20);
        assert!(p.ends_with("..."));
    }

    #[test]
    fn test_entry_preview_multiline() {
        let e = ClipboardEntry::new(EntryType::Text, "".into(), "line1\nline2\nline3".into());
        assert_eq!(e.preview(50), "line1");
    }

    #[test]
    fn test_entry_content_equals() {
        let e1 = ClipboardEntry::new(EntryType::Text, "a.exe".into(), "hello".into());
        let e2 = ClipboardEntry::new(EntryType::Text, "b.exe".into(), "hello".into());
        let e3 = ClipboardEntry::new(EntryType::Text, "a.exe".into(), "world".into());
        assert!(e1.content_equals(&e2));
        assert!(!e1.content_equals(&e3));
    }

    #[test]
    fn test_entry_flags_roundtrip() {
        let f = EntryFlags { pinned: true };
        assert_eq!(EntryFlags::from_byte(f.to_byte()).pinned, true);
        let f2 = EntryFlags { pinned: false };
        assert_eq!(EntryFlags::from_byte(f2.to_byte()).pinned, false);
    }

    #[test]
    fn test_entry_type_roundtrip() {
        assert_eq!(EntryType::from_u8(0), Some(EntryType::Text));
        assert_eq!(EntryType::from_u8(1), Some(EntryType::PlainText));
        assert_eq!(EntryType::from_u8(2), Some(EntryType::FileDrop));
        assert_eq!(EntryType::from_u8(255), None);
    }
}
