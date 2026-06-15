use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use vt100::Screen as VtScreen;

/// Renders a vt100 Screen into a ratatui buffer cell by cell.
pub struct PaneWidget<'a> {
    pub screen: &'a VtScreen,
}

impl<'a> Widget for PaneWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for row in 0..area.height {
            for col in 0..area.width {
                let cell = self.screen.cell(row, col);
                let x = area.x + col;
                let y = area.y + row;

                if let Some(c) = cell {
                    let ch = c.contents();
                    let text = if ch.is_empty() { " ".to_string() } else { ch.to_string() };

                    let fg = vt_color_to_ratatui(c.fgcolor());
                    let bg = vt_color_to_ratatui(c.bgcolor());
                    let mut style = Style::default().fg(fg).bg(bg);

                    if c.bold() {
                        style = style.add_modifier(ratatui::style::Modifier::BOLD);
                    }
                    if c.italic() {
                        style = style.add_modifier(ratatui::style::Modifier::ITALIC);
                    }
                    if c.underline() {
                        style = style.add_modifier(ratatui::style::Modifier::UNDERLINED);
                    }

                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(&text);
                        cell.set_style(style);
                    }
                }
            }
        }
    }
}

fn vt_color_to_ratatui(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
