# Brain Harness T260 Branch Reconciliation Plan

Date: 2026-06-04
Status: completed read-only branch reconciliation plan. This slice inspected current source,
upstream source, store query support, tests, and Claude Bridge critique. It did not push, pull,
rebase, merge, set upstream, publish a PR, edit code, edit harness files, run native Claude, mutate
lifecycle or M6 state, refresh runtime, delete, rollback, force-kill, or change user-owned files.

## Research Question

Given T259's fresh `origin/main...HEAD` state of `2 372` and predicted telemetry conflicts, what is
the smallest safe reconciliation strategy for the next implementation slice?

## Hypotheses

- Preferred: the current branch already subsumes upstream commit `711c736` with a deeper telemetry
  implementation, so future reconciliation should merge `origin/main` into this branch, preserve
  current telemetry code where it is a strict superset, and validate with telemetry and workspace
  tests.
- Null: upstream has behavior that current HEAD lacks, so preserving HEAD in conflicts would drop
  required functionality.
- Simpler alternative: rebase the branch onto `origin/main`. Rejected for now because the branch is
  372 commits ahead and the overlap is in files changed repeatedly by this branch.
- Failure: auto-merged files or schema/query differences create type-correct but semantically wrong
  telemetry behavior.

## Source Findings

- Current HEAD already defines `RealSessionEvalAppliedFilters` in
  `engram-core/src/telemetry.rs` and includes it in `RealSessionEvalReport`.
- Current `TelemetryService::real_session_eval_report_scoped(limit, project, scenario_id, arm)`
  uses `TelemetryRepo::list_traces_scoped` plus `list_feedback_for_traces`, so scoped reports apply
  the limit after database-level scope filtering and anchor feedback to the sampled trace set.
- Current `TelemetryRepo` stores `project`, defines `idx_trace_project`, and supports scoped trace
  queries over project, scenario, arm, and intent.
- Current MCP `telemetry_new` passes `request.project`, `request.scenario_id`, and `request.arm` to
  `real_session_eval_report_scoped`.
- Current tests include `mcp_real_session_eval_reports_applied_filters`,
  `scoped_real_session_eval_applies_limit_after_scope_filters`, and coverage tests for feedback
  trace counts, outcome trace counts, external-session feedback counts, and memory-judgment
  coverage.
- Upstream `711c736` adds an older, smaller version of the same feature: applied filters on
  real-session eval reports, project filtering, and a focused MCP applied-filter test.

Conclusion: current HEAD appears to subsume upstream `711c736`, but this must be validated during
the actual merge. The untested edge cases to keep visible are empty-string filters, no-match
filters, and any auto-merged core/MCP field-chain mismatch.

## Reconciliation Plan

Recommended future implementation slice:

1. Start from a clean working tree that still excludes root `AGENTS.md`.
2. Re-run `git fetch origin`, `git status --short --branch`, `git rev-list --left-right --count
   origin/main...HEAD`, and `git merge-base origin/main HEAD` so the merge target is fresh.
3. Use a regular merge of `origin/main` into `yuval.meiri/memory-os-phase0`, not a 372-commit
   rebase. Do not use a broad `-s ours` merge strategy; it would hide what was reviewed.
4. For conflicts in `engram-index/src/telemetry.rs` and
   `engram-tests/tests/telemetry_tests.rs`, preserve the current HEAD implementation where it
   subsumes upstream, but explicitly verify that upstream's applied-filter behavior remains covered
   by tests.
5. Inspect auto-merged `engram-core/src/telemetry.rs` and `engram-mcp/src/tools.rs` as a field
   chain: `RealSessionEvalAppliedFilters` fields, service signature, repo query filters, MCP
   request passthrough, and JSON report output must agree.
6. Confirm store schema/query support remains aligned: `project` is persisted, `idx_trace_project`
   exists, scoped trace query conditions bind project/scenario/arm/intent correctly, and feedback
   is selected from sampled trace IDs.
7. Leave remote backup, upstream setup, PR publication, and push policy as separate remote-mutation
   decisions after the merge validates.

## Validation Gate

The implementation slice should not be considered complete until at least:

- `cargo fmt --all --check`
- `cargo test -p engram-tests --test telemetry_tests`
- `cargo test -p engram-tests`
- `cargo check --workspace`
- `cargo clippy --all-targets -- -D warnings`, unless a pre-existing lint backlog is documented
  before the merge
- `git diff --check`
- obligation detection/doctor, exact docs indexing, current-plan capture, orient check, and
  telemetry feedback

If full integration tests or clippy are too expensive for the first pass, record that as residual
risk rather than treating targeted telemetry tests as complete branch reconciliation evidence.

## Decision

T260 does not reconcile the branch. It converts T259's conflict discovery into an implementation
plan. The next branch-sync implementation should merge, not rebase; preserve current telemetry
semantics where they subsume upstream; inspect auto-merged semantic edges; and validate broadly
before any push, upstream setup, PR publication, or backup-push policy decision.
