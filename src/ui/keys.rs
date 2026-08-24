use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Help,
    CloseOverlay,
    Up,
    Down,
    Update,
    UpdateAll,
    Add,
    Remove,
}

pub struct KeyMap;

impl KeyMap {
    pub fn map(ev: KeyEvent) -> Option<Action> {
        let m = ev.modifiers;
        match ev.code {
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char('c') if m.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
            KeyCode::Char('?') => Some(Action::Help),
            KeyCode::Esc => Some(Action::CloseOverlay),
            KeyCode::Char('j') | KeyCode::Down => Some(Action::Down),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::Up),
            KeyCode::Char('u') | KeyCode::Enter => Some(Action::Update),
            KeyCode::Char('U') => Some(Action::UpdateAll),
            KeyCode::Char('a') => Some(Action::Add),
            KeyCode::Char('d') => Some(Action::Remove),
            _ => None,
        }
    }
}

pub struct Binding {
    pub keys: &'static str,
    pub desc: &'static str,
}

pub const HELP: &[Binding] = &[
    Binding {
        keys: "j/k, ↓/↑",
        desc: "naviguer dans la liste",
    },
    Binding {
        keys: "Enter / u",
        desc: "mettre à jour le dossier sélectionné",
    },
    Binding {
        keys: "U",
        desc: "tout mettre à jour",
    },
    Binding {
        keys: "a",
        desc: "ajouter un dossier à suivre",
    },
    Binding {
        keys: "d",
        desc: "retirer le dossier sélectionné",
    },
    Binding {
        keys: "(popup sudo)",
        desc: "si un script demande sudo : saisir le mot de passe, Enter valide",
    },
    Binding {
        keys: "q / Ctrl-C",
        desc: "quitter",
    },
    Binding {
        keys: "?",
        desc: "basculer l'aide",
    },
    Binding {
        keys: "Esc",
        desc: "fermer l'aide / annuler l'ajout",
    },
];
