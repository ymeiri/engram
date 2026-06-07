# T138 Critical Validation Baseline

Date: 2026-06-02
Status: completed validation and narrow CI-lint fix
Scope: non-destructive repository validation after T137

No harness install, hook/settings/adapter write, binary install, daemon restart, lifecycle archive
or apply, migration action, schema/storage/index change, public MCP change, ranking change,
document-index behavior change, or `orient` payload change was run for T138.

## Research Question

Can the current post-T137 repository pass the critical Brain Harness validation surfaces, and if not
is the failure narrow enough to fix without crossing any approval gate?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The current source passes critical tests and lint, while installed harness repair, M6, and lifecycle cleanup remain separately gated. |
| Null | The repo has an ordinary validation failure that blocks a production-readiness claim. |
| Simpler alternative | Do no validation and keep relying on read-only readiness audits. |
| Failure | A green validation baseline gets overstated as installed harness readiness, M6 completion, lifecycle hygiene, broad ranking quality, or cross-harness production readiness. |

## Measurement

T138 used only local repository validation and one mechanical source edit triggered by CI lint:

- `cargo fmt --all --check`
- `cargo test -p engram-tests --test harness_tests`
- `cargo test -p engram-tests --test memory_tests orient`
- `cargo test -p engram-tests --test obligation_tests`
- `cargo test -p engram-tests --test telemetry_tests`
- `cargo test -p engram-tests --test lint_tests`
- `cargo test -p engram-tests --test brain_harness_eval_tests`
- `cargo test -p engram-tests --test search_tests current`
- `cargo check -p engram-cli`
- `cargo test -p engram-cli invalid_rfc3339_timestamp_error_names_cursor_timestamp`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
- `git diff --check`

The final `cargo test --all-targets` run passed on the final tree. Model-download-dependent tests
remained ignored by their existing test attributes.

## Finding

The first clippy run found one existing CI-lint failure:

```text
error: items after a test module
engram-cli/src/main.rs:2923:1
```

The fix was intentionally mechanical: move the existing
`invalid_rfc3339_timestamp_error_names_cursor_timestamp` test module from the middle of
`engram-cli/src/main.rs` to the end of the file. The test body and assertions were not changed.

## Results

| Area | T138 result | Evidence |
| --- | --- | --- |
| Formatting | Passed | `cargo fmt --all --check` |
| CLI cursor timestamp test | Passed | `cargo test -p engram-cli invalid_rfc3339_timestamp_error_names_cursor_timestamp` |
| CLI compile check | Passed | `cargo check -p engram-cli` |
| CI lint | Passed after mechanical test-module move | `cargo clippy --all-targets -- -D warnings` |
| Harness rendering and hook-event coverage | Passed | `cargo test -p engram-tests --test harness_tests`; full suite also covered `engram-index` harness tests. |
| Lean/current-plan `orient` coverage | Passed | `cargo test -p engram-tests --test memory_tests orient`; full suite also covered `engram-index` orient tests. |
| Current-plan direct-search fixtures | Passed | `cargo test -p engram-tests --test search_tests current` and full `search_tests`. |
| Obligations, telemetry, lint | Passed | Targeted MCP integration tests and full suite. |
| Full repository tests | Passed | `cargo test --all-targets` on final tree. |
| Diff hygiene | Passed | `git diff --check` |

## Completion Matrix Delta

| Area | T138 status | Evidence |
| --- | --- | --- |
| Critical repository validation | Validated | Full workspace tests, targeted Brain Harness tests, `engram-cli` check, clippy, format, and diff checks passed. |
| CI-lint readiness | Improved | Existing `items_after_test_module` failure fixed without behavioral change. |
| Installed harness readiness | Still missing, gated | T138 did not run `harness(install)`, status, doctor, hook edits, or settings edits. T135 remains the exact repair gate. |
| M6 migration completion | Still gated | No migration inventory, review export, status, prioritize, apply, deletion, or cleanup ran. |
| Lifecycle hygiene | Still gated | No memory archive, `lint(apply_safe)`, or handoff semantics change ran. T136 active-handoff noise remains. |
| Cross-harness production readiness | Still not complete | T138 did not validate installed Claude Code, Codex, Gemini CLI, Cursor, or generic harness readiness. |

## Decision

T138 gives a stronger final-tree repository validation baseline and fixes a narrow clippy issue. It
does not change the product gate sequence: T135 remains the next product-moving exact approval gate,
with M6 migration completion and lifecycle cleanup still separate approval-gated work.

## Stop Conditions For Follow-Up

Stop and ask before any follow-up that would:

- run `harness(action="install")` or edit installed hooks, settings, adapters, commands, skills, or
  user-owned files;
- run migration inventory/review/export/status/prioritize/apply, deletion, cleanup, or quarantine
  candidate inspection without exact approval;
- archive, supersede, reject, delete, or otherwise change MemoryItem lifecycle state;
- change ranking, `orient`, public MCP, schema/storage/index, document-index behavior, or
  `handoff(update)` semantics;
- treat this validation baseline as proof of installed harness readiness or migration completion.
