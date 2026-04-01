# engram

> Your AI coding assistant's persistent memory

**engram** is a Personal Knowledge Augmentation System (PKAS) — a local-first, agent-agnostic knowledge layer that provides AI coding agents with persistent context, memory, and intelligence.

## The Problem

AI coding agents suffer from:

1. **Context Loss** ("The Hotdog Problem"): Agents don't know your project-specific terms, repos, tools, or jargon. Every session starts from zero.

2. **Session Amnesia**: After compaction or new sessions, decisions and rationale are lost. There's no "git log" for agent reasoning.

3. **Repeated Searches**: Same documentation searches, same lookups, same tool confusion — session after session.

4. **Siloed Knowledge**: Your custom commands work in one agent but not another. Knowledge is agent-specific.

## The Solution

engram provides a **6-layer knowledge system** accessible via the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/):

| Layer | Purpose | Status | Examples |
|-------|---------|--------|----------|
| **1. Entity Knowledge** | Your repos, tools, terminology | ✅ Complete | "MCP" = Model Context Protocol |
| **2. Session History** | What happened, decisions, rationale | ✅ Complete | "We chose OAuth over API keys because..." |
| **3. Document Knowledge** | Chunked docs, semantic search | ✅ Complete | Search "orchestrator" → get relevant section |
| **4. Tool Intelligence** | Tool usage patterns, recommendations | ✅ Complete | "For Go builds, use bzl (95% success rate)" |
| **5. Session Coordination** | Parallel session awareness | ✅ Complete | "Session B is also touching auth-service" |
| **6. Document Intelligence** | Canonical doc resolver, dedupe | ✅ Complete | 6,318 .md files → one canonical registry |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     AI Coding Agents                            │
│              (Claude Code, Cursor, Gemini CLI, etc.)            │
└─────────────────────────────┬───────────────────────────────────┘
                              │ MCP Protocol (JSON-RPC over stdio)
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      engram MCP Server                          │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │   Layer 1   │  │   Layer 2   │  │        Layer 3          │ │
│  │  Entities   │  │  Sessions   │  │       Documents         │ │
│  │     ✅      │  │     ✅      │  │          ✅             │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │   Layer 4   │  │   Layer 5   │  │        Layer 6          │ │
│  │    Tools    │  │   Coord     │  │     Doc Intelligence    │ │
│  │     ✅      │  │     ✅      │  │          ✅             │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   SurrealDB (Embedded)                          │
│              Relational + Graph + Vector Search                 │
│                   ~/.engram/data/                               │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Installation

```bash
# Build from source
git clone <repo-url>
cd engram
cargo build --release

# The binary will be at target/release/engram
```

### Initialize

```bash
# Initialize the database (creates ~/.engram/data/)
engram init

# Initialize the knowledge system (creates ~/.engram/knowledge/)
engram knowledge init
```

### Configure MCP Client

Add to your Claude Code config (`~/.config/claude-code/settings.json`):

```json
{
  "mcpServers": {
    "engram": {
      "command": "/path/to/engram",
      "args": ["serve"]
    }
  }
}
```

Or for Cursor, add to your MCP configuration file.

### Start the MCP Server

```bash
# Start the MCP server (for AI agents)
engram serve

# Or with in-memory storage for testing
engram serve --memory
```

## CLI Reference

### Layer 1: Entity Knowledge

Manage your knowledge graph of repos, tools, services, concepts, and more.

```bash
# Create entities
engram entity create web-ui -t repo -d "Main frontend repository"
engram entity create MCP -t concept -d "Model Context Protocol"
engram entity create postgres -t service -d "PostgreSQL database"
engram entity create bzl -t tool -d "Bazel build wrapper"

# Entity types: repo, tool, concept, deployment, topic, workflow, person, team, service

# List entities
engram entity list                    # All entities
engram entity list -t service         # Filter by type

# Search entities
engram entity search api              # Search by name

# View entity details (includes relationships, aliases, observations)
engram entity show MCP

# Create relationships
engram entity relate web-ui -t depends_on api-service
engram entity relate api-service -t uses postgres
engram entity relate web-ui -t owned_by frontend-team

# Relationship types: depends_on, uses, deployed_via, owned_by, documents, related_to

# Add aliases (alternative names for term detection)
engram entity alias MCP "Model Context Protocol"
engram entity alias bzl "bazel"

# Add observations (facts, notes, insights)
engram entity observe MCP "Used by Claude Code and Cursor for tool integration" -s "anthropic docs"

# Delete an entity (cascades to relationships, aliases, observations)
engram entity delete old-service

# Statistics
engram entity stats
```

### Layer 2: Session History

Track your coding sessions with decisions, events, and rationale.

```bash
# Start a new session
engram session start --agent claude-code --project my-app --goal "Add auth"

# Log events during the session
engram session log -t decision "Chose OAuth over API keys" --context "Better security for user data"
engram session log -t observation "Found existing auth module at src/auth/"
engram session log -t error "Tests failing in CI" --source "github-actions"

# List sessions
engram session list                        # All sessions
engram session list --status active        # Only active sessions

# View session details with events
engram session show <session-id>

# Search across all sessions
engram session search "auth"               # Find events mentioning "auth"

# End the session
engram session end --summary "Implemented OAuth login flow"

# Statistics
engram session stats
```

### Layer 3: Document Search

Index and search documentation using semantic similarity.

```bash
# Index a file or directory
engram index ~/projects/docs/
engram index ./README.md

# Search indexed documents
engram search-docs "authentication flow"
engram search-docs "how to deploy" --limit 10 --score 0.5

# View index statistics
engram stats
```

### Layer 4: Tool Intelligence

Track tool usage patterns and get recommendations based on historical success rates.

```bash
# First, register tools as entities
engram entity create bzl -t tool -d "Bazel build wrapper"
engram entity create cargo -t tool -d "Rust package manager"

# Log tool usage with outcome
engram tool log bzl -o success -c "building go service"
engram tool log bzl -o success -c "building go service"
engram tool log cargo -o failed -c "building go service"

# Outcome types: success, partial, failed, switched

# Get recommendations for a context
engram tool recommend "building go service"
# Output: bzl (100% confidence) - Based on 2 previous usages with 100% success rate

# View statistics for a tool
engram tool stats bzl

# View overall tool intelligence statistics
engram tool stats

# List recent tool usages
engram tool list
engram tool list --outcome success   # Filter by outcome

# Search usage history
engram tool search "go service"
```

### Layer 5: Session Coordination

Track parallel sessions and detect conflicts when multiple agents work on the same project.

```bash
# Register a session for coordination
engram coord register <session-id> -a claude-code -p my-project -g "Implementing auth"
engram coord register <session-id> -a cursor -p my-project -g "Fixing tests" -c auth-service -c user-api

# The session ID can be any valid UUID. Components are optional for conflict detection.

# Send heartbeat to keep session active (prevents stale cleanup)
engram coord heartbeat <session-id>

# Set the current file being edited (detects file conflicts)
engram coord set-file <session-id> -f src/auth.rs
engram coord set-file <session-id>  # Clear file

# Update components being worked on
engram coord set-components <session-id> -c auth-service -c billing

# Check for conflicts with other sessions
engram coord conflicts <session-id>
# Shows both component overlaps and file conflicts

# List all active sessions
engram coord list
engram coord list -p my-project  # Filter by project

# Unregister a session
engram coord unregister <session-id>

# View statistics
engram coord stats
```

### Layer 6: Knowledge Management

Manage your personal knowledge repository with version detection and deduplication.

```bash
# Initialize knowledge system
engram knowledge init

# Scan a directory for documents
engram knowledge scan ~/projects/my-repo/docs --repo my-repo

# Import a document (copies to ~/.engram/knowledge/)
engram knowledge import ./guide.md --name "Setup Guide" -t howto

# Register a document (reference only, doesn't copy)
engram knowledge register ./ADR-001.md --name "ADR: Auth Strategy" -t adr

# Document types: adr, runbook, howto, research, design, readme, changelog

# List all knowledge documents
engram knowledge list

# Find duplicate documents
engram knowledge duplicates

# Detect version chains (e.g., guide-v1.md, guide-v2.md)
engram knowledge versions

# Statistics
engram knowledge stats
```

### Shortcuts

```bash
# Quick entity creation
engram add entity web-ui -t repo -d "Frontend repo"

# Quick alias
engram add alias "Model Context Protocol" --entity MCP

# Search entities (not documents)
engram search "frontend"
```

## MCP Tools Reference

When connected via MCP, AI agents have access to these tools:

### Layer 1: Entity Knowledge (8 tools)

| Tool | Description |
|------|-------------|
| `entity_create` | Create an entity (repo, tool, service, concept, etc.) |
| `entity_list` | List all entities, optionally filtered by type |
| `entity_search` | Search entities by name |
| `entity_get` | Get full entity details with relationships and observations |
| `entity_relate` | Create a relationship between two entities |
| `entity_alias` | Add an alias for an entity |
| `entity_observe` | Add an observation (fact/note) about an entity |
| `entity_stats` | Get entity statistics |

### Layer 2: Session History (7 tools)

| Tool | Description |
|------|-------------|
| `session_start` | Start a new coding session |
| `session_end` | End a session with optional summary |
| `session_get` | Get session details with all events |
| `session_list` | List sessions (filter by status, agent, project) |
| `session_log` | Log an event (decision, observation, error, etc.) |
| `session_search` | Search events across all sessions |
| `session_stats` | Get session statistics |

### Layer 3: Document Search (3 tools)

| Tool | Description |
|------|-------------|
| `search_docs` | Search indexed documents using semantic similarity |
| `index_docs` | Index documents from a file or directory |
| `get_stats` | Get document index statistics |

### Layer 4: Tool Intelligence (6 tools)

| Tool | Description |
|------|-------------|
| `tool_log_usage` | Log a tool usage with outcome (success/partial/failed/switched) |
| `tool_recommend` | Get tool recommendations for a given context |
| `tool_get_stats` | Get usage statistics for a specific tool |
| `tool_list_usages` | List recent tool usages, optionally filtered by outcome |
| `tool_search` | Search tool usage history by context |
| `tool_intel_stats` | Get overall tool intelligence statistics |

### Layer 5: Session Coordination (8 tools)

| Tool | Description |
|------|-------------|
| `coord_register` | Register a session for coordination with other parallel sessions |
| `coord_unregister` | Unregister a session when ending work |
| `coord_heartbeat` | Send heartbeat to keep session active |
| `coord_set_file` | Set current file being edited (detects conflicts) |
| `coord_set_components` | Update components being worked on |
| `coord_check_conflicts` | Check for component and file conflicts with other sessions |
| `coord_list` | List all active sessions, optionally filtered by project |
| `coord_stats` | Get coordination statistics |

### Layer 6: Knowledge Management (8 tools)

| Tool | Description |
|------|-------------|
| `knowledge_init` | Initialize the personal knowledge repository |
| `knowledge_scan` | Scan a directory for markdown documents |
| `knowledge_register` | Register a document reference (doesn't copy) |
| `knowledge_import` | Import a document to personal repo (copies file) |
| `knowledge_list` | List all registered knowledge documents |
| `knowledge_find_duplicates` | Find duplicate documents by content hash |
| `knowledge_detect_versions` | Detect version chains and recommend canonical |
| `knowledge_stats` | Get knowledge statistics |

## Technology Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| **Language** | Rust | Compiler catches errors; single binary distribution |
| **Database** | SurrealDB | Multi-model (relational + graph + vector); embedded |
| **Protocol** | MCP | Agent-agnostic; Claude Code + Cursor support |
| **Embeddings** | fastembed | Local inference; no API calls |

## Project Structure

```
engram/
├── engram-core/    # Domain types, invariants (zero external deps)
├── engram-store/   # SurrealDB adapter, queries, migrations
├── engram-embed/   # fastembed wrapper, batching, caching
├── engram-index/   # Chunking, ingestion, services
├── engram-mcp/     # MCP server endpoints
├── engram-cli/     # CLI application
└── engram-tests/   # Integration tests
```

## Data Storage

All data is stored locally:

```
~/.engram/
├── data/           # SurrealDB database (RocksDB backend)
└── knowledge/      # Personal knowledge repository (git-initialized)
    ├── adr/
    ├── runbook/
    ├── howto/
    ├── research/
    ├── design/
    ├── readme/
    └── changelog/
```

## Development

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug engram serve

# Run clippy
cargo clippy --all-targets --all-features

# Format code
cargo fmt
```

## Roadmap

- [x] **Layer 3**: Document Knowledge — semantic search over indexed docs
- [x] **Layer 6**: Document Intelligence — canonical resolver, dedupe, versioning
- [x] **Layer 1**: Entity Knowledge — graph of repos, tools, concepts, relationships
- [x] **Layer 2**: Session History — decisions, events, rationale tracking
- [x] **Layer 4**: Tool Intelligence — usage learning, recommendations
- [x] **Layer 5**: Session Coordination — parallel session awareness, conflict detection

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

- Named after [engrams](https://en.wikipedia.org/wiki/Engram_(neuropsychology)) — the hypothetical means by which memories are stored
- Inspired by Vannevar Bush's [Memex](https://en.wikipedia.org/wiki/Memex) vision (1945)
- Built on the [Model Context Protocol](https://modelcontextprotocol.io/) specification
