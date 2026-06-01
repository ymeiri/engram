# Brain Harness T78 Controlled Observable Task Audit

Status: Completed; feedback submitted
Date: 2026-06-01
Scope: Prospective, genuine current-work tasks with transcript-visible outcomes

This slice is evidence-quality work only. It must not run M6 inspection, migration status,
prioritize, review apply, candidate decisions, deletion, lifecycle mutation, document indexing,
harness writes, schema/storage/index changes, ranking changes, public MCP changes, or `orient`
payload changes.

## Research Question

Can prospective, pre-registered, genuine current-work tasks produce independently assessable
non-`plan_work` outcome traces using only existing `orient`, `search`, telemetry, repo state, and
the current transcript?

Secondary question: when outcomes are assessable, can feedback honestly name which returned memory
was used or rejected without importing later context or relaxing the T77 rubric?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Two `verify_decision` and two `follow_user_preference` current-work tasks will produce task-outcome-assessable traces whose outcomes are visible from this transcript and repo state. |
| Null | Even prospective current-work tasks remain retrieval-only or ambiguous because the transcript/repo evidence does not link retrieval to outcome tightly enough. |
| Simpler alternative | Produce only a design note for richer outcome-link instrumentation. |
| Failure | Tasks are selected or reclassified to satisfy intent coverage, feedback is submitted before all tasks are classified, or a confidence-gate result is treated as authorization for gated work. |

## Consultation Summary

AI Council recall surfaced prior guidance that flat `BrainHarnessIntent` is weak secondary
metadata and passive feedback coverage cannot prove Brain Harness confidence. AI Council broadcast
and Claude Bridge both recommended a prospective controlled observable-task audit over a design-only
instrumentation note: it can create real outcome evidence now, while still preserving gates.

The key constraint from Claude Bridge is that the tasks must be genuine current work, not invented
from the intent vocabulary. T78 therefore uses only tasks required by this continuation turn's
startup, validation, preference, and commit discipline.

## Pre-Registered Tasks

Run exactly these four retrieval tasks after this pre-registration is committed:

| Task | Intent | Retrieval call | Why this is genuine current work | Success criterion | Outcome evidence |
| --- | --- | --- | --- | --- | --- |
| T78-V1 | `verify_decision` | `search(project="engram", intent="verify_decision", query="T78 current plan after T77 next non-gated observable task audit", limit=8)` | The goal requires current-plan / next-step retrieval to stay validated after each slice. | Active T77 current-plan memory `019e82a1-affd-7d93-b712-8a08faaa8338` appears above stale repository current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915`. | Search result ordering in the trace and transcript. |
| T78-V2 | `verify_decision` | `orient(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="verify_decision", response_shape="lean", prompt="Verify the active current plan and hard gates before executing T78 controlled observable tasks.")` | `orient` is the hot-path task-boundary entrypoint and must remain compact while surfacing the active plan/gates. | Lean `orient` includes active T77 current-plan memory `019e82a1-affd-7d93-b712-8a08faaa8338` in candidate/top guidance and does not rank stale repository current-plan memory above it. | Lean orient response in the trace and transcript. |
| T78-P1 | `follow_user_preference` | `search(project="engram", intent="follow_user_preference", query="user software design philosophy Ousterhout evidence no unrequested features small slices Engram", limit=8)` | The user explicitly requires Ousterhout design, evidence over confidence, no unrequested features, and small slices. | Reviewed user preference memory `019e6924-256b-7093-b1c5-286ec4d02461` is returned and the T78 implementation remains documentation/evidence-only with no product-surface behavior change. | Search result ordering plus final git diff/stat. |
| T78-P2 | `follow_user_preference` | `search(project="engram", intent="follow_user_preference", query="Engram commit discipline stage only intended files root AGENTS.md untracked leave untouched", limit=8)` | The user explicitly requires commit discipline and root `AGENTS.md` is currently untracked/user-owned. | Commit discipline guidance is returned if available; the T78 commits stage only intended documentation files and leave root `AGENTS.md` untracked. | Search result plus `git status --short`, staged diff, and final status. |

## Classification Rubric

Classify each task after all four retrieval calls complete:

- `ASSESSABLE_TASK_OUTCOME`: the expected behavior and outcome are visible from the trace plus
  current transcript/repo state.
- `ASSESSABLE_RETRIEVAL_ONLY`: retrieval can be judged, but the task outcome or memory
  contribution cannot be judged without importing later or external context.
- `NO_OUTCOME_CONTEXT`: the trace and transcript do not show what happened after retrieval.
- `AMBIGUOUS_EXPECTED_BEHAVIOR`: the success criterion is not concrete enough.
- `GATE_CONFLICT`: assessment or execution would require a gated operation.

Only `ASSESSABLE_TASK_OUTCOME` traces may receive outcome feedback with `task_success`,
`preference_adhered`, `repeated_context_questions`, or `bad_memory_used`.

## Measurement

Report:

- total pre-registered tasks,
- task-outcome assessable count,
- retrieval-only count,
- unassessable counts by reason,
- feedback submissions,
- whether any confidence report was run.

Primary success for this slice is not a confidence-gate pass. Success is a trustworthy answer about
whether prospective genuine tasks can produce assessable non-plan feedback with today's surfaces.

## Stop Rules

- Do not add tasks, expand the intent list, or change success criteria after this file is
  committed.
- Submit feedback only after all four tasks are completed and classified.
- If fewer than two tasks per intent are `ASSESSABLE_TASK_OUTCOME`, submit no feedback for that
  intent.
- If no feedback is submitted, do not run `real_session_eval`.
- If feedback is submitted, run `telemetry(action="real_session_eval", project="engram",
  limit=50)` exactly once at the end.
- Treat any confidence-gate pass as diagnostic only. It does not authorize migration, lifecycle
  writes, harness writes, ranking changes, schema/storage/index changes, document-index actions,
  public MCP changes, or `orient` expansion.

## Execution Results

T78 ran exactly the four pre-registered retrieval tasks after preregistration commit `eee76d2`.
No task was added, removed, or reclassified after execution began.

| Task | Trace | Classification | Result |
| --- | --- | --- | --- |
| T78-V1 | `019e82a7-c5f1-7c73-987c-63f31d105a92` | `ASSESSABLE_TASK_OUTCOME` | Direct search returned active T77 current-plan memory `019e82a1-affd-7d93-b712-8a08faaa8338` first and stale repository current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` lower. |
| T78-V2 | `019e82a7-c6b7-7742-ae61-f244a67bb4c9` | `ASSESSABLE_TASK_OUTCOME` | Lean `orient` returned active T77 current-plan memory first in candidate IDs and Brain Loop top items; stale repository current-plan memory did not outrank it. |
| T78-P1 | `019e82a7-c86e-7aa3-a7fd-109edf7a9672` | `ASSESSABLE_TASK_OUTCOME` | Search returned reviewed user software-design preference `019e6924-256b-7093-b1c5-286ec4d02461` first; T78 remained documentation/evidence-only and did not change product behavior. |
| T78-P2 | `019e82a7-c9ff-7d01-b4cd-8f802044bca8` | `ASSESSABLE_TASK_OUTCOME` | Search returned reviewed commit-discipline preference `019e03be-a9a5-7db2-848d-eb26ef78bcb5` first; preregistration commit staged only the intended T78 doc file and `git status --short` still showed only root `AGENTS.md` as untracked. |

Summary:

| Metric | Count |
| --- | ---: |
| Pre-registered tasks | 4 |
| Task-outcome assessable | 4 |
| Retrieval-only assessable | 0 |
| Unassessable | 0 |
| Feedback submitted | 4 |

Feedback IDs:

- T78-V1: `019e82a8-82da-7b22-bd31-cfed0f458fb7`
- T78-V2: `019e82a8-9499-7c03-b983-08c5387281cc`
- T78-P1: `019e82a8-a7be-7fb2-8c4e-6b7db8cc5f58`
- T78-P2: `019e82a8-ba3e-7fc3-a5b8-0dd0ecde4ced`

Because feedback was submitted, T78 ran `telemetry(action="real_session_eval", project="engram",
limit=50)` exactly once after the feedback submissions. The report generated at
`2026-06-01T10:09:19.374901Z` returned:

- `trace_count=50`
- `feedback_trace_count=30`
- `feedback_coverage=0.6000000238418579`
- `memory_judgment_coverage=1.0`
- `outcome_trace_count=30`
- `outcome_coverage=0.6000000238418579`
- `task_failure_count=0`
- `bad_memory_used_count=0`
- `wrong_scope_memory_count=0`
- `confidence_gate.passed=true`

The confidence-gate pass is diagnostic only. T78 deliberately selected four current-work tasks with
transcript-visible outcomes, so it proves that prospective task design can produce assessable
non-plan feedback with today's surfaces. It does not prove broad historical organic outcome
coverage, production readiness, or permission for M6, lifecycle writes, harness writes, ranking
changes, schema/storage/index changes, document-index actions, public MCP changes, or `orient`
expansion.
