# Brain Harness T336 Project-Scoped Lint

Date: 2026-06-07
Status: implemented and locally validated

## Scope

T336 adds an optional project filter to Memory OS lint so agents can ask for current project health
without unrelated historical project obligations dominating the report.

This is a code slice. It changes:

- `engram-index/src/lint.rs`
- `engram-mcp/src/tools.rs`
- `engram-mcp/src/server.rs`
- `engram-cli/src/main.rs`
- `engram-tests/tests/lint_tests.rs`

It does not archive memory, run `lint apply_safe`, mutate obligations, change `orient`, change
ranking, run native Claude, change harness files, or touch user-owned files.

## Research Question

Can lint provide a project-focused health report without losing the existing global lint behavior?

## Hypotheses

| Type | Hypothesis | Evidence |
| --- | --- | --- |
| Preferred | Add an optional project filter while preserving unscoped lint as the default. | `LintOptions.project`, MCP `project`, and CLI `--scope-project` are optional. |
| Null | Global lint is sufficient for project closeout. | Rejected because fresh global lint still surfaces unrelated historical obligations while `obligations doctor` for `engram` is clean. |
| Simpler alternative | Only document that global lint is noisy. | Rejected because a small code path can make the health check directly useful. |
| Failure | Scoped lint hides broadly applicable guidance or changes lifecycle writes. | Avoided by keeping global/user memory visible and leaving `apply_safe` gated by existing safe-action rules. |

## Behavior

When a project is supplied:

- memory lint includes global and user memory;
- memory lint includes project-scoped items whose project name matches case-insensitively;
- memory lint includes task-scoped items whose parent project name matches;
- stale active session lint filters sessions to the project;
- open obligation lint filters obligations with the same scope rules;
- unrelated project, repository, entity, session, and custom-scope memory are excluded because the
  current lint request has no repository/entity resolver context.

Unscoped lint remains unchanged.

## Validation

Focused validation passed:

- `cargo test -p engram-index lint_project_scope_filters_memory_obligations_and_sessions`
- `cargo test -p engram-tests --test lint_tests`
- `cargo run -p engram-cli -- lint --data-dir <tempdir> run --scope-project engram --limit 5 --json`

The first CLI smoke against the live global store failed with the existing SurrealDB lock because
the daemon owns `~/.engram/data/LOCK`; the isolated temp-store CLI smoke passed and returned an
empty report. This confirms the flag and command path without requiring daemon downtime.

## Non-Claims

T336 does not claim lifecycle cleanup is complete, does not make broad `lint apply_safe` safe, and
does not close native Claude, effective-hook, live host-label, hosted CI, direct legacy deletion, or
production parity gates.
