use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug)]
pub enum Event {
    Key(KeyModifiers, KeyCode),
    PtyOutput { pane_id: uuid::Uuid, data: Vec<u8> },
    Resize(u16, u16),
    Tick,
}
