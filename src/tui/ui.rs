use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, LayoutMode, Mode};
use super::widgets::pane::PaneWidget;

pub fn render(frame: &mut Frame, app: &App) {
    let Ok(mgr) = app.terminals.lock() else { return };

    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    render_tab_bar(frame, app, chunks[0], &mgr);

    if app.zoomed {
        render_single_pane(frame, app, chunks[1], mgr.active_index, true, &mgr);
    } else {
        render_pane_layout(frame, app, chunks[1], &mgr);
    }

    render_status_bar(frame, app, chunks[2], &mgr);

    drop(mgr); // release lock before popups

    if app.show_help {
        render_help_overlay(frame);
    }
    if app.mode == Mode::Rename {
        render_rename_popup(frame, app);
    }
}

fn render_pane_layout(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    mgr: &crate::terminal::manager::TerminalManager,
) {
    if mgr.is_empty() {
        render_empty_hint(frame, app, area);
        return;
    }

    match app.layout {
        LayoutMode::Single => {
            render_single_pane(frame, app, area, mgr.active_index, true, mgr);
        }
        LayoutMode::VSplit if mgr.sessions().len() >= 2 => {
            let halves = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);
            let prev = mgr.active_index.saturating_sub(1);
            render_single_pane(frame, app, halves[0], prev, prev == mgr.active_index, mgr);
            render_single_pane(frame, app, halves[1], mgr.active_index, true, mgr);
        }
        LayoutMode::HSplit if mgr.sessions().len() >= 2 => {
            let halves = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);
            let prev = mgr.active_index.saturating_sub(1);
            render_single_pane(frame, app, halves[0], prev, prev == mgr.active_index, mgr);
            render_single_pane(frame, app, halves[1], mgr.active_index, true, mgr);
        }
        _ => {
            render_single_pane(frame, app, area, mgr.active_index, true, mgr);
        }
    }
}

fn render_single_pane(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    index: usize,
    is_active: bool,
    mgr: &crate::terminal::manager::TerminalManager,
) {
    let Some(session) = mgr.sessions().get(index) else { return };

    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let zoom_indicator = if app.zoomed { " [ZOOM] " } else { "" };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(format!(" {}{} ", session.label, zoom_indicator));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Ok(screen_guard) = session.screen.lock() {
        frame.render_widget(PaneWidget { screen: screen_guard.screen() }, inner);
    }
}

fn render_empty_hint(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = if app.mode == Mode::Prefix {
        "PREFIX — waiting for command..."
    } else {
        "No terminals open.\nCtrl+A then c to create one."
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::Gray)),
        inner,
    );
}

fn render_tab_bar(
    frame: &mut Frame,
    _app: &App,
    area: Rect,
    mgr: &crate::terminal::manager::TerminalManager,
) {
    if mgr.is_empty() {
        let line = Line::from(vec![
            Span::styled(" holocron ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" — Ctrl+A then c to create a terminal"),
        ]);
        frame.render_widget(
            Paragraph::new(line).style(Style::default().bg(Color::DarkGray)),
            area,
        );
        return;
    }

    let active = mgr.active_index;
    let mut spans = vec![Span::raw(" ")];
    for (i, s) in mgr.sessions().iter().enumerate() {
        let label = format!(" {}:{} ", i, s.label);
        let style = if i == active {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray).bg(Color::DarkGray)
        };
        spans.push(Span::styled(label, style));
        spans.push(Span::raw(" "));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::DarkGray)),
        area,
    );
}

fn render_status_bar(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    mgr: &crate::terminal::manager::TerminalManager,
) {
    let mode_span = match app.mode {
        Mode::Normal if app.zoomed => {
            Span::styled(" ZOOM ", Style::default().fg(Color::Black).bg(Color::Magenta))
        }
        Mode::Normal => {
            Span::styled(" NORMAL ", Style::default().fg(Color::Black).bg(Color::Cyan))
        }
        Mode::Prefix => {
            Span::styled(" PREFIX ", Style::default().fg(Color::Black).bg(Color::Yellow))
        }
        Mode::Rename => {
            Span::styled(" RENAME ", Style::default().fg(Color::Black).bg(Color::Green))
        }
    };

    let pane_info = if let Some(s) = mgr.active() {
        Span::styled(
            format!(" {}×{} ", s.cols, s.rows),
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::raw("")
    };

    let hints = Span::styled(
        " Ctrl+A: prefix  ?:help  q:quit",
        Style::default().fg(Color::DarkGray),
    );

    frame.render_widget(Paragraph::new(Line::from(vec![mode_span, pane_info, hints])), area);
}

fn render_rename_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup_width = 40u16.min(area.width.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = area.height / 2;
    let popup_area = Rect::new(x, y, popup_width, 3);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title(" Rename pane (Enter to confirm, Esc to cancel) ");

    frame.render_widget(Paragraph::new(format!("{}_", app.input_buf)).block(block), popup_area);
}

fn render_help_overlay(frame: &mut Frame) {
    let area = frame.area();
    let popup_width = 54u16.min(area.width.saturating_sub(4));
    let popup_height = 18u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let help = vec![
        Line::from(Span::styled(
            "Keybindings  (prefix = Ctrl+A)",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from("  Prefix + c        New terminal"),
        Line::from("  Prefix + n / p    Next / previous pane"),
        Line::from("  Prefix + h/j/k/l  Navigate panes"),
        Line::from("  Prefix + 0-9      Jump to pane by number"),
        Line::from("  Prefix + ,        Rename current pane"),
        Line::from("  Prefix + x        Kill current pane"),
        Line::from("  Prefix + z        Toggle zoom (fullscreen)"),
        Line::from("  Prefix + %        Split vertical"),
        Line::from("  Prefix + \"        Split horizontal"),
        Line::from("  Prefix + ?        Toggle this help"),
        Line::from("  Prefix + q        Quit"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Help — press ? to close ");

    frame.render_widget(Paragraph::new(help).block(block), popup_area);
}
