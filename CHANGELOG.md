# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Drafted the `v0.2.0` GA release notes artifact with install, upgrade, first-run,
  and known-limitation guidance for final release review.
- Added a `v0.2.0` GA release-owner approval runbook that requires fresh exact-head gate evidence,
  names the post-approval commands, and preserves default-deny release boundaries.

### Changed
- Refreshed the native Claude production-gate preflight baseline to the current Claude Code
  `2.1.173` path/hash so GA checks fail only on live blockers, not stale expected metadata.
- Removed beta-specific wording from Homebrew formula rendering and packaging docs so the same
  renderer can produce GA-ready formula text after the final version bump.
- Made local release packaging fail closed on tracked changes by default, with
  `ALLOW_TRACKED_CHANGES=1` reserved for explicit development rehearsals.
- Hardened local release packaging so malformed `ALLOW_TRACKED_CHANGES` values fail closed before
  release binary builds or artifact writes.
- Hardened published release install verification so GitHub release checks fail on draft releases
  or unexpected prerelease/stable state before downloading assets.
- Hardened published release install verification so release tags must match the workspace package
  version before release metadata or asset validation runs.
- Hardened published release install verification so downloaded release assets must match a local
  release tag that peels to the expected packaged Git commit.
- Hardened published release install verification so downloaded GitHub release proof requires
  exactly the expected archive/checksum assets and validates their GitHub asset digests.
- Hardened published release install verification so published proof requires a verified local tag
  signature and a remote Git tag object/peeled commit matching the expected release head.
- Hardened published release install verification so local `--asset-dir` rehearsals report asset
  install evidence without claiming published install verification.
- Hardened release repository targeting so the GA release gate and published-release verifier reject
  accidental repository overrides unless explicitly allowed for local rehearsals.
- Hardened package install smoke verification so `MANIFEST.json` is parsed with `jq` instead of
  ad hoc text matching when checking release metadata and packaged file hashes.
- Hardened package install smoke verification so `.sha256` files must contain exactly one
  SHA-256 line naming the expected release archive.
- Hardened Homebrew formula rendering so it requires the adjacent `.sha256` asset to name and
  hash the same release archive before formula text is written.
- Hardened Homebrew formula rendering so it verifies the packaged `MANIFEST.json` release identity,
  tracked-change provenance, and `Cargo.lock` hash before formula text is written.
- Hardened Homebrew formula rendering so it rejects unsafe archive members and verifies packaged
  payload hashes against `MANIFEST.json` before formula text is written.
- Hardened Homebrew formula rendering so accidental `HOMEBREW_RELEASE_BASE_URL` overrides fail
  closed unless explicitly allowed for local rehearsals.
- Hardened release packaging so the generated `MANIFEST.json` is validated with `jq` and checked
  against staged payload hashes before the release archive is created.
- Hardened the GA release gate so full owner-review evidence includes Homebrew formula render,
  Ruby syntax, and beta-wording validation after package/install smoke.
- Hardened the GA release gate so owner-review evidence defaults to the `main` branch and reports
  the expected branch in text and JSON output.
- Hardened the GA release gate with a disk-space preflight before local CI/package smoke, so low
  free space fails early with an actionable cleanup message instead of surfacing later as Cargo
  build errors.
- Hardened the GA release gate disk-space preflight so `--json` mode emits structured failure
  evidence before exiting nonzero when local cleanup is required.
- Hardened the GA release gate disk-space threshold so `RELEASE_GATE_MIN_FREE_KIB` overrides fail
  closed unless explicitly allowed for local rehearsals.
- Added non-destructive disk cleanup evidence to the GA release gate: low-space JSON now reports the
  local shortfall and generated artifact candidate sizes before requiring cleanup approval.
- Hardened the GA release gate so owner-review evidence checks that the intended `v0.2.0` local
  tag, remote Git tag, and GitHub release are all still unavailable before local release validation
  or publication steps.
- Hardened hosted CI tests by restoring and warming the fastembed model cache before serialized
  integration tests start daemon processes.
- Refreshed the GA readiness matrix so the latest `0cdc0cd` hardening head is clearly separated
  from historical full-gate evidence while disk cleanup still blocks final owner-review proof.
- Hardened the GA release-owner approval runbook so the current disk-space blocker is explicit and
  final post-approval gate assertions require `disk_space.state=passed`.
- Refreshed the GA readiness matrix with recent `4978711` exact-head hosted CI and disk-preflight
  evidence while preserving the release-owner cleanup approval gate.
- Refreshed the GA readiness matrix with exact-head `35a5b9c` package manifest payload-hash guard
  evidence while preserving the disk cleanup approval gate.
- Refreshed the GA readiness matrix with exact-head `6ba403e` disk-threshold override guard
  evidence and local release-packaging override validation.
- Refreshed the GA readiness matrix with current-main hosted CI evidence plus isolated Codex and
  Cursor setup adapter install/status rehearsal results.
- Refreshed the GA readiness matrix with setup-path hosted CI, canonical vault, and read-only
  native Claude preflight evidence; the native Claude production gate remains blocked by an
  already-running CLI process and is not claimed as closed.
- Refreshed the GA readiness matrix with native Claude preflight evidence from the GA release-gate
  hardening checkpoint; the gate remains blocked only by an already-running native Claude CLI
  process.
- Shifted release-facing setup and security docs from beta-only wording to a `0.2.x` support scope
  while preserving the current fact that `v0.2.0-beta.2` is still the latest published artifact.
- Added a GA-capable release gate report script for current-main CI/package evidence, while keeping
  the beta PR gate command as a compatibility wrapper.
- Hardened the GA release gate report so the intended release version is distinct from the current
  workspace package version and the gate remains version-blocked until the `0.2.0` bump lands.
- Hardened the GA release gate report so final release notes must retain explicit scope limits for
  native Claude proof and broad lifecycle/M6 mutation claims before owner review.
- Bumped the workspace package metadata to `0.2.0` for final GA validation; tag, artifact,
  Homebrew, and GitHub release publication remain gated on exact-head CI and full release-gate
  evidence.
- Refreshed the GA readiness matrix with pre-runbook exact-head `0.2.0` CI, full release-gate,
  package smoke, and Homebrew formula evidence; publication still requires explicit release-owner
  approval and a fresh full gate on the release head.

### Fixed
- Kept work-management integration tests offline-deterministic by using the no-embedder
  `WorkService` fixture for CRUD and MCP coverage, avoiding incidental model downloads on hosted
  CI.
- Serialized the multi-session daemon integration test harness so hosted CI does not start one
  daemon per test concurrently and time out waiting for `/health`.
- Hardened the multi-session daemon integration test harness so daemon startup failures include
  child-process status and daemon stdout/stderr tails, with a longer readiness window for cold CI
  runners.

## [0.2.0-beta.2] - 2026-06-09

### Added
- `engram setup` provides a guided, dry-run-first setup path for Claude Code, Codex, and Cursor.
- First `orient` now prompts the agent to ask which existing docs, runbooks, notes, ADRs, or
  knowledge folders to ingest when the document index is empty.
- Homebrew packaging support for macOS Apple Silicon beta installs.
- `engram warmup embeddings` prepares and verifies the local fastembed ONNX model cache before
  first agent use or offline/sandboxed work.
- `engram daemon status` now reports daemon spawn provenance when available, including the
  executable path/version that started the daemon and warnings for path, version, or pid/port
  metadata drift.
- `scripts/native-claude-gate-preflight.sh` collects read-only production-gate evidence for
  native Claude prompt-bearing, effective-hook, and live host-label proof readiness, with
  machine-readable `--json` output for release and GA automation. It defaults its post-release
  branch expectation to `main` and supports `--expected-branch <branch>` for explicit non-main
  checks.
- `scripts/beta-release-gate-report.sh` now supports machine-readable `--json` output for
  release-owner evidence automation while preserving text output and evidence-only semantics. When
  hosted CI is verified as a pre-step blocker, the JSON report embeds the hosted verifier's
  structured success object under `hosted_ci.verifier`; it also exposes the release gate state,
  release-owner-review readiness, hosted-CI fallback-decision requirement, and remaining release
  actions.
- `scripts/verify-published-release-install.sh` verifies published release archive/checksum assets
  by downloading them from GitHub and running the existing package install smoke against those
  assets, with local `--asset-dir` validation for pre-publish rehearsal.
- `scripts/verify-hosted-ci-prestep-blocker.sh` now supports machine-readable `--json` output for
  direct hosted-CI waiver-condition automation while preserving fail-closed checks. It also accepts
  `--event <event>` for explicit post-merge push-run verification while defaulting to
  `pull_request`.

### Changed
- README, MCP setup, security, and contribution docs now describe the beta install/setup path and
  open-source project expectations.
- Cold embedding-model initialization now prints an explicit first-run download/cache warning before
  fastembed loads the model.
- RocksDB lock conflicts now include recovery guidance and daemon startup failures include recent
  daemon log output so stale lock and active-daemon cases are easier to diagnose.

## [0.2.0-beta.1] - 2026-06-07

### Added
- Initial local/Codex Brain Loop beta scope with lean `orient`, current-plan retrieval,
  used-memory candidate IDs, obligation summaries, telemetry feedback, generated vault support,
  and review-gated Memory OS flows.
- Local CI-equivalent validation script (`scripts/local-ci.sh`) for exact-head release checks when
  hosted GitHub Actions is externally blocked.
- Local release packaging script (`scripts/package-release.sh`) that builds the release binary,
  verifies its version, and writes a tarball plus SHA-256 checksum under `dist/`.
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
