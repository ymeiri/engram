# Brain Harness T238 Telemetry Intent Coverage Follow-Through

Date: 2026-06-04
Status: completed docs-only telemetry feedback follow-through. No runtime, lifecycle, migration,
harness, source, ranking, `orient`, public MCP, schema/storage/index, document-index behavior,
deletion, rollback, old-binary, or user-owned-file change was executed.

## Scope

T238 follows through on the T237 postflight telemetry gap. T237's final current-plan memory recorded
that postflight feedback improved the rolling report but left the confidence gate false because only
two intents had feedback. T238 tests whether scoring a real third-intent verification trace changes
that gate.

This slice writes only telemetry feedback and documentation. It does not execute the pending T233
runtime-refresh packet, mutate lifecycle state, run M6/migration/quarantine actions, or change any
retrieval behavior.

## Research Question

Does submitting legitimate feedback for a `verify_decision` trace, then scoring the material startup
retrieval traces used by this turn, make the rolling `real_session_eval(project="engram", limit=50)`
confidence gate pass, and what would that prove?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The third-intent condition will be satisfied after verification feedback, but the rolling gate may still fail because fresh unscored startup/search traces depress feedback coverage. Any pass would remain weak operational evidence, not completion proof. |
| Null | The third-intent feedback alone makes the confidence gate pass and can be treated as completion evidence. |
| Simpler alternative | Do not score the trace and keep pointing at T237. Rejected because the trace was assessable and materially tested the stated telemetry gap. |
| Failure | The slice chases a numerical pass by scoring marginal traces, treats telemetry as approval for T233/M6/lifecycle work, or changes telemetry formulas instead of recording the measured state. |

## Measurement

Pre-scoring verification trace:

- `telemetry(action="get_trace", trace_id="019e91f8-f329-7280-bb7a-2e63487319ba")`
  confirmed a `verify_decision` `orient` trace for the T237 decision.
- Feedback `019e91fa-f1ad-7f22-959e-80b51b06d687` scored that trace with
  `task_success=true`, usefulness/correctness `5`, noise `2`, `bad_memory_used=false`, and
  used-memory attribution for the T237 current plan, telemetry weak-signal rule, research-method
  rule, Ousterhout preference, and M6 gate.

After that verification-only scoring, `telemetry(action="real_session_eval", project="engram",
limit=50)` generated at `2026-06-04T09:33:25.549794Z` returned:

- `trace_count=50`
- `feedback_count=17`
- `feedback_trace_count=17`
- `feedback_coverage=0.3400000035762787`
- `distinct_intent_count=5`
- `task_failure_count=0`
- `bad_memory_used_count=0`
- `wrong_scope_memory_count=0`
- `missing_context_count=0`
- `confidence_gate.passed=false`
- confidence-gate reason: feedback coverage was 34%, below the 50% threshold.

T238 then scored the material startup/resume retrieval traces used by this turn:

- startup `orient` trace `019e91fa-073c-79c3-90f1-0d7ac840fc2e`
- current-plan search trace `019e91fa-304f-7b71-ac6e-22595bbb87fc`
- architecture search trace `019e91fa-31d6-7e12-9ef7-006d3eed37f4`
- Memory OS plan search trace `019e91fa-3354-77c0-9aae-611b3be8b61d`
- preference search trace `019e91fa-34d1-7260-9626-8721dd37b199`
- risk search trace `019e91fa-3652-7d73-98b1-e932affabfc7`
- resume-session handoff search trace `019e91fa-96fb-7530-90ce-6f425d0b02e7`

After feedback discipline caught up for those traces, `telemetry(action="real_session_eval",
project="engram", limit=50)` generated at `2026-06-04T09:34:52.915401Z` returned:

- `trace_count=50`
- `feedback_count=24`
- `feedback_trace_count=24`
- `feedback_coverage=0.47999998927116394`
- `memory_judgment_coverage=1.0`
- `distinct_intent_count=5`
- `distinct_operation_count=2`
- `task_failure_count=0`
- `bad_memory_used_count=0`
- `wrong_scope_memory_count=0`
- `missing_context_count=0`
- `stale_memory_count=8`
- `external_session_trace_count=18`
- `unspecified_external_session_trace_count=32`
- `external_session_feedback_count=9`
- `confidence_gate.passed=false`
- confidence-gate reason: feedback coverage was 48%, below the 50% threshold.

Fresh companion checks:

- `lint(action="run", limit=20)` still reported wrong-scope active-memory feedback and
  superseded-active lifecycle pressure with `applied_safe_actions=0`.
- `obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")`
  returned `open=[]` and `warnings=[]`.
- `git status --short` showed only the known user-owned untracked root `AGENTS.md`.

## Interpretation

The original two-intent blocker is no longer the current blocker. The rolling report now has
feedback across five intents in the sampled window.

The gate still fails because feedback coverage is 48%. The result is close to the threshold, which
means the numerical pass/fail state is sensitive to recent unscored retrievals. That sensitivity is
itself useful evidence: real-session telemetry is operational hygiene and weak signal, not an
independent completion proof.

The healthy parts of the report still matter: sampled feedback has zero task failures, zero
bad-memory-used records, zero missing-context reports, and zero wrong-scope memory judgments. But
that does not authorize runtime refresh, M6 apply/deferral, lifecycle cleanup, `orient` expansion,
ranking changes, telemetry formula changes, or broad Brain Harness completion.

## Completion Matrix Delta

| Area | State After T238 | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Evidence loop | Partial, rolling gate still false | Feedback discipline raised coverage from 34% to 48%; five intents represented | One or more legitimate feedback-bearing traces could flip the numeric gate, so treat it as weak and rolling |
| Task outcome signal | Healthy in sampled feedback | `task_failure_count=0`, `bad_memory_used_count=0`, `missing_context_count=0` | Agent-reported, rolling, and not a controlled artifact proof |
| External-session joinability | Still sparse | `external_session_trace_count=18/50`, `unspecified_external_session_trace_count=32/50` | T217/T229 source fallback still awaits T233 runtime refresh and host label adoption |
| Lifecycle pressure | Still present | `lint(run, limit=20)` wrong-scope and superseded-active findings, zero safe actions applied | Exact lifecycle approvals only; no `lint apply_safe` from this audit |
| Runtime refresh | Still pending | T237 showed T233 remains fresh; T238 did not repeat or execute T233 first checks | Exact T233 runtime execution remains the product-moving gate |
| M6 migration | Still gated | M6 docs and feedback continue to mark migration completion unresolved | Human dispositions or explicit deferral; no apply/status/prioritize action from T238 |

## Decision

T238 keeps the Brain Harness goal incomplete. Telemetry feedback discipline improved the rolling
window but did not pass the confidence gate. Even a future numerical pass must remain weaker than
source fixtures, cross-harness validation, and controlled dogfood evidence.

The next product-moving gate remains exact T233 runtime refresh/live validation. M6 migration and
lifecycle cleanup remain separate explicit gates.

## Validation

Validation for this docs-only slice:

- actual Engram `orient`, direct searches, work context, handoff, repo docs, and git state read
  before planning;
- telemetry feedback submitted for the assessable `verify_decision` trace and material startup
  retrieval traces;
- two read-only `telemetry(action="real_session_eval", project="engram", limit=50)` reports;
- read-only `lint(action="run", limit=20)` and obligation doctor checks;
- `git diff --check`;
- exact document indexing for this report and `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`;
- document-search visibility probe for T238;
- post-commit `orient`, obligation doctor, current-plan capture, and telemetry feedback.

No Rust build or test is required because T238 changes documentation only and does not touch
binary-relevant source.
