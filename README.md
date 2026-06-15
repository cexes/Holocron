# holocron

Keyboard-driven TUI terminal multiplexer with MCP integration for AI orchestration — runs inside your terminal, cross-platform, no mouse required.

---

## The problem it solves

When working with multiple AI agents at the same time (Claude Code, Codex, OpenCode), each one runs in a separate terminal window. You constantly switch between them to check what each agent is doing, with no unified view and no way to coordinate them from a single place.

**Holocron solves this on two fronts:**

1. **Visual multiplexer** — all terminals in one place, keyboard navigation, no mouse
2. **MCP bridge** — an orchestrator Claude can read and write to any pane on demand

You talk to one Claude and it acts on the others. You stay in control.

---

## What it looks like in practice

```
┌─────────────────────────────────────────────────────────────────┐
│  TERMINAL 1 — holocron running                                  │
│                                                                 │
│  ┌────────────────────┬──────────────────────────────────────┐  │
│  │ 0:claude-code      │ 1:opencode                           │  │
│  │                    │                                      │  │
│  │  > implementing    │  > reviewing frontend...             │  │
│  │    auth module...  │                                      │  │
│  │                    │                                      │  │
│  └────────────────────┴──────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 2:tests                                                  │   │
│  │   ✓ 42 passed  ✗ 1 failed                               │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
└──────────────────┬──────────────────────────────────────────────┘
                   │ IPC socket
                   │
┌──────────────────▼──────────────────────────────────────────────┐
│  TERMINAL 2 — you talking to the orchestrator Claude            │
│                                                                 │
│  you:    "what is happening across all terminals?"              │
│  claude: [list_terminals] [read_terminal 0] [read_terminal 1]   │
│          "Terminal 0: Claude Code implementing auth.            │
│           Terminal 1: OpenCode reviewing frontend.              │
│           Terminal 2: 1 test failing in user.spec.ts"           │
│                                                                 │
│  you:    "tell terminal 0 to focus on the failing test"         │
│  claude: [send_command 0 "focus on the failing user.spec.ts"]   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Installation

**Prerequisites:** Rust 1.75+ ([rustup.rs](https://rustup.rs))

```bash
git clone <repo>
cd holocron
cargo install --path .
```

The `holocron` binary is then available globally in your PATH.

---

## Running

```bash
holocron          # open the TUI
holocron --mcp    # MCP bridge mode (called automatically by Claude)
```

Once open, create your first terminal with `Ctrl+A` then `c`.

---

## Keybindings

Default prefix: **Ctrl+A** (configurable)

| Keys | Action |
|------|--------|
| `Prefix + c` | Create new terminal |
| `Prefix + n / p` | Next / previous pane |
| `Prefix + h/j/k/l` | Navigate panes (vim-style) |
| `Prefix + 0-9` | Jump to pane by number |
| `Prefix + ,` | Rename current pane |
| `Prefix + x` | Kill current pane |
| `Prefix + z` | Zoom (fullscreen) current pane |
| `Prefix + %` | Split vertical |
| `Prefix + "` | Split horizontal |
| `Prefix + ?` | Help overlay |
| `Prefix + q` | Quit |

---

## Claude MCP Integration

Holocron is already registered in `~/.claude.json`. To use it:

1. Open `holocron` in one terminal
2. Open `claude` in another terminal
3. Run `/mcp` to confirm `holocron` appears in the list
4. Ask Claude to interact with your terminals

### Available MCP tools

| Tool | Description |
|------|-------------|
| `list_terminals` | List all open panes |
| `read_terminal` | Read the last N lines of output from a pane |
| `send_command` | Send a command to a pane's stdin |
| `create_terminal` | Open a new pane |
| `get_terminal_info` | Get details of a pane (id, label, dimensions) |

---

## How it works

Holocron uses two modes in a single binary to keep TUI stdio separate from MCP stdio:

```
Claude Code
    │ stdio (MCP protocol)
    ▼
holocron --mcp        ← bridge: translates MCP → IPC
    │ local socket (/tmp/holocron-{session}.sock)
    ▼
holocron              ← TUI: manages panes and PTYs
    │ portable-pty (ConPTY on Windows, Unix PTY on Mac/Linux)
    ▼
[ pane 0 ]  [ pane 1 ]  [ pane 2 ]
```

---

## Configuration

File: `~/.config/holocron/config.toml`

```toml
[prefix_key]
key = "a"
ctrl = true

shell = "/bin/zsh"
scrollback_lines = 10000

[theme]
active_pane_border = "cyan"
inactive_pane_border = "gray"
```

---

## Stack

| Layer | Crate | Reason |
|-------|-------|--------|
| TUI | `ratatui` + `crossterm` | crossterm handles Windows ConPTY natively |
| Cross-platform PTY | `portable-pty` | ConPTY on Windows, Unix PTY on Mac/Linux |
| ANSI/VT100 | `vt100` | Parses escape codes from child processes |
| Async runtime | `tokio` | PTYs + IPC + events concurrently |
| MCP protocol | `rmcp` | Rust MCP SDK, stdio transport |
| IPC | `interprocess` | Unix sockets + Windows named pipes |

---

## License

MIT
