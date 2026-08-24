use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::backend::Backend;
use ratatui::Terminal;

use crate::install::discover;
use crate::install::store;
use crate::install::target::InstallTarget;
use crate::install::tracked::TrackedTarget;
use crate::ui::keys::{Action, KeyMap};
use crate::ui::layout;
use crate::ui::panels::{add_prompt, help, output, password_prompt, statusbar, target_list};

const TOAST_TTL: Duration = Duration::from_secs(4);

pub struct App {
    targets: Vec<TrackedTarget>,
    cursor: usize,
    add_input: Option<String>,
    password_prompt: Option<PasswordEntry>,
    sudo_password: Option<String>,
    show_help: bool,
    toast: Option<(String, Instant)>,
    quit: bool,
}

/// Saisie en cours du mot de passe sudo pour une cible donnée.
struct PasswordEntry {
    target: usize,
    input: String,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            targets: Vec::new(),
            cursor: 0,
            add_input: None,
            password_prompt: None,
            sudo_password: None,
            show_help: false,
            toast: None,
            quit: false,
        };
        app.load_targets();
        app
    }

    /// Charge les dossiers suivis depuis la config et fait naître chaque cible.
    /// Les dossiers dont le script a disparu sont retirés de la config (auto
    /// nettoyage) et signalés.
    fn load_targets(&mut self) {
        let mut broken = 0;
        for folder in store::load() {
            match discover::discover(&folder) {
                Ok(targets) => {
                    for target in targets {
                        self.targets.push(TrackedTarget::new(target));
                    }
                }
                Err(_) => broken += 1,
            }
        }
        if broken > 0 {
            // On réécrit la config sans les dossiers cassés.
            let _ = self.persist();
            self.set_toast(format!("{broken} missing folder(s) dropped"));
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        while !self.quit {
            for tracked in &mut self.targets {
                tracked.pump();
            }
            self.handle_password_requests();
            self.expire_toast();

            terminal.draw(|frame| {
                let layout = layout::compute(frame.area());
                target_list::render(frame, layout.target_list, &self.targets, self.cursor);
                output::render(frame, layout.output, self.targets.get(self.cursor));
                let toast = self
                    .toast
                    .as_ref()
                    .filter(|(_, t)| t.elapsed() < TOAST_TTL)
                    .map(|(s, _)| s.as_str());
                statusbar::render(frame, layout.statusbar, toast);
                if self.show_help {
                    help::render(frame, frame.area());
                }
                if let Some(ref input) = self.add_input {
                    add_prompt::render(frame, frame.area(), input);
                }
                if let Some(entry) = &self.password_prompt {
                    if let Some(tracked) = self.targets.get(entry.target) {
                        password_prompt::render(
                            frame,
                            frame.area(),
                            tracked.name(),
                            tracked.awaiting_prompt(),
                            &entry.input,
                        );
                    }
                }
            })?;

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Release {
                        if self.password_prompt.is_some() {
                            self.handle_password_key(key);
                        } else if self.add_input.is_some() {
                            self.handle_add_key(key);
                        } else if let Some(action) = KeyMap::map(key) {
                            self.handle_action(action);
                        }
                    }
                }
            }
        }
        self.clear_sudo_password();
        Ok(())
    }

    /// Repère une cible en attente de mot de passe. Si on en a déjà saisi un
    /// cette session et que c'est la première demande de cette cible, on le
    /// réinjecte sans rien demander ; sinon on ouvre la popup contextualisée.
    fn handle_password_requests(&mut self) {
        if self.password_prompt.is_some() {
            return;
        }
        let Some(idx) = self.targets.iter().position(|t| t.is_awaiting_password()) else {
            return;
        };
        let first_request = self.targets[idx].password_sends() == 0;
        if first_request {
            if let Some(password) = self.sudo_password.clone() {
                self.targets[idx].provide_password(password);
                return;
            }
        }
        self.password_prompt = Some(PasswordEntry {
            target: idx,
            input: String::new(),
        });
    }

    fn handle_password_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                if let Some(entry) = self.password_prompt.take() {
                    if let Some(tracked) = self.targets.get_mut(entry.target) {
                        tracked.cancel();
                    }
                    self.set_toast("password prompt cancelled".to_string());
                }
            }
            KeyCode::Enter => {
                if let Some(entry) = self.password_prompt.take() {
                    self.sudo_password = Some(entry.input.clone());
                    if let Some(tracked) = self.targets.get_mut(entry.target) {
                        tracked.provide_password(entry.input);
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(entry) = self.password_prompt.as_mut() {
                    entry.input.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Some(entry) = self.password_prompt.as_mut() {
                    entry.input.push(c);
                }
            }
            _ => {}
        }
    }

    /// Efface le mot de passe en cache en écrasant son contenu avant libération.
    fn clear_sudo_password(&mut self) {
        if let Some(mut password) = self.sudo_password.take() {
            // SAFETY : on écrit des octets nuls (NUL est de l'UTF-8 valide), puis
            // on libère la chaîne. Le but est de ne pas laisser le secret traîner.
            unsafe {
                password.as_bytes_mut().iter_mut().for_each(|b| *b = 0);
            }
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.quit = true,
            Action::Help => self.show_help = !self.show_help,
            Action::CloseOverlay => self.show_help = false,
            Action::Up => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            Action::Down => {
                if self.cursor + 1 < self.targets.len() {
                    self.cursor += 1;
                }
            }
            Action::Update => self.update_selected(),
            Action::UpdateAll => self.update_all(),
            Action::Add => self.add_input = Some(String::new()),
            Action::Remove => self.remove_selected(),
        }
    }

    fn update_selected(&mut self) {
        let Some(tracked) = self.targets.get_mut(self.cursor) else {
            return;
        };
        if tracked.is_updating() {
            self.set_toast("update already running".to_string());
            return;
        }
        if let Err(e) = tracked.launch() {
            let name = tracked.name().to_string();
            self.set_toast(format!("cannot launch {name}: {e}"));
        }
    }

    fn update_all(&mut self) {
        let mut launched = 0;
        for tracked in &mut self.targets {
            if !tracked.is_updating() && tracked.launch().is_ok() {
                launched += 1;
            }
        }
        self.set_toast(format!("{launched} update(s) launched"));
    }

    fn remove_selected(&mut self) {
        if self.cursor >= self.targets.len() {
            return;
        }
        let removed = self.targets.remove(self.cursor);
        if self.cursor >= self.targets.len() {
            self.cursor = self.targets.len().saturating_sub(1);
        }
        let _ = self.persist();
        self.set_toast(format!("{} no longer tracked", removed.name()));
    }

    fn handle_add_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.add_input = None;
            }
            KeyCode::Enter => {
                if let Some(input) = self.add_input.take() {
                    self.submit_add(input);
                }
            }
            KeyCode::Backspace => {
                if let Some(input) = self.add_input.as_mut() {
                    input.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Some(input) = self.add_input.as_mut() {
                    input.push(c);
                }
            }
            _ => {}
        }
    }

    fn submit_add(&mut self, raw: String) {
        let raw = raw.trim();
        if raw.is_empty() {
            return;
        }
        let folder = expand_tilde(raw);
        match discover::discover(&folder) {
            Ok(targets) => self.track_new(targets),
            Err(e) => self.set_toast(format!("cannot add: {e}")),
        }
    }

    /// Ajoute au suivi les cibles encore inconnues (identité = leur script) et
    /// signale le résultat. Les cibles déjà suivies sont ignorées en silence.
    fn track_new(&mut self, targets: Vec<InstallTarget>) {
        let mut added = 0;
        let mut last_name = String::new();
        for target in targets {
            let already_tracked = self
                .targets
                .iter()
                .any(|t| t.target().script() == target.script());
            if already_tracked {
                continue;
            }
            last_name = target.name().to_string();
            self.targets.push(TrackedTarget::new(target));
            added += 1;
        }
        if added == 0 {
            self.set_toast("already tracked".to_string());
            return;
        }
        self.cursor = self.targets.len() - 1;
        let _ = self.persist();
        match added {
            1 => self.set_toast(format!("{last_name} now tracked")),
            n => self.set_toast(format!("{n} targets now tracked")),
        }
    }

    fn persist(&self) -> Result<()> {
        let folders: Vec<PathBuf> = self
            .targets
            .iter()
            .map(|t| t.target().folder().to_path_buf())
            .collect();
        store::save(&folders)
    }

    fn expire_toast(&mut self) {
        if let Some((_, t)) = &self.toast {
            if t.elapsed() >= TOAST_TTL {
                self.toast = None;
            }
        }
    }

    fn set_toast(&mut self, msg: String) {
        self.toast = Some((msg, Instant::now()));
    }
}

/// Remplace un `~` initial par le dossier personnel de l'utilisateur.
fn expand_tilde(raw: &str) -> PathBuf {
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    if raw == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(raw)
}
