# MCP Integration

`holocron` exposes your terminal panes to Claude via the Model Context Protocol (MCP).
Claude can list, read, and write to terminals **on demand** — it never acts autonomously.

## Setup

1. Install `holocron` and make sure it's in your PATH.

2. Add to Claude Code's MCP config (`~/.claude/mcp.json`):

```json
{
  "mcpServers": {
    "holocron": {
      "command": "holocron",
      "args": ["--mcp"]
    }
  }
}
```

3. Start `holocron` in a terminal. Open your panes.

4. In Claude Code, you can now ask:
   - *"list my terminals"*
   - *"what is terminal 2 doing?"*
   - *"send 'npm test' to terminal 3"*

## How it works

```
Claude Code (MCP client)
    │ stdio (JSON-RPC)
    ▼
holocron --mcp  ← spawned by Claude Code
    │ IPC socket
    ▼
holocron (TUI)  ← already running
    │
    ▼
[ your terminal panes ]
```

Claude Code spawns `holocron --mcp` as a subprocess. That process connects to the running TUI instance via a local socket and bridges requests.

## Available tools

### `list_terminals`
Returns all active panes with their IDs, labels, and which is active.

**No arguments.**

Example response:
```json
[
  { "id": "550e8400-...", "label": "backend", "index": 0, "is_active": false },
  { "id": "6ba7b810-...", "label": "claude-code", "index": 1, "is_active": true }
]
```

### `read_terminal`
Returns the last N lines of visible output from a pane.

| Argument | Type | Required | Description |
|---|---|---|---|
| `id` | string (UUID) | yes | Terminal ID from `list_terminals` |
| `lines` | number | no | Lines to return (default: 50) |

### `send_command`
Sends text to a pane's stdin (Enter is appended automatically).

| Argument | Type | Required | Description |
|---|---|---|---|
| `id` | string (UUID) | yes | Terminal ID |
| `command` | string | yes | Command text |

### `get_terminal_info`
Returns label, dimensions (cols×rows), and ID for a single pane.

| Argument | Type | Required | Description |
|---|---|---|---|
| `id` | string (UUID) | yes | Terminal ID |

### `create_terminal`
Creates a new pane in the running TUI session.

| Argument | Type | Required | Description |
|---|---|---|---|
| `label` | string | yes | Human-readable label |
| `command` | string | no | Shell command (default: `$SHELL`) |

## Cost model

Claude only reads terminals when **you ask**. There is no background polling.
A typical query (list + read one terminal) costs ~1-2k tokens — a few cents per day at heavy usage.
