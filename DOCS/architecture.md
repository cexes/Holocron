# Architecture

## Overview

`holocron` is a single Rust binary with three operating modes:

```
holocron              → TUI mode
holocron --headless   → headless mode (manager + IPC server, no UI)
holocron --mcp        → MCP bridge mode
```

## Why three modes?

The TUI uses stdin/stdout for rendering and keyboard input. MCP also uses stdin/stdout for its protocol. They cannot share the same process stdio.

**Solution:** Whoever owns the `TerminalManager` (the TUI, or a headless daemon) also holds an IPC server (local socket). The `--mcp` bridge is a thin process that reads MCP from stdio and forwards to whichever of those is running via IPC.

Headless mode exists so the bridge always has something to connect to even
if the user never opened the TUI: `run_bridge` checks whether the session
recorded on disk is actually reachable (`IpcClient::is_alive`), and if not
(no session file, or a stale one left by a crash) it spawns
`holocron --headless` detached from itself and waits for it to come up. The
headless daemon then runs independently — it is not a child of the bridge
process — so panes survive across multiple bridge connects/disconnects, the
same way a tmux server outlives any single client.

## Full flow

```
Claude Code
    │ stdin/stdout (MCP JSON-RPC)
    ▼
holocron --mcp          ← IpcClient connects to socket
    │                      (auto-spawns holocron --headless if nothing's alive)
    │ interprocess socket
    │ /tmp/holocron-{uuid}.sock (Linux/macOS)
    │ \\.\pipe\holocron-{uuid} (Windows)
    ▼
holocron (TUI)  OR  holocron --headless   ← IpcServer + TerminalManager
    │ portable-pty
    ▼
[ bash / zsh / cmd ]  [ claude code ]  [ npm test ]
```

## TUI event loop

```
tokio::select! {
    crossterm key event  → keybindings handler → App state → re-render
    PTY stdout bytes     → vt100 parser → Screen state → re-render
    terminal resize      → resize all PTYs → re-render
    IPC request          → TerminalManager action → IPC response
}
```

## Module map

```
src/
├── main.rs          CLI (clap), dispatch to TUI or MCP mode
├── app.rs           App struct: all state, TerminalManager, mode
├── config.rs        Config: keybindings, shell, theme (TOML)
├── error.rs         Custom error types (thiserror)
│
├── tui/
│   ├── mod.rs       run(): setup crossterm, run_loop(), teardown
│   ├── events.rs    Event enum (Key, PtyOutput, Resize, Tick)
│   ├── keybindings.rs  Prefix key state machine, Action enum
│   ├── ui.rs        Root render fn, composes all widgets
│   └── widgets/
│       ├── tab_bar.rs   Tab bar with pane labels
│       ├── pane.rs      PaneWidget: renders vt100::Screen cell by cell
│       └── status_bar.rs  Mode indicator, dimensions, hints
│
├── terminal/
│   ├── manager.rs   TerminalManager: create/kill/nav/list sessions
│   ├── session.rs   TerminalSession: PTY process + background reader
│   └── screen.rs    Screen: wraps vt100::Parser, exposes cell grid
│
├── ipc/
│   ├── protocol.rs  IpcRequest / IpcResponse (serde JSON over socket)
│   ├── server.rs    IpcServer: tokio listener, dispatches to manager
│   └── client.rs    IpcClient: connects to running TUI, sends requests
│
├── headless.rs      run(): manager + IPC server, no TUI, graceful shutdown
│
└── mcp/
    ├── mod.rs       run_bridge(): entry point for --mcp mode, auto-spawns
    │                headless sessions when nothing's alive
    ├── server.rs    DevTerminalMcpServer: rmcp tool_router, ServerHandler
    └── tools.rs     TerminalTools: delegates to IpcClient
```

## Session discovery

When the TUI or a headless daemon starts, it writes a session UUID to:
- `~/.local/share/holocron/session` (Linux/macOS)
- `%LOCALAPPDATA%\holocron\session` (Windows)

`holocron --mcp` reads this file to know which IPC socket to connect to,
and probes the socket (`IpcClient::is_alive`) before trusting it — a file
left behind by a crashed process doesn't mean the session is actually up.
If there's no live session, the bridge spawns `holocron --headless` and
polls until the new session file + socket are ready (up to ~5s) before
proceeding.

Pass `--session <uuid>` to override (useful for multiple sessions).

Known limitation: the TUI always starts a brand-new session rather than
attaching to an existing headless one, so if a headless daemon already has
panes open and you then open the TUI, the TUI won't see them (and a new
headless daemon would no longer be reachable as "the" session either). This
is fine for the common case — open the TUI first if you want to watch
Claude's panes live — but isn't a general multi-attach system.

## Cross-platform PTY

`portable-pty` abstracts:
- **Linux/macOS**: POSIX `openpty()` / `forkpty()`
- **Windows**: ConPTY (`CreatePseudoConsole`) — requires Windows 10 1809+

No WSL required on Windows.
