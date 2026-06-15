# Roadmap

## v0.1 — Current (MVP)

- [x] Cross-platform TUI (Windows/macOS/Linux)
- [x] Multiple terminal panes with keyboard navigation
- [x] Prefix key system (Ctrl+A, tmux-style)
- [x] Create, rename, kill, zoom panes
- [x] Split view (vertical / horizontal)
- [x] MCP server: list, read, send, create terminals
- [x] IPC bridge (TUI ↔ MCP via local socket)
- [x] Configurable keybindings and shell (TOML)

## v0.2 — Planned

- [ ] Session persistence (restore panes on restart)
- [ ] Multiple named sessions (`holocron new my-session`)
- [ ] Scrollback navigation (`Prefix + [` to scroll, like tmux)
- [ ] Copy mode (select and copy terminal text)
- [x] Pane status monitor: RUNNING / IDLE / WAITING overlay per pane (`Prefix + s`)
- [ ] Pane synchronization (broadcast to all panes)

## v0.3 — Future

- [ ] Plugin system for custom MCP tools
- [ ] Layouts: save and restore pane arrangements
- [ ] Notification on pane activity (bell, visual)
- [ ] `holocron attach` to rejoin a detached session
- [ ] Multiple Claude sessions per terminal session

## Not planned

- Mouse support (keyboard-first is a core design principle)
- GUI / Electron version
- Cloud sessions
