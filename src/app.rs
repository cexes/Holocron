use crate::{config::Config, terminal::manager::TerminalManager};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Prefix,
    Rename,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Single,
    VSplit,
    HSplit,
}

pub struct App {
    pub config: Config,
    pub mode: Mode,
    pub layout: LayoutMode,
    pub zoomed: bool,
    pub should_quit: bool,
    pub show_help: bool,
    pub show_status: bool,
    pub input_buf: String,
    /// Shared with the IPC server so Claude sees the same panes the TUI shows.
    pub terminals: Arc<Mutex<TerminalManager>>,
    pub pty_rx: mpsc::UnboundedReceiver<(Uuid, Vec<u8>)>,
}

impl App {
    pub fn with_manager(
        config: Config,
        terminals: Arc<Mutex<TerminalManager>>,
        pty_rx: mpsc::UnboundedReceiver<(Uuid, Vec<u8>)>,
    ) -> Self {
        Self {
            config,
            terminals,
            pty_rx,
            mode: Mode::Normal,
            layout: LayoutMode::Single,
            zoomed: false,
            should_quit: false,
            show_help: false,
            show_status: false,
            input_buf: String::new(),
        }
    }

    pub fn enter_prefix_mode(&mut self) {
        self.mode = Mode::Prefix;
    }

    pub fn exit_prefix_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn new_pane(&mut self, cols: u16, rows: u16) -> anyhow::Result<()> {
        let mut mgr = self.terminals.lock().unwrap();
        let n = mgr.sessions().len() + 1;
        mgr.create(format!("terminal {n}"), cols, rows)?;
        Ok(())
    }

    pub fn toggle_zoom(&mut self) {
        if !self.terminals.lock().unwrap().is_empty() {
            self.zoomed = !self.zoomed;
        }
    }

    pub fn split_vertical(&mut self, cols: u16, rows: u16) -> anyhow::Result<()> {
        self.layout = LayoutMode::VSplit;
        let mut mgr = self.terminals.lock().unwrap();
        let n = mgr.sessions().len() + 1;
        mgr.create(format!("terminal {n}"), cols / 2, rows)?;
        Ok(())
    }

    pub fn split_horizontal(&mut self, cols: u16, rows: u16) -> anyhow::Result<()> {
        self.layout = LayoutMode::HSplit;
        let mut mgr = self.terminals.lock().unwrap();
        let n = mgr.sessions().len() + 1;
        mgr.create(format!("terminal {n}"), cols, rows / 2)?;
        Ok(())
    }

    pub fn write_to_active(&self, data: &[u8]) {
        if let Ok(mgr) = self.terminals.lock() {
            if let Some(session) = mgr.active() {
                let _ = session.write(data);
            }
        }
    }
}
