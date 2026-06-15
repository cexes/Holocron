use crate::terminal::session::TerminalSession;
use anyhow::Result;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct TerminalManager {
    sessions: Vec<TerminalSession>,
    pub active_index: usize,
    output_tx: mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
    shell: String,
}

impl TerminalManager {
    pub fn new(shell: String, output_tx: mpsc::UnboundedSender<(Uuid, Vec<u8>)>) -> Self {
        Self {
            sessions: Vec::new(),
            active_index: 0,
            output_tx,
            shell,
        }
    }

    pub fn create(&mut self, label: impl Into<String>, cols: u16, rows: u16) -> Result<Uuid> {
        let session = TerminalSession::spawn(
            label,
            &self.shell,
            cols,
            rows,
            self.output_tx.clone(),
        )?;
        let id = session.id;
        self.sessions.push(session);
        self.active_index = self.sessions.len() - 1;
        Ok(id)
    }

    pub fn kill(&mut self, id: Uuid) {
        if let Some(pos) = self.sessions.iter().position(|s| s.id == id) {
            self.sessions.remove(pos);
            if self.active_index >= self.sessions.len() && !self.sessions.is_empty() {
                self.active_index = self.sessions.len() - 1;
            }
        }
    }

    pub fn kill_active(&mut self) {
        if let Some(session) = self.sessions.get(self.active_index) {
            let id = session.id;
            self.kill(id);
        }
    }

    pub fn next(&mut self) {
        if !self.sessions.is_empty() {
            self.active_index = (self.active_index + 1) % self.sessions.len();
        }
    }

    pub fn prev(&mut self) {
        if !self.sessions.is_empty() {
            self.active_index =
                (self.active_index + self.sessions.len() - 1) % self.sessions.len();
        }
    }

    pub fn jump_to(&mut self, index: usize) {
        if index < self.sessions.len() {
            self.active_index = index;
        }
    }

    pub fn active(&self) -> Option<&TerminalSession> {
        self.sessions.get(self.active_index)
    }

    pub fn active_mut(&mut self) -> Option<&mut TerminalSession> {
        self.sessions.get_mut(self.active_index)
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut TerminalSession> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    pub fn sessions(&self) -> &[TerminalSession] {
        &self.sessions
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn rename_active(&mut self, new_label: String) {
        if let Some(s) = self.sessions.get_mut(self.active_index) {
            s.label = new_label;
        }
    }

    pub fn resize_all(&mut self, cols: u16, rows: u16) {
        for session in &mut self.sessions {
            let _ = session.resize(cols, rows);
        }
    }

    /// Returns snapshot of all sessions for MCP list_terminals.
    pub fn list_info(&self) -> Vec<SessionInfo> {
        self.sessions
            .iter()
            .enumerate()
            .map(|(i, s)| SessionInfo {
                id: s.id,
                label: s.label.clone(),
                index: i,
                is_active: i == self.active_index,
            })
            .collect()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub label: String,
    pub index: usize,
    pub is_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manager() -> (TerminalManager, mpsc::UnboundedReceiver<(Uuid, Vec<u8>)>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let shell = crate::config::Config::default_shell();
        (TerminalManager::new(shell, tx), rx)
    }

    #[test]
    fn starts_empty() {
        let (mgr, _rx) = make_manager();
        assert!(mgr.is_empty());
    }

    #[test]
    fn next_and_prev_wrap() {
        let (tx, _rx) = mpsc::unbounded_channel::<(Uuid, Vec<u8>)>();
        let shell = crate::config::Config::default_shell();
        let mut mgr = TerminalManager::new(shell, tx);
        mgr.create("a", 80, 24).unwrap();
        mgr.create("b", 80, 24).unwrap();
        assert_eq!(mgr.active_index, 1);
        mgr.next();
        assert_eq!(mgr.active_index, 0);
        mgr.prev();
        assert_eq!(mgr.active_index, 1);
    }
}
