# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Build
cargo build                          # Debug build
cargo build --release                # Optimized release build

# Test
cargo test                           # Run all tests
cargo test --all-targets             # Run all tests including examples
cargo test <test_name>               # Run a single test

# Lint & Format
cargo fmt                            # Format code
cargo fmt --all --check              # Check formatting (CI)
cargo clippy --all-targets -- -D warnings  # Run linter (CI)

# Run (multi-session support via daemon)
engram init                          # Initialize database (~/.engram/data/)
engram serve                         # Start MCP (auto-starts daemon, runs proxy)
engram serve --project myproject     # Project-specific daemon (isolated data)
engram serve --http --port 8765      # Run as HTTP daemon directly
engram serve --memory                # Start with in-memory storage (testing)
RUST_LOG=debug engram serve          # Verbose logging

# Daemon management
engram daemon status                 # Check if global daemon is running
engram daemon status --project foo   # Check project-specific daemon
engram daemon start                  # Start daemon (auto-select port 8765-8774)
engram daemon stop                   # Stop running daemon
engram daemon logs                   # View daemon logs
```

## Architecture Overview

engram is a Personal Knowledge Augmentation System (PKAS) that provides AI coding agents with persistent memory via the Model Context Protocol (MCP).

### Crate Structure (dependency order)

```
engram-core     Zero external deps. Domain types: Entity, Session, Document, Chunk, Tool, Coordination, Knowledge, Work
    ↓
engram-store    SurrealDB adapter. Repository traits and implementations for all domain types
    ↓
engram-embed    fastembed wrapper. Local ONNX embeddings (384 dimensions)
    ↓
engram-index    Business logic services: DocumentService, EntityService, SessionService,
                ToolIntelService, CoordinationService, KnowledgeService, WorkService
    ↓
engram-mcp      MCP server (rmcp crate). EngramServer exposes 21 tools via JSON-RPC over stdio
    ↓
engram-cli      CLI application. Single main.rs using clap derive
```

### Seven Knowledge Layers

1. **Entity Knowledge** - Graph of repos, tools, services, concepts with relationships, aliases, observations
2. **Session History** - Coding session tracking with decisions, events, rationale
3. **Document Knowledge** - Semantic search over chunked markdown documents
4. **Tool Intelligence** - Tool usage patterns and success-rate-based recommendations
5. **Session Coordination** - Parallel session awareness and conflict detection
6. **Knowledge Management** - Document registry with version detection and deduplication
7. **Work Management** - Projects, tasks, PRs with entity connections and scoped observations

### Multi-Session Architecture

engram supports multiple AI agent sessions sharing the same knowledge base through a transparent daemon proxy:

```
┌─────────────────┐     ┌─────────────────┐
│  Claude Code 1  │     │  Claude Code 2  │
│  (stdio proxy)  │     │  (stdio proxy)  │
└────────┬────────┘     └────────┬────────┘
         │                       │
         └───────────┬───────────┘
                     │ HTTP
              ┌──────┴──────┐
              │   Daemon    │
              │ (HTTP MCP)  │
              └──────┬──────┘
                     │
              ┌──────┴──────┐
              │  SurrealDB  │
              │  (RocksDB)  │
              └─────────────┘
```

- **Default mode**: `engram serve` auto-starts a daemon (if needed) and runs as stdio proxy
- **Daemon mode**: `engram serve --http` runs the HTTP server directly
- **Project isolation**: `engram serve --project myproject` uses separate daemon/data per project
- **Port range**: Daemons auto-select ports 8765-8774

### Data Flow

```
AI Agent → MCP (stdio/JSON-RPC) → Proxy → HTTP → EngramServer → Services
         → Repositories (engram-store) → SurrealDB (embedded RocksDB)
```

### Key Files

- `engram-mcp/src/tools.rs` - All 21 MCP tool implementations (largest file)
- `engram-mcp/src/server.rs` - EngramServer with stdio and HTTP server modes
- `engram-cli/src/main.rs` - CLI entry point with all subcommands
- `engram-cli/src/daemon.rs` - Daemon management (start, stop, health checks)
- `engram-cli/src/proxy.rs` - Stdio-to-HTTP proxy for transparent daemon access
- `engram-store/src/repos/*.rs` - Database queries for each domain type

### Storage

- Global database: `~/.engram/data/` (SurrealDB with RocksDB backend)
- Project database: `~/.engram/projects/<name>/data/` (isolated per project)
- Daemon files: `~/.engram/daemon.{port,pid,log}` (or in project subdir)
- Knowledge repo: `~/.engram/knowledge/` (git-initialized, organized by doc type)

## Testing

Integration tests are in `engram-tests/tests/` with one file per layer:
- `entity_tests.rs`, `session_tests.rs`, `document_tests.rs`
- `tool_intel_tests.rs`, `coordination_tests.rs`, `knowledge_tests.rs`
- `work_tests.rs` (projects, tasks, PRs, entity connections, scoped observations)
- `semantic_search_tests.rs`, `search_tests.rs` (unified cross-layer search)

Tests use in-memory SurrealDB (`kv-mem` feature) and `insta` for YAML snapshots.

## Code Conventions

- MSRV: Rust 1.80
- Max line width: 100 characters
- Max function arguments: 8
- Max cognitive complexity: 25
- UUID v7 for all IDs (time-sortable)
- `thiserror` for library errors, `anyhow` for application errors
