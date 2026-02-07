# BufferVault - Release Notes

## [0.1.1] - 2026-02-07

### Ameliore
- Documentation exhaustive ajoutee aux 40 fichiers sources (.rs)
- Headers de module avec description d'architecture, securite et portabilite
- References RFC/FIPS ajoutees aux modules crypto (AES: FIPS 197, GCM: SP 800-38D, SHA-256: FIPS 180-4, HMAC: RFC 2104, PBKDF2: RFC 8018)
- Commentaires SAFETY sur tous les blocs `unsafe` et pointeurs statiques
- Documentation des structs d'etat UI (PopupState, SidebarState, PermanentState, ManagerState, SplashState, DpiContext, ThemePalette, RenderContext)
- Documentation des schemas binaires dans storage/format.rs et storage/vault.rs
- Documentation complete de la struct App et de ses methodes (cycle de vie, messages, wndproc)
- Documentation des 7 fichiers mod.rs avec descriptions architecturales

### Corrige
- Doc-test en echec sur `extract_filename` dans system/process.rs (fonction privee non accessible depuis un doc-test externe, corrige avec `ignore`)

### Inchange
- Aucune modification fonctionnelle du code
- 77 tests unitaires passent sans regression

---

## [0.1.0] - Unreleased
- Infrastructure de base : capture presse-papiers via Win32 API
- Stockage chiffre (AES-256-GCM, PBKDF2, DPAPI)
- Historique en ring buffer avec rotation FIFO
- Mode d'affichage popup
- Raccourci clavier global configurable (defaut : Win+Shift+V)
- Injection presse-papiers et collage automatique (SendInput)
- Icone zone de notification avec menu contextuel
- Fichier de configuration (format cle-valeur)
- Zeroing securise des buffers sensibles
- Ecriture atomique du fichier vault

---

## Roadmap

| Version | Description |
|---------|-------------|
| 0.1.0   | Fondations : capture, chiffrement, popup, raccourci |
| 0.2.0   | Modes sidebar et permanent, themes, config complete |
| 0.3.0   | Menu contextuel Windows, epinglage, recherche, exclusions |
| 0.4.0   | Multi-ecrans, DPI, polish UI |
| 0.5.0   | Optimisations performance, stabilite longue duree |
| 1.0.0   | Premiere version stable |

---

*Format : [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/)*  
*Versioning : [Semantic Versioning](https://semver.org/lang/fr/)*
