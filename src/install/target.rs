use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use anyhow::{Context, Result};
use portable_pty::{ChildKiller, CommandBuilder, PtySize};

/// Un message émis par un script de mise à jour en cours d'exécution.
pub enum UpdateMessage {
    /// Une ligne de sortie (stdout et stderr fusionnés par le terminal).
    Line(String),
    /// Le script attend un mot de passe : l'invite détectée dans le flux
    /// (typiquement `[sudo] password for …`).
    PasswordPrompt(String),
    /// Le script s'est terminé ; `true` si son code de retour vaut 0.
    Finished(bool),
}

/// Poignée vers une mise à jour en cours. On peut lire sa sortie (`rx`), lui
/// transmettre un mot de passe et l'interrompre — les trois gestes qu'on pose
/// sur un process vivant. Le pseudo-terminal et le `Child` restent cachés.
pub struct RunningUpdate {
    pub rx: Receiver<UpdateMessage>,
    password_tx: Sender<String>,
    killer: Box<dyn ChildKiller + Send + Sync>,
}

impl RunningUpdate {
    /// Transmet un mot de passe au script (écrit dans le terminal, suivi d'un
    /// retour chariot). Sans effet si le script est déjà terminé.
    pub fn provide_password(&self, password: String) {
        let _ = self.password_tx.send(password);
    }

    /// Interrompt la mise à jour en tuant le process (utilisé quand l'utilisateur
    /// annule la saisie du mot de passe).
    pub fn cancel(&mut self) {
        let _ = self.killer.kill();
    }
}

/// Une cible d'installation : un dossier suivi qui contient un script de mise
/// à jour. Objet de domaine immuable — elle connaît son chemin et son script,
/// et sait se mettre à jour elle-même.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallTarget {
    name: String,
    folder: PathBuf,
    script: PathBuf,
}

impl InstallTarget {
    pub fn new(
        name: impl Into<String>,
        folder: impl Into<PathBuf>,
        script: impl Into<PathBuf>,
    ) -> Self {
        Self {
            name: name.into(),
            folder: folder.into(),
            script: script.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn folder(&self) -> &Path {
        &self.folder
    }

    pub fn script(&self) -> &Path {
        &self.script
    }

    /// Lance le script de mise à jour de cette cible et renvoie le flux de sa
    /// sortie. La cible sait se mettre à jour elle-même : elle détient son
    /// dossier de travail et son script.
    ///
    /// Le script tourne dans un pseudo-terminal (PTY) : ainsi un `sudo` du
    /// script voit un vrai terminal et peut réclamer un mot de passe, qu'on
    /// détecte et réinjecte. `LC_ALL=C` force une invite déterministe.
    pub fn update(&self) -> Result<RunningUpdate> {
        let pty = portable_pty::native_pty_system()
            .openpty(PtySize::default())
            .context("cannot open the pseudo-terminal")?;

        let mut command = CommandBuilder::new("bash");
        command.arg(&self.script);
        command.cwd(&self.folder);
        command.env("LC_ALL", "C");

        let reader = pty
            .master
            .try_clone_reader()
            .context("cannot read from the terminal")?;
        let writer = pty
            .master
            .take_writer()
            .context("cannot write to the terminal")?;
        let mut child = pty
            .slave
            .spawn_command(command)
            .with_context(|| format!("cannot launch {}", self.script.display()))?;
        let killer = child.clone_killer();
        // On ferme notre extrémité « esclave » : le lecteur recevra ainsi l'EOF
        // quand le script aura terminé.
        drop(pty.slave);

        let (tx, rx) = mpsc::channel();
        let (password_tx, password_rx) = mpsc::channel::<String>();

        let tx_reader = tx.clone();
        let reader_thread = thread::spawn(move || read_output(reader, tx_reader));
        thread::spawn(move || write_passwords(writer, password_rx));

        // Le master reste vivant dans ce thread le temps de l'exécution, puis on
        // attend la fin des lecteurs avant d'annoncer le code de sortie : aucune
        // ligne n'est perdue.
        let master = pty.master;
        thread::spawn(move || {
            let _ = reader_thread.join();
            let success = child.wait().map(|status| status.success()).unwrap_or(false);
            drop(master);
            let _ = tx.send(UpdateMessage::Finished(success));
        });

        Ok(RunningUpdate {
            rx,
            password_tx,
            killer,
        })
    }
}

/// Lit le flux du terminal : émet chaque ligne complète, et signale une invite
/// de mot de passe quand le tampon partiel (sans retour ligne) y ressemble.
fn read_output(mut reader: Box<dyn Read + Send>, tx: Sender<UpdateMessage>) {
    let mut chunk = [0u8; 1024];
    let mut pending: Vec<u8> = Vec::new();
    let mut prompt_signaled = false;

    loop {
        let read = match reader.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        pending.extend_from_slice(&chunk[..read]);

        while let Some(eol) = pending.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = pending.drain(..=eol).collect();
            let text = strip_line_endings(&String::from_utf8_lossy(&line));
            if tx.send(UpdateMessage::Line(text)).is_err() {
                return;
            }
            // Une nouvelle ligne est passée : une éventuelle invite suivante
            // (ex. « Sorry, try again. » puis re-demande) pourra resignaler.
            prompt_signaled = false;
        }

        if !pending.is_empty() && !prompt_signaled {
            let partial = String::from_utf8_lossy(&pending);
            if looks_like_password_prompt(&partial) {
                if tx
                    .send(UpdateMessage::PasswordPrompt(partial.trim().to_string()))
                    .is_err()
                {
                    return;
                }
                prompt_signaled = true;
            }
        }
    }

    let rest = strip_line_endings(&String::from_utf8_lossy(&pending));
    if !rest.is_empty() {
        let _ = tx.send(UpdateMessage::Line(rest));
    }
}

/// Écrit dans le terminal les mots de passe reçus, suivis d'un retour chariot.
/// Le tampon est remis à zéro après écriture pour ne pas laisser traîner le
/// secret en mémoire.
fn write_passwords(mut writer: Box<dyn Write + Send>, password_rx: Receiver<String>) {
    while let Ok(password) = password_rx.recv() {
        let mut bytes = password.into_bytes();
        bytes.push(b'\n');
        let ok = writer
            .write_all(&bytes)
            .and_then(|_| writer.flush())
            .is_ok();
        bytes.iter_mut().for_each(|b| *b = 0);
        if !ok {
            break;
        }
    }
}

fn strip_line_endings(line: &str) -> String {
    line.trim_end_matches(['\r', '\n']).to_string()
}

/// Reconnaît une invite de mot de passe : la dernière ligne du tampon (encore
/// sans retour ligne, signe qu'un programme attend une saisie) se termine par
/// `:` et évoque un mot de passe. Volontairement étroite — testée seule.
fn looks_like_password_prompt(buffer: &str) -> bool {
    let last_line = buffer.rsplit('\n').next().unwrap_or(buffer).trim_end();
    if !last_line.ends_with(':') {
        return false;
    }
    let lowered = last_line.to_lowercase();
    lowered.contains("password") || lowered.contains("[sudo]") || lowered.contains("mot de passe")
}

#[cfg(test)]
mod tests {
    use super::looks_like_password_prompt;

    #[test]
    fn it_recognizes_sudo_and_generic_password_prompts() {
        assert!(looks_like_password_prompt("[sudo] password for bob: "));
        assert!(looks_like_password_prompt("Password:"));
        assert!(looks_like_password_prompt(
            "première ligne\n[sudo] password for bob: "
        ));
        assert!(looks_like_password_prompt("Mot de passe : "));
    }

    #[test]
    fn it_ignores_regular_output() {
        assert!(!looks_like_password_prompt("téléchargement... 50%"));
        assert!(!looks_like_password_prompt(""));
        assert!(!looks_like_password_prompt(
            "password mis à jour avec succès"
        ));
        assert!(!looks_like_password_prompt(
            "Étapes restantes :\nclonage en cours"
        ));
    }
}
