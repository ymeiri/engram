# MCP Setup Guide

Ready-made configurations for connecting engram to your AI coding agent.

## Claude Code

Add to `~/.claude.json`:

```json
{
  "mcpServers": {
    "engram": {
      "command": "/absolute/path/to/engram",
      "args": ["serve"]
    }
  }
}
```

For project-specific memory (isolated data per project):

```json
{
  "mcpServers": {
    "engram": {
      "command": "/absolute/path/to/engram",
      "args": ["serve", "--project", "my-project"]
    }
  }
}
```

## Cursor

Add to your Cursor MCP configuration (Settings > MCP):

```json
{
  "mcpServers": {
    "engram": {
      "command": "/absolute/path/to/engram",
      "args": ["serve"]
    }
  }
}
```

## Windsurf

Add to your Windsurf MCP configuration:

```json
{
  "mcpServers": {
    "engram": {
      "command": "/absolute/path/to/engram",
      "args": ["serve"]
    }
  }
}
```

## Multi-Session Setup

If you run multiple agents on the same project, engram automatically shares knowledge through a background daemon:

```
Agent 1 (Claude Code)  ──┐
                          ├── HTTP ──> engram daemon ──> SurrealDB
Agent 2 (Cursor)       ──┘
```

No extra configuration needed. `engram serve` auto-starts the daemon on first launch and subsequent agents connect automatically.

### Project Isolation

Each `--project` flag creates an isolated daemon with separate data:

```bash
# These two agents share memory:
engram serve --project backend

# This agent has its own isolated memory:
engram serve --project frontend
```

Daemon files are stored in `~/.engram/` (global) or `~/.engram/projects/<name>/` (per-project).

## Verify Setup

After configuring your agent, test the connection:

1. Ask your agent: *"Remember that this project uses OAuth for authentication."*
2. Start a **new session** (close and reopen the agent).
3. Ask: *"What authentication approach does this project use?"*

If engram is working, the agent recalls your OAuth decision from step 1.

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Agent doesn't see engram | Restart the MCP client after editing config |
| Permission denied | Use an absolute path to the binary; ensure it's executable |
| Server won't start | Run `engram serve` directly in terminal to see errors |
| Daemon port conflict | Check `engram daemon status`; stop with `engram daemon stop` |
| Need verbose logs | `RUST_LOG=debug engram serve` |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |
| `ENGRAM_DATA_DIR` | `~/.engram` | Override default data directory |
