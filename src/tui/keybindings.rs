use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    NewPane,
    NextPane,
    PrevPane,
    KillPane,
    RenamePane,
    ZoomPane,
    SplitVertical,
    SplitHorizontal,
    JumpToPane(usize),
    ShowHelp,
    Quit,
    NavigateLeft,
    NavigateDown,
    NavigateUp,
    NavigateRight,
    ForwardKey(KeyModifiers, KeyCode),
    None,
}

pub fn resolve_prefix_action(modifiers: KeyModifiers, code: KeyCode) -> Action {
    if modifiers != KeyModifiers::NONE {
        return Action::None;
    }
    match code {
        KeyCode::Char('c') => Action::NewPane,
        KeyCode::Char('n') => Action::NextPane,
        KeyCode::Char('p') => Action::PrevPane,
        KeyCode::Char('x') => Action::KillPane,
        KeyCode::Char(',') => Action::RenamePane,
        KeyCode::Char('z') => Action::ZoomPane,
        KeyCode::Char('%') => Action::SplitVertical,
        KeyCode::Char('"') => Action::SplitHorizontal,
        KeyCode::Char('?') => Action::ShowHelp,
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('h') => Action::NavigateLeft,
        KeyCode::Char('j') => Action::NavigateDown,
        KeyCode::Char('k') => Action::NavigateUp,
        KeyCode::Char('l') => Action::NavigateRight,
        KeyCode::Char(c) if c.is_ascii_digit() => {
            Action::JumpToPane(c.to_digit(10).unwrap() as usize)
        }
        _ => Action::None,
    }
}
