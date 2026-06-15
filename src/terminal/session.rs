use crate::terminal::screen::Screen;
use anyhow::{Context, Result};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize};
use std::{
    io::Write,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct TerminalSession {
    pub id: Uuid,
    pub label: String,
    pub screen: Arc<Mutex<Screen>>,
    master: Box<dyn MasterPty + Send>,
    /// Writer is stored so it can be reused across multiple keystrokes.
    /// take_writer() can only be called once on the PTY master.
    writer: Mutex<Box<dyn Write + Send>>,
    _child: Box<dyn Child + Send + Sync>,
    pub cols: u16,
    pub rows: u16,
}

impl TerminalSession {
    pub fn spawn(
        label: impl Into<String>,
        shell: &str,
        cols: u16,
        rows: u16,
        output_tx: mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
    ) -> Result<Self> {
        let id = Uuid::new_v4();
        let pty_system = portable_pty::native_pty_system();

        let pair = pty_system
            .openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .context("failed to open PTY")?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");

        let child = pair.slave.spawn_command(cmd).context("failed to spawn shell")?;

        let screen = Arc::new(Mutex::new(Screen::new(rows, cols)));
        let reader = pair.master.try_clone_reader().context("failed to clone PTY reader")?;
        let writer = pair.master.take_writer().context("failed to take PTY writer")?;

        let screen_clone = Arc::clone(&screen);
        std::thread::spawn(move || {
            read_pty_output(reader, screen_clone, output_tx, id);
        });

        Ok(Self {
            id,
            label: label.into(),
            screen,
            writer: Mutex::new(writer),
            master: pair.master,
            _child: child,
            cols,
            rows,
        })
    }

    pub fn write(&self, data: &[u8]) -> Result<()> {
        self.writer
            .lock()
            .map_err(|_| anyhow::anyhow!("PTY writer lock poisoned"))?
            .write_all(data)
            .context("failed to write to PTY")
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.cols = cols;
        self.rows = rows;
        self.master
            .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| anyhow::anyhow!("resize failed: {e}"))?;
        if let Ok(mut s) = self.screen.lock() {
            s.resize(rows, cols);
        }
        Ok(())
    }
}

fn read_pty_output(
    mut reader: Box<dyn std::io::Read + Send>,
    screen: Arc<Mutex<Screen>>,
    tx: mpsc::UnboundedSender<(Uuid, Vec<u8>)>,
    id: Uuid,
) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let data = buf[..n].to_vec();
                if let Ok(mut s) = screen.lock() {
                    s.process(&data);
                }
                let _ = tx.send((id, data));
            }
            Err(_) => break,
        }
    }
}
