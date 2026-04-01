# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- MCP setup guide for Claude Code, Cursor, and Windsurf (`docs/MCP_SETUP.md`)

## [0.1.0] - 2026-04-01

### Added

#### Layer 1: Entity Knowledge
- Entity types: repo, tool, concept, deployment, topic, workflow, person, team, service
- Relationship types: depends_on, uses, deployed_via, owned_by, documents, related_to
- Entity repository with SurrealDB graph storage
- Entity service with business logic (create, resolve by name/alias, relate)
- CLI commands: `entity create`, `list`, `show`, `search`, `relate`, `alias`, `observe`, `delete`, `stats`
- MCP tools: `entity_create`, `entity_list`, `entity_search`, `entity_get`, `entity_relate`, `entity_alias`, `entity_observe`, `entity_stats`

#### Layer 2: Session History
- Session tracking with project, agent, goal, status, and summary
- Event types: decision, command, file_change, tool_use, error, milestone, observation
- Session repository with SurrealDB storage
- Session service with business logic (start, end, log events, search)
- CLI commands: `session start`, `end`, `list`, `show`, `log`, `search`, `stats`
- MCP tools: `session_start`, `session_end`, `session_get`, `session_list`, `session_log`, `session_search`, `session_stats`
- Cross-session event search for finding past decisions and rationale
- Automatic session lookup when session ID not specified

#### Layer 4: Tool Intelligence
- Tool usage tracking with outcome types: success, partial, failed, switched
- Context-based tool recommendations using historical success rates
- Automatic preference learning from usage patterns
- Tool statistics with success rate calculation
- Tool repository with SurrealDB storage (`engram-store/src/repos/tool.rs`)
- Tool intelligence service (`engram-index/src/tool_intel.rs`)
- CLI commands: `tool log`, `recommend`, `stats`, `list`, `search`
- MCP tools: `tool_log_usage`, `tool_recommend`, `tool_get_stats`, `tool_list_usages`, `tool_search`, `tool_intel_stats`

#### Layer 5: Session Coordination
- Active session registration with agent, project, goal, and components
- Heartbeat mechanism for session liveness tracking
- Component-based conflict detection (overlapping components between sessions)
- File-based conflict detection (multiple sessions editing same file)
- Stale session cleanup with configurable timeout (default 30 minutes)
- Coordination repository with SurrealDB storage (`engram-store/src/repos/coordination.rs`)
- Coordination service (`engram-index/src/coordination.rs`)
- CLI commands: `coord register`, `unregister`, `heartbeat`, `set-file`, `set-components`, `conflicts`, `list`, `stats`
- MCP tools: `coord_register`, `coord_unregister`, `coord_heartbeat`, `coord_set_file`, `coord_set_components`, `coord_check_conflicts`, `coord_list`, `coord_stats`

#### Layer 3: Document Search
- Markdown parsing and intelligent chunking
- Local embeddings via fastembed (ONNX-based, no API calls)
- Vector storage and semantic similarity search in SurrealDB
- CLI commands: `index`, `search-docs`, `stats`
- MCP tools: `search_docs`, `index_docs`, `get_stats`

#### Layer 6: Knowledge Management
- Knowledge document registry with content hashing
- Document types: adr, runbook, howto, research, design, readme, changelog
- Version chain detection (e.g., guide-v1.md → guide-v2.md)
- Duplicate detection by content hash
- Import (copies to ~/.engram/knowledge/) vs Register (reference only)
- Directory scanning with sync records
- CLI commands: `knowledge init`, `scan`, `import`, `register`, `list`, `duplicates`, `versions`, `stats`
- MCP tools: `knowledge_init`, `knowledge_scan`, `knowledge_register`, `knowledge_import`, `knowledge_list`, `knowledge_find_duplicates`, `knowledge_detect_versions`, `knowledge_stats`

#### Infrastructure
- 7-crate workspace structure (core, store, embed, index, mcp, cli, tests)
- SurrealDB embedded database with RocksDB backend
- MCP server with stdio transport
- Comprehensive CLI with clap
- Integration test suite

### Changed
- N/A

### Fixed
- SurrealDB datetime deserialization using custom `SurrealDateTime` enum
- Raw SurQL queries to avoid SDK serialization issues with complex types

