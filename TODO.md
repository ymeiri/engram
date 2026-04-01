# engram TODO

> Tracking document for the Personal Knowledge Augmentation System (PKAS)

## Project Status Overview

| Layer | Name | Status | Description |
|-------|------|--------|-------------|
| 1 | Entity Knowledge | ✅ Complete | Knowledge graph for repos, tools, services, concepts |
| 2 | Session History | ✅ Complete | Decisions, events, rationale tracking |
| 3 | Document Search | ✅ Complete | Semantic search over indexed documents |
| 4 | Tool Intelligence | ✅ Complete | Tool usage learning, recommendations |
| 5 | Session Coordination | ✅ Complete | Parallel session awareness, conflict detection |
| 6 | Knowledge Management | ✅ Complete | Personal knowledge repo, versioning, deduplication |

---

## Completed Work

### Layer 1: Entity Knowledge ✅

**Crates modified:** `engram-core`, `engram-store`, `engram-index`, `engram-mcp`, `engram-cli`

- [x] Core entity types: `Entity`, `Relationship`, `Alias`, `Observation`
- [x] Entity types: repo, tool, concept, deployment, topic, workflow, person, team, service
- [x] Relationship types: depends_on, uses, deployed_via, owned_by, documents, related_to
- [x] SurrealDB graph storage with `RELATE` statements
- [x] Entity repository (`engram-store/src/repos/entity.rs`)
- [x] Entity service with business logic (`engram-index/src/entity.rs`)
- [x] CLI commands: create, list, show, search, relate, alias, observe, delete, stats
- [x] MCP tools (8 tools): entity_create, entity_list, entity_search, entity_get, entity_relate, entity_alias, entity_observe, entity_stats

**Implementation notes:**
- Uses raw SurQL queries to avoid SurrealDB SDK serialization issues
- Custom `SurrealDateTime` enum handles SurrealDB's native datetime format
- Graph queries use SurrealDB's graph traversal syntax

### Layer 3: Document Search ✅

**Crates modified:** `engram-embed`, `engram-store`, `engram-index`, `engram-mcp`, `engram-cli`

- [x] Markdown parsing and chunking
- [x] fastembed integration for local embeddings (no API calls)
- [x] Vector storage in SurrealDB
- [x] Semantic similarity search
- [x] CLI commands: index, search-docs, stats
- [x] MCP tools (3 tools): search_docs, index_docs, get_stats

**Implementation notes:**
- Uses `fastembed` crate for ONNX-based local inference
- Embedding model downloaded on first use (~100MB)
- Chunks stored with source file path and line numbers

### Layer 6: Knowledge Management ✅

**Crates modified:** `engram-core`, `engram-store`, `engram-index`, `engram-mcp`, `engram-cli`

- [x] Knowledge document registry
- [x] Document types: adr, runbook, howto, research, design, readme, changelog
- [x] Content hashing for deduplication
- [x] Version chain detection (e.g., guide-v1.md, guide-v2.md)
- [x] Import (copy to ~/.engram/knowledge/) vs Register (reference only)
- [x] Directory scanning with sync records
- [x] CLI commands: init, scan, import, register, list, duplicates, versions, stats
- [x] MCP tools (8 tools): knowledge_init, knowledge_scan, knowledge_register, knowledge_import, knowledge_list, knowledge_find_duplicates, knowledge_detect_versions, knowledge_stats

**Architecture decision (from expert panel consultation):**
> Files remain the source of truth; engram acts as an intelligent index layer.
> This preserves existing workflows while adding knowledge graph capabilities.

### Layer 2: Session History ✅

**Crates modified:** `engram-core`, `engram-store`, `engram-index`, `engram-mcp`, `engram-cli`

- [x] Core session types: `Session`, `Event`, `SessionStatus`, `EventType`, `SessionStats`
- [x] Event types: decision, command, file_change, tool_use, error, milestone, observation
- [x] Session repository (`engram-store/src/repos/session.rs`)
- [x] Session service with business logic (`engram-index/src/session.rs`)
- [x] CLI commands: start, end, list, show, log, search, stats
- [x] MCP tools (7 tools): session_start, session_end, session_get, session_list, session_log, session_search, session_stats

**Implementation notes:**
- Sessions track project, agent, goal, status, summary
- Events have content, context, source fields for rich logging
- Cross-session search finds events matching queries across all sessions
- Automatic session lookup (most recent active) when session ID not specified

### Layer 4: Tool Intelligence ✅

**Crates modified:** `engram-core`, `engram-store`, `engram-index`, `engram-mcp`, `engram-cli`

- [x] Core tool types: `ToolOutcome` (Display/FromStr), `ToolStats`, existing `ToolUsage`, `ToolPreference`, `ToolRecommendation`
- [x] Tool outcome types: success, partial, failed, switched
- [x] Tool repository (`engram-store/src/repos/tool.rs`)
- [x] Tool intelligence service with business logic (`engram-index/src/tool_intel.rs`)
- [x] CLI commands: log, recommend, stats, list, search
- [x] MCP tools (6 tools): tool_log_usage, tool_recommend, tool_get_stats, tool_list_usages, tool_search, tool_intel_stats

**Implementation notes:**
- Tools must be registered as entities (type: tool) before logging usage
- Recommendations based on context matching and historical success rates
- Preferences learned automatically when tool used successfully ≥2 times in same context
- Uses `meta::id(id)` in SurrealDB queries to extract string IDs from Thing types

### Layer 5: Session Coordination ✅

**Crates modified:** `engram-core`, `engram-store`, `engram-index`, `engram-mcp`, `engram-cli`

- [x] Core coordination types: `ActiveSession`, `ConflictInfo`
- [x] Session registration with agent, project, goal, components
- [x] Heartbeat mechanism for session liveness
- [x] Component-based conflict detection (overlapping components)
- [x] File-based conflict detection (same file being edited)
- [x] Stale session cleanup (configurable timeout)
- [x] Coordination repository (`engram-store/src/repos/coordination.rs`)
- [x] Coordination service (`engram-index/src/coordination.rs`)
- [x] CLI commands: register, unregister, heartbeat, set-file, set-components, conflicts, list, stats
- [x] MCP tools (8 tools): coord_register, coord_unregister, coord_heartbeat, coord_set_file, coord_set_components, coord_check_conflicts, coord_list, coord_stats

**Implementation notes:**
- Sessions register with components (e.g., "auth-service", "user-api") for conflict detection
- Conflicts detected when multiple sessions touch same components or files
- Heartbeat mechanism prevents stale sessions from blocking (default 30 min timeout)
- Independent of Layer 2 sessions - can register any valid UUID

---

## Known Issues & Technical Debt

### SurrealDB Integration
- [ ] **DateTime deserialization**: Required custom `SurrealDateTime` enum to handle SurrealDB's native format
- [ ] **SDK limitations**: Using raw SurQL strings instead of typed queries due to serialization issues
- [ ] **Connection pooling**: Currently creates new connections; may need pooling for performance

### Build & Development
- [ ] **Compilation time**: Full release build takes ~8 minutes due to heavy dependencies (SurrealDB, ONNX)
- [ ] **Binary size**: Release binary is large (~50MB) due to embedded ML models
- [ ] **Disk space**: Cargo artifacts can consume 10-20GB; need periodic `cargo clean`

### Testing
- [ ] **Integration tests**: Some tests require model downloads (marked `#[ignore]`)
- [ ] **Test isolation**: Tests share database; occasional flakiness
- [ ] Add more unit tests for edge cases

### Documentation
- [ ] Create `docs/architecture.md` with system design
- [ ] Create `docs/schema.md` with SurrealDB schema documentation
- [ ] Create `docs/mcp-tools.md` with detailed tool documentation
- [ ] Add `CONTRIBUTING.md` for open source contributors
- [ ] Update `CHANGELOG.md` with recent changes

---

## Future Ideas & Enhancements

### From Expert Panel Consultation
- [ ] **Bidirectional sync**: Changes in ~/.engram/knowledge/ reflect back to source files
- [ ] **Semantic deduplication**: Detect near-duplicates, not just exact hash matches
- [ ] **Progressive migration**: Help users gradually migrate scattered docs to canonical locations
- [ ] **Git integration**: Correlate knowledge with git history

### General Enhancements
- [ ] **Web UI**: Simple dashboard for browsing knowledge graph
- [ ] **Export formats**: Export knowledge to JSON, Markdown, or other formats
- [ ] **Backup/restore**: Easy backup of ~/.engram/ directory
- [ ] **Multi-workspace**: Support multiple projects with isolated knowledge bases
- [ ] **Embedding model selection**: Allow users to choose different embedding models
- [ ] **Incremental indexing**: Only re-index changed files
- [ ] **Watch mode**: Auto-reindex on file changes

### MCP Protocol Enhancements
- [ ] **Resources**: Expose knowledge as MCP resources, not just tools
- [ ] **Prompts**: Pre-built prompts for common knowledge queries
- [ ] **Streaming**: Stream large search results

---

## Development Notes

### Crate Structure
```
engram/
├── engram-core/    # Domain types (zero external deps)
├── engram-store/   # SurrealDB adapter, repositories
├── engram-embed/   # fastembed wrapper
├── engram-index/   # Business logic services
├── engram-mcp/     # MCP server and tools
├── engram-cli/     # CLI application
└── engram-tests/   # Integration tests
```

### Key Commands
```bash
# Build
cargo build --release

# Test (some tests require model download)
cargo test

# Run with verbose logging
RUST_LOG=debug ./target/release/engram serve

# Clean build artifacts (frees 10-20GB)
cargo clean
```

### Data Storage
```
~/.engram/
├── data/           # SurrealDB database (RocksDB backend)
└── knowledge/      # Personal knowledge repository (git-initialized)
    ├── adr/
    ├── runbook/
    ├── howto/
    └── ...
```

---

*Last updated: 2026-01-29*
