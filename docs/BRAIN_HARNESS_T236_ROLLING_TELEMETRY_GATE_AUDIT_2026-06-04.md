# Brain Harness T236 Rolling Telemetry Gate Audit

Date: 2026-06-04
Status: completed read-only telemetry audit. No runtime, lifecycle, migration, harness, or source
change was executed.

## Scope

This slice records the current `telemetry(action="real_session_eval", project="engram", limit=50)`
state after T235. It also adds the missing tail matrix note for T235.

T236 does not change telemetry formulas, ranking, `orient`, public MCP, schema/storage/index,
document-index behavior, lifecycle state, M6/migration/quarantine state, harness files/settings/
hooks/adapters, runtime configuration, native Claude state, deletion, rollback, old-binary state, or
user-owned files.

## Research Question

Does the current rolling telemetry report prove the evidence loop is complete enough for the Brain
Harness goal, or does it remain a partial signal that must keep completion gated?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The latest rolling telemetry report remains useful but incomplete: it shows zero task failures and zero bad-memory-used records, but confidence still fails because feedback covers too few intents and external-session joinability is sparse. |
| Null | The latest rolling report passes all confidence criteria and can support completion claims without further evidence. |
| Simpler alternative | Leave the older T72 telemetry snapshot in the matrix and do not add another audit. |
| Failure | The audit treats rolling feedback as proof of migration readiness, runtime refresh, lifecycle cleanup, or broad Brain Harness completion. |

## Measurement

Read-only evidence:

- `telemetry(action="real_session_eval", project="engram", limit=50)` generated at
  `2026-06-04T09:20:10.384236Z`.
- `trace_count=50`.
- `feedback_trace_count=33`.
- `feedback_coverage=0.6600000262260437`.
- `memory_judgment_coverage=1.0`.
- `task_failure_count=0`.
- `bad_memory_used_count=0`.
- `stale_memory_count=3`.
- `wrong_scope_memory_count=0`.
- `missing_context_count=0`.
- `external_session_trace_count=4`.
- `unspecified_external_session_trace_count=46`.
- `distinct_intent_count=2`.
- `confidence_gate.passed=false` because feedback covers only two intents and the gate requires at
  least three intents with feedback.
- `lint(action="run", limit=30)` still reported wrong-scope active-memory signals and
  superseded-active lifecycle debt. It applied zero safe actions.
- `git status --short` showed only the known user-owned untracked root `AGENTS.md`.

## Interpretation

The current rolling report is better than a failing task-outcome state: recent sampled traces with
feedback have no task failures, no bad-memory-used records, no missing-context reports, and no
wrong-scope memory judgments. That is useful operational evidence.

It is not completion proof. The confidence gate is false because intent diversity is too narrow,
external-session labels remain sparse, and lifecycle lint still reports active stale/wrong-scope and
superseded-memory pressure. The report therefore cannot authorize M6 apply, lifecycle cleanup,
runtime refresh, broad ranking changes, `orient` expansion, harness writes, or a Brain Harness
completion claim.

## Completion Matrix Delta

| Area | State After T236 | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Evidence loop | Partial, current rolling gate false | `real_session_eval` report generated `2026-06-04T09:20:10.384236Z` | Need feedback across at least three intents and stronger host joinability before completion claims |
| Task outcome signal | Healthy in sampled feedback | `task_failure_count=0`, `bad_memory_used_count=0` | Rolling window only; not a substitute for fixed scenarios or user approval |
| External-session joinability | Sparse | `external_session_trace_count=4/50`, `unspecified_external_session_trace_count=46` | T217/T229 source fallback awaits T233 runtime refresh and real host labels |
| Lifecycle pressure | Still present | `lint(run, limit=30)` wrong-scope and superseded-active findings, zero safe actions applied | Exact lifecycle approvals only; no `lint apply_safe` from this audit |
| M6 migration | Still gated | Confidence gate false and M6 docs remain undecided | Human dispositions or explicit deferral; apply still needs dry-run, rollback, and approval |

## Validation

Validation for this docs-only slice:

- read-only `telemetry(real_session_eval)` report
- read-only `lint(run, limit=30)` report
- `git diff --check`
- exact document indexing for this report and `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- document-search visibility probe for T236
- post-commit `orient` and obligation checks

No Rust build or test is required because T236 changes documentation only.
