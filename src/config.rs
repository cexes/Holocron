use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub prefix_key: PrefixKey,
    pub shell: Option<String>,
    pub scrollback_lines: usize,
    pub theme: Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefixKey {
    pub key: char,
    pub ctrl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub active_pane_border: String,
    pub inactive_pane_border: String,
    pub tab_active: String,
    pub tab_inactive: String,
    pub status_bar: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix_key: PrefixKey { key: 'a', ctrl: true },
            shell: None,
            scrollback_lines: 10_000,
            theme: Theme::default(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            active_pane_border: "cyan".into(),
            inactive_pane_border: "gray".into(),
            tab_active: "cyan bold".into(),
            tab_inactive: "gray".into(),
            status_bar: "dark_gray".into(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("reading config at {}", path.display()))?;
        toml::from_str(&raw).with_context(|| "parsing config TOML")
    }

    pub fn default_shell() -> String {
        std::env::var("SHELL")
            .or_else(|_| std::env::var("COMSPEC"))
            .unwrap_or_else(|_| {
                if cfg!(windows) {
                    "cmd.exe".into()
                } else {
                    "/bin/sh".into()
                }
            })
    }

    pub fn shell_command(&self) -> String {
        self.shell.clone().unwrap_or_else(Self::default_shell)
    }

    pub fn matches_prefix(&self, modifiers: KeyModifiers, code: KeyCode) -> bool {
        let key_match = match code {
            KeyCode::Char(c) => c == self.prefix_key.key,
            _ => false,
        };
        let mod_match = if self.prefix_key.ctrl {
            modifiers == KeyModifiers::CONTROL
        } else {
            modifiers == KeyModifiers::NONE
        };
        key_match && mod_match
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("holocron")
        .join("config.toml")
}
