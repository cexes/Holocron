# Architecture

## Overview

`holocron` is a single Rust binary with two operating modes:

```
holocron            → TUI mode
holocron --mcp      → MCP bridge mode
```

## Why two modes?

The TUI uses stdin/stdout for rendering and keyboard input. MCP also uses stdin/stdout for its protocol. They cannot share the same process stdio.

**Solution:** The TUI process holds an IPC server (local socket). The `--mcp` bridge is a thin process that reads MCP from stdio and forwards to the TUI via IPC.

## Full flow

```
Claude Code
    │ stdin/stdout (MCP JSON-RPC)
    ▼
holocron --mcp          ← IpcClient connects to socket
    │ interprocess socket
    │ /tmp/holocron-{uuid}.sock (Linux/macOS)
    │ \\.\pipe\holocron-{uuid} (Windows)
    ▼
holocron (TUI)           ← IpcServer + TerminalManager
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
└── mcp/
    ├── mod.rs       run_bridge(): entry point for --mcp mode
    ├── server.rs    DevTerminalMcpServer: rmcp tool_router, ServerHandler
    └── tools.rs     TerminalTools: delegates to IpcClient
```

## Session discovery

When the TUI starts, it writes a session UUID to:
- `~/.local/share/holocron/session` (Linux/macOS)
- `%LOCALAPPDATA%\holocron\session` (Windows)

`holocron --mcp` reads this file to know which IPC socket to connect to.

Pass `--session <uuid>` to override (useful for multiple sessions).

## Cross-platform PTY

`portable-pty` abstracts:
- **Linux/macOS**: POSIX `openpty()` / `forkpty()`
- **Windows**: ConPTY (`CreatePseudoConsole`) — requires Windows 10 1809+

No WSL required on Windows.
