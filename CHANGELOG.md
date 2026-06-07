# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0-beta.1] - 2026-06-07

### Added
- Initial local/Codex Brain Loop beta scope with lean `orient`, current-plan retrieval,
  used-memory candidate IDs, obligation summaries, telemetry feedback, generated vault support,
  and review-gated Memory OS flows.
- Memory OS harness adapter rendering for Codex, generic harnesses, Claude Code, Gemini CLI, and
  Cursor, with lifecycle guidance kept advisory and approval-gated.
- Generated Markdown vault support for durable local inspection of MemoryItems, knowledge commits,
  repositories, entities, and projects.

### Changed
- Public setup docs now state that the beta-supported path is local/Codex; multi-host parity,
  native Claude prompt-bearing proof, live host-label proof, and effective-hook proof remain
  follow-up work.
- Default `engram serve` stdio/proxy behavior now rejects incompatible HTTP-only or memory-only
  flags instead of silently starting a different persistence mode.

### Fixed
- Scoped Memory OS search and final-response obligation guidance so project/cwd context is not
  lost in the supported local/Codex beta path.
- Refreshed the installed local `engram` binary, Codex generated adapter, and global daemon from the
  beta candidate so installed local/Codex guidance matches source.
- Escaped graph node ID examples in Rustdoc comments so the Docs CI job no longer reports invalid
  HTML tag warnings for `memory:<...>` examples.
- Claude harness status now warns when generated `SessionStart` or `SessionEnd` hook files are
  installed but Claude settings do not register the corresponding required hook.
- CI checkout runtime drift by moving workflow checkout steps to `actions/checkout@v5`.

### Known Limitations
- Native Claude prompt-bearing proof, effective-hook visibility, live Claude host-label proof,
  direct legacy deprecation/deletion, broad lifecycle cleanup, broad `lint apply_safe`, exhaustive
  telemetry completeness, OIDC/Vault/native-Claude auth/debugging, full multi-host parity, and new
  feature work are not part of this beta gate.
- The current Rustdoc warning set was closed in this candidate; future Rustdoc polish remains
  production-hardening work, not an initial-beta gate.
- Already-open agent UI sessions may need a fresh session or tool reload before they pick up the
  refreshed installed Codex skill text.

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
