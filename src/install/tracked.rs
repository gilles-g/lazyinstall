use std::collections::VecDeque;
use std::sync::mpsc::TryRecvError;

use anyhow::Result;

use crate::install::target::{InstallTarget, RunningUpdate, UpdateMessage};

const MAX_LOG_LINES: usize = 1000;

/// État du suivi d'une cible — volatile, propre à la session courante.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateState {
    /// Jamais lancée pendant cette session.
    Idle,
    /// Script en cours d'exécution.
    Updating,
    /// Dernière exécution réussie.
    Succeeded,
    /// Dernière exécution échouée.
    Failed(String),
}

impl UpdateState {
    pub fn label(&self) -> &str {
        match self {
            UpdateState::Idle => "idle",
            UpdateState::Updating => "updating…",
            UpdateState::Succeeded => "up to date",
            UpdateState::Failed(_) => "FAILED",
        }
    }
}

/// Une cible suivie : l'objet de domaine immuable assorti de son état
/// d'exécution volatile et de la sortie de sa dernière mise à jour.
///
/// La cible dit *ce que c'est* ; ce wrapper dit *ce qu'on en fait ici et
/// maintenant*. Il sait se lancer et faire avancer son propre flux.
pub struct TrackedTarget {
    target: InstallTarget,
    state: UpdateState,
    running: Option<RunningUpdate>,
    logs: VecDeque<String>,
    awaiting_password: Option<String>,
    password_sends: u32,
}

impl TrackedTarget {
    pub fn new(target: InstallTarget) -> Self {
        Self {
            target,
            state: UpdateState::Idle,
            running: None,
            logs: VecDeque::new(),
            awaiting_password: None,
            password_sends: 0,
        }
    }

    pub fn target(&self) -> &InstallTarget {
        &self.target
    }

    pub fn name(&self) -> &str {
        self.target.name()
    }

    pub fn state(&self) -> &UpdateState {
        &self.state
    }

    pub fn is_updating(&self) -> bool {
        matches!(self.state, UpdateState::Updating)
    }

    pub fn logs(&self) -> &VecDeque<String> {
        &self.logs
    }

    /// La cible attend-elle un mot de passe ? (un `sudo` du script l'a réclamé)
    pub fn is_awaiting_password(&self) -> bool {
        self.awaiting_password.is_some()
    }

    /// L'invite de mot de passe en attente, le cas échéant.
    pub fn awaiting_prompt(&self) -> Option<&str> {
        self.awaiting_password.as_deref()
    }

    /// Nombre de mots de passe déjà transmis à la mise à jour en cours. Sert à
    /// distinguer une première demande d'une re-demande (mot de passe refusé).
    pub fn password_sends(&self) -> u32 {
        self.password_sends
    }

    /// Lance la mise à jour : démarre le script de la cible et bascule en
    /// suivi. Sans effet si une mise à jour tourne déjà.
    pub fn launch(&mut self) -> Result<()> {
        if self.is_updating() {
            return Ok(());
        }
        let running = self.target.update()?;
        self.logs.clear();
        self.state = UpdateState::Updating;
        self.running = Some(running);
        self.awaiting_password = None;
        self.password_sends = 0;
        Ok(())
    }

    /// Transmet un mot de passe à la mise à jour en cours et lève l'attente.
    pub fn provide_password(&mut self, password: String) {
        if let Some(running) = &self.running {
            running.provide_password(password);
            self.password_sends += 1;
        }
        self.awaiting_password = None;
    }

    /// Interrompt la mise à jour en cours (annulation de la saisie). La cible
    /// finira en échec au tour de `pump()` suivant.
    pub fn cancel(&mut self) {
        if let Some(running) = &mut self.running {
            running.cancel();
        }
        self.awaiting_password = None;
    }

    /// Fait avancer le flux de la mise à jour en cours : récupère sans bloquer
    /// les lignes disponibles et applique la fin du script s'il a terminé.
    pub fn pump(&mut self) {
        let Some(running) = self.running.take() else {
            return;
        };

        let mut finished: Option<bool> = None;
        loop {
            match running.rx.try_recv() {
                Ok(UpdateMessage::Line(line)) => self.push_log(line),
                Ok(UpdateMessage::PasswordPrompt(prompt)) => {
                    self.awaiting_password = Some(prompt);
                }
                Ok(UpdateMessage::Finished(success)) => finished = Some(success),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    finished.get_or_insert(false);
                    break;
                }
            }
        }

        match finished {
            Some(true) => self.state = UpdateState::Succeeded,
            Some(false) => {
                self.state = UpdateState::Failed("script exited with an error".to_string());
            }
            // Pas encore fini : on remet la poignée en place pour le prochain tour.
            None => self.running = Some(running),
        }
    }

    fn push_log(&mut self, line: String) {
        self.logs.push_back(line);
        if self.logs.len() > MAX_LOG_LINES {
            self.logs.pop_front();
        }
    }
}
