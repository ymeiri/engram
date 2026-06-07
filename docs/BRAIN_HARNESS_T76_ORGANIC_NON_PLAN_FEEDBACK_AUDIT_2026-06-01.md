# Brain Harness T76 Organic Non-Plan Feedback Audit

Status: Completed instrumentation slice; organic scoring deferred
Date: 2026-06-01
Scope: Existing organic `follow_user_preference` and `verify_decision` traces in telemetry

This audit began as read-only except for ordinary telemetry feedback and documentation/memory
capture. It did not run M6 inspection, migration status, prioritize, review apply, candidate
decisions, deletion, lifecycle mutation, document indexing, harness writes, schema/storage/index
changes, ranking changes, or `orient` payload changes.

The audit found an instrumentation blocker before trace scoring: `telemetry(action="list_traces")`
accepted an `intent` field but did not apply it. The approved follow-up implementation slice fixed
that existing field behavior only. It did not add MCP request parameters, add tools, change storage
schema/indexes, change ranking, expand `orient`, or submit non-plan feedback.

## Research Question

Can existing non-`plan_work` traces provide independently assessable feedback evidence without
creating new synthetic tasks solely to satisfy the rolling confidence gate?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Existing organic non-plan traces contain enough concrete evidence to score at least one non-plan intent honestly, while keeping the result diagnostic rather than gate-changing. |
| Null | Existing non-plan traces are too sparse or too weakly assessable; the T75 confidence-gate failure should remain as a real evidence gap. |
| Simpler alternative | Create new fixed read-only tasks for `follow_user_preference` and `verify_decision`. |
| Failure | Retroactive scoring turns into cherry-picking or confidence-gate gaming and is used to imply M6, lifecycle, harness, ranking, document-index, schema/storage/index, public MCP, or `orient` approval. |

## Consultation

AI Council recall found prior guidance that flat intent coverage is secondary metadata and passive
feedback coverage cannot establish Brain Harness confidence. A fresh Council broadcast agreed that
non-plan feedback scoring is legitimate only when pre-registered, falsifiable, externally
assessable, and explicitly gate-neutral.

Claude Bridge recommended retroactive annotation of existing organic traces over new fixed tasks.
Its core concern was evaluator-controlled stimulus: if Codex creates new tasks for missing intent
labels, the result can become metric seeding even when the tasks are read-only.

## Pre-Registered Measurement

This section was written before opening any candidate trace bodies for T76.

Organic trace boundary:

- Candidate traces must have been created before the T76 startup orient trace
  `019e8283-8595-7ee3-b271-3dbedcc81e86`.
- Traces created by this T76 audit are excluded from scoring.
- The audit inspects at most the newest ten existing traces for each target intent:
  `follow_user_preference` and `verify_decision`.

Assessability criteria:

- A `follow_user_preference` trace is assessable only when its query or prompt names a preference
  task, returns a stable preference/rule MemoryItem relevant to that task, and the task outcome can
  be checked against repo docs, Engram memory, or the current transcript without subjective
  interpretation.
- A `verify_decision` trace is assessable only when its query or prompt names a concrete decision,
  gate, document, or claim, and the correct action can be checked against repo docs, Engram memory,
  git state, or tool output.
- A trace is not assessable merely because it has an intent label.
- If fewer than three traces for an intent satisfy the criterion, T76 records that scarcity and
  does not treat the intent as meaningfully covered.
- All traces that meet the criterion within the inspected set must be scored, including misses,
  stale/wrong-scope results, or bad-memory-use cases.

Feedback rules:

- Feedback must record used and rejected memory IDs when assessable.
- Scoring must describe the observed behavior, not optimize for aggregate coverage.
- Any rolling `confidence_gate.passed=true` result after T76 is diagnostic only. It must not
  authorize or relax M6, lifecycle, harness, ranking, document-index, schema/storage/index, public
  MCP, or `orient` gates.

## Results

The pre-registered scoring run stopped before opening or scoring candidate trace bodies.

Observed blocker:

- `telemetry(action="stats_by_intent")` showed historical `follow_user_preference` and
  `verify_decision` traces exist.
- `telemetry(action="list_traces", intent=...)` returned the newest project traces overall rather
  than traces for the requested intent.
- Source inspection confirmed the MCP request type already exposed `intent`, traces were persisted
  with `intent_key`, and the store schema already defined `idx_trace_intent`, but the
  `list_traces` code path forwarded only project, scenario, and arm filters.

Implementation result:

- Wired the existing `intent` request field through `telemetry(action="list_traces")`.
- Canonicalized intent aliases through `BrainHarnessIntent::parse`, so `intent="preference"`
  filters the stored `follow_user_preference` intent key.
- Left `query` unchanged because it is documented as operation context, not a list filter.
- Added deterministic MCP coverage where project-only or intent-only filtering would each return
  the wrong trace if the combined filter were ignored.

Validation:

- `cargo fmt --all --check`
- `cargo test -p engram-tests --test telemetry_tests mcp_telemetry_list_traces_filters_by_intent`
- `cargo test -p engram-tests --test telemetry_tests`

Post-commit live validation:

- Installed the T76 build into `/Users/yuval.meiri/.local/bin/engram`.
- Restarted the global daemon on port 8765.
- `telemetry(action="list_traces", project="engram", intent="follow_user_preference", limit=5)`
  returned five traces, all with `intent="follow_user_preference"`.
- `telemetry(action="list_traces", project="engram", intent="verify_decision", limit=3)`
  returned three traces, all with `intent="verify_decision"`.

T76 does not claim new non-plan confidence-gate evidence. Organic scoring should be rerun as a
separate pre-registered audit. Because the live validation calls exposed candidate trace bodies,
those viewed traces must not be reused as if they were blind organic scoring evidence.
