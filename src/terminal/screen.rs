/// Wraps vt100::Parser and exposes a renderable cell grid.
pub struct Screen {
    parser: vt100::Parser,
}

impl Screen {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            parser: vt100::Parser::new(rows, cols, 10_000),
        }
    }

    pub fn process(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.parser.set_size(rows, cols);
    }

    pub fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }

    /// Returns plain-text lines for MCP read_terminal tool.
    pub fn text_lines(&self, last_n: usize) -> Vec<String> {
        let screen = self.parser.screen();
        let (rows, cols) = screen.size();
        let start = (rows as usize).saturating_sub(last_n);
        (start..rows as usize)
            .map(|r| {
                let mut line = String::new();
                for c in 0..cols {
                    let contents = screen
                        .cell(r as u16, c)
                        .map(|cell| cell.contents().to_string())
                        .unwrap_or_else(|| " ".to_string());
                    if contents.is_empty() {
                        line.push(' ');
                    } else {
                        line.push_str(&contents);
                    }
                }
                line.trim_end().to_owned()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processes_plain_text() {
        let mut screen = Screen::new(24, 80);
        screen.process(b"hello world");
        // text lands on row 0; fetch all 24 rows to include it
        let lines = screen.text_lines(24);
        assert!(lines.iter().any(|l| l.contains("hello world")));
    }

    #[test]
    fn processes_ansi_colors() {
        let mut screen = Screen::new(24, 80);
        screen.process(b"\x1b[31mred text\x1b[0m");
        let lines = screen.text_lines(24);
        assert!(lines.iter().any(|l| l.contains("red text")));
    }
}
