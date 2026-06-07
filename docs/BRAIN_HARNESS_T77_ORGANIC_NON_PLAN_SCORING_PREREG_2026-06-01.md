# Brain Harness T77 Organic Non-Plan Scoring Pre-Registration

Status: Completed; no scoring feedback submitted
Date: 2026-06-01
Scope: Existing organic non-`plan_work` telemetry traces, excluding T76-contaminated trace bodies

This slice is evidence-quality work only. It must not run M6 inspection, migration status,
prioritize, review apply, candidate decisions, deletion, lifecycle mutation, document indexing,
harness writes, schema/storage/index changes, ranking changes, public MCP changes, or `orient`
payload changes.

## Research Question

Can existing organic non-plan telemetry traces provide independently assessable feedback evidence
after excluding T76-viewed trace bodies, without creating synthetic tasks or treating intent
coverage as production readiness?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | At least one non-plan intent has enough unseen, independently assessable pre-T76 traces to submit honest feedback while keeping confidence-gate output diagnostic only. |
| Null | Existing unseen non-plan traces are too sparse or lack enough task/outcome context; scarcity or instrumentation limits should remain the result. |
| Simpler alternative | Create new read-only non-plan tasks to seed coverage. |
| Failure | The audit loosens exclusions or assessability after seeing trace bodies, turning retrospective scoring into confidence-gate gaming. |

## Consultation Summary

AI Council recall again surfaced the prior decision that flat `BrainHarnessIntent` is weak
secondary metadata and passive telemetry/feedback coverage cannot prove Brain Harness readiness.

AI Council broadcast agreed T77 is sound only with a frozen cutoff, explicit contamination
blacklist, deterministic sampling, strict assessability criteria, reason-coded exclusions, and a
hard rule that any confidence-gate pass remains diagnostic. Claude Bridge agreed, adding that the
selection policy must be written before any new `list_traces` call because the current surface
returns full trace bodies, not metadata-only rows.

## Frozen Boundaries

Cutoff:

- Exclude every trace with `created_at >= 2026-06-01T09:28:38.037734Z`, the created timestamp of
  T76 startup orient trace `019e8283-8595-7ee3-b271-3dbedcc81e86`.

T76-contaminated trace IDs:

- `019e8283-e111-78c2-831b-c76854bb6a06`
- `019e8272-2470-7121-9ee0-11e837499355`
- `019e8271-a25e-7351-840d-d9daba821aba`
- `019e8271-05f0-7130-a5e0-6721d9da1d9d`
- `019e8240-0096-7453-8c7e-cb68e7a0ff3d`
- `019e8272-22ea-71e1-bca1-502b0e2e0585`
- `019e8271-a1ab-75b0-b9e7-3f0eaa467427`
- `019e8271-0538-79d2-bb9e-38f3c2acd04f`

Target intents:

- `follow_user_preference`
- `verify_decision`

## Fixed Inspection Policy

The current telemetry surface does not provide a metadata-only trace listing. Therefore the first
fixed-window `list_traces` call is treated as trace-body inspection. The audit will not call
`list_traces` repeatedly to shop for better candidates.

For each target intent, run exactly one fixed-window call:

- `telemetry(action="list_traces", project="engram", intent=<intent>, limit=20)`

For every trace returned in that window:

1. Exclude if its ID is in the T76 contamination list.
2. Exclude if `created_at` is at or after the cutoff.
3. Otherwise classify it as `assessable` or `unassessable` using the rubric below.
4. Record all classifications, including misses and unassessable cases.

If fewer than three assessable traces remain for an intent within the fixed window, record
`INSUFFICIENT_SAMPLE` for that intent and do not submit feedback for that intent. Do not increase
the limit, search other prompts, or widen the intent list during T77.

## Assessability Rubric

A trace is independently assessable only when the trace body itself plus allowed frozen context can
answer both questions:

1. What behavior was expected?
2. Did the retrieved memory materially support that behavior without stale, wrong-scope, or
   misleading memory use?

Allowed context:

- The trace body.
- The returned MemoryItem bodies for IDs in the trace.
- Repo docs and git output that existed before the trace timestamp.
- This conversation only for traces from this current thread when the relevant user request and
  assistant action are visible in the transcript.

Not allowed:

- T76-viewed trace bodies as context for another trace.
- Later outcomes created after the trace timestamp.
- Inferences from the intent label alone.
- Scoring a trace because it would improve intent coverage.
- Changing the rubric or threshold after seeing candidate trace bodies.

Reason codes:

- `POST_CUTOFF`
- `T76_CONTAMINATED`
- `NO_TASK_CONTEXT`
- `NO_OUTCOME_CONTEXT`
- `AMBIGUOUS_EXPECTED_BEHAVIOR`
- `ASSESSABLE_RETRIEVAL_ONLY`
- `ASSESSABLE_TASK_OUTCOME`

Only `ASSESSABLE_TASK_OUTCOME` traces may receive feedback with `task_success`,
`preference_adhered`, `repeated_context_questions`, or `bad_memory_used`. Retrieval-only traces may
be reported in the audit, but T77 must not submit outcome feedback for them.

## Feedback And Stop Rules

- Do not call `real_session_eval` during trace inspection.
- Submit feedback only after all fixed-window candidates for both target intents are classified.
- If an intent has fewer than three `ASSESSABLE_TASK_OUTCOME` traces, submit no feedback for that
  intent and record scarcity.
- If no intent has at least three `ASSESSABLE_TASK_OUTCOME` traces, submit no T77 scoring feedback
  at all and record the blocker.
- If feedback is submitted, call `real_session_eval(project="engram", limit=50)` once at the end.
- Any `confidence_gate.passed=true` result is diagnostic only and does not approve M6, lifecycle,
  harness, ranking, schema/storage/index, document-index, public MCP, or `orient` work.

## Planned Evidence Output

T77 will produce a report table with counts by intent:

- total returned in fixed window
- post-cutoff exclusions
- T76-contaminated exclusions
- unassessable counts by reason
- retrieval-only assessable count
- task-outcome assessable count
- feedback submissions

The valid success state is not "gate passed." The valid success state is a trustworthy answer to
whether existing unseen organic non-plan traces can support honest outcome feedback today.

## Fixed-Window Inspection Results

T77 ran the two pre-registered fixed-window calls exactly once:

- `telemetry(action="list_traces", project="engram", intent="follow_user_preference", limit=20)`
- `telemetry(action="list_traces", project="engram", intent="verify_decision", limit=20)`

No `real_session_eval` call was made during inspection.

### Summary Table

| Intent | Returned | Post-cutoff | T76-contaminated | Older unseen | Retrieval-only assessable | Task-outcome assessable | Feedback submitted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `follow_user_preference` | 20 | 2 | 4 | 14 | 14 | 0 | 0 |
| `verify_decision` | 20 | 1 | 3 | 16 | 16 | 0 | 0 |

### Classification Notes

`follow_user_preference`:

- Post-cutoff: current T77 search trace `019e8299-883c-7d31-af69-aecabe0f7926` and T76 search
  trace `019e8283-e111-78c2-831b-c76854bb6a06`.
- T76-contaminated older traces: `019e8272-2470-7121-9ee0-11e837499355`,
  `019e8271-a25e-7351-840d-d9daba821aba`,
  `019e8271-05f0-7130-a5e0-6721d9da1d9d`, and
  `019e8240-0096-7453-8c7e-cb68e7a0ff3d`.
- The 14 older unseen traces were retrieval-only assessable: their queries asked for user
  preferences or commit discipline and generally returned relevant preference/rule memory. They did
  not include enough downstream assistant response or task outcome context to score
  `task_success`, `preference_adhered`, `repeated_context_questions`, or `bad_memory_used`.

`verify_decision`:

- Post-cutoff: current T77 risk/gate search trace `019e8299-89d2-7511-8f6d-623f3bf56d73`.
- T76-contaminated older traces: `019e8272-22ea-71e1-bca1-502b0e2e0585`,
  `019e8271-a1ab-75b0-b9e7-3f0eaa467427`, and
  `019e8271-0538-79d2-bb9e-38f3c2acd04f`.
- The 16 older unseen traces were retrieval-only assessable: their queries asked about concrete
  gates or decisions and often returned relevant gate/migration/harness context. They did not
  include enough downstream assistant response or task outcome context to score outcome fields
  without importing later audit reports or evaluator memory.

### Result

Under the pre-registered rules, neither target intent had three `ASSESSABLE_TASK_OUTCOME` traces.
T77 therefore submitted no scoring feedback and did not call `real_session_eval` at the end.

This is useful negative evidence: existing organic non-plan trace bodies can support retrieval
quality spot checks, but they are not sufficient by themselves for honest historical outcome
scoring. The next non-gated evidence slice should either add a metadata-only trace listing and/or
capture richer task-outcome links for future traces, or pre-register a controlled non-synthetic
task set whose outcomes are observable from the current transcript. This result does not authorize
M6, lifecycle writes, harness writes, ranking changes, schema/storage/index changes, document-index
actions, public MCP changes, or `orient` expansion.
