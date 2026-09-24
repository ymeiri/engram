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
registers or updates the user-scope Claude MCP server named `engram` with `--profile agent`. This
bounded profile exposes `orient`, `memory`, `repo`, `search`, and the narrow `harness`/`obligations`
actions needed by generated hooks. The default Claude Code hook set is low-overhead: session start,
compaction, and session end. In-session Engram usage is driven by generated Claude commands and
agent instructions, not per-prompt or per-tool hooks. Codex and Cursor use generated
skills/instructions; they still need MCP configured so the agent can call Engram tools.

By default, setup writes under your home directory. Use `--root .` from a repository if you want
project-local agent files.

## Claude Code

Guided setup:

```bash
engram setup --agent claude-code
engram setup --agent claude-code --write
```

To opt into the higher-overhead runtime enforcement profile that registers prompt/tool/final
response MCP hooks:

```bash
engram setup --agent claude-code --write --enforcement graduated
```

Use `--enforcement strict` only when you want Claude Code to keep blocking gated actions until the
required Engram obligations are resolved or explicitly skipped.

By default, Claude settings are written to `.claude/settings.json` when `--write` is approved. For
personal, gitignored settings:

```bash
engram setup --agent claude-code --write --settings-target settings.local.json
```

The Claude Code write step runs the equivalent of:

```bash
claude mcp add -s user engram -- /absolute/path/to/engram serve --profile agent
```

If a user-scope `engram` entry already exists, setup replaces it with the resolved current Engram
binary path. Restart Claude Code after setup so the hooks and MCP entry are loaded.

Manual MCP configuration in `~/.claude.json` is still supported for advanced setups:

```json
{
  "mcpServers": {
    "engram": {
      "command": "/absolute/path/to/engram",
      "args": ["serve", "--profile", "agent"]
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
      "args": ["serve", "--project", "my-project", "--profile", "agent"]
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
args = ["serve", "--profile", "agent"]
```

For project-specific memory:

```toml
[mcp_servers.engram]
command = "/absolute/path/to/engram"
args = ["serve", "--project", "my-project", "--profile", "agent"]
```

To compare the generated and installed harness files with the Engram MCP command the host is
configured to launch, run:

```bash
engram harness status --harness codex --root . --attest-host-configuration --json
engram harness status --harness claude-code --root . --attest-host-configuration --json
```

This check is provider-free and does not launch the configured Engram server. Codex attestation
uses `codex mcp get engram --json`, which resolves Codex configuration precedence. Claude Code's
native `mcp get` health-checks approved servers, so Engram instead inspects the known local,
project, and user config sources statically and reports
`resolved_configuration_verified: false`. Environment values and obviously sensitive argument
values are redacted. In both cases, `running_host_loaded_verified` and `live_runtime_verified`
remain false: use a fresh native-host trace and `engram contract --profile agent --verify-runtime`
for those separate claims.

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

The administrative `docs` MCP tool is available with `--profile full`. The bounded agent profile
uses the explicit CLI ingestion commands above.

## Verified Procedures

Generated Codex and Claude Code guidance tells agents to call
`memory(action="procedure_match", query=..., project=..., cwd=..., conditions={...})` when a task
or failure suggests a reusable workflow. Only procedures with matching scope, exact prerequisites,
freshness, and an unchanged verification receipt are returned. An abstention means the agent must
inspect the repository or tool's authoritative sources instead of applying a near match.

For stable file-backed prerequisites, procedure cards can map a prerequisite key through
`procedure.prerequisite_sources` to a scalar TOML `key_path` in a safe, regular, Git-tracked
`relative_path`. Engram resolves these sources from the active checkout during `procedure_match`;
the caller supplies `conditions` only for prerequisites without a source. Source observations are
value-redacted and caller text cannot override them. Unsafe, missing, untracked, oversized, invalid,
or non-scalar sources cause abstention.

Agents may record a successful workflow as a structured `kind="procedure"` candidate, but the
candidate stays `needs_review`. After inspecting a JSON receipt containing the exact `command`,
`exit_code`, captured `output`, and observed `conditions`, activate it explicitly:

```bash
engram memory verify-procedure <memory-id> \
  --receipt .engram/proofs/example.json \
  --expires-in-days 30 \
  --confirm
```

Engram stores only the receipt path and SHA-256, not its captured output, and rechecks the receipt
at retrieval time. When a repository-scoped receipt is inside a checkout with the same stable
repository identity, Engram stores it relative to the checkout root and resolves it against the
active checkout at retrieval time.

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

Daemon files are stored in `~/.engram/` globally or `~/.engram/projects/<name>/` per project. Set
`ENGRAM_HOME` on the Engram server process to move the complete state root—including daemon
metadata, bearer tokens, and project data—to an isolated absolute directory. This is useful for
controlled evaluations and portable installations; every client that manages the same daemon must
use the same value.

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
| `ENGRAM_HOME` | `~/.engram` | Override the complete Engram state root |
| `ENGRAM_EMBED_CACHE_DIR` | `~/.engram/cache/fastembed` | Override embedding model cache directory |
