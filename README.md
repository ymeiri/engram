# engram

[![Build](https://github.com/ymeiri/engram/actions/workflows/ci.yml/badge.svg)](https://github.com/ymeiri/engram/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)

> Persistent memory for AI coding agents. One binary. Local and private.

Your AI coding assistant forgets everything between sessions — project conventions, architectural decisions, what you were working on, which tools work best. **engram remembers.**

**Without engram:**
> "What auth approach does this project use?"
> *"I don't have context about previous decisions..."*

**With engram:**
> "What auth approach does this project use?"
> *"This project uses OAuth. You chose it over API keys on Jan 12 for delegated partner access."*

engram is a local-first memory system purpose-built for AI coding agents. It connects via [MCP](https://modelcontextprotocol.io/) (Model Context Protocol) so any compatible agent — Claude Code, Cursor, Windsurf — gains persistent knowledge across sessions with zero cloud dependencies.

## Quick Start

### 1. Install

```bash
# From source (Rust 1.80+)
git clone https://github.com/ymeiri/engram.git
cd engram && cargo build --release
# Binary: ./target/release/engram
```

> Pre-built binaries coming soon. See [Releases](https://github.com/ymeiri/engram/releases).

### 2. Initialize and start

```bash
engram init      # Creates ~/.engram/data/
engram serve     # Starts MCP server
```

### 3. Connect to your agent

Add to your Claude Code config (`~/.claude.json`):

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

For Cursor or Windsurf, see the full [MCP Setup Guide](docs/MCP_SETUP.md).

### 4. Verify it works

Open your agent and try:

1. *"Remember that this project uses OAuth, not API keys."*
2. Start a **new session**.
3. *"What auth approach does this project use?"*

If engram is connected, your agent recalls the decision from step 1.

## What It Does

engram gives your coding agent MCP tools that let it read, search, and write to its own persistent memory. You code naturally; the agent decides when to store knowledge for the future.

| Capability | What it remembers | Example |
|-----------|-------------------|---------|
| **Project knowledge** | Repos, services, tools, concepts, conventions | "MCP" = Model Context Protocol |
| **Session memory** | Decisions, rationale, events across sessions | "We chose OAuth over API keys because..." |
| **Document recall** | Semantic search over indexed docs | Search "deploy" → relevant runbook section |
| **Tool memory** | Which tools succeed in which context | "For Go builds, use bzl (95% success)" |

<details>
<summary>Advanced capabilities</summary>

| Capability | What it does | Example |
|-----------|-------------|---------|
| **Session coordination** | Awareness of parallel agent sessions | "Session B is also editing auth-service" |
| **Work tracking** | Projects, tasks, PRs with entity connections | "PR #42 is blocked by #38" |
| **Knowledge management** | Doc registry with version detection and dedup | 6,318 .md files → one canonical registry |

</details>

## Why engram?

**Local and private.** Single Rust binary with an embedded database. No Docker, no Postgres, no API keys, no cloud. Your data stays in `~/.engram/`.

**Built for coding agents.** Not a chatbot memory system — engram understands repos, PRs, tasks, tool usage patterns, and multi-session coordination. It knows what a pull request *is*.

**MCP-native.** Not an adapter or wrapper. Built from the ground up for the Model Context Protocol, so it works with any MCP-compatible agent out of the box.

**Minimal setup.** `engram init && engram serve`, add one MCP config entry, and your agent has persistent memory. No schema definitions, no ontology configuration, no embedding API keys.

## How It Compares

|  | engram | General memory systems | Cloud memory APIs |
|--|--------|------------------------|-------------------|
| Built for coding agents | Yes | No | Partial |
| Local / no cloud required | Yes | Varies | No |
| MCP-native | Yes | No | No |
| Understands PRs, repos, tasks | Yes | No | Partial |
| Single binary, zero infra | Yes | No | No |

## Architecture

```
AI Coding Agent (Claude Code, Cursor, Windsurf, ...)
        │
        │ MCP (JSON-RPC over stdio)
        ▼
   engram server ── MCP tools
        │
        ▼
   SurrealDB (embedded) ── RocksDB + vector search
   ~/.engram/data/
```

engram is built as a Rust workspace of 6 crates:

```
engram-core     Domain types (zero external deps)
engram-store    SurrealDB adapter (relational + graph + vector)
engram-embed    Local ONNX embeddings via fastembed
engram-index    Business logic services
engram-mcp      MCP server (tools via JSON-RPC)
engram-cli      CLI application
```

### Multi-Session Support

Multiple agents can share the same knowledge base through a transparent daemon:

```bash
engram serve                        # Auto-starts daemon, runs as stdio proxy
engram serve --project myproject    # Project-isolated daemon and data
engram daemon status                # Check daemon health
```

## MCP Tools

Your agent gets these capabilities through MCP tools automatically. Here are the essentials:

| Tool | What it does |
|------|-------------|
| `entity` | Create/get/search/relate entities (repos, tools, services, concepts) |
| `entity_observe` | Add observations (facts, notes) about entities |
| `session` | Start/end/search coding sessions with event logging |
| `search` | Unified search across all knowledge layers |
| `docs` | Index and search documents with semantic similarity |
| `tool` | Log tool usage and get success-rate-based recommendations |
| `coord` | Register sessions, detect conflicts with parallel agents |
| `work_project` | Track projects with status and entity connections |
| `work_task` | Track tasks with priorities and dependencies |
| `work_pr` | Track pull requests across repos |

<details>
<summary>Full tool reference</summary>

See the [docs/](docs/) directory for complete tool parameters and examples.

</details>

## CLI Reference

engram also provides a full CLI for manual interaction:

```bash
# Entities
engram entity create web-ui -t repo -d "Main frontend repository"
engram entity relate web-ui -t depends_on api-service
engram entity observe MCP "Used by Claude Code for tool integration"

# Sessions
engram session start --agent claude-code --project my-app --goal "Add auth"
engram session log -t decision "Chose OAuth over API keys"
engram session end --summary "Implemented OAuth login flow"

# Documents
engram index ~/projects/docs/
engram search-docs "authentication flow"

# Tool intelligence
engram tool log bzl -o success -c "building go service"
engram tool recommend "building go service"

# Coordination
engram coord register <session-id> -a claude-code -p my-project
engram coord conflicts <session-id>

# Work management
engram work project create "Backend Rewrite" -s active
engram work task create <project-id> "Migrate auth service" -p high
```

## Development

```bash
cargo build                    # Build all crates
cargo test                     # Run tests
cargo clippy --all-targets -- -D warnings
cargo fmt
RUST_LOG=debug engram serve    # Verbose logging
```

## Technology

| Component | Choice | Why |
|-----------|--------|-----|
| **Language** | Rust | Single binary, no runtime deps, memory safety |
| **Database** | SurrealDB (embedded) | Multi-model: relational + graph + vector in one DB |
| **Protocol** | MCP | Agent-agnostic standard by Anthropic |
| **Embeddings** | fastembed (ONNX) | Local inference, no API calls, total privacy |

## Troubleshooting

- **Agent doesn't see engram?** Restart your MCP client after editing config.
- **Permission denied?** Make sure the binary path in your MCP config is absolute and executable.
- **Verify server starts:** Run `engram serve` directly in a terminal to check for errors.
- **Verbose logging:** `RUST_LOG=debug engram serve` for detailed output.

## License

Apache License 2.0. See [LICENSE](LICENSE).

## Acknowledgments

Named after [engrams](https://en.wikipedia.org/wiki/Engram_(neuropsychology)) — the hypothetical means by which memories are stored. Inspired by Vannevar Bush's [Memex](https://en.wikipedia.org/wiki/Memex) vision (1945). Built on the [Model Context Protocol](https://modelcontextprotocol.io/).
