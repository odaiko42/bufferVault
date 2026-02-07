# BufferVault - Release Notes

## [0.1.3] - 2026-02-07

### Ameliore
- **ui/manager.rs** : refactoring complet -- extraction de `draw_checkbox()`, `draw_edit_mode()`, `draw_normal_entry()` depuis le monolithique `draw_manager_entry()`. Ajout de 15+ constantes nommees, `BvResult` sur `create_window()`, `is_editing()` accesseur, `#[must_use]` / `#[inline]` sur les methodes pures
- **ui/popup.rs** : nettoyage du code mort dans `paint()`, ajout de `POPUP_WIDTH_BASE` / `SCROLL_STEP`, retour `BvResult` sur `create_window()`
- **ui/sidebar.rs** : ajout de `scroll()`, constantes `FALLBACK_VISIBLE_COUNT` / `SCROLL_STEP`, retour `BvResult` sur `create_window()`
- **ui/permanent.rs** : ajout de `scroll()`, 5 constantes nommees, retour `BvResult` sur `create_window()`
- **ui/renderer.rs** : suppression du parametre `_dpi` inutilise dans `create_font()`, ajout de `fade_step_count()` utilitaire, constantes `PIN_INDICATOR_WIDTH` / `PREVIEW_MAX_CHARS` / `FONT_FACE`
- **ui/splash.rs** : extraction de 15+ constantes depuis les magic numbers (dimensions, couleurs, polices), `draw_border()` helper, `alpha()` accesseur
- **ui/mod.rs** : facade re-exports pour tous les types publics UI, fonctions de validation `validated_visible_count()` / `validated_selection()`
- **Gestion d'erreur** : les 4 fenetres (popup, sidebar, permanent, manager) retournent `BvResult` sur `create_window()` avec gestion dans `app.rs`
- **Documentation** : commentaires SAFETY detailles sur tous les blocs unsafe GDI, doc-comments JSDoc complets sur toutes les methodes publiques
- **573 tests unitaires** (vs 484 en v0.1.2), 0 warnings

### Nouveaux tests (89 nouveaux tests)
- `ui/manager.rs` : 35 tests (10 -> 35) -- is_editing, toggle twice/empty, indices desc all/none/empty, checked count all/none/empty, start_edit single line/out of bounds/empty/sets is_editing, confirm no edit/clears editing, cancel resets, scroll clamp, hide clears, destroy null, constantes
- `ui/popup.rs` : 16 tests (6 -> 16) -- constantes, destroy, hide search, scroll boundaries/max clamp, search push/pop resets selection, move up adjusts scroll
- `ui/sidebar.rs` : 15 tests (4 -> 15) -- constantes, destroy, toggle null, visible count null, scroll up/down/empty, move up adjusts scroll
- `ui/permanent.rs` : 15 tests (3 -> 15) -- constantes, class name, destroy, toggle null, visible count null, scroll up/down/empty, move up adjusts scroll
- `ui/renderer.rs` : 12 tests (3 -> 12) -- fade_step_count normal/zero/one/equals/exceeds, font face, preview max chars, search bar, padding
- `ui/splash.rs` : 14 tests (3 -> 14) -- dimensions, couleurs, polices, fade duration, layout fits, timer ids unique, bg dark, title bright, font larger
- `ui/mod.rs` : 12 tests (0 -> 12) -- facade re-exports, validated_visible_count zero/valid/too large, validated_selection valid/empty/out of range, bounds coherence

### Corrige
- `popup.rs` : code mort supprime dans `paint()` (variables `display_entries` / `display_slice` jamais utilisees)
- `app.rs` : 3 appels `create_window()` mis a jour pour gerer `BvResult` avec log d'erreur

---

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
