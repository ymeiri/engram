# engram

[![Build](https://github.com/ymeiri/engram/actions/workflows/ci.yml/badge.svg)](https://github.com/ymeiri/engram/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)

> Local-first persistent memory for AI coding agents via MCP. One binary. Private by default.

Your AI coding assistant forgets everything between sessions — project conventions, architectural decisions, what you were working on, which tools work best. **engram remembers.**

**Without engram:**
> "What auth approach does this project use?"
> *"I don't have context about previous decisions..."*

**With engram:**
> "What auth approach does this project use?"
> *"This project uses OAuth. You chose it over API keys on Jan 12 for delegated partner access."*

engram is a local-first memory system and knowledge base purpose-built for AI coding agents. It
connects via [MCP](https://modelcontextprotocol.io/) (Model Context Protocol) so Claude Code, Codex,
Cursor, and other compatible agents can use semantic search, session memory, document recall, and
tool history across sessions without embedding API calls.

> **Beta scope:** v0.2.0-beta.2 supports guided local setup for Claude Code, Codex,
> and Cursor. Other MCP-compatible hosts may work with the same server, but they are not part of
> this beta support matrix.
>
> **First-run cache note:** semantic search uses a local ONNX embedding model. First use may download
> `all-MiniLM-L6-v2` from Hugging Face into the local cache unless you pre-warm it with
> `engram warmup embeddings`.

## Quick Start

### 1. Install

Homebrew is the preferred path for macOS Apple Silicon:

```bash
brew tap ymeiri/engram
brew install engram
engram --version
```

The Homebrew formula is scoped to Apple Silicon macOS. Other platforms can install from a release
artifact or build from source.

From a published beta release artifact:

```bash
version=0.2.0-beta.2
archive="engram-${version}-aarch64-apple-darwin.tar.gz"

curl -LO "https://github.com/ymeiri/engram/releases/download/v${version}/${archive}"
curl -LO "https://github.com/ymeiri/engram/releases/download/v${version}/${archive}.sha256"
shasum -a 256 -c "${archive}.sha256"
tar -xzf "${archive}"

mkdir -p "$HOME/.local/bin"
install -m 755 "engram-${version}-aarch64-apple-darwin/engram" "$HOME/.local/bin/engram"
export PATH="$HOME/.local/bin:$PATH"
engram --version
```

See [Releases](https://github.com/ymeiri/engram/releases) for published artifacts.

From source with Rust 1.80+:

```bash
git clone https://github.com/ymeiri/engram.git
cd engram
cargo build --release

mkdir -p "$HOME/.local/bin"
install -m 755 ./target/release/engram "$HOME/.local/bin/engram"
export PATH="$HOME/.local/bin:$PATH"
engram --version
```

### 2. Warm up embeddings for offline use

```bash
engram warmup embeddings
```

The warmup command prepares the local model cache and confirms embeddings work before you connect an
agent. This is optional when you are online, but recommended before offline or sandboxed use.

### 3. Initialize storage

```bash
engram init      # Creates ~/.engram/data/
```

`engram init` is intentionally script-safe and non-interactive. Use `engram setup` for the human
agent setup flow.

### 4. Configure your agent

Run the guided setup wizard:

```bash
engram setup
```

For non-interactive setup, select the agent explicitly. The default is a dry-run:

```bash
engram setup --agent claude-code
engram setup --agent codex
engram setup --agent cursor
```

After reviewing the planned files, write the agent adapter and hook configuration:

```bash
engram setup --agent codex --write
```

Setup supports Claude Code, Codex, and Cursor in this beta. It uses the same harness adapter
installer as `engram harness install`, keeps writes approval-gated, and does not overwrite
user-owned files unless you opt into the lower-level harness command.

By default, setup writes under your home directory. Use `--root .` from a repository if you want
project-local agent files.

Manual MCP configuration is also supported. For example, Claude Code can use:

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

For Codex and Cursor examples, see the full [MCP Setup Guide](docs/MCP_SETUP.md).

### 5. First orient and knowledge ingestion

Restart your agent after writing setup files. In the agent, ask:

> Run orient for this project.

On first orient, if no documents are indexed yet, Engram tells the agent to ask which existing
project docs, runbooks, notes, ADRs, or knowledge folders you want it to ingest. Preview ingestion
before writing:

```bash
engram index --plan ./docs
engram index ./docs --recursive
```

You can also let the agent use the MCP `docs` tool to plan and index approved paths.

### 6. Verify memory works

Open your agent and try:

1. *"Remember that this project uses OAuth, not API keys."*
2. Start a **new session**.
3. *"What auth approach does this project use?"*

If engram is connected, your agent recalls the decision from step 1.

## What To Expect

- `engram init` only prepares local storage. It does not change agent settings.
- `engram setup` shows a dry-run plan by default. Use `--write` only after reviewing the planned
  adapter and hook files.
- `engram serve` is the MCP server command your agent runs. In default mode it starts or connects
  to a local daemon so multiple sessions can share memory safely.
- First `orient` gives the agent a project context packet. If the document index is empty, it asks
  the user which existing knowledge should be ingested before indexing anything.
- Indexed documents, memory items, sessions, tool history, and work context are stored locally under
  `~/.engram/` unless you use a project-specific or explicit data directory.

## Uninstall

Stop Engram before removing the binary:

```bash
engram daemon stop 2>/dev/null || true
```

If you used project-specific daemons, stop each project daemon first:

```bash
engram daemon stop --project my-project 2>/dev/null || true
```

Remove generated agent setup files from the same root where you ran `engram setup --write`.
For the default home-directory setup:

```bash
# Codex
rm -rf ~/.codex/skills/engram-memory-session
rm -rf ~/.codex/skills/engram-resume-session
rm -f ~/AGENTS.engram.md

# Cursor
rm -rf ~/.cursor/skills/engram-memory-session
rm -rf ~/.cursor/skills/engram-resume-session
rm -rf ~/.cursor/skills/engram-end-session
```

For Claude Code project setup, remove the generated files from that project root:

```bash
rm -f .claude/commands/engram-memory-session.md
rm -f .claude/commands/engram-resume-session.md
rm -f .claude/commands/engram-end-session.md
rm -f .claude/hooks/engram-session-start.sh
rm -f .claude/hooks/engram-stop-nudge.sh
rm -f .claude/hooks/engram-session-end.sh
rm -f .claude/engram-settings-snippet.json
rm -f AGENTS.engram.md
```

If you added Engram manually to Claude Code, Codex, or Cursor MCP settings, remove the `engram`
MCP server entry from that agent's config file.

Then uninstall the binary:

```bash
# Homebrew
brew uninstall ymeiri/engram/engram
brew untap ymeiri/engram

# Manual install
rm -f ~/.local/bin/engram
```

By default, keep `~/.engram/` so local memory can be restored later. To remove local memory without
deleting it immediately, move it aside:

```bash
mv ~/.engram ~/.engram.backup.$(date +%Y%m%d%H%M%S)
```

Permanent data deletion is destructive:

```bash
rm -rf ~/.engram
```

Verify removal:

```bash
command -v engram || echo "engram binary removed"
```

## What It Does

engram gives your coding agent MCP tools that let it read, search, and write to its own persistent memory. You code naturally; the agent decides when to store knowledge for the future.

| Capability | What it remembers | Example |
|-----------|-------------------|---------|
| **Project knowledge** | Repos, services, tools, concepts, conventions | "Auth owns login, billing consumes its JWT claims" |
| **Session memory** | Decisions, rationale, events across sessions | "OAuth was chosen for delegated partner access" |
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

**Local and private.** Single Rust binary with an embedded database. No Docker, no Postgres, and
no embedding API keys. Your data stays in `~/.engram/`; first embedding use may download the local
ONNX model unless you pre-warm the cache.

**Built for coding agents.** Not a chatbot memory system — engram understands repos, PRs, tasks, tool usage patterns, and multi-session coordination. It knows what a pull request *is*.

**MCP-native.** Not an adapter or wrapper. Built from the ground up for the Model Context
Protocol. Guided beta setup covers Claude Code, Codex, and Cursor.

**Minimal setup.** Install the binary, run `engram init`, then use `engram setup` to configure a
supported local agent. No schema definitions, no ontology configuration, no embedding API keys.

## Where It Fits

engram is a good fit when you want a local memory layer for AI coding agents that work across
multiple sessions, repositories, and tools.

| Use engram when... | Consider something else when... |
|--------------------|---------------------------------|
| You want Claude Code, Codex, or Cursor to remember project decisions across sessions | You need a hosted company knowledge base for humans |
| You want memory stored locally on your machine | You need managed cloud retention, access control, and audit logs |
| You want MCP tools for entities, sessions, documents, work items, and coordination | You only need a generic vector database or search API |
| You want a single local binary with no database service to operate | You already have a production knowledge platform your agents can query directly |

## Architecture

```
AI Coding Agent (Claude Code, Codex, Cursor, ...)
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
cargo build --locked           # Build all crates with the committed dependency graph
cargo test --locked            # Run tests with the committed dependency graph
cargo clippy --locked --all-targets -- -D warnings
cargo fmt
./scripts/local-ci.sh          # Run the CI-equivalent release gate locally
./scripts/package-release.sh   # Build a local release tarball, manifest, and checksum
./scripts/package-install-smoke.sh  # Verify manifest, install, and packaged /health
./scripts/render-homebrew-formula.sh  # Render tap-ready Homebrew formula from the tarball
./scripts/beta-release-gate-report.sh  # Collect PR/CI/package gate evidence and release-review state
./scripts/verify-published-release-install.sh  # Verify downloaded release assets after publishing
./scripts/verify-hosted-ci-prestep-blocker.sh  # Verify hosted pre-step CI blocker; use --event push for main runs
./scripts/native-claude-gate-preflight.sh  # Check native Claude proof readiness; use --expected-branch for non-main checks
RUST_LOG=debug engram serve    # Verbose logging
```

Release package builds fail on tracked working-tree or index changes by default. Set
`ALLOW_TRACKED_CHANGES=1` only for local development rehearsals.

Embedding model files are cached under `~/.engram/cache/fastembed` by default. Set
`ENGRAM_EMBED_CACHE_DIR` to use a pre-warmed or shared cache; upstream `FASTEMBED_CACHE_DIR` and
`HF_HOME` are also honored. If the cache is cold, `engram serve`, `engram index`, `engram search`,
and other semantic paths may need network access for the one-time model download.

## Technology

| Component | Choice | Why |
|-----------|--------|-----|
| **Language** | Rust | Single binary, no runtime deps, memory safety |
| **Database** | SurrealDB (embedded) | Multi-model: relational + graph + vector in one DB |
| **Protocol** | MCP | Agent-agnostic standard by Anthropic |
| **Embeddings** | fastembed (ONNX) | Local inference; no embedding API calls after cache warmup |

## Troubleshooting

- **Agent doesn't see engram?** Restart your MCP client after editing config.
- **Permission denied?** Make sure the binary path in your MCP config is absolute and executable.
- **Verify server starts:** Run `engram serve` directly in a terminal to check for errors.
- **Verbose logging:** `RUST_LOG=debug engram serve` for detailed output.
- **First run pauses at embeddings:** Run `engram warmup embeddings` in a terminal. It prints the
  cache path and explains whether a one-time Hugging Face model download may occur.
- **Database lock conflict:** Run `engram daemon status` to check whether another Engram daemon owns
  the RocksDB store. If no Engram process is using the store and a crash left a stale lock, remove
  the reported `LOCK` file and retry. Check `engram daemon logs` for the original startup error.

## Open Source

- Contributions: see [CONTRIBUTING.md](CONTRIBUTING.md) for local development, tests, and PR
  expectations.
- Security: see [SECURITY.md](SECURITY.md) for vulnerability reporting and supported beta
  versions. Do not open public issues for private security reports.
- Conduct: see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
- License: Apache-2.0.

## License

Apache License 2.0. See [LICENSE](LICENSE).

## Acknowledgments

Named after [engrams](https://en.wikipedia.org/wiki/Engram_(neuropsychology)) — the hypothetical means by which memories are stored. Inspired by Vannevar Bush's [Memex](https://en.wikipedia.org/wiki/Memex) vision (1945). Built on the [Model Context Protocol](https://modelcontextprotocol.io/).
