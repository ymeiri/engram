# Brain Harness T81 Feedback Outcome Pointer Proxy Audit

Status: Complete; read-only telemetry proxy audit only
Date: 2026-06-01
Scope: Recent `AgentFeedback.note` and `missing_context` outcome evidence pointers

This slice does not change source behavior, telemetry schema, storage, indexes, public MCP request
parameters, ranking, harness adapters/hooks, migration, lifecycle state, document indexing, or the
`orient` payload. It submits no outcome feedback for the sampled rows.

## Research Question

Do recent Engram feedback rows already contain enough free-text outcome evidence pointers to
justify asking for a future structured outcome-evidence implementation?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Recent notes mostly describe retrieval/process use rather than durable outcome evidence; a future implementation needs either a controlled artifact path or a more explicit collection pilot. |
| Null | Existing `note` and `missing_context` fields already carry enough inspectable outcome pointers to support structured provenance work. |
| Simpler alternative | Treat any feedback row with `task_success=true` as outcome evidence. |
| Failure | Infer outcome evidence from positive self-report without a visible transcript, commit, test, user review, or controlled outcome artifact pointer. |

## Measurement

T81 sampled the latest 20 project feedback rows using:

- `telemetry(action="list_feedback", project="engram", limit=20)`

The audit classified only the returned feedback text and fields. It did not inspect hidden
transcripts, infer missing outcomes, mutate feedback, or run `real_session_eval`.

A representative T78 trace was checked with:

- `telemetry(action="get_trace", trace_id="019e82a7-c86e-7aa3-a7fd-109edf7a9672")`

That trace had `intent="follow_user_preference"` and returned memory/result IDs, but no
`scenario_id` or `arm`. This matters because even a controlled T78 row still depends on external
documentation for controlled-task grouping and outcome classification.

## Results

| Measure | Count | Interpretation |
| --- | ---: | --- |
| Feedback rows sampled | 20 | Latest project-scoped feedback rows returned by MCP telemetry. |
| Rows with non-empty `note` | 20 | Notes are consistently populated. |
| Rows with non-empty `missing_context` | 0 | Missing-context is not currently carrying outcome evidence in this window. |
| Rows with task outcome fields populated | 20 | All sampled rows report `task_success`, `preference_adhered`, `repeated_context_questions`, and `bad_memory_used`. |
| Rows with explicit `ASSESSABLE_TASK_OUTCOME` label | 4 | These are the four T78 rows and are the only clear outcome-assessability labels in the sample. |
| Rows with durable transcript/commit/test/user-review artifact pointer in free text | 0 | No sampled row carries a structured or directly inspectable outcome evidence reference. |
| Rows that should be treated as retrieval/process feedback without external evidence | 16 | Positive outcome fields exist, but notes describe retrieval order, gate preservation, or stale-memory rejection rather than downstream task outcome evidence. |

## Classification

The 20-row sample supports the T80 decision:

- `task_success=true` is common, but it is mostly self-report.
- `note` is useful for explaining retrieval and attribution decisions.
- `missing_context` is not enough in this sample because it is absent.
- T78's four rows are more assessable only because the surrounding T78 report and transcript state
  were pre-registered and reviewed; the feedback rows themselves do not encode a durable evidence
  pointer.
- No sampled row provides a machine-readable link to transcript, commit, test output, user review,
  or controlled outcome artifact.

## Decision

Do not request schema/API work yet.

The latest feedback window shows good note hygiene but weak outcome-evidence linkage. A future
`outcome_evidence` field or controlled-outcome storage path would need a collection mechanism, not
just new fields. The current evidence supports one of two next steps:

1. Pre-register a controlled outcome artifact pilot that writes a document artifact linking trace
   IDs to transcript/repo evidence, without changing storage or public MCP shape.
2. Run a larger read-only proxy audit over more feedback rows and trace bodies to estimate how often
   existing notes can be independently linked to outcome artifacts.

Neither path authorizes schema/storage/index changes, public MCP changes, harness writes,
migration, lifecycle mutation, ranking changes, document indexing, or `orient` expansion.
