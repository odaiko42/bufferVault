# BufferVault - Release Notes

## [0.1.2] - 2026-02-07

### Ameliore
- **Gestion d'erreur** : `autostart.rs` reecrit avec retour `BvResult` au lieu de `bool` pour toute interaction registre
- **Detection theme systeme** : `is_system_dark_mode()` lit la cle de registre `AppsUseLightTheme` (HKCU) au lieu d'un stub `false`
- **Testabilite** : 145 tests unitaires (x2 vs v0.1.1) avec couverture significativement elargie
- **Documentation** : JSDoc/Safety amelioree sur `tray.rs`, `win32.rs`, `splash.rs`, `renderer.rs`

### Nouveaux tests (68 nouveaux tests)
- `error.rs` : 10 tests (Display de chaque variante, From<io::Error>, Debug, BvResult ok/err)
- `system/win32.rs` : 14 tests (to/from_wstring, rgb, loword/hiword, csprng_fill, null constants, get_env_var)
- `system/tray.rs` : 3 tests (NID initialisation, tooltip court, tooltip troncature)
- `system/autostart.rs` : 2 tests (get_exe_path, is_autostart)
- `system/process.rs` : 3 tests supplementaires (case-insensitive, deep path, trailing backslash)
- `ui/manager.rs` : 9 tests (new, toggle check, toggle all, checked indices, confirm/cancel edit, start edit, bounds)
- `ui/popup.rs` : 7 tests (new, search push/pop, move up/down, scroll, resolve selected, hide clears)
- `ui/sidebar.rs` : 4 tests (new, move up at zero, move down boundary, move empty)
- `ui/permanent.rs` : 3 tests (new, move up at zero, move down boundary)
- `ui/splash.rs` : 3 tests (constants, timer IDs, fade convergence)
- `ui/renderer.rs` : 3 tests (constants positifs, font sizes negatives, search bar hauteur)
- `ui/theme.rs` : 8 tests supplementaires (unknown defaults, palette contrast, shared colors, system mode, debug, clone)

### Corrige
- `autostart.rs` : refactoring complet avec helper `open_reg_key()`, elimination de code duplique
- `app.rs` : mise a jour de l'appel `toggle_autostart()` pour gerer `BvResult` avec log d'erreur

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
