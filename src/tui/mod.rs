pub mod events;
pub mod keybindings;
pub mod ui;
pub mod widgets;

use crate::{
    app::{App, Mode},
    config::Config,
    ipc::server::{cleanup_session_file, run_ipc_server, write_session_file},
    terminal::manager::TerminalManager,
};
use anyhow::Result;
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event as CEvent, EventStream, KeyCode,
        KeyEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use keybindings::{resolve_prefix_action, Action};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

pub async fn run(config: Config) -> Result<()> {
    let session_id = Uuid::new_v4().to_string();

    // Single manager shared between the TUI and the IPC/MCP server.
    // This ensures terminals created by Claude appear in the TUI immediately.
    let (pty_tx, pty_rx) = tokio::sync::mpsc::unbounded_channel::<(Uuid, Vec<u8>)>();
    let manager: Arc<Mutex<TerminalManager>> =
        Arc::new(Mutex::new(TerminalManager::new(config.shell_command(), pty_tx)));

    write_session_file(&session_id)?;

    let ipc_mgr = Arc::clone(&manager);
    let ipc_session = session_id.clone();
    tokio::spawn(async move {
        if let Err(e) = run_ipc_server(ipc_session, ipc_mgr).await {
            tracing::error!("IPC server error: {e}");
        }
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, config, manager, pty_rx).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    cleanup_session_file();
    result
}

async fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    config: Config,
    manager: Arc<Mutex<TerminalManager>>,
    pty_rx: tokio::sync::mpsc::UnboundedReceiver<(Uuid, Vec<u8>)>,
) -> Result<()> {
    let mut app = App::with_manager(config, manager, pty_rx);
    let size = terminal.size()?;
    let pane_rows = size.height.saturating_sub(2);
    let _ = app.new_pane(size.width, pane_rows);

    let mut event_stream = EventStream::new();

    loop {
        // Always drain PTY output before rendering
        while app.pty_rx.try_recv().is_ok() {}

        terminal.draw(|frame| ui::render(frame, &app))?;

        // Block until EITHER a keyboard event arrives OR PTY output arrives.
        // No polling — we wake up exactly when there is work to do.
        tokio::select! {
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(CEvent::Key(key))) if key.kind == KeyEventKind::Press => {
                        handle_key(&mut app, key.modifiers, key.code, terminal)?;
                    }
                    Some(Ok(CEvent::Resize(cols, rows))) => {
                        let pane_rows = rows.saturating_sub(2);
                        app.terminals.lock().unwrap().resize_all(cols, pane_rows);
                    }
                    None => break,
                    _ => {}
                }
            }
            // PTY output arrived — loop back immediately to drain and redraw
            _ = app.pty_rx.recv() => {}
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_key<B: ratatui::backend::Backend>(
    app: &mut App,
    modifiers: crossterm::event::KeyModifiers,
    code: KeyCode,
    terminal: &mut Terminal<B>,
) -> Result<()> {
    match app.mode {
        Mode::Normal => {
            if app.config.matches_prefix(modifiers, code) {
                app.enter_prefix_mode();
            } else {
                let bytes = key_to_bytes(modifiers, code);
                if !bytes.is_empty() {
                    app.write_to_active(&bytes);
                }
            }
        }
        Mode::Prefix => {
            app.exit_prefix_mode();
            let size = terminal.size()?;
            let pane_rows = size.height.saturating_sub(2);

            match resolve_prefix_action(modifiers, code) {
                Action::Quit => app.quit(),
                Action::ShowHelp => app.show_help = !app.show_help,
                Action::ShowStatus => app.show_status = !app.show_status,
                Action::NewPane => app.new_pane(size.width, pane_rows)?,
                Action::NextPane => app.terminals.lock().unwrap().next(),
                Action::PrevPane => app.terminals.lock().unwrap().prev(),
                Action::KillPane => app.terminals.lock().unwrap().kill_active(),
                Action::ZoomPane => app.toggle_zoom(),
                Action::SplitVertical => app.split_vertical(size.width, pane_rows)?,
                Action::SplitHorizontal => app.split_horizontal(size.width, pane_rows)?,
                Action::RenamePane => {
                    app.input_buf.clear();
                    app.mode = Mode::Rename;
                }
                Action::NavigateLeft | Action::NavigateUp => app.terminals.lock().unwrap().prev(),
                Action::NavigateRight | Action::NavigateDown => app.terminals.lock().unwrap().next(),
                Action::JumpToPane(n) => app.terminals.lock().unwrap().jump_to(n),
                _ => {}
            }
        }
        Mode::Rename => match code {
            KeyCode::Enter => {
                let label = app.input_buf.drain(..).collect();
                app.terminals.lock().unwrap().rename_active(label);
                app.mode = Mode::Normal;
            }
            KeyCode::Esc => {
                app.input_buf.clear();
                app.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                app.input_buf.pop();
            }
            KeyCode::Char(c) => app.input_buf.push(c),
            _ => {}
        },
    }
    Ok(())
}

fn key_to_bytes(modifiers: crossterm::event::KeyModifiers, code: KeyCode) -> Vec<u8> {
    use crossterm::event::KeyModifiers;
    match code {
        KeyCode::Char(c) => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                let b = c.to_ascii_lowercase() as u8;
                if b >= b'a' && b <= b'z' {
                    return vec![b - b'a' + 1];
                }
            }
            let mut buf = [0u8; 4];
            c.encode_utf8(&mut buf).as_bytes().to_vec()
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![127],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Esc => vec![27],
        KeyCode::Up => vec![27, b'[', b'A'],
        KeyCode::Down => vec![27, b'[', b'B'],
        KeyCode::Right => vec![27, b'[', b'C'],
        KeyCode::Left => vec![27, b'[', b'D'],
        KeyCode::Home => vec![27, b'[', b'H'],
        KeyCode::End => vec![27, b'[', b'F'],
        KeyCode::Delete => vec![27, b'[', b'3', b'~'],
        KeyCode::PageUp => vec![27, b'[', b'5', b'~'],
        KeyCode::PageDown => vec![27, b'[', b'6', b'~'],
        _ => vec![],
    }
}
