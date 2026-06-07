# Brain Harness T257 Post-T256 Telemetry Window Correction

Date: 2026-06-04
Status: completed docs-only telemetry correction. No native Claude run, hook or settings edit,
harness install, lifecycle archive, `lint apply_safe`, M6/migration/quarantine action,
ranking/`orient`, public MCP, schema/storage/index, document-index behavior change, branch
reconciliation, deletion, rollback, force-kill, runtime refresh, legacy simplification, or
user-owned-file change was executed.

## Scope

T257 corrects the startup-facing completion matrix after post-T256 telemetry feedback changed the
rolling 20-trace window. T256 accurately recorded the telemetry reports generated before the T256
commit. After T256 current-plan capture and trace feedback, a newer 20-trace eval kept high
coverage and clean outcomes but failed the confidence gate because only two intents had feedback in
that small rolling window. The 50-trace window still passed.

This slice updates only:

- `docs/BRAIN_HARNESS_ARCHITECTURE.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- this report

## Research Question

How should the matrix describe telemetry after the latest post-T256 feedback made the 20-trace
window fail the intent-feedback subgate while the 50-trace window still passes?

## Evidence

- T256 git commit `f16ea57` was followed by current-plan MemoryItem
  `019e92af-e79d-7ad1-805e-9dd07766eb49`.
- Post-T256 lean `orient` trace `019e92b0-098e-7cc1-9da1-2fa67adb64ad` returned the T256 current
  plan first and reported no open obligations.
- Feedback records `019e92b0-471b-7822-8ee6-946483fbe502` and
  `019e92b0-4723-7753-bd38-bbc177cb0341` scored the material T256 search/orient traces.
- `telemetry(action="real_session_eval", project="engram", limit=20)` generated at
  `2026-06-04T12:51:26.859552Z` reported `feedback_coverage=0.949999988079071`,
  `task_failure_count=0`, `bad_memory_used_count=0`, `wrong_scope_memory_count=0`,
  `missing_context_count=0`, but `confidence_gate.passed=false` because only two intents had
  feedback in the 20-trace window.
- `telemetry(action="real_session_eval", project="engram", limit=50)` generated at
  `2026-06-04T12:51:26.931086Z` reported `feedback_coverage=0.9399999976158142`, four intents,
  clean outcome counters, and `confidence_gate.passed=true`.
- `git status --short --branch` still showed only the known user-owned untracked root `AGENTS.md`.

## Decision

Update the matrix to say telemetry remains sampled and window-sensitive. The 50-trace window is the
current passing confidence window. The latest 20-trace window has high feedback coverage and clean
outcome counters but does not currently pass the confidence gate because intent-feedback diversity
is too narrow. This correction does not change the core completion gates: M6, lifecycle,
prompt-bearing native Claude execution, effective hook visibility, host-label adoption, and branch
synchronization remain incomplete.

## Validation

Validation for this docs-only correction:

- post-T256 `orient` verification;
- post-T256 telemetry evals for `limit=20` and `limit=50`;
- `git status --short --branch`;
- `git diff --check`;
- exact document indexing for this report, `docs/BRAIN_HARNESS_ARCHITECTURE.md`, and
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`;
- document-search visibility for T257;
- `obligations(action="doctor", project="engram")`;
- focused commit with only intended repo docs.
