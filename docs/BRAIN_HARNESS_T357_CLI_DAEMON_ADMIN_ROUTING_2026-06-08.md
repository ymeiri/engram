# Brain Harness T357 CLI Daemon Admin Routing

Date: 2026-06-08
Status: implemented, source-validated, local-CI validated, and package-smoked

## Scope

T357 hardens the local/Codex beta CLI path for daemon-backed Memory OS administration. Before this
slice, installed CLI commands such as:

```text
engram lint run --scope-project engram --vault-path /Users/yuval.meiri/.engram/vault --json
engram vault status /Users/yuval.meiri/.engram/vault --json
```

could try to open `~/.engram/data` directly while the global daemon was already running, causing a
RocksDB lock error instead of using the healthy daemon.

T357 changes only `engram-cli/src/main.rs`. When no explicit `--data-dir` is supplied, the CLI now
prefers a healthy global or project daemon for one-shot `lint`, `obligations`, and `vault` commands
by using the existing MCP `proxy::call_tool_once` path. Explicit `--data-dir` still uses direct
RocksDB access, and commands still fall back to direct access when no healthy matching daemon is
available.

## Research Question

Can Engram make common beta admin commands work while the daemon owns the store, without changing
storage semantics, daemon behavior, MCP request shapes, or the supported beta scope?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Reusing the existing one-shot MCP daemon helper for CLI `lint`, `obligations`, and `vault` commands avoids RocksDB lock conflicts while preserving output shape and direct fallback behavior. | Supported. |
| Null | CLI admin commands should keep direct RocksDB access only; users can stop the daemon before running them. | Rejected for the local/Codex beta path because daemon ownership is the normal multi-session operating mode. |
| Simpler alternative | Document the lock workaround without changing code. | Rejected because it leaves first-run admin commands brittle while a small existing proxy path already exists. |
| Failure | The CLI confuses data-store project selection with `--scope-project`, routes explicit `--data-dir` through the daemon, or changes vault JSON output wrappers. | Avoided with focused argument-builder tests and installed smokes. |

## Implementation

- Added a shared `call_daemon_tool_if_available` helper that:
  - skips daemon routing when `--data-dir` is supplied;
  - selects the global or project daemon based on the existing top-level `--project` data-store
    option;
  - returns `None` when the matching daemon is unavailable, preserving direct fallback.
- Routed `Commands::Lint`, `Commands::Obligations`, and `Commands::Vault` through that helper
  before opening the store directly.
- Kept `--scope-project` as the lint/obligation filter and top-level `--project` as data-store
  selection.
- Added focused unit tests for lint, obligation, and vault daemon argument construction.

## Validation

Source validation:

```text
cargo fmt --all --check
git diff --check
cargo check -p engram-cli
cargo test -p engram-cli
```

Live source-binary smokes with the global daemon still running:

```text
cargo run -q -p engram-cli -- lint run --scope-project engram --vault-path /Users/yuval.meiri/.engram/vault --limit 3 --json
cargo run -q -p engram-cli -- lint run --scope-project engram --vault-path /Users/yuval.meiri/.engram/vault --limit 3
cargo run -q -p engram-cli -- vault status /Users/yuval.meiri/.engram/vault --json
cargo run -q -p engram-cli -- vault status /Users/yuval.meiri/.engram/vault
```

Installed-path refresh:

```text
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
```

This replaced installed hash:

```text
a47edffa8c8ed955a311adac85033ce8a28235c37007b5149ea81a8ffeb456de
```

with:

```text
fa91efbd228683dae608881f5828bdc1ffe55b67376e414653f8ac8eb92ba8c9
```

Installed-path smokes passed while the existing global daemon remained running as PID `64693` on
port `8765`:

```text
/Users/yuval.meiri/.local/bin/engram lint run --scope-project engram --vault-path /Users/yuval.meiri/.engram/vault --limit 3 --json
/Users/yuval.meiri/.local/bin/engram vault status /Users/yuval.meiri/.engram/vault --json
/Users/yuval.meiri/.local/bin/engram obligations doctor --limit 3 --json
```

The lint and vault commands returned daemon-backed JSON instead of the previous
`~/.engram/data/LOCK` failure. The unscoped obligations doctor also routed successfully; its output
remains intentionally global unless callers use the MCP doctor with explicit `project` and `cwd`.

Full local CI-equivalent validation passed:

```text
./scripts/local-ci.sh
```

Release package/install validation also passed:

```text
./scripts/package-install-smoke.sh
```

The package smoke rebuilt the release tarball, verified
`engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz.sha256`, installed the packaged binary into a
temporary prefix, confirmed `engram 0.2.0-beta.1`, started packaged
`engram serve --http --memory`, and verified `/health` returned:

```json
{"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

## Gate Impact

T357 improves the supported local/Codex beta operator path. It does not mark PR #3 ready, merge,
tag, publish, close hosted CI, run native Claude, prove effective-hook visibility, prove live host
labels, mutate lifecycle state, run broad `lint apply_safe`, or change the supported beta scope.
