// BufferVault - Orchestrateur principal
// Connecte tous les composants : clipboard, history, storage, UI
// Gere la boucle de messages Win32 et le cycle de vie de l'application
//
// Ce fichier est le coeur de BufferVault. Il orchestre l'initialisation
// de tous les sous-systemes (clipboard, crypto, storage, UI) et gere
// la boucle de messages Win32 qui pilote l'application.
//
// # Architecture
// L'application est mono-thread : la boucle de messages unique garantit
// que tous les handlers sont appeles sequentiellement. Le pointeur
// global APP_PTR est utilise par les wndproc pour acceder a l'etat
// de l'application sans recourir a des RefCell ou Mutex.
//
// # Cycle de vie
// 1. `App::new()` : chargement config, cle maitre, historique
// 2. `App::run()` : enregistrement classes fenetres, boucle de messages
// 3. `App::cleanup()` : sauvegarde finale, liberation des ressources
//
// # Messages Win32 geres
// - WM_CLIPBOARDUPDATE : nouvelle copie detectee
// - WM_HOTKEY           : raccourci clavier global active
// - WM_TIMER            : sauvegarde automatique periodique
// - WM_TRAY_CALLBACK    : interaction avec l'icone tray
// - WM_PAINT/WM_KEYDOWN : rendu et navigation dans les fenetres UI

use crate::clipboard::{injector, monitor};
use crate::config::settings::{DisplayMode, Settings};
use crate::constants::*;
use crate::crypto::dpapi;
use crate::error::BvResult;
use crate::history::ring::HistoryRing;
use crate::storage::vault;
use crate::system::{autostart, hotkey, process, tray};
use crate::ui::dpi::DpiContext;
use crate::ui::popup::PopupState;
use crate::ui::sidebar::SidebarState;
use crate::ui::permanent::PermanentState;
use crate::ui::manager::{self, ManagerState};
use crate::ui::splash::{self, SplashState};
use crate::ui::theme;
use crate::ui::window;
use crate::system::win32::*;

/// Pointeur global vers l'App, accessible depuis tous les wndprocs.
///
/// # Safety
/// BufferVault est mono-thread (boucle de messages unique).
/// Ce pointeur est initialise au debut de `run()` et invalide dans `cleanup()`.
/// Il ne doit jamais etre accede en dehors du thread principal Win32.
static mut APP_PTR: *mut App = std::ptr::null_mut();

/// ID du menu contextuel : afficher/masquer
const TRAY_CMD_SHOW: u16 = 1;
/// ID du menu contextuel : quitter
const TRAY_CMD_QUIT: u16 = 2;
/// ID du menu contextuel : vider l'historique
const TRAY_CMD_CLEAR: u16 = 3;
/// ID du menu contextuel : a propos
const TRAY_CMD_ABOUT: u16 = 4;
/// ID du menu contextuel : gerer l'historique
const TRAY_CMD_MANAGE: u16 = 5;
/// ID du menu contextuel : demarrage automatique
const TRAY_CMD_STARTUP: u16 = 6;

/// Application principale BufferVault.
///
/// Structure centrale qui detient tous les composants de l'application :
/// historique, configuration, cle de chiffrement, etats des fenetres UI.
///
/// # Thread Safety
/// Cette structure n'est PAS thread-safe (pas de Sync/Send).
/// Elle est concue pour etre utilisee exclusivement depuis le thread
/// principal de la boucle de messages Win32.
pub struct App {
    /// Handle de la fenetre cachee (boucle de messages)
    hwnd: HWND,
    /// Historique des entrees
    history: HistoryRing,
    /// Configuration
    settings: Settings,
    /// Cle maitre dechiffree
    master_key: Vec<u8>,
    /// Etat du popup
    popup: PopupState,
    /// Etat de la sidebar
    sidebar: SidebarState,
    /// Etat de la fenetre permanente
    permanent: PermanentState,
    /// Etat du gestionnaire d'historique
    manager: ManagerState,
    /// Contexte DPI
    dpi: DpiContext,
    /// Splash screen (Some pendant l'affichage, None apres)
    splash: Option<SplashState>,
    /// Flag pour ignorer la prochaine notification clipboard
    /// (quand c'est notre propre injection)
    ignore_next_clipboard: bool,
}

impl App {
    /// Cree et initialise l'application.
    ///
    /// Charge la configuration, la cle maitre DPAPI et l'historique
    /// depuis le fichier vault chiffre. Retourne une erreur si la cle
    /// maitre ne peut pas etre chargee/creee ou si le vault est corrompu.
    ///
    /// # Errors
    /// - `BvError::Crypto` : echec du chargement/creation de la cle DPAPI
    /// - `BvError::Storage` : echec de lecture du fichier vault
    /// - `BvError::Integrity` : fichier vault corrompu (HMAC invalide)
    pub fn new() -> BvResult<Self> {
        // Charger la configuration
        let default_settings = Settings::default();
        let config_path = default_settings.config_path();
        let settings = Settings::load(&config_path);

        // Charger ou creer la cle maitre via DPAPI
        let key_path = settings.keystore_path();
        let master_key = dpapi::load_or_create_master_key(&key_path)?;

        // Creer l'historique
        let mut history = HistoryRing::new(settings.max_history);

        // Charger le vault existant
        let vault_path = settings.vault_path();
        let entries = vault::load_vault(&vault_path, &master_key)?;
        history.load_from(entries);

        Ok(Self {
            hwnd: NULL_HWND,
            history,
            settings,
            master_key,
            popup: PopupState::new(DEFAULT_VISIBLE_ITEMS),
            sidebar: SidebarState::new(),
            permanent: PermanentState::new(),
            manager: ManagerState::new(),
            dpi: DpiContext::new(),
            splash: None,
            ignore_next_clipboard: false,
        })
    }

    /// Initialise les composants Win32 et demarre la boucle de messages.
    ///
    /// Sequence d'initialisation :
    /// 1. Enregistrement des classes de fenetres Win32
    /// 2. Creation de la fenetre cachee (receptrice de messages)
    /// 3. Affichage du splash screen
    /// 4. Enregistrement du clipboard listener
    /// 5. Enregistrement du hotkey global
    /// 6. Ajout de l'icone tray
    /// 7. Creation de la fenetre UI selon le mode configure
    /// 8. Boucle de messages (bloquante)
    /// 9. Nettoyage des ressources
    ///
    /// # Errors
    /// - `BvError::Win32` : echec d'enregistrement de classe ou creation de fenetre
    /// - `BvError::Clipboard` : echec d'enregistrement du listener
    pub fn run(&mut self) -> BvResult<()> {
        // Enregistrer les classes de fenetres
        window::register_class(
            window::MAIN_CLASS,
            Self::wndproc_main,
            CS_HREDRAW | CS_VREDRAW,
        )?;
        window::register_class(
            window::POPUP_CLASS,
            Self::wndproc_popup,
            CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
        )?;
        window::register_class(
            window::SIDEBAR_CLASS,
            Self::wndproc_popup, // Reutilise le meme wndproc
            CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
        )?;
        window::register_class(
            "BufferVaultPermanent",
            Self::wndproc_popup,
            CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
        )?;
        window::register_class(
            manager::MANAGER_CLASS,
            Self::wndproc_manager,
            CS_HREDRAW | CS_VREDRAW,
        )?;
        window::register_class(
            splash::SPLASH_CLASS,
            Self::wndproc_splash,
            CS_HREDRAW | CS_VREDRAW,
        )?;

        // Creer la fenetre cachee
        self.hwnd = window::create_hidden_window(window::MAIN_CLASS)?;

        // Mettre a jour le DPI
        self.dpi = DpiContext::from_hwnd(self.hwnd);

        // Stocker le pointeur this dans GWLP_USERDATA
        // SAFETY: On stocke un pointeur raw vers self. Il reste valide
        // tant que la boucle de messages tourne dans la meme scope.
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, self as *mut App as isize);
            APP_PTR = self as *mut App;
        }

        // Afficher le splash screen
        self.splash = Some(SplashState::show(&self.dpi));

        // Enregistrer le clipboard listener
        monitor::register_listener(self.hwnd)?;

        // Enregistrer le hotkey global (non fatal si deja pris)
        if let Err(e) = hotkey::register_global_hotkey(
            self.hwnd,
            self.settings.hotkey_modifiers,
            self.settings.hotkey_vk,
        ) {
            eprintln!("Warning: hotkey registration failed: {}", e);
            eprintln!("Hint: the hotkey may already be used by another application.");
        }

        // Ajouter l'icone tray
        tray::add_tray_icon(self.hwnd, "BufferVault")?;

        // Timer de sauvegarde automatique
        // SAFETY: appel FFI Win32.
        unsafe {
            SetTimer(self.hwnd, TIMER_AUTOSAVE, AUTO_SAVE_INTERVAL_MS, std::ptr::null());
        }

        // Creer la fenetre UI selon le mode
        self.popup.visible_count = self.settings.visible_items;
        self.popup.create_window(&self.dpi);

        match self.settings.display_mode {
            DisplayMode::Sidebar => {
                self.sidebar.create_window(&self.dpi);
                self.sidebar.toggle(); // Afficher par defaut
            }
            DisplayMode::Permanent => {
                self.permanent.create_window(&self.dpi);
                self.permanent.toggle();
            }
            _ => {}
        }

        // Boucle de messages Win32
        self.message_loop();

        // Nettoyage
        self.cleanup();

        Ok(())
    }

    /// Boucle de messages Win32.
    fn message_loop(&self) {
        // SAFETY: boucle de messages standard Win32.
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, NULL_HWND, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    /// Gere le message WM_CLIPBOARDUPDATE.
    ///
    /// Appele a chaque modification du presse-papiers. Ignore la notification
    /// si elle provient de notre propre injection (`ignore_next_clipboard`).
    /// Verifie les exclusions d'application et la taille maximale avant
    /// d'ajouter l'entree a l'historique.
    fn on_clipboard_update(&mut self) {
        if self.ignore_next_clipboard {
            self.ignore_next_clipboard = false;
            return;
        }

        // Detecter l'application source
        let source = process::get_foreground_process_name();

        // Verifier les exclusions
        if self.settings.is_app_excluded(&source) {
            return;
        }

        // Lire le clipboard
        if let Some(entry) = monitor::capture_clipboard(self.hwnd, source) {
            // Verifier la taille maximale
            if entry.content_size() <= self.settings.max_entry_size {
                self.history.push(entry);
                // Rafraichir les fenetres visibles
                self.refresh_visible_ui();
            }
        }
    }

    /// Gere le message WM_HOTKEY (raccourci clavier global).
    ///
    /// Bascule la visibilite de la fenetre UI active selon le mode
    /// d'affichage configure (popup, sidebar ou permanent).
    fn on_hotkey(&mut self) {
        match self.settings.display_mode {
            DisplayMode::Popup | DisplayMode::Minimal => {
                if self.popup.visible {
                    self.popup.hide();
                } else {
                    self.popup.show(self.history.as_slice(), &self.dpi);
                }
            }
            DisplayMode::Sidebar => {
                self.sidebar.toggle();
            }
            DisplayMode::Permanent => {
                self.permanent.toggle();
            }
        }
    }

    /// Gere la selection d'un element (touche Entree dans le popup).
    ///
    /// Copie le contenu de l'entree selectionnee dans le presse-papiers
    /// sans coller automatiquement. L'utilisateur choisit ou et quand
    /// coller avec Ctrl+V. Ferme le popup si `close_after_select` est active.
    fn on_select(&mut self) {
        let index = match self.settings.display_mode {
            DisplayMode::Popup | DisplayMode::Minimal => {
                self.popup.resolve_selected_index(self.history.as_slice())
            }
            DisplayMode::Sidebar => {
                let idx = self.sidebar.selected;
                if idx < self.history.len() { Some(idx) } else { None }
            }
            DisplayMode::Permanent => {
                let idx = self.permanent.selected;
                if idx < self.history.len() { Some(idx) } else { None }
            }
        };

        if let Some(idx) = index {
            if let Some(entry) = self.history.get(idx) {
                let text = entry.content.clone();

                // Fermer le popup si configure
                if self.settings.close_after_select {
                    self.popup.hide();
                }

                // Ignorer notre propre modification du clipboard
                self.ignore_next_clipboard = true;

                // Placer le texte dans le presse-papiers sans coller automatiquement.
                // L'utilisateur choisit ou et quand coller (Ctrl+V ou clic droit).
                let _ = injector::set_clipboard_text(self.hwnd, &text);
            }
        }
    }

    /// Gere la touche Delete pour supprimer un element.
    fn on_delete(&mut self) {
        let idx = match self.settings.display_mode {
            DisplayMode::Popup | DisplayMode::Minimal => self.popup.selected,
            DisplayMode::Sidebar => self.sidebar.selected,
            DisplayMode::Permanent => self.permanent.selected,
        };

        if idx < self.history.len() {
            self.history.remove(idx);
            self.refresh_visible_ui();
        }
    }

    /// Gere le toggle pin (double-clic ou Ctrl+P).
    fn on_toggle_pin(&mut self) {
        let idx = match self.settings.display_mode {
            DisplayMode::Popup | DisplayMode::Minimal => self.popup.selected,
            DisplayMode::Sidebar => self.sidebar.selected,
            DisplayMode::Permanent => self.permanent.selected,
        };

        if idx < self.history.len() {
            self.history.toggle_pin(idx);
            self.refresh_visible_ui();
        }
    }

    /// Gere les clics sur l'icone tray.
    fn on_tray_message(&mut self, lparam: LPARAM) {
        let msg = (lparam & 0xFFFF) as u32;
        match msg {
            WM_LBUTTONDBLCLK => {
                self.on_hotkey();
            }
            WM_RBUTTONDOWN => {
                let startup_on = autostart::is_autostart_enabled();
                let items = [
                    ("Afficher/Masquer", TRAY_CMD_SHOW, false),
                    ("Gerer l'historique...", TRAY_CMD_MANAGE, false),
                    ("Vider l'historique", TRAY_CMD_CLEAR, false),
                    ("", 0, false),
                    ("Lancer au demarrage", TRAY_CMD_STARTUP, startup_on),
                    ("A propos...", TRAY_CMD_ABOUT, false),
                    ("", 0, false),
                    ("Quitter", TRAY_CMD_QUIT, false),
                ];
                let cmd = tray::show_tray_menu(self.hwnd, &items);
                match cmd {
                    TRAY_CMD_SHOW => self.on_hotkey(),
                    TRAY_CMD_CLEAR => {
                        self.history.clear_unpinned();
                        self.refresh_visible_ui();
                    }
                    TRAY_CMD_ABOUT => {
                        self.show_about_dialog();
                    }
                    TRAY_CMD_MANAGE => {
                        self.manager.show(self.history.len(), &self.dpi);
                    }
                    TRAY_CMD_STARTUP => {
                        autostart::toggle_autostart();
                    }
                    TRAY_CMD_QUIT => {
                        self.save_and_quit();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Gere le timer de sauvegarde automatique (WM_TIMER).
    ///
    /// Sauvegarde le vault sur disque si l'historique a ete modifie
    /// depuis la derniere sauvegarde, puis applique la politique de
    /// retention (suppression des entrees plus anciennes que `retention_days`).
    fn on_timer(&mut self) {
        if self.history.is_dirty() {
            self.save_vault();
            self.history.reset_dirty();
        }

        // Appliquer la retention
        self.history.apply_retention(self.settings.retention_days);
    }

    /// Sauvegarde le vault chiffre sur disque.
    ///
    /// Utilise une ecriture atomique (fichier temporaire + rename) pour
    /// eviter la corruption en cas de crash ou coupure de courant.
    fn save_vault(&self) {
        let path = self.settings.vault_path();
        let entries = self.history.to_vec();
        if let Err(e) = vault::save_vault(&path, &entries, &self.master_key) {
            eprintln!("Failed to save vault: {}", e);
        }
    }

    /// Rafraichit les fenetres UI visibles.
    fn refresh_visible_ui(&self) {
        if self.popup.visible {
            window::invalidate(self.popup.hwnd);
        }
        if self.sidebar.visible {
            window::invalidate(self.sidebar.hwnd);
        }
        if self.permanent.visible {
            window::invalidate(self.permanent.hwnd);
        }
    }

    /// Affiche la boite de dialogue "A propos".
    fn show_about_dialog(&self) {
        let version = env!("CARGO_PKG_VERSION");
        let text = format!(
            "BufferVault v{}\n\n\
             Gestionnaire d'historique du presse-papiers\n\
             avec chiffrement AES-256-GCM\n\n\
             Auteur : Emmanuel Forgues\n\
             (c) 2025 - Tous droits reserves\n\n\
             Pure Rust - Zero dependance externe",
            version
        );
        let wtitle = to_wstring("BufferVault - A propos");
        let wtext = to_wstring(&text);
        // SAFETY: appel FFI Win32 pour afficher la boite de dialogue.
        unsafe {
            MessageBoxW(
                self.hwnd,
                wtext.as_ptr(),
                wtitle.as_ptr(),
                MB_OK | MB_ICONINFORMATION,
            );
        }
    }

    /// Sauvegarde l'historique et quitte l'application.
    ///
    /// Envoie WM_QUIT pour terminer la boucle de messages Win32
    /// apres avoir persiste le vault sur disque.
    fn save_and_quit(&mut self) {
        self.save_vault();
        // SAFETY: appel FFI Win32.
        unsafe { PostQuitMessage(0) };
    }

    /// Nettoyage des ressources Win32 a la fermeture.
    ///
    /// Sequence de nettoyage :
    /// 1. Invalidation du pointeur global APP_PTR
    /// 2. Sauvegarde finale du vault
    /// 3. Desenregistrement du clipboard listener
    /// 4. Desenregistrement du hotkey global
    /// 5. Retrait de l'icone tray
    /// 6. Arret du timer de sauvegarde
    /// 7. Destruction de toutes les fenetres
    fn cleanup(&mut self) {
        // Invalider le pointeur global
        // SAFETY: mono-thread, on quitte la boucle de messages.
        unsafe { APP_PTR = std::ptr::null_mut(); }

        // Sauvegarder une derniere fois
        self.save_vault();

        // Retirer l'ecouteur clipboard
        monitor::unregister_listener(self.hwnd);

        // Retirer le hotkey
        hotkey::unregister_global_hotkey(self.hwnd);

        // Retirer l'icone tray
        tray::remove_tray_icon(self.hwnd);

        // Tuer le timer
        // SAFETY: appel FFI Win32.
        unsafe { KillTimer(self.hwnd, TIMER_AUTOSAVE) };

        // Detruire les fenetres
        self.popup.destroy();
        self.sidebar.destroy();
        self.permanent.destroy();
        self.manager.destroy();
        window::destroy(self.hwnd);
    }

    // --- Window procedures ---

    /// WndProc de la fenetre principale cachee.
    ///
    /// Recoit les messages systeme (clipboard, hotkey, timer, tray)
    /// et les dispatche vers les handlers de l'App.
    ///
    /// # Safety
    /// - Le pointeur `app` est recupere depuis GWLP_USERDATA, valide
    ///   tant que la boucle de messages tourne dans `run()`.
    /// - Fonction appelee exclusivement par le dispatch Win32 (mono-thread).
    unsafe extern "system" fn wndproc_main(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let app = window::get_user_data::<App>(hwnd);
        if app.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        let app = &mut *app;

        match msg {
            WM_CLIPBOARDUPDATE => {
                app.on_clipboard_update();
                0
            }
            WM_HOTKEY => {
                app.on_hotkey();
                0
            }
            WM_TIMER => {
                app.on_timer();
                0
            }
            WM_TRAY_CALLBACK => {
                app.on_tray_message(lparam);
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            WM_ENDSESSION => {
                app.save_vault();
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    /// WndProc des fenetres popup/sidebar/permanent.
    ///
    /// Gere les messages de rendu (WM_PAINT), navigation (WM_KEYDOWN),
    /// recherche (WM_CHAR), defilement (WM_MOUSEWHEEL) et selection
    /// (WM_LBUTTONDOWN, WM_LBUTTONDBLCLK).
    ///
    /// # Safety
    /// - Utilise le pointeur global APP_PTR, valide dans le thread principal.
    /// - Les appels GDI sont effectues dans le scope BeginPaint/EndPaint.
    unsafe extern "system" fn wndproc_popup(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let app = APP_PTR;
        if app.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        let app = &mut *app;

        // Determiner quelle fenetre est concernee
        let is_popup = hwnd == app.popup.hwnd;
        let is_sidebar = hwnd == app.sidebar.hwnd;
        let is_permanent = hwnd == app.permanent.hwnd;

        // Obtenir la palette selon le theme
        let palette = match app.settings.theme {
            crate::config::settings::ThemeMode::Dark => theme::get_palette(theme::ThemeMode::Dark),
            crate::config::settings::ThemeMode::Light => theme::get_palette(theme::ThemeMode::Light),
        };

        match msg {
            WM_PAINT => {
                let entries = app.history.as_slice();
                if is_popup {
                    app.popup.paint(entries, palette);
                } else if is_sidebar {
                    app.sidebar.paint(entries, palette);
                } else if is_permanent {
                    app.permanent.paint(entries, palette);
                } else {
                    return DefWindowProcW(hwnd, msg, wparam, lparam);
                }
                0
            }
            WM_KEYDOWN => {
                let entries_len = app.history.len();
                match wparam as u32 {
                    VK_ESCAPE => {
                        if is_popup {
                            app.popup.hide();
                        } else if is_sidebar {
                            app.sidebar.toggle();
                        } else if is_permanent {
                            app.permanent.toggle();
                        }
                        0
                    }
                    VK_UP => {
                        if is_popup { app.popup.move_up(entries_len); }
                        else if is_sidebar { app.sidebar.move_up(entries_len); }
                        else if is_permanent { app.permanent.move_up(entries_len); }
                        0
                    }
                    VK_DOWN => {
                        if is_popup { app.popup.move_down(entries_len); }
                        else if is_sidebar { app.sidebar.move_down(entries_len); }
                        else if is_permanent { app.permanent.move_down(entries_len); }
                        0
                    }
                    VK_RETURN => {
                        app.on_select();
                        0
                    }
                    VK_DELETE => {
                        app.on_delete();
                        0
                    }
                    0x08 => {
                        // VK_BACK - effacer le dernier caractere de recherche
                        if is_popup {
                            app.popup.search_pop();
                        }
                        0
                    }
                    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
                }
            }
            WM_CHAR => {
                // Recherche incrementale dans le popup
                let ch = wparam as u32;
                if is_popup && ch >= 0x20 && ch < 0x7F {
                    if let Some(c) = char::from_u32(ch) {
                        app.popup.search_push(c);
                    }
                }
                0
            }
            WM_MOUSEWHEEL => {
                let delta = hiword_w(wparam);
                let entries_len = app.history.len();
                if is_popup {
                    app.popup.scroll(delta as i32, entries_len);
                }
                0
            }
            WM_LBUTTONDOWN => {
                // Clic pour selectionner un element
                let y = hiword_l(lparam) as i32;
                if is_popup {
                    let item_h = app.dpi.scale_i32(crate::ui::renderer::ITEM_HEIGHT_BASE);
                    if item_h > 0 {
                        let idx = app.popup.scroll_offset + (y / item_h) as usize;
                        if idx < app.history.len() {
                            app.popup.selected = idx;
                            app.on_select();
                        }
                    }
                }
                0
            }
            WM_LBUTTONDBLCLK => {
                app.on_toggle_pin();
                0
            }
            WM_KILLFOCUS => {
                // Cacher le popup quand il perd le focus
                if is_popup {
                    app.popup.hide();
                }
                0
            }
            WM_ERASEBKGND => {
                1 // On gere le fond via double buffering
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    /// WndProc de la fenetre du gestionnaire d'historique.
    ///
    /// Gere deux modes : navigation (defilement, selection, suppression)
    /// et edition (modification inline du contenu d'une entree).
    ///
    /// # Safety
    /// - Utilise le pointeur global APP_PTR, valide dans le thread principal.
    /// - Le mode edition manipule un buffer interne, pas de memoire partagee.
    unsafe extern "system" fn wndproc_manager(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let app = APP_PTR;
        if app.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        let app = &mut *app;

        let palette = match app.settings.theme {
            crate::config::settings::ThemeMode::Dark => theme::get_palette(theme::ThemeMode::Dark),
            crate::config::settings::ThemeMode::Light => theme::get_palette(theme::ThemeMode::Light),
        };

        match msg {
            WM_PAINT => {
                let entries = app.history.as_slice();
                app.manager.paint(entries, palette, &app.dpi);
                0
            }
            WM_KEYDOWN => {
                let count = app.history.len();
                // Verifier si Ctrl est enfonce
                let ctrl = (GetKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0;

                if app.manager.editing_index.is_some() {
                    // Mode edition actif
                    match wparam as u32 {
                        VK_RETURN => {
                            if let Some((idx, new_content)) = app.manager.confirm_edit() {
                                if let Some(entry) = app.history.get_mut(idx) {
                                    entry.content = new_content;
                                }
                            }
                            0
                        }
                        VK_ESCAPE => {
                            app.manager.cancel_edit();
                            0
                        }
                        0x08 => {
                            // VK_BACK
                            if app.manager.edit_cursor > 0 {
                                app.manager.edit_cursor -= 1;
                                app.manager.edit_buffer.remove(app.manager.edit_cursor);
                                window::invalidate(hwnd);
                            }
                            0
                        }
                        VK_DELETE => {
                            if app.manager.edit_cursor < app.manager.edit_buffer.len() {
                                app.manager.edit_buffer.remove(app.manager.edit_cursor);
                                window::invalidate(hwnd);
                            }
                            0
                        }
                        37 => {
                            // VK_LEFT
                            if app.manager.edit_cursor > 0 {
                                app.manager.edit_cursor -= 1;
                                window::invalidate(hwnd);
                            }
                            0
                        }
                        39 => {
                            // VK_RIGHT
                            if app.manager.edit_cursor < app.manager.edit_buffer.len() {
                                app.manager.edit_cursor += 1;
                                window::invalidate(hwnd);
                            }
                            0
                        }
                        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
                    }
                } else {
                    // Mode navigation
                    match wparam as u32 {
                        VK_UP => {
                            app.manager.move_up(count, &app.dpi);
                            0
                        }
                        VK_DOWN => {
                            app.manager.move_down(count, &app.dpi);
                            0
                        }
                        VK_SPACE => {
                            app.manager.toggle_check();
                            0
                        }
                        VK_F2 => {
                            let entries = app.history.as_slice();
                            app.manager.start_edit(entries);
                            0
                        }
                        VK_DELETE => {
                            // Suppression par lot des elements coches ou de l'element courant
                            let indices = app.manager.checked_indices_desc();
                            if indices.is_empty() {
                                // Supprimer l'element sous le curseur
                                let idx = app.manager.cursor;
                                if idx < app.history.len() {
                                    app.history.remove(idx);
                                    let new_count = app.history.len();
                                    if app.manager.cursor >= new_count && new_count > 0 {
                                        app.manager.cursor = new_count - 1;
                                    }
                                    app.manager.checked = vec![false; new_count];
                                    app.refresh_visible_ui();
                                    window::invalidate(hwnd);
                                }
                            } else {
                                // Supprimer les elements coches (du plus grand index au plus petit)
                                for idx in &indices {
                                    app.history.remove(*idx);
                                }
                                let new_count = app.history.len();
                                if app.manager.cursor >= new_count && new_count > 0 {
                                    app.manager.cursor = new_count - 1;
                                }
                                app.manager.checked = vec![false; new_count];
                                app.refresh_visible_ui();
                                window::invalidate(hwnd);
                            }
                            0
                        }
                        VK_RETURN => {
                            // Entree = copier dans le buffer et fermer
                            let idx = app.manager.cursor;
                            if idx < app.history.len() {
                                if let Some(entry) = app.history.get(idx) {
                                    let text = entry.content.clone();
                                    app.ignore_next_clipboard = true;
                                    let _ = injector::set_clipboard_text(app.hwnd, &text);
                                }
                                app.manager.hide();
                            }
                            0
                        }
                        VK_ESCAPE => {
                            app.manager.hide();
                            0
                        }
                        VK_A if ctrl => {
                            // Ctrl+A : tout selectionner/deselectionner
                            app.manager.toggle_all();
                            0
                        }
                        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
                    }
                }
            }
            WM_CHAR => {
                // Saisie en mode edition
                if app.manager.editing_index.is_some() {
                    let ch = wparam as u32;
                    if ch >= 0x20 {
                        if let Some(c) = char::from_u32(ch) {
                            app.manager.edit_buffer.insert(app.manager.edit_cursor, c);
                            app.manager.edit_cursor += c.len_utf8();
                            window::invalidate(hwnd);
                        }
                    }
                }
                0
            }
            WM_MOUSEWHEEL => {
                let delta = hiword_w(wparam);
                let count = app.history.len();
                app.manager.scroll(delta as i32, count, &app.dpi);
                0
            }
            WM_LBUTTONDOWN => {
                let x = loword_l(lparam) as i32;
                let y = hiword_l(lparam) as i32;
                let count = app.history.len();
                app.manager.on_checkbox_click(x, y, &app.dpi, count);
                0
            }
            WM_LBUTTONDBLCLK => {
                // Double-clic = commencer l'edition
                let entries = app.history.as_slice();
                app.manager.start_edit(entries);
                0
            }
            WM_CLOSE => {
                app.manager.hide();
                0
            }
            WM_ERASEBKGND => {
                1
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    /// WndProc du splash screen.
    ///
    /// Gere l'affichage initial et l'animation de fade-out progressif.
    /// Le splash est detruit automatiquement quand l'opacite atteint 0.
    ///
    /// # Safety
    /// - Utilise le pointeur global APP_PTR pour acceder a l'etat splash.
    /// - Les timers Win32 garantissent un appel sequentiel.
    unsafe extern "system" fn wndproc_splash(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let app = APP_PTR;
        if app.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        let app = &mut *app;

        match msg {
            WM_PAINT => {
                if let Some(ref splash) = app.splash {
                    splash.paint();
                }
                0
            }
            WM_TIMER => {
                let timer_id = wparam;
                let should_destroy = if let Some(ref mut splash) = app.splash {
                    splash.on_timer(timer_id)
                } else {
                    false
                };
                if should_destroy {
                    app.splash = None;
                }
                0
            }
            WM_ERASEBKGND => 1,
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
