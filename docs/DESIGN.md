# BufferVault - Document de conception

> Outil Windows de gestion d'historique du presse-papiers avec chiffrement

**Version** : 0.1.0  
**Date** : 2026-02-07  
**Langage** : Rust (sans dependances externes)  
**Plateforme** : Windows 10/11 (x86_64)

---

## Table des matieres

1. [Vision et objectifs](#1-vision-et-objectifs)
2. [Exigences fonctionnelles](#2-exigences-fonctionnelles)
3. [Exigences non-fonctionnelles](#3-exigences-non-fonctionnelles)
4. [Architecture](#4-architecture)
5. [Securite et chiffrement](#5-securite-et-chiffrement)
6. [Interface utilisateur](#6-interface-utilisateur)
7. [Integration Windows](#7-integration-windows)
8. [Structure du code](#8-structure-du-code)
9. [Gestion de configuration](#9-gestion-de-configuration)
10. [Cycle de vie des donnees](#10-cycle-de-vie-des-donnees)
11. [Gestion des versions](#11-gestion-des-versions)
12. [Plan d'implementation](#12-plan-dimplementation)
13. [Contraintes techniques](#13-contraintes-techniques)
14. [Glossaire](#14-glossaire)

---

## 1. Vision et objectifs

### 1.1 Vision

BufferVault est un outil systeme Windows leger et securise qui intercepte chaque operation de copie (Ctrl+C, menu contextuel "Copier") et conserve un historique chiffre de tous les elements copies. L'utilisateur peut a tout moment rappeler cet historique via un raccourci clavier configurable, parcourir les entrees et coller l'element de son choix, comme s'il venait d'etre copie.

### 1.2 Objectifs principaux

| # | Objectif | Priorite |
|---|----------|----------|
| O1 | Capturer automatiquement toute copie dans le presse-papiers Windows | Critique |
| O2 | Stocker l'historique de maniere chiffree (AES-256-GCM) | Critique |
| O3 | Permettre le rappel de l'historique par raccourci clavier configurable | Critique |
| O4 | Offrir plusieurs modes d'affichage configurables | Haute |
| O5 | Permettre le collage via Ctrl+V ou clic droit apres selection | Critique |
| O6 | Fonctionner sans aucune dependance externe (pure Rust + Win32 API) | Critique |
| O7 | Limiter chaque fichier source a 800 lignes maximum | Haute |
| O8 | Maintenir un fichier de notes de version (RELEASE_NOTES.md) | Haute |

### 1.3 Public cible

Utilisateurs Windows avances, developpeurs, professionnels manipulant regulierement des donnees sensibles (mots de passe, tokens, extraits de code).

---

## 2. Exigences fonctionnelles

### 2.1 Capture du presse-papiers

| ID | Exigence | Detail |
|----|----------|--------|
| F01 | Surveillance continue | BufferVault s'enregistre comme auditeur du presse-papiers Windows via `AddClipboardFormatListener` |
| F02 | Types supportes | Texte brut (CF_UNICODETEXT), texte riche (CF_TEXT), chemins de fichiers (CF_HDROP) |
| F03 | Deduplication | Si le contenu copie est identique au dernier element, il n'est pas duplique dans l'historique |
| F04 | Horodatage | Chaque entree est associee a un timestamp UTC |
| F05 | Taille maximale par entree | Configurable (defaut : 1 Mo). Les copies depassant cette taille sont tronquees avec indication |
| F06 | Source applicative | Le nom du processus source est capture si possible via `GetForegroundWindow` + `GetWindowThreadProcessId` |

### 2.2 Historique

| ID | Exigence | Detail |
|----|----------|--------|
| F10 | Capacite | Nombre maximal d'entrees configurable (defaut : 500) |
| F11 | Persistance | L'historique est sauvegarde sur disque de facon chiffree |
| F12 | Rotation | Quand la capacite maximale est atteinte, les entrees les plus anciennes sont supprimees (FIFO) |
| F13 | Suppression manuelle | L'utilisateur peut supprimer une entree individuellement ou purger tout l'historique |
| F14 | Recherche | Recherche textuelle dans l'historique via le panneau d'affichage |
| F15 | Epinglage | L'utilisateur peut epingler des entrees pour qu'elles ne soient jamais supprimees par la rotation |
| F16 | Categories | Possibilite d'etiqueter les entrees (optionnel, v0.3+) |

### 2.3 Selection et collage

| ID | Exigence | Detail |
|----|----------|--------|
| F20 | Rappel de l'historique | Un raccourci clavier global (defaut : `Win+Shift+V`) ouvre le panneau d'historique |
| F21 | Selection | Clic simple ou navigation clavier (fleches + Entree) pour selectionner une entree |
| F22 | Injection dans le presse-papiers | L'entree selectionnee est placee dans le presse-papiers Windows |
| F23 | Collage automatique | Apres selection, l'outil peut optionnellement simuler un `Ctrl+V` dans l'application active |
| F24 | Collage par clic droit | L'historique reste accessible dans le menu contextuel Windows (shell extension) |
| F25 | Fermeture automatique | Le panneau se ferme apres selection (configurable) |
| F26 | Annulation | Echap ferme le panneau sans modifier le presse-papiers |

### 2.4 Configuration utilisateur

| ID | Exigence | Detail |
|----|----------|--------|
| F30 | Fichier de config | Format TOML-like (parse interne, pas de dependance) stocke dans `%APPDATA%\BufferVault\config.txt` |
| F31 | Raccourci clavier | Configurable (combinaison de modificateurs + touche) |
| F32 | Mode d'affichage | Choix parmi les modes decrits en section 6 |
| F33 | Nombre d'elements visibles | Nombre d'entrees affichees avant defilement |
| F34 | Position du panneau | Position a l'ecran (coin, centre, pres du curseur) |
| F35 | Theme | Clair / Sombre / Systeme |
| F36 | Demarrage automatique | Option pour lancer BufferVault au demarrage de Windows |
| F37 | Exclusions | Liste d'applications dont les copies sont ignorees |
| F38 | Duree de retention | Duree maximale de conservation des entrees (defaut : 30 jours) |

---

## 3. Exigences non-fonctionnelles

### 3.1 Performance

| ID | Exigence | Cible |
|----|----------|-------|
| NF01 | Empreinte memoire au repos | < 15 Mo |
| NF02 | Temps de capture d'une copie | < 5 ms |
| NF03 | Temps d'affichage du panneau | < 100 ms |
| NF04 | Temps de chiffrement/dechiffrement | < 10 ms par entree |
| NF05 | Utilisation CPU au repos | < 0.1% |

### 3.2 Fiabilite

| ID | Exigence | Detail |
|----|----------|--------|
| NF10 | Disponibilite | L'outil doit tourner en arriere-plan sans crash pendant des semaines |
| NF11 | Recuperation | En cas de corruption du fichier d'historique, l'outil redemarre avec un historique vide sans crash |
| NF12 | Graceful shutdown | Sauvegarde propre de l'etat a l'arret de Windows ou de l'application |

### 3.3 Securite

| ID | Exigence | Detail |
|----|----------|--------|
| NF20 | Chiffrement au repos | AES-256-GCM implemente en pure Rust |
| NF21 | Cle de chiffrement | Derivee via PBKDF2-HMAC-SHA256 a partir d'un secret utilisateur + DPAPI Windows |
| NF22 | Memoire securisee | Les buffers contenant des donnees dechiffrees sont zeros avant liberation |
| NF23 | Pas de logs en clair | Aucune donnee de presse-papiers n'apparait dans les logs |
| NF24 | Verrouillage auto | Apres une periode d'inactivite configurable, l'historique se verrouille |
| NF25 | Protection integrite | HMAC sur chaque entree pour detecter toute alteration |

### 3.4 Compatibilite

| ID | Exigence | Detail |
|----|----------|--------|
| NF30 | OS | Windows 10 version 1903+ et Windows 11 |
| NF31 | Architecture | x86_64 |
| NF32 | DPI | Support du scaling DPI (100%, 125%, 150%, 200%) |
| NF33 | Multi-ecrans | Affichage correct sur configurations multi-moniteurs |

---

## 4. Architecture

### 4.1 Vue d'ensemble

```
+------------------------------------------------------------------+
|                        BufferVault                                |
|                                                                   |
|  +------------------+    +------------------+    +--------------+ |
|  | Clipboard        |    | Crypto           |    | Config       | |
|  | Monitor          |--->| Engine           |--->| Manager      | |
|  | (Win32 API)      |    | (AES-256-GCM)    |    | (parse/save) | |
|  +------------------+    +------------------+    +--------------+ |
|          |                       |                      |         |
|          v                       v                      v         |
|  +------------------+    +------------------+    +--------------+ |
|  | History          |    | Storage          |    | Hotkey       | |
|  | Manager          |--->| Engine           |    | Manager      | |
|  | (in-memory ring) |    | (file I/O)       |    | (Win32 API)  | |
|  +------------------+    +------------------+    +--------------+ |
|          |                                              |         |
|          v                                              v         |
|  +------------------+    +------------------+    +--------------+ |
|  | UI               |    | Paste            |    | Tray         | |
|  | Renderer         |--->| Injector         |    | Icon         | |
|  | (Win32 GDI/DWM)  |    | (SendInput)      |    | (Shell_Notify| |
|  +------------------+    +------------------+    |  IconW)      | |
|                                                  +--------------+ |
+------------------------------------------------------------------+
            |                       |
            v                       v
    +---------------+      +----------------+
    | config.txt    |      | vault.dat      |
    | (settings)    |      | (encrypted     |
    |               |      |  history)      |
    +---------------+      +----------------+
```

### 4.2 Description des composants

#### 4.2.1 Clipboard Monitor
- S'enregistre aupres de Windows via `AddClipboardFormatListener`
- Recoit les messages `WM_CLIPBOARDUPDATE`
- Lit le contenu du presse-papiers via `OpenClipboard` / `GetClipboardData`
- Delegue au History Manager

#### 4.2.2 Crypto Engine
- Implementation pure Rust d'AES-256-GCM (pas de dependance OpenSSL/ring)
- Derivation de cle via PBKDF2-HMAC-SHA256 (100 000 iterations minimum)
- Generation de nonces uniques via CSPRNG Windows (`BCryptGenRandom`)
- Zeroing securise des buffers sensibles

#### 4.2.3 History Manager
- Structure de donnees en anneau (ring buffer) en memoire
- Gestion FIFO avec support d'epinglage
- Index de recherche simple (sous-chaine)
- Expose l'historique au UI Renderer

#### 4.2.4 Storage Engine
- Serialisation binaire compacte (format proprietaire simple)
- Ecriture atomique (write-to-temp + rename)
- Sauvegarde periodique (configurable, defaut : toutes les 30 secondes si changement)
- Fichier : `%APPDATA%\BufferVault\vault.dat`

#### 4.2.5 Config Manager
- Parse un format cle-valeur simple (similaire TOML simplifie)
- Valeurs par defaut pour toutes les options
- Rechargement a chaud (detection de modification du fichier)

#### 4.2.6 UI Renderer
- Fenetre Win32 sans bordure avec transparence (WS_EX_LAYERED, WS_EX_TOOLWINDOW)
- Dessin via GDI+ ou Direct2D selon disponibilite
- Plusieurs modes d'affichage (voir section 6)
- Gestion clavier et souris

#### 4.2.7 Hotkey Manager
- Enregistrement de raccourci global via `RegisterHotKey`
- Support de combinaisons complexes (Win+Shift+V, Ctrl+Alt+H, etc.)
- Modification dynamique sans redemarrage

#### 4.2.8 Paste Injector
- Place l'element selectionne dans le presse-papiers via `SetClipboardData`
- Simule optionnellement Ctrl+V via `SendInput` pour collage automatique
- Restaure le focus sur l'application cible

#### 4.2.9 Tray Icon
- Icone dans la zone de notification Windows
- Menu contextuel : Ouvrir historique, Parametres, Verrouiller, Purger, Quitter
- Double-clic : ouvre le panneau d'historique

### 4.3 Flux de donnees principal

```
1. Utilisateur copie (Ctrl+C)
2. Windows envoie WM_CLIPBOARDUPDATE
3. Clipboard Monitor lit le contenu
4. Deduplication (comparaison avec derniere entree)
5. Crypto Engine chiffre l'entree
6. History Manager ajoute l'entree au ring buffer
7. Storage Engine persiste sur disque (periodiquement)

--- Plus tard ---

8. Utilisateur appuie sur Win+Shift+V
9. Hotkey Manager detecte le raccourci
10. UI Renderer affiche le panneau avec l'historique
11. Utilisateur selectionne une entree (clic ou clavier)
12. Crypto Engine dechiffre l'entree
13. Paste Injector place dans le presse-papiers
14. Paste Injector simule Ctrl+V (optionnel)
15. UI Renderer ferme le panneau
```

---

## 5. Securite et chiffrement

### 5.1 Modele de menaces

| Menace | Mitigation |
|--------|------------|
| Acces au fichier vault.dat par un tiers | Chiffrement AES-256-GCM |
| Vol de la cle de chiffrement | Cle derivee via DPAPI (liee a la session Windows) |
| Lecture memoire par un processus malveillant | Zeroing des buffers, duree minimale en clair |
| Alteration du fichier d'historique | HMAC d'integrite par entree + header global |
| Rejeu / reutilisation de nonce | Nonce unique par entree (96 bits, CSPRNG) |
| Brute-force sur la cle | PBKDF2 avec 100 000+ iterations |

### 5.2 Schema de chiffrement

```
Derivation de cle :
  master_key = DPAPI_Protect(user_secret)
  salt = random(32 bytes) -- genere une fois, stocke dans le header du vault
  derived_key = PBKDF2_HMAC_SHA256(master_key, salt, iterations=100000, key_len=32)

Chiffrement d'une entree :
  nonce = BCryptGenRandom(12 bytes)
  (ciphertext, tag) = AES_256_GCM_Encrypt(derived_key, nonce, plaintext, aad=entry_metadata)
  stored = nonce || tag || ciphertext

Dechiffrement d'une entree :
  nonce = stored[0..12]
  tag = stored[12..28]
  ciphertext = stored[28..]
  plaintext = AES_256_GCM_Decrypt(derived_key, nonce, ciphertext, tag, aad=entry_metadata)
```

### 5.3 Format du fichier vault.dat

```
Offset  | Taille     | Contenu
--------|------------|----------------------------------
0x00    | 8 bytes    | Magic number : "BVAULT01"
0x08    | 4 bytes    | Version du format (u32 LE)
0x0C    | 32 bytes   | Salt PBKDF2
0x2C    | 4 bytes    | Nombre d'iterations PBKDF2 (u32 LE)
0x30    | 32 bytes   | HMAC-SHA256 du header
0x50    | 4 bytes    | Nombre d'entrees (u32 LE)
0x54    | variable   | Entrees chiffrees (voir ci-dessous)

Format d'une entree :
Offset  | Taille     | Contenu
--------|------------|----------------------------------
0x00    | 4 bytes    | Taille totale de l'entree (u32 LE)
0x04    | 8 bytes    | Timestamp UTC (i64 LE, secondes epoch)
0x0C    | 1 byte     | Type (0=texte, 1=riche, 2=fichier)
0x0D    | 1 byte     | Flags (bit 0: epingle)
0x0E    | 2 bytes    | Taille du nom de source (u16 LE)
0x10    | variable   | Nom de source (UTF-8)
variable| 12 bytes   | Nonce AES-GCM
variable| 16 bytes   | Tag AES-GCM
variable| variable   | Donnees chiffrees
```

### 5.4 Gestion de la cle maitre

```
Premier lancement :
  1. Generer un secret aleatoire de 32 bytes
  2. Proteger ce secret via DPAPI (CryptProtectData)
  3. Stocker le blob DPAPI dans %APPDATA%\BufferVault\keystore.bin

Lancement suivant :
  1. Lire le blob DPAPI depuis keystore.bin
  2. Dechiffrer via CryptUnprotectData
  3. Utiliser le secret + salt pour deriver la cle AES

Avantage : La cle est liee a la session Windows de l'utilisateur.
Personne d'autre (meme administrateur) ne peut dechiffrer sans la session.
```

---

## 6. Interface utilisateur

### 6.1 Modes d'affichage

BufferVault propose plusieurs modes d'affichage, configurables par l'utilisateur.

#### 6.1.1 Mode Popup (defaut)

```
+------------------------------------------+
| BufferVault                    [x] [ ] |
|------------------------------------------|
| [loupe] Rechercher...                    |
|------------------------------------------|
| [pin] Il y a 2 min  | notepad.exe        |
|   "Le texte copie precedemment qui..."  |
|------------------------------------------|
|   Il y a 5 min  | vscode.exe            |
|   "function calculateTotal(items) {"    |
|------------------------------------------|
|   Il y a 12 min | chrome.exe             |
|   "https://example.com/api/endpoint"    |
|------------------------------------------|
|   Il y a 1h     | explorer.exe           |
|   "C:\Users\Docs\rapport.pdf"           |
|------------------------------------------|
|       v Afficher plus (496 restants)     |
+------------------------------------------+
```

**Comportement** :
- Apparait au centre de l'ecran (ou pres du curseur, configurable)
- Se ferme apres selection ou Echap
- Navigable au clavier (fleches haut/bas, Entree pour selectionner)
- Nombre d'elements visibles : configurable (defaut : 8)
- Largeur et hauteur ajustables

#### 6.1.2 Mode Barre laterale

```
+----+  
| B  |  <- Onglet visible en permanence (bord droit de l'ecran)
| V  |
+----+

    Au clic ou survol :

+----+--------------------------------------+
| B  | BufferVault                          |
| V  |--------------------------------------|
|    | Rechercher...                        |
+----+--------------------------------------|
     | "Le texte copie precedemm..."       |
     | "function calculateTotal..."         |
     | "https://example.com/api..."         |
     | "C:\Users\Docs\rapport..."           |
     +--------------------------------------+
```

**Comportement** :
- Onglet fin (20-30px) colle au bord de l'ecran (cote configurable)
- Au clic : le panneau glisse depuis le bord
- Se referme apres selection ou perte de focus
- L'onglet reste toujours visible (topmost configurable)

#### 6.1.3 Mode Liste permanente

```
+--------------------------------------+
| BufferVault            [_] [pin] [x] |
|--------------------------------------|
| 1. "Le texte copie prece..."  [pin]  |
| 2. "function calculateTo..."  [pin]  |
| 3. "https://example.com/..."  [pin]  |
| 4. "C:\Users\Docs\rappor..."  [pin]  |
| 5. "SELECT * FROM users..."   [pin]  |
+--------------------------------------+
```

**Comportement** :
- Petite fenetre toujours visible (topmost)
- Redimensionnable et deplacable
- Nombre d'elements affiches : configurable (defaut : 5)
- Clic simple sur une entree = copie dans le presse-papiers
- Double-clic = copie + colle automatiquement
- Opacity configurable (semi-transparent au repos)
- Se souvient de sa position entre les sessions

#### 6.1.4 Mode Minimal (icone de notification uniquement)

- Aucun panneau visible en permanence
- L'historique n'est accessible que via le raccourci clavier
- Ou via clic droit sur l'icone de la zone de notification
- Le panneau popup apparait temporairement

### 6.2 Elements communs a tous les modes

| Element | Detail |
|---------|--------|
| Apercu du texte | Premiere ligne, tronquee a N caracteres (configurable, defaut : 50) |
| Horodatage | Format relatif ("Il y a 3 min") ou absolu (configurable) |
| Source | Nom de l'application source (icone si possible) |
| Indicateur d'epinglage | Icone pin cliquable |
| Barre de recherche | Filtrage instantane par sous-chaine |
| Compteur | Nombre total d'entrees dans l'historique |

### 6.3 Interactions

| Action | Resultat |
|--------|----------|
| Clic gauche sur entree | Selection : place dans presse-papiers, ferme panneau |
| Double-clic sur entree | Selection + collage automatique (Ctrl+V simule) |
| Clic droit sur entree | Menu : Copier, Coller, Epingler, Supprimer |
| Ctrl+chiffre (1-9) | Selection rapide des 9 premieres entrees |
| Fleches haut/bas | Navigation dans la liste |
| Entree | Selectionne l'entree en surbrillance |
| Echap | Ferme le panneau sans action |
| Suppr | Supprime l'entree en surbrillance |
| Ctrl+Suppr | Purge tout l'historique (avec confirmation) |
| Ctrl+F | Focus sur la barre de recherche |

### 6.4 Theme et apparence

```
[theme]
mode = "dark"           # dark | light | system
opacity = 0.95          # 0.0 a 1.0
font_size = 13          # en points
font_family = "Segoe UI"
accent_color = "#4A9EFF"
border_radius = 8       # en pixels
```

---

## 7. Integration Windows

### 7.1 APIs Win32 utilisees

| API | Usage |
|-----|-------|
| `AddClipboardFormatListener` | Enregistrement surveillance presse-papiers |
| `RemoveClipboardFormatListener` | Desenregistrement |
| `OpenClipboard` / `CloseClipboard` | Acces au presse-papiers |
| `GetClipboardData` | Lecture du contenu |
| `SetClipboardData` | Ecriture dans le presse-papiers |
| `EmptyClipboard` | Vidage avant ecriture |
| `RegisterHotKey` / `UnregisterHotKey` | Raccourci clavier global |
| `SendInput` | Simulation de frappe (Ctrl+V) |
| `Shell_NotifyIconW` | Icone de zone de notification |
| `CreateWindowExW` | Creation de fenetres |
| `SetLayeredWindowAttributes` | Transparence |
| `GetForegroundWindow` | Identification de la fenetre active |
| `GetWindowThreadProcessId` | Identification du processus source |
| `SetForegroundWindow` | Restauration du focus |
| `CryptProtectData` / `CryptUnprotectData` | DPAPI pour la cle maitre |
| `BCryptGenRandom` | Generation de nombres aleatoires cryptographiques |
| `GetDpiForWindow` | Support DPI |
| `MonitorFromWindow` | Support multi-ecrans |
| `RegisterWindowMessageW` | Communication inter-process |

### 7.2 Demarrage automatique

Deux methodes proposees (configurable) :

1. **Registre Windows** : Cle `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
2. **Dossier Startup** : Raccourci dans `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup`

### 7.3 Integration menu contextuel (v0.3+)

Extension shell Windows optionnelle pour ajouter un sous-menu "BufferVault" dans le menu contextuel systeme :
- Registre : `HKCR\*\shell\BufferVault`
- Affiche les N dernieres entrees dans un sous-menu
- Necessite elevation pour l'enregistrement initial

### 7.4 Icone zone de notification

```
Menu contextuel de l'icone :
  - Ouvrir l'historique
  - Parametres...
  ---
  - Verrouiller
  - Purger l'historique
  ---
  - Demarrer avec Windows [x]
  ---
  - A propos
  - Quitter
```

---

## 8. Structure du code

### 8.1 Contrainte : 800 lignes maximum par fichier

Chaque fichier source Rust est limite a 800 lignes. Cette contrainte impose une decomposition modulaire stricte.

### 8.2 Organisation des fichiers

```
buffervault/
  Cargo.toml
  RELEASE_NOTES.md
  docs/
    DESIGN.md              <- Ce document
  src/
    main.rs                <- Point d'entree, boucle de messages (~200 lignes)
    app.rs                 <- Etat global de l'application, orchestration (~400 lignes)
    clipboard/
      mod.rs               <- Module clipboard, re-exports (~50 lignes)
      monitor.rs           <- Surveillance du presse-papiers (~300 lignes)
      injector.rs          <- Injection presse-papiers + SendInput (~250 lignes)
    crypto/
      mod.rs               <- Module crypto, re-exports (~50 lignes)
      aes_gcm.rs           <- Implementation AES-256-GCM pure Rust (~700 lignes)
      pbkdf2.rs            <- Derivation de cle PBKDF2-HMAC-SHA256 (~400 lignes)
      sha256.rs            <- Implementation SHA-256 pure Rust (~500 lignes)
      ghash.rs             <- Multiplication GF(2^128) pour GCM (~400 lignes)
      dpapi.rs             <- Wrappers DPAPI Windows (~200 lignes)
      secure_buf.rs        <- Buffer avec zeroing automatique (~150 lignes)
    history/
      mod.rs               <- Module history, re-exports (~50 lignes)
      entry.rs             <- Structure ClipboardEntry (~150 lignes)
      ring.rs              <- Ring buffer avec epinglage (~400 lignes)
      search.rs            <- Recherche dans l'historique (~200 lignes)
    storage/
      mod.rs               <- Module storage, re-exports (~50 lignes)
      vault.rs             <- Lecture/ecriture du fichier vault.dat (~500 lignes)
      format.rs            <- Serialisation/deserialisation binaire (~400 lignes)
    config/
      mod.rs               <- Module config, re-exports (~50 lignes)
      parser.rs            <- Parseur format cle-valeur (~350 lignes)
      settings.rs          <- Structure de configuration + defauts (~300 lignes)
    ui/
      mod.rs               <- Module UI, re-exports (~50 lignes)
      window.rs            <- Creation et gestion fenetre Win32 (~600 lignes)
      renderer.rs          <- Dessin GDI des elements (~700 lignes)
      popup.rs             <- Mode popup (~400 lignes)
      sidebar.rs           <- Mode barre laterale (~400 lignes)
      permanent.rs         <- Mode liste permanente (~350 lignes)
      theme.rs             <- Couleurs et styles (~200 lignes)
      dpi.rs               <- Gestion DPI et scaling (~150 lignes)
    system/
      mod.rs               <- Module system, re-exports (~50 lignes)
      hotkey.rs            <- Gestion raccourci clavier global (~250 lignes)
      tray.rs              <- Icone de notification (~350 lignes)
      win32.rs             <- Bindings et constantes Win32 (~600 lignes)
      process.rs           <- Identification processus source (~200 lignes)
    error.rs               <- Types d'erreur centralises (~150 lignes)
    constants.rs           <- Constantes globales (~100 lignes)
```

### 8.3 Modules et responsabilites

| Module | Responsabilite | Dependances internes |
|--------|---------------|---------------------|
| `main` | Initialisation, boucle de messages | `app`, `system` |
| `app` | Orchestration des composants | Tous |
| `clipboard` | Capture et injection presse-papiers | `system::win32`, `history` |
| `crypto` | Chiffrement, derivation, DPAPI | `system::win32` |
| `history` | Gestion de l'historique en memoire | `crypto` |
| `storage` | Persistance sur disque | `crypto`, `history` |
| `config` | Lecture/ecriture configuration | Aucune |
| `ui` | Affichage et interaction utilisateur | `history`, `config`, `system` |
| `system` | Bindings Win32, hotkey, tray | Aucune |

### 8.4 Build system

```toml
# Cargo.toml
[package]
name = "buffervault"
version = "0.1.0"
edition = "2021"
authors = ["BufferVault Team"]
description = "Secure clipboard history manager for Windows"

[dependencies]
# Aucune dependance externe

[build-dependencies]
# Aucune

[profile.release]
opt-level = 3
lto = true
strip = true
panic = "abort"

# Lien avec les DLLs systeme Windows (pas de crate externe)
# Les liens sont faits via #[link(name = "...")] dans le code
```

Librairies systeme Windows liees directement :
- `user32.dll` (fenetres, presse-papiers, hotkey)
- `kernel32.dll` (processus, fichiers)
- `gdi32.dll` (dessin)
- `crypt32.dll` (DPAPI)
- `bcrypt.dll` (CSPRNG)
- `shell32.dll` (notification)
- `dwmapi.dll` (composition, optionnel)

---

## 9. Gestion de configuration

### 9.1 Format du fichier config.txt

```ini
# BufferVault Configuration
# Emplacement : %APPDATA%\BufferVault\config.txt

[general]
max_history = 500
max_entry_size_kb = 1024
retention_days = 30
auto_start = true
language = "fr"

[hotkey]
# Modificateurs : win, ctrl, alt, shift
# Touches : a-z, 0-9, f1-f12, etc.
modifier = "win+shift"
key = "v"

[display]
# Mode : popup | sidebar | permanent | minimal
mode = "popup"
visible_items = 8
preview_length = 50
position = "center"          # center | cursor | top-left | top-right | bottom-left | bottom-right
close_after_select = true
show_source = true
show_timestamp = true
timestamp_format = "relative" # relative | absolute

[sidebar]
edge = "right"               # left | right
width = 350
auto_hide = true

[permanent]
width = 300
height = 250
opacity_idle = 0.6
opacity_hover = 0.95
remember_position = true

[theme]
mode = "dark"                # dark | light | system
opacity = 0.95
font_size = 13
accent_color = "#4A9EFF"
border_radius = 8

[security]
lock_timeout_minutes = 30
clear_on_lock = false
pbkdf2_iterations = 100000

[exclusions]
# Applications dont les copies sont ignorees
apps = ["KeePass.exe", "1Password.exe", "LastPass.exe"]
```

### 9.2 Valeurs par defaut

Toutes les valeurs ont un defaut raisonnable. Si le fichier de configuration n'existe pas au premier lancement, il est cree avec les valeurs par defaut et des commentaires explicatifs.

### 9.3 Rechargement a chaud

Le fichier de configuration est surveille via `ReadDirectoryChangesW`. Tout changement est applique sans redemarrage (sauf le raccourci global qui necessite un re-enregistrement).

---

## 10. Cycle de vie des donnees

### 10.1 Cycle d'une entree

```
Copie detectee
     |
     v
Deduplication -----> [Doublon] --> Ignore
     |
     v [Nouveau]
Capture metadonnees (timestamp, source, type)
     |
     v
Chiffrement AES-256-GCM
     |
     v
Insertion dans le ring buffer (memoire)
     |
     v
Sauvegarde periodique sur disque (vault.dat)
     |
     v
[Rotation] Si capacite atteinte et non epingle --> Suppression
     |
     v
[Retention] Si age > retention_days et non epingle --> Suppression
     |
     v
[Purge manuelle] Utilisateur demande suppression --> Zeroing + Suppression
```

### 10.2 Sauvegarde sur disque

- Declenchee toutes les 30 secondes si des modifications existent
- Declenchee immediatement a l'arret de l'application
- Ecriture atomique : ecriture dans un fichier temporaire, puis rename
- En cas d'echec : retry 3 fois avec delai exponentiel

### 10.3 Demarrage

```
1. Charger la configuration (ou creer les defauts)
2. Lire le blob DPAPI (ou generer au premier lancement)
3. Deriver la cle de chiffrement
4. Charger vault.dat (ou creer un vault vide)
5. Dechiffrer les entrees en memoire
6. Appliquer la retention (supprimer les entrees expirees)
7. Enregistrer le listener presse-papiers
8. Enregistrer le raccourci clavier global
9. Creer l'icone de notification
10. Entrer dans la boucle de messages Windows
```

### 10.4 Arret propre

```
1. Recevoir WM_CLOSE ou WM_ENDSESSION
2. Desenregistrer le listener presse-papiers
3. Desenregistrer le raccourci clavier
4. Supprimer l'icone de notification
5. Chiffrer et sauvegarder l'historique sur disque
6. Zeroing des buffers sensibles en memoire
7. Quitter
```

---

## 11. Gestion des versions

### 11.1 Schema de versioning

Le projet suit le Semantic Versioning (SemVer) : `MAJOR.MINOR.PATCH`

- **MAJOR** : Changements incompatibles (format vault.dat, API de config)
- **MINOR** : Nouvelles fonctionnalites retrocompatibles
- **PATCH** : Corrections de bugs

### 11.2 Fichier RELEASE_NOTES.md

Un fichier `RELEASE_NOTES.md` a la racine du projet documente chaque version :
- Date de release
- Nouvelles fonctionnalites
- Corrections de bugs
- Changements cassants
- Notes de migration

### 11.3 Roadmap

| Version | Contenu | Statut |
|---------|---------|--------|
| 0.1.0 | Infrastructure de base : capture presse-papiers, stockage chiffre, affichage popup simple | A faire |
| 0.2.0 | Modes d'affichage supplementaires (sidebar, permanent), configuration complete | A faire |
| 0.3.0 | Integration menu contextuel Windows, epinglage, recherche | A faire |
| 0.4.0 | Support multi-ecrans, DPI, themes clair/sombre | A faire |
| 0.5.0 | Optimisations performance, stabilite, polish UI | A faire |
| 1.0.0 | Premiere version stable complete | A faire |

---

## 12. Plan d'implementation

### Phase 1 - Fondations (v0.1.0)

| Etape | Module | Description | Priorite |
|-------|--------|-------------|----------|
| 1.1 | `system::win32` | Bindings Win32 de base (types, constantes, fonctions) | P0 |
| 1.2 | `crypto::sha256` | Implementation SHA-256 | P0 |
| 1.3 | `crypto::pbkdf2` | Implementation PBKDF2-HMAC-SHA256 | P0 |
| 1.4 | `crypto::aes_gcm` | Implementation AES-256-GCM | P0 |
| 1.5 | `crypto::ghash` | Multiplication GF(2^128) pour GCM | P0 |
| 1.6 | `crypto::dpapi` | Wrappers DPAPI | P0 |
| 1.7 | `crypto::secure_buf` | Buffer securise avec zeroing | P0 |
| 1.8 | `clipboard::monitor` | Surveillance presse-papiers | P0 |
| 1.9 | `history::entry` | Structure ClipboardEntry | P0 |
| 1.10 | `history::ring` | Ring buffer | P0 |
| 1.11 | `storage::format` | Serialisation binaire | P0 |
| 1.12 | `storage::vault` | Lecture/ecriture vault.dat | P0 |
| 1.13 | `config::parser` | Parseur de configuration | P1 |
| 1.14 | `config::settings` | Structure Settings | P1 |
| 1.15 | `system::hotkey` | Raccourci clavier global | P0 |
| 1.16 | `ui::window` | Fenetre Win32 de base | P0 |
| 1.17 | `ui::renderer` | Dessin GDI basique | P0 |
| 1.18 | `ui::popup` | Mode popup | P0 |
| 1.19 | `clipboard::injector` | Injection + SendInput | P0 |
| 1.20 | `system::tray` | Icone notification | P1 |
| 1.21 | `main` + `app` | Assemblage et boucle principale | P0 |
| 1.22 | `error` | Types d'erreur | P0 |
| 1.23 | `constants` | Constantes | P0 |

### Phase 2 - Modes d'affichage (v0.2.0)

| Etape | Description |
|-------|-------------|
| 2.1 | Mode barre laterale |
| 2.2 | Mode liste permanente |
| 2.3 | Configuration complete et rechargement a chaud |
| 2.4 | Themes clair/sombre/systeme |

### Phase 3 - Fonctionnalites avancees (v0.3.0)

| Etape | Description |
|-------|-------------|
| 3.1 | Integration menu contextuel Windows |
| 3.2 | Epinglage d'entrees |
| 3.3 | Recherche dans l'historique |
| 3.4 | Exclusion d'applications |

### Phase 4 - Polish (v0.4.0 - v0.5.0)

| Etape | Description |
|-------|-------------|
| 4.1 | Support multi-ecrans |
| 4.2 | Support DPI |
| 4.3 | Optimisations memoire et CPU |
| 4.4 | Tests de stabilite longue duree |
| 4.5 | Documentation utilisateur |

---

## 13. Contraintes techniques

### 13.1 Zero dependances externes

Le projet n'utilise **aucun crate externe**. Toutes les fonctionnalites sont implementees en pure Rust ou via appels directs aux APIs Windows.

Justification :
- Reduction de la surface d'attaque (supply chain)
- Controle total du code cryptographique
- Binaire minimal (< 2 Mo attendu)
- Pas de gestion de versions de dependances

### 13.2 Implementations necessaires en pur Rust

| Composant | Complexite | Remarques |
|-----------|-----------|-----------|
| AES-256 | Haute | Tables S-box, MixColumns, ShiftRows, KeyExpansion |
| GCM mode | Haute | Multiplication GF(2^128), GHASH, compteur |
| SHA-256 | Moyenne | Compression Merkle-Damgard, padding |
| HMAC-SHA256 | Faible | Enveloppe autour de SHA-256 |
| PBKDF2 | Faible | Iterations d'HMAC-SHA256 |
| Win32 bindings | Moyenne | FFI, structures alignees, pointeurs |
| Parseur config | Faible | Tokenizer simple, lignes cle=valeur |
| Serialisation binaire | Moyenne | Read/write bytes, endianness |
| GDI rendering | Haute | CreateFont, TextOut, FillRect, etc. |

### 13.3 Fichiers < 800 lignes

Chaque fichier `.rs` doit rester sous 800 lignes (commentaires et lignes vides inclus). En cas de depassement, le module doit etre decoupe en sous-modules.

### 13.4 Securite du code

- Aucun `unwrap()` sur des operations faillibles en production (utiliser `match` ou `?`)
- Minimiser l'usage de `unsafe` (confine aux appels FFI Win32)
- Tous les blocs `unsafe` doivent etre commentes avec la justification de securite
- Les buffers contenant des secrets doivent utiliser `SecureBuf` avec zeroing

---

## 14. Glossaire

| Terme | Definition |
|-------|------------|
| AES-256-GCM | Advanced Encryption Standard avec cle de 256 bits en mode Galois/Counter Mode |
| CSPRNG | Cryptographically Secure Pseudo-Random Number Generator |
| DPAPI | Data Protection API, API Windows pour la protection de secrets lies a la session utilisateur |
| GCM | Galois/Counter Mode, mode de chiffrement authentifie |
| GDI | Graphics Device Interface, API de dessin Windows |
| GHASH | Fonction de hachage universelle utilisee dans GCM |
| HMAC | Hash-based Message Authentication Code |
| PBKDF2 | Password-Based Key Derivation Function 2 |
| Ring buffer | Structure de donnees circulaire FIFO |
| S-box | Substitution box, table de substitution utilisee dans AES |
| SemVer | Semantic Versioning, schema de versioning (MAJOR.MINOR.PATCH) |
| Topmost | Fenetre toujours au premier plan (flag WS_EX_TOPMOST) |
| DPAPI blob | Donnee chiffree par DPAPI, dechiffrable uniquement par le meme utilisateur Windows |
| Vault | Fichier de stockage chiffre de l'historique |

---

*Document genere pour BufferVault v0.1.0 - Ce document est vivant et sera mis a jour au fil du developpement.*
