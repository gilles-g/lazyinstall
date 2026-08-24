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
        desc: "move through the list",
    },
    Binding {
        keys: "Enter / u",
        desc: "update the selected folder",
    },
    Binding {
        keys: "U",
        desc: "update everything",
    },
    Binding {
        keys: "a",
        desc: "add a folder to track",
    },
    Binding {
        keys: "d",
        desc: "stop tracking the selected folder",
    },
    Binding {
        keys: "(sudo popup)",
        desc: "if a script asks for sudo: type the password, Enter confirms",
    },
    Binding {
        keys: "q / Ctrl-C",
        desc: "quit",
    },
    Binding {
        keys: "?",
        desc: "toggle help",
    },
    Binding {
        keys: "Esc",
        desc: "close help / cancel the add",
    },
];
