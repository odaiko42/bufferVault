// BufferVault - Parseur format cle-valeur
// Format simple : sections [nom], cle = valeur, commentaires #
//
// Ce module implemente un parseur de configuration minimaliste
// qui ne depend d'aucune crate externe (pas de serde, toml, etc.).
//
// # Format supporte
// - Sections : `[section_name]`
// - Cle-valeur : `key = value`
// - Guillemets : `key = "value with spaces"`
// - Commentaires : `# ligne entiere` ou `key = value # inline`
// - Listes : `key = ["a", "b", "c"]`
// - Valeurs sans section sont affectees a la section "general"
//
// # Serialisation
// `serialize_config` produit un fichier deterministe (sections et cles
// triees alphabetiquement). Les valeurs avec espaces sont auto-quotees.
//
// # Portabilite
// Ce module est en pur Rust, sans dependance Win32.

use std::collections::HashMap;

/// Resultat du parsing : sections contenant des paires cle-valeur.
pub type ParsedConfig = HashMap<String, HashMap<String, String>>;

/// Parse un fichier de configuration au format cle-valeur avec sections.
///
/// Format supporte :
/// - Sections : `[section_name]`
/// - Cle-valeur : `key = value` ou `key = "value with spaces"`
/// - Commentaires : lignes commencant par `#`
/// - Listes : `key = ["a", "b", "c"]`
pub fn parse_config(text: &str) -> ParsedConfig {
    let mut config = ParsedConfig::new();
    let mut current_section = String::from("general");

    for line in text.lines() {
        let trimmed = line.trim();

        // Ignorer les lignes vides et commentaires
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Detection de section
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].trim().to_string();
            continue;
        }

        // Parsing cle = valeur
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            let raw_value = trimmed[eq_pos + 1..].trim();

            // Supprimer les commentaires inline (apres #, mais pas dans les strings)
            let value = strip_inline_comment(raw_value);
            let value = strip_quotes(&value);

            config
                .entry(current_section.clone())
                .or_default()
                .insert(key, value);
        }
    }

    config
}

/// Supprime les commentaires inline (apres #) en respectant les guillemets.
fn strip_inline_comment(s: &str) -> String {
    let mut in_quotes = false;
    let mut in_brackets = false;
    for (i, c) in s.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            '[' if !in_quotes => in_brackets = true,
            ']' if !in_quotes => in_brackets = false,
            '#' if !in_quotes && !in_brackets => {
                return s[..i].trim().to_string();
            }
            _ => {}
        }
    }
    s.to_string()
}

/// Supprime les guillemets autour d'une valeur.
fn strip_quotes(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Serialise une configuration en texte.
pub fn serialize_config(config: &ParsedConfig) -> String {
    let mut out = String::new();
    // Trier les sections pour un resultat deterministe
    let mut sections: Vec<_> = config.keys().collect();
    sections.sort();

    for section in sections {
        out.push_str(&format!("[{}]\n", section));
        if let Some(pairs) = config.get(section) {
            let mut keys: Vec<_> = pairs.keys().collect();
            keys.sort();
            for key in keys {
                let value = &pairs[key];
                // Utiliser des guillemets si la valeur contient des espaces
                if value.contains(' ') || value.contains('#') {
                    out.push_str(&format!("{} = \"{}\"\n", key, value));
                } else {
                    out.push_str(&format!("{} = {}\n", key, value));
                }
            }
        }
        out.push('\n');
    }

    out
}

/// Parse une valeur comme booleen.
pub fn parse_bool(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Some(true),
        "false" | "no" | "0" | "off" => Some(false),
        _ => None,
    }
}

/// Parse une valeur comme u32.
pub fn parse_u32(value: &str) -> Option<u32> {
    value.trim().parse().ok()
}

/// Parse une valeur comme usize.
pub fn parse_usize(value: &str) -> Option<usize> {
    value.trim().parse().ok()
}

/// Parse une liste de type ["a", "b", "c"].
pub fn parse_string_list(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Vec::new();
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    inner
        .split(',')
        .map(|s| strip_quotes(s.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let text = r#"
[general]
max_history = 500
auto_start = true

[theme]
mode = "dark"
"#;
        let config = parse_config(text);
        assert_eq!(config["general"]["max_history"], "500");
        assert_eq!(config["general"]["auto_start"], "true");
        assert_eq!(config["theme"]["mode"], "dark");
    }

    #[test]
    fn test_parse_comments() {
        let text = "# This is a comment\nkey = value # inline comment\n";
        let config = parse_config(text);
        assert_eq!(config["general"]["key"], "value");
    }

    #[test]
    fn test_parse_quoted_value() {
        let text = "name = \"Hello World\"\n";
        let config = parse_config(text);
        assert_eq!(config["general"]["name"], "Hello World");
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("yes"), Some(true));
        assert_eq!(parse_bool("invalid"), None);
    }

    #[test]
    fn test_parse_string_list() {
        let list = parse_string_list(r#"["KeePass.exe", "1Password.exe"]"#);
        assert_eq!(list, vec!["KeePass.exe", "1Password.exe"]);
    }

    #[test]
    fn test_parse_empty_list() {
        let list = parse_string_list("[]");
        assert!(list.is_empty());
    }

    #[test]
    fn test_serialize_roundtrip() {
        let text = "[general]\nkey = value\n\n[theme]\nmode = dark\n\n";
        let config = parse_config(text);
        let serialized = serialize_config(&config);
        let reparsed = parse_config(&serialized);
        assert_eq!(config, reparsed);
    }
}
