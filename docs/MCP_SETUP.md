# MCP Setup Guide

Ready-made configurations for connecting engram to supported local AI coding agents.

> **0.2.x support scope:** guided setup supports Claude Code, Codex, and Cursor. Other
> MCP-compatible hosts may work with `engram serve`, but they are not part of the supported
> 0.2.x setup matrix.

## Recommended Setup

Run the setup wizard:

```bash
engram setup
```

For automation or docs, pass the agent explicitly. Setup is a dry-run unless `--write` is present:

```bash
engram setup --agent claude-code
engram setup --agent codex
engram setup --agent cursor
```

After reviewing the planned files:

```bash
engram setup --agent claude-code --write
engram setup --agent codex --write
engram setup --agent cursor --write
```

Claude Code exposes lifecycle hooks, so setup writes hook files, merges Claude settings, and
registers or updates the user-scope Claude MCP server named `engram`. Codex and Cursor use
generated skills/instructions; they still need MCP configured so the agent can call Engram tools.

By default, setup writes under your home directory. Use `--root .` from a repository if you want
project-local agent files.

## Claude Code

Guided setup:

```bash
engram setup --agent claude-code
engram setup --agent claude-code --write
```

By default, Claude settings are written to `.claude/settings.json` when `--write` is approved. For
personal, gitignored settings:

```bash
engram setup --agent claude-code --write --settings-target settings.local.json
```

The Claude Code write step runs the equivalent of:

```bash
claude mcp add -s user engram -- /absolute/path/to/engram serve
```

If a user-scope `engram` entry already exists, setup replaces it with the resolved current Engram
binary path. Restart Claude Code after setup so the hooks and MCP entry are loaded.

Manual MCP configuration in `~/.claude.json` is still supported for advanced setups:

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

For project-specific memory:

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

## Codex

Guided setup installs Codex skills under `.codex/skills/`:

```bash
engram setup --agent codex
engram setup --agent codex --write
```

Codex MCP configuration uses TOML. Add Engram to the Codex config:

```toml
[mcp_servers.engram]
command = "/absolute/path/to/engram"
args = ["serve"]
```

For project-specific memory:

```toml
[mcp_servers.engram]
command = "/absolute/path/to/engram"
args = ["serve", "--project", "my-project"]
```

## Cursor

Guided setup installs Cursor Agent skills under `.cursor/skills/`:

```bash
engram setup --agent cursor
engram setup --agent cursor --write
```

Add Engram to Cursor's MCP configuration:

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

## First Orient

After setup, restart the agent and ask it to run `orient` for the current project. If no documents
are indexed yet, Engram will tell the agent to ask which existing docs, runbooks, notes, ADRs, or
knowledge folders should be ingested.

Preview ingestion before writing:

```bash
engram index --plan ./docs
engram index ./docs --recursive
```

Agents can also use the MCP `docs` tool with `action="plan"` before indexing approved paths.

## Multi-Session Setup

If you run multiple agents on the same project, engram shares knowledge through a background daemon:

```
Agent 1 (Claude Code)  ──┐
                          ├── HTTP ──> engram daemon ──> SurrealDB
Agent 2 (Codex/Cursor) ──┘
```

No extra daemon configuration is required. `engram serve` auto-starts the daemon on first launch and
subsequent agents connect automatically.

### Project Isolation

Each `--project` flag creates an isolated daemon with separate data:

```bash
# These agents share memory:
engram serve --project backend

# This agent has its own isolated memory:
engram serve --project frontend
```

Daemon files are stored in `~/.engram/` globally or `~/.engram/projects/<name>/` per project.

## Verify Setup

After configuring your agent:

1. Ask your agent to run `orient` for the current project.
2. Ask it to remember a small project decision.
3. Start a new session.
4. Ask it to recall that decision.

If engram is working, the agent can use its MCP tools to recall the stored decision.

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Agent does not see engram | Restart the MCP client after editing config |
| Permission denied | Use an absolute path to the binary and ensure it is executable |
| Server will not start | Run `engram serve` directly in a terminal to see errors |
| Daemon port conflict | Check `engram daemon status`; stop with `engram daemon stop` |
| First semantic call pauses | Run `engram warmup embeddings` to prepare the local model cache |
| Need verbose logs | `RUST_LOG=debug engram serve` |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |
| `ENGRAM_DATA_DIR` | `~/.engram` | Override default data directory |
| `ENGRAM_EMBED_CACHE_DIR` | `~/.engram/cache/fastembed` | Override embedding model cache directory |
