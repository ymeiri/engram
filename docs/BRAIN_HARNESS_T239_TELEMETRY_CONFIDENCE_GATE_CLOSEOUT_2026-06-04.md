# Brain Harness T239 Telemetry Confidence Gate Closeout

Date: 2026-06-04
Status: completed docs-only telemetry closeout after T238 feedback. No runtime, lifecycle,
migration, harness, source, ranking, `orient`, public MCP, schema/storage/index, document-index
behavior, deletion, rollback, old-binary, or user-owned-file change was executed.

## Scope

T239 records the rolling telemetry state after the T238 closeout feedback was submitted and the
next startup `orient` trace entered the sample window. T238 accurately recorded its two reports at
the time: the final T238 report failed the confidence gate at 48% feedback coverage. After T238
commit, indexing, current-plan capture, and closeout feedback, the sampled report moved to the gate
threshold.

This slice writes only documentation. It does not execute the pending T233 runtime-refresh packet,
mutate lifecycle state, run M6/migration/quarantine actions, or change retrieval behavior.

## Research Question

After T238 closeout feedback, does the current rolling
`telemetry(action="real_session_eval", project="engram", limit=50)` report pass the conservative
confidence gate, and what does that prove for the Brain Harness completion matrix?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The rolling report may now pass at or near the threshold because legitimate closeout feedback caught up with material retrieval traces. A pass is useful operational evidence, not completion proof, and remains sensitive to fresh unscored traces. |
| Null | Once the telemetry gate passes, telemetry can be marked complete and used to authorize runtime refresh, lifecycle cleanup, or M6 work. |
| Simpler alternative | Leave the matrix at T238's 48% result. Rejected because the next report was observed after legitimate feedback and changes the current gate state. |
| Failure | The slice treats a rolling numeric pass as broad quality proof, changes telemetry formulas, or uses the report to bypass exact approval gates. |

## Measurement

Observed immediately after T238 closeout feedback, before this report was written:

- `telemetry(action="real_session_eval", project="engram", limit=50)` generated at
  `2026-06-04T09:39:09.473084Z` returned `feedback_count=26`,
  `feedback_coverage=0.5199999809265137`, `distinct_intent_count=5`, zero task failures, zero
  bad-memory-used records, zero wrong-scope memory judgments, zero missing-context reports, and
  `confidence_gate.passed=true`.

Fresh recheck after the post-compaction startup `orient` trace entered the rolling window:

- `telemetry(action="real_session_eval", project="engram", limit=50)` generated at
  `2026-06-04T09:40:20.573429Z` returned:
  - `trace_count=50`
  - `feedback_count=25`
  - `feedback_trace_count=25`
  - `feedback_coverage=0.5`
  - `memory_judgment_coverage=1.0`
  - `distinct_intent_count=4`
  - `distinct_operation_count=2`
  - `external_session_trace_count=18`
  - `unspecified_external_session_trace_count=32`
  - `external_session_feedback_count=9`
  - `task_success_count=25`
  - `task_failure_count=0`
  - `preference_adhered_count=25`
  - `preference_violated_count=0`
  - `repeated_context_question_count=0`
  - `bad_memory_used_count=0`
  - `stale_memory_count=8`
  - `wrong_scope_memory_count=0`
  - `missing_context_count=0`
  - `confidence_gate.passed=true`

The gate reasons list was empty. The report still recommended setting real external session IDs
and adding scenario/arm labels for controlled eval traces.

Companion evidence remained unchanged from T238:

- `lint(action="run", limit=20)` still reported wrong-scope active-memory feedback and
  superseded-active lifecycle pressure with no safe action applied.
- `obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")`
  stayed clean.
- `git status --short` showed only the known user-owned untracked root `AGENTS.md`.

## Interpretation

The current rolling telemetry report now passes the conservative gate, but only exactly at the 50%
coverage threshold after one fresh unscored startup trace entered the sample. That is the important
engineering signal: feedback discipline is now good enough for the sampled window, while the pass is
still rolling, agent-assessed, and sensitive to normal work.

The healthy outcome counters are useful: no sampled feedback reports task failure, harmful memory
use, wrong-scope memory, missing context, repeated context questions, or preference violations.
However, the report is not a substitute for source fixtures, installed-runtime validation,
cross-harness behavioral proof, or human decisions on M6 and lifecycle cleanup.

## Completion Matrix Delta

| Area | State After T239 | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Evidence loop | Current rolling gate passes at threshold | Fresh report passed with `feedback_coverage=0.5`, `feedback_count=25`, and four feedback intents | Rolling sample can flip with new unscored traces; still weak operational evidence |
| Task outcome signal | Healthy in sampled feedback | `task_failure_count=0`, `bad_memory_used_count=0`, `missing_context_count=0`, `wrong_scope_memory_count=0` | Agent-reported and not controlled artifact proof |
| External-session joinability | Still incomplete | `external_session_trace_count=18/50`, `unspecified_external_session_trace_count=32/50` | T217/T229 runtime fallback still awaits T233; hosts still need real labels |
| Lifecycle pressure | Still present | T238 lint evidence still shows wrong-scope and superseded-active findings with no safe action applied | Exact lifecycle approvals only; no `lint apply_safe` from this audit |
| Runtime refresh | Still pending | Telemetry pass does not install source changes or validate live runtime | Exact T233 runtime execution remains the product-moving gate |
| M6 migration | Still gated | T239 does not inspect, decide, apply, or defer M6 candidates | Human dispositions or explicit deferral remain required |

## Decision

T239 updates the current completion matrix from "telemetry gate false at 48%" to "telemetry gate
currently passes at the 50% threshold." The Brain Harness goal remains incomplete because runtime
refresh/live validation, M6 disposition or deferral, and gated lifecycle cleanup remain unresolved.

The next product-moving gate remains exact T233 runtime refresh/live validation. T239 does not
authorize T233 execution, M6 actions, lifecycle archive, harness writes, or any source behavior
change.

## Validation

Validation for this docs-only slice:

- actual Engram `orient`, direct search context, repo docs, telemetry, lint, obligation, and git
  state were reviewed before planning;
- two post-T238 telemetry reports were compared, with the fresh report anchoring the current state;
- no source or runtime files were edited;
- planned validation is `git diff --check`, exact document indexing for this report and
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`, document-search visibility, commit, current-plan
  capture, obligation doctor, and telemetry feedback.

No Rust build or test is required because T239 changes documentation only and does not touch
binary-relevant source.
