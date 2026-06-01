# Brain Harness T80 Outcome Link Decision Packet

Status: Complete; read-only source and design audit only
Date: 2026-06-01
Scope: Outcome assessability after T77, T78, and T79

This slice does not change source behavior, telemetry schema, storage, indexes, public MCP request
parameters, ranking, harness adapters/hooks, migration, lifecycle state, document indexing, or the
`orient` payload.

## Research Question

What is the smallest non-gated design decision that improves Engram's ability to reason honestly
about whether retrieved memory improved real agent outcomes?

This follows three recent facts:

- T77 showed older organic `follow_user_preference` and `verify_decision` traces were
  retrieval-only assessable, with no honest downstream task outcome context.
- T78 showed controlled current-work tasks can be outcome-assessable when transcript/repo state
  provides a visible link from retrieval to result.
- T79 showed Claude Bridge project-harness replication failed before Engram traces existed because
  the allowed Engram tools were unavailable.

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Keep real-session telemetry lightweight and classify outcome evidence outside the hot path; define a separate controlled-outcome link contract for future implementation. |
| Null | Existing `AgentFeedback` fields are already sufficient if agents write better notes, so no design distinction is needed. |
| Simpler alternative | Add documentation only saying `task_success` is weak self-report. |
| Failure | Add schema/API fields before proving they will be populated, or blur weak real-session feedback with independently judged controlled outcomes. |

## Measurement Before Implementation

T80 measures only design readiness from existing evidence:

- inspect current trace, feedback, real-session report, and controlled eval source;
- compare source behavior with T77/T78/T79 evidence;
- consult AI Council and Claude Bridge for design blind spots;
- produce a decision packet with no code or schema changes.

Success is a concrete future-facing contract: what existing fields can honestly prove, what they
cannot prove, and what evidence must exist before any future implementation.

## Completion Matrix

| Area | Current state | T80 judgment |
| --- | --- | --- |
| Runtime retrieval traces | `BrainHarnessTrace` stores operation, intent, scenario, arm, query, project, returned IDs, latency, warnings, and timestamps. | Implemented for retrieval/process telemetry. It does not store downstream outcome evidence. |
| Agent feedback | `AgentFeedback` stores memory attribution, missing context, scores, task outcome fields, suggested changes, and notes. | Implemented as weak agent-reported feedback. It does not record who judged an outcome or where the outcome evidence is. |
| Real-session eval | `real_session_eval` aggregates outcome fields and memory judgments, with a conservative gate. | Useful monitoring signal. It cannot distinguish self-report from transcript-visible or independent judgment. |
| Controlled eval schema | `brain_harness_eval.rs` has stricter controlled outcomes and rejects `EvalJudge::UsingAgent`. | Validated as the right strong-evidence direction. It is separate from live telemetry. |
| Organic historical scoring | T77 found zero task-outcome assessable traces in the sampled old windows. | Missing outcome links remain the blocker. |
| Controlled current-work scoring | T78 produced four `ASSESSABLE_TASK_OUTCOME` traces from transcript-visible outcomes. | Validated for prospective controlled tasks only. |
| Cross-harness replication | T79 failed before trace generation because Claude Bridge did not expose allowed Engram tools. | Risky/partial. Treat as harness tool-exposure evidence, not retrieval evidence. |

## Source Audit

`engram-core/src/telemetry.rs` defines `BrainHarnessTrace` as the retrieval/process event. It knows
what was asked and what IDs were returned, but not what happened after the retrieval.

`AgentFeedback` has task outcome fields:

- `task_success`
- `preference_adhered`
- `repeated_context_questions`
- `bad_memory_used`

Those fields are useful, but they are agent-reported. The struct does not include a judgment source,
evidence reference, transcript pointer, or outcome artifact link.

`engram-index/src/telemetry.rs` treats feedback as outcome-bearing when any of those four fields is
present. `real_session_eval` aggregates the counts, but it does not know whether the outcome came
from self-report, transcript inspection, human review, an eval agent, or an automated harness.

`engram-core/src/brain_harness_eval.rs` already defines the stricter model for controlled evidence:
`BrainHarnessEvalOutcome` includes an `EvalJudgment`, and validation rejects
`EvalJudge::UsingAgent`. This source boundary is important: strong outcome evidence exists as a
controlled-eval concept, not as a property of ordinary real-session feedback.

## Consultation Summary

AI Council recall found no exact prior decision for this T80 outcome-link question. AI Council
broadcast agreed on the main direction:

- do not add `AgentFeedback` / `TelemetryRequest` fields in T80;
- preserve real-session telemetry as weak retrieval/process evidence;
- define outcome assessability tiers and a rubric now;
- use a separate controlled-outcome artifact/linking process as the likely future direction;
- require evidence that structured provenance will be populated before schema/API work.

Claude Bridge critique agreed with the no-schema stance but raised two blind spots:

- a tier taxonomy is decorative unless it includes field roles and consumer-facing criteria;
- requiring proof of future field population can be circular unless a proxy audit or pilot is
  defined.

T80 incorporates those critiques below.

## Decision

Do not change live telemetry or public MCP request shape now.

For current Engram evidence work:

1. Treat `BrainHarnessTrace` as retrieval/process evidence.
2. Treat `AgentFeedback` task outcome fields as weak signals unless the feedback is paired with
   transcript-visible or independently judged evidence.
3. Use `brain_harness_eval.rs` controlled outcome types as the stronger evidence model.
4. Define a future controlled-outcome link contract before adding storage/API fields.

## Field Evidence Roles

| Field or source | Honest evidence role now | Not sufficient for |
| --- | --- | --- |
| `BrainHarnessTrace.query`, `returned_memory_ids`, `returned_result_ids` | What the agent asked and what Engram returned. | Whether the task succeeded after retrieval. |
| `used_memory_ids`, `rejected_memory_ids`, `stale_memory_ids`, `wrong_scope_memory_ids` | Agent or evaluator attribution judgment. Useful when corroborated by transcript/repo state. | Independent proof that memory helped or hurt. |
| `missing_context` | Retrieval/process failure clue when it names context the agent expected. | Task failure proof by itself. |
| `task_success`, `preference_adhered`, `repeated_context_questions`, `bad_memory_used` | Weak outcome signal unless externally corroborated. | Strong outcome evidence without a visible or independent judge. |
| `note` | Free-form explanation or pointer. | Machine-readable evidence unless it cites inspectable artifacts. |
| `BrainHarnessEvalOutcome` + `EvalJudgment` | Strong controlled outcome evidence when validated. | Organic real-session coverage by itself. |

## Outcome Assessability Classes

| Class | Definition | Allowed scoring use |
| --- | --- | --- |
| `RETRIEVAL_ONLY` | Retrieval result can be inspected, but downstream task outcome is absent. | Score retrieval relevance/noise only; do not score `task_success`. |
| `SELF_REPORTED_OUTCOME` | Feedback has outcome fields but no inspectable outcome evidence. | Count as weak operational signal only. |
| `TRANSCRIPT_VISIBLE_OUTCOME` | Transcript, repo diff, test output, or other durable context shows what happened after retrieval. | Score task outcome if the assessor cites the evidence. |
| `CONTROLLED_LINKED_OUTCOME` | A controlled outcome artifact links trace ID(s) to an independent human/eval-agent/automated-harness judgment. | Strongest scoring path for Brain Harness claims. |
| `HARNESS_INCONCLUSIVE` | Harness/tooling failed before an assessable retrieval/outcome chain existed. | Record as infrastructure evidence only. |

## Existing Field Rubric

Existing `note` and `missing_context` are sufficient for retrieval/process analysis only when they
name concrete context, memory IDs, task constraints, or failure symptoms.

They are not sufficient for task-outcome scoring unless they point to inspectable evidence, such as:

- a transcript section that shows the final answer or decision;
- a repo diff, commit, or test result created after the retrieval;
- a user correction or acceptance;
- an independently judged controlled outcome record.

If the assessor cannot inspect the downstream outcome, the correct classification is
`RETRIEVAL_ONLY` or `SELF_REPORTED_OUTCOME`, not `ASSESSABLE_TASK_OUTCOME`.

## Future Controlled-Outcome Link Contract

A future implementation should be considered only with explicit approval. The minimum useful
contract is a separate controlled-outcome link, not an expansion of `orient`:

| Field | Purpose |
| --- | --- |
| `outcome_id` | Stable outcome artifact ID. |
| `trace_ids` | One or more `BrainHarnessTrace` IDs being judged. |
| `scenario_id` and `arm` | Controlled-eval grouping, aligned with existing telemetry labels. |
| `judge` and `judge_source` | Human, eval-agent, or automated-harness judgment, not the using agent. |
| `evidence_refs` | Transcript, commit, diff, test, user-review, or report references. |
| `outcome_fields` | Task success, preference adherence, repeated questions, bad-memory use, and notes. |
| `created_at` | Audit timestamp. |

This contract can be implemented as a document artifact, generated report, or storage record later.
Any stored schema, MCP API, or new tool surface requires explicit approval.

## Future Evidence Gates

Before adding `outcome_evidence` fields to `AgentFeedback` or `TelemetryRequest`, run a proxy audit
or pilot that answers:

- What fraction of recent feedback `note` / `missing_context` values already cite outcome-visible
  artifacts?
- How often do self-reported outcomes disagree with transcript-visible or independently judged
  outcomes?
- Can controlled outcome artifacts be linked to trace IDs without adding synchronous work to the
  `orient` hot path?
- Can Claude/native harnesses expose the necessary Engram tools before the outcome-link process is
  used for cross-harness claims?

If those answers are missing, structured provenance fields risk becoming another sparse self-report
channel rather than stronger evidence.

## Next Non-Gated Slice

The next non-gated slice should be a read-only proxy audit of recent `AgentFeedback` notes and
missing-context values. It should sample existing feedback without writes, classify whether each row
contains transcript-visible outcome pointers, and estimate whether a future structured
outcome-evidence field would have meaningful population. This would not authorize schema/API work;
it would decide whether asking for that approval is evidence-backed.
