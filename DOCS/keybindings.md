# Keybindings Reference

Prefix key: **Ctrl+A** (default, configurable in `~/.config/holocron/config.toml`)

## Pane management

| Keys | Action |
|---|---|
| `Prefix + c` | Create new terminal pane |
| `Prefix + x` | Kill current pane |
| `Prefix + ,` | Rename current pane (Enter to confirm, Esc to cancel) |
| `Prefix + z` | Toggle zoom (fullscreen current pane) |
| `Prefix + m` | Toggle master pane (see below) |
| `Prefix + %` | Split view — vertical (side by side) |
| `Prefix + "` | Split view — horizontal (top / bottom) |

## Navigation

| Keys | Action |
|---|---|
| `Prefix + n` | Next pane |
| `Prefix + p` | Previous pane |
| `Prefix + h` | Navigate left |
| `Prefix + j` | Navigate down |
| `Prefix + k` | Navigate up |
| `Prefix + l` | Navigate right |
| `Prefix + 0-9` | Jump to pane by index |

## General

| Keys | Action |
|---|---|
| `Prefix + ?` | Toggle help overlay |
| `Prefix + q` | Quit |

## Master pane

`Prefix + m` marks the current pane as **master** (tagged `[MASTER]` in the tab bar, pane border, and status bar). While the master pane is the active one, creating a new terminal — whether via `Prefix + c` or the MCP `create_terminal` tool — no longer steals focus; the new pane spawns in the background and the master stays in view. Navigate away manually (`Prefix + n/p/h/j/k/l` or jump-to-index) and the next terminal creation will focus normally again. Press `Prefix + m` again on the master pane to unset it; killing the master pane also clears the flag.

## Changing the prefix key

Edit `~/.config/holocron/config.toml`:

```toml
[prefix_key]
key = "b"     # change to Ctrl+B like tmux
ctrl = true
```
