// BufferVault - Serialisation/deserialisation binaire
// Format proprietaire compact pour les entrees du vault
//
// Ce module serialise les ClipboardEntry en format binaire compact
// pour le stockage dans le fichier vault chiffre.
//
// # Format d'une entree
// ```text
// [timestamp]    i64 LE (8 octets)
// [entry_type]   u8 (1 octet)
// [flags]        u8 (1 octet)
// [source_len]   u16 LE (2 octets)
// [source]       source_len octets UTF-8
// [content_len]  u32 LE (4 octets)
// [content]      content_len octets UTF-8
// ```
//
// # Format d'un vecteur d'entrees
// ```text
// [count]        u32 LE (4 octets)
// Pour chaque entree :
//   [entry_size]  u32 LE (4 octets)
//   [entry_data]  entry_size octets
// ```
//
// # Robustesse
// Chaque champ est valide avant lecture (taille restante verifiee).
// Les chaines invalides sont traitees via from_utf8_lossy.
//
// # Portabilite
// Ce module est en pur Rust, sans dependance Win32.

use crate::history::entry::{ClipboardEntry, EntryFlags, EntryType};
use crate::error::{BvError, BvResult};

/// Serialise une entree en format binaire.
///
/// Format :
/// - timestamp : i64 LE (8 octets)
/// - entry_type : u8 (1 octet)
/// - flags : u8 (1 octet)
/// - source_len : u16 LE (2 octets)
/// - source : source_len octets (UTF-8)
/// - content_len : u32 LE (4 octets)
/// - content : content_len octets (UTF-8)
pub fn serialize_entry(entry: &ClipboardEntry) -> Vec<u8> {
    let source_bytes = entry.source_app.as_bytes();
    let content_bytes = entry.content.as_bytes();
    let total = 8 + 1 + 1 + 2 + source_bytes.len() + 4 + content_bytes.len();

    let mut buf = Vec::with_capacity(total);
    buf.extend_from_slice(&entry.timestamp.to_le_bytes());
    buf.push(entry.entry_type as u8);
    buf.push(entry.flags.to_byte());
    buf.extend_from_slice(&(source_bytes.len() as u16).to_le_bytes());
    buf.extend_from_slice(source_bytes);
    buf.extend_from_slice(&(content_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(content_bytes);
    buf
}

/// Deserialise une entree depuis un format binaire.
/// Retourne l'entree et le nombre d'octets consommes.
pub fn deserialize_entry(data: &[u8]) -> BvResult<(ClipboardEntry, usize)> {
    let mut pos = 0;

    // timestamp (8)
    if data.len() < pos + 8 {
        return Err(BvError::Integrity("Entry too short for timestamp".into()));
    }
    let timestamp = i64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
    pos += 8;

    // entry_type (1)
    if data.len() < pos + 1 {
        return Err(BvError::Integrity("Entry too short for type".into()));
    }
    let entry_type = EntryType::from_u8(data[pos])
        .ok_or_else(|| BvError::Integrity(format!("Invalid entry type: {}", data[pos])))?;
    pos += 1;

    // flags (1)
    if data.len() < pos + 1 {
        return Err(BvError::Integrity("Entry too short for flags".into()));
    }
    let flags = EntryFlags::from_byte(data[pos]);
    pos += 1;

    // source_len (2)
    if data.len() < pos + 2 {
        return Err(BvError::Integrity("Entry too short for source_len".into()));
    }
    let source_len = u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
    pos += 2;

    // source
    if data.len() < pos + source_len {
        return Err(BvError::Integrity("Entry too short for source".into()));
    }
    let source_app = String::from_utf8_lossy(&data[pos..pos + source_len]).to_string();
    pos += source_len;

    // content_len (4)
    if data.len() < pos + 4 {
        return Err(BvError::Integrity("Entry too short for content_len".into()));
    }
    let content_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;

    // content
    if data.len() < pos + content_len {
        return Err(BvError::Integrity("Entry too short for content".into()));
    }
    let content = String::from_utf8_lossy(&data[pos..pos + content_len]).to_string();
    pos += content_len;

    let entry = ClipboardEntry {
        timestamp,
        entry_type,
        flags,
        source_app,
        content,
    };

    Ok((entry, pos))
}

/// Serialise un vecteur d'entrees.
pub fn serialize_entries(entries: &[ClipboardEntry]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for entry in entries {
        let data = serialize_entry(entry);
        // Prefixer chaque entree par sa taille pour faciliter le parsing
        buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
        buf.extend_from_slice(&data);
    }
    buf
}

/// Deserialise un vecteur d'entrees.
pub fn deserialize_entries(data: &[u8]) -> BvResult<Vec<ClipboardEntry>> {
    if data.len() < 4 {
        return Err(BvError::Integrity("Data too short for entry count".into()));
    }

    let count = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let mut entries = Vec::with_capacity(count.min(10000));
    let mut pos = 4;

    for _ in 0..count {
        if pos + 4 > data.len() {
            return Err(BvError::Integrity("Data truncated before entry size".into()));
        }
        let entry_size = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        if pos + entry_size > data.len() {
            return Err(BvError::Integrity("Data truncated in entry body".into()));
        }
        let (entry, consumed) = deserialize_entry(&data[pos..pos + entry_size])?;
        if consumed != entry_size {
            return Err(BvError::Integrity("Entry size mismatch".into()));
        }
        entries.push(entry);
        pos += entry_size;
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry() -> ClipboardEntry {
        ClipboardEntry {
            timestamp: 1700000000,
            entry_type: EntryType::Text,
            flags: EntryFlags { pinned: true },
            source_app: "notepad.exe".into(),
            content: "Hello, World!".into(),
        }
    }

    #[test]
    fn test_entry_roundtrip() {
        let entry = make_entry();
        let data = serialize_entry(&entry);
        let (decoded, consumed) = deserialize_entry(&data).unwrap();
        assert_eq!(consumed, data.len());
        assert_eq!(decoded.timestamp, entry.timestamp);
        assert_eq!(decoded.entry_type, entry.entry_type);
        assert_eq!(decoded.flags.pinned, entry.flags.pinned);
        assert_eq!(decoded.source_app, entry.source_app);
        assert_eq!(decoded.content, entry.content);
    }

    #[test]
    fn test_entries_roundtrip() {
        let entries = vec![
            make_entry(),
            ClipboardEntry {
                timestamp: 1700000001,
                entry_type: EntryType::FileDrop,
                flags: EntryFlags::default(),
                source_app: "explorer.exe".into(),
                content: "C:\\file.txt".into(),
            },
        ];
        let data = serialize_entries(&entries);
        let decoded = deserialize_entries(&data).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].content, "Hello, World!");
        assert_eq!(decoded[1].content, "C:\\file.txt");
    }

    #[test]
    fn test_deserialize_truncated() {
        let result = deserialize_entries(&[0, 0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_entries() {
        let data = serialize_entries(&[]);
        let decoded = deserialize_entries(&data).unwrap();
        assert!(decoded.is_empty());
    }
}
