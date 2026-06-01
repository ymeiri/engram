# Brain Harness T83 T82 Second-Reader Review

Status: Pre-registered; execution pending
Date: 2026-06-01
Scope: Read-only second-reader review of the T82 controlled outcome artifact

This slice validates evidence discipline only. It must not change source behavior, telemetry
schema, storage, indexes, public MCP request parameters, ranking, harness adapters/hooks,
migration, lifecycle state, document indexing, or the `orient` payload. It must not add new T82
rows or write telemetry feedback for T82's sampled rows.

## Research Question

Can a second reader classify the five T82 rows from the artifact and cited evidence refs without
hidden transcript context, and does that review strengthen or weaken the case for a future
artifact-format approval packet?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A read-only Claude Bridge review can independently confirm that T82's field shape is usable, while keeping reviewer agreement `PENDING` or partial where evidence refs are insufficient. |
| Null | The second reader cannot reproduce the classifications from the artifact and refs, so T82 remains narrative provenance rather than a reviewable artifact shape. |
| Simpler alternative | Stop at the T82 self-authored artifact and defer second-reader validation. |
| Failure | Treat Claude's agreement as proof of production readiness, allow writes/tools beyond read-only review, or use the review to justify schema/API/storage changes. |

## Measurement

Run one Claude Bridge call with:

- `harness="isolated"`
- `write=false`
- no Bash allowlist
- no tool allowlist

Claude receives the T82 artifact content and the relevant T78/T79/T80/T81 line excerpts already
captured by Codex. Claude must not edit files, call tools, or inspect hidden state.

Success criteria:

- Claude returns one row-level classification table for T82-1 through T82-5.
- Each row includes agreement or disagreement with the T80 class, evidence-strength judgment, and
  a short reason.
- Claude explicitly identifies whether T82-5 should remain `SELF_REPORTED_OUTCOME`.
- Claude states whether the artifact shape is reviewable enough for a future approval packet.

Stop rules:

- Do not retry if Claude asks for hidden transcript access or tools.
- Do not change T82 row criteria after seeing Claude output.
- Do not submit outcome feedback for T82 sampled rows.
- Do not run `real_session_eval`.
- Do not treat agreement as approval for schema/storage/public MCP/harness/ranking/lifecycle/
  migration/document-index/`orient` changes.

## Execution Prompt

```text
Read-only second-reader review for Engram T83. Do not edit files, run tools, or rely on hidden
transcript context.

Context: T82 created a doc-only controlled outcome artifact. It has five rows: four T78
ASSESSABLE_TASK_OUTCOME trace/feedback pairs and one T79 startup feedback row that T82 classifies
as SELF_REPORTED_OUTCOME. The artifact deliberately refuses causal claims and does not authorize
schema/API/storage/public MCP/harness/ranking/lifecycle/migration/document-index/orient changes.

Task:
1. Review the T82 artifact text and cited evidence excerpts supplied in this prompt.
2. For each row T82-1 through T82-5, state:
   - agree/disagree with T82's T80 class,
   - evidence strength,
   - whether the durable refs are enough for a second reader,
   - one sentence of reasoning.
3. State whether T82-5 should remain SELF_REPORTED_OUTCOME.
4. State whether the artifact shape is reviewable enough for a future approval packet, without
   recommending implementation yet.

Return a compact table and a short conclusion.
```

## Evidence Excerpts To Provide

- T82 artifact: `docs/BRAIN_HARNESS_T82_CONTROLLED_OUTCOME_ARTIFACT_PILOT_2026-06-01.md`
- T78 pre-registration/results: `docs/BRAIN_HARNESS_T78_CONTROLLED_OBSERVABLE_TASK_AUDIT_2026-06-01.md`
  lines 47-50 and 100-122.
- T79 harness-inconclusive result: `docs/BRAIN_HARNESS_T79_CLAUDE_BRIDGE_OBSERVABLE_TASK_AUDIT_2026-06-01.md`
  lines 124-145.
- T80 outcome classes and existing-field rubric:
  `docs/BRAIN_HARNESS_T80_OUTCOME_LINK_DECISION_PACKET_2026-06-01.md` lines 124-147.
- T81 proxy audit result:
  `docs/BRAIN_HARNESS_T81_FEEDBACK_OUTCOME_POINTER_PROXY_AUDIT_2026-06-01.md` lines 42-65.
