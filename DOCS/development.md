# Development Guide

## Prerequisites

- Rust 1.75+ (`rustup update stable`)
- On Windows: Windows 10 version 1809+ (for ConPTY support)

## Build

```bash
cargo build           # debug build
cargo build --release # release build
```

## Run

```bash
cargo run             # TUI mode
cargo run -- --mcp   # MCP bridge mode
```

## Test

```bash
cargo test            # all tests
cargo test terminal   # terminal module only
```

## Lint

```bash
cargo clippy          # lint
cargo fmt             # format
cargo fmt --check     # check formatting (CI)
```

## Install locally

```bash
cargo install --path .
holocron          # now available in $PATH
```

## Project structure

See [architecture.md](architecture.md) for the full module map and data flow.

## Adding a new MCP tool

1. Add the request/response variant to `src/ipc/protocol.rs`
2. Handle it in `src/ipc/server.rs` `handle_request()`
3. Add a method to `src/mcp/tools.rs` `TerminalTools`
4. Add the `#[tool]` method in `src/mcp/server.rs`

## Logging

Logs go to:
- Linux/macOS: `~/.local/share/holocron/holocron.log`
- Windows: `%LOCALAPPDATA%\holocron\holocron.log`

Set `RUST_LOG=debug` to increase verbosity.

## IPC socket location

- Linux/macOS: `$XDG_RUNTIME_DIR/holocron-{uuid}.sock` (falls back to `~/.local/share/holocron/`)
- Windows: `\\.\pipe\holocron-{uuid}`

## Release checklist

- [ ] `cargo test` passes
- [ ] `cargo clippy` clean
- [ ] `cargo fmt --check` passes
- [ ] Update version in `Cargo.toml`
- [ ] Update `DOCS/roadmap.md`
- [ ] Build release binaries for all three platforms
