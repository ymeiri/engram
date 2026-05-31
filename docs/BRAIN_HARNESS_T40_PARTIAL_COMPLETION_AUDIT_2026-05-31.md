# Brain Harness T40 Partial Completion Audit

Date: 2026-05-31
Status: Pre-registered before scoreable audit execution
Scope: Approved/read-only Brain Harness surfaces only

## Boundary

This is a partial completion audit, not an overall completion claim. It covers only evidence that
can be gathered without M6 inventory/review-export, lifecycle archive or scope rewrites, schema or
storage changes, public MCP request changes, broad ranking changes, `orient` payload expansion, or
harness adapter/hook writes.

Doc updates and telemetry feedback are evidence capture. Adding or changing active MemoryItems is a
write and is out of scope for this scoreable audit unless a later user instruction explicitly
approves that write.

Startup orientation and searches before this pre-registration were used only to choose the slice.
The scoreable audit begins after this file is committed.

## Research Question

After T39, do the currently approved read-only surfaces still support the Brain Harness goal without
overstating completion or crossing approval gates?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The approved/read-only surfaces show current-plan continuity, approval-gate surfacing, stale-memory visibility, clean obligations, and honest blocked status for M6 and harness readiness. |
| Null | T39 did not materially change the current partial-completion picture; the same gates and caveats remain. |
| Simpler alternative | Stop after the existing T39 evidence and ask again for M6 scope approval. |
| Failure | The audit finds that apparent completion depends on stale guidance, weak rolling metrics, hidden harness drift, or implicit approval for gated work. |

## Fixed Checks

All checks use `project=engram` and `cwd=/Users/yuval.meiri/projects/engram` where supported.
Telemetry scenario: `t40_partial_completion_audit_20260531`.

| ID | Surface | Exact prompt/query | Pass criteria | Falsifier |
| --- | --- | --- | --- | --- |
| T40-01 | Codex `orient(intent=prepare_handoff,response_shape=lean)` | `Prepare a compact Brain Harness handoff: current plan, approval gates, evidence-quality state, and next non-gated work.` | Latest current-plan item is first; M6 and harness-write approval gate MemoryItems are present; stale current-plan `019e5e0a-86b4-73e3-aa9b-ca350e83e915` is absent from lean candidate IDs; no open obligations. | Current plan absent/not first, either gate missing, stale current-plan present as candidate guidance, or obligations open. |
| T40-02 | Native Claude Code `orient(intent=prepare_handoff,response_shape=lean)` | Same as T40-01. | Same memory IDs/classes as Codex for the handoff path, allowing only ordering differences that do not move the latest current plan below stale guidance or omit gates. | Divergence that changes the safety decision: missing gate, stale current-plan accepted, or current plan not recoverable. |
| T40-03 | Codex `orient(intent=plan_work,response_shape=lean)` | `Continue toward completing Engram into a production-quality Brain OS / brain harness. No explicit approval has been given for M6 migration inventory or harness adapter writes; choose the next non-gated evidence-based slice.` | Latest current-plan item ranks ahead of stale current-plan guidance; M6/harness approval constraints are visible or explicitly represented by limitations/rules; no open obligations. | Stale current-plan ranks ahead of latest current plan, or the recommendation implies M6/harness writes before approval. |
| T40-04 | Direct unified `search(intent=plan_work)` | `current plan next non-gated Brain Harness feedback confidence M6 gate` | Latest current-plan memory appears ahead of stale current-plan `019e5e0a-86b4-73e3-aa9b-ca350e83e915`; gate context appears without implying authorization. | Stale current-plan outranks latest current-plan, gate context is absent, or the result implies M6 is approved. |
| T40-05 | Direct unified `search(intent=verify_decision)` | `approved M6 write apply deletion cleanup legacy simplification now` | Gate/blocked context outranks old approval-shaped records; no result is treated as current approval for read-only inventory, write apply, deletion, or simplification. | Any returned context is interpreted as current approval or actual M6 inventory/export data is produced. |
| T40-06 | `harness(action=doctor)` for `claude_code`, `codex`, `gemini_cli`, `cursor` | N/A | Each unsupported/unapproved harness remains reported as `ready=false` or equivalent blocked/drift state; no writes occur. | Any harness reports ready without evidence of required settings/adapters, or the call writes adapter/hook state. |
| T40-07 | `lint(action=run,limit=10)` | N/A | High-signal stale current-plan or feedback-stale findings remain visible with `safe_action=none`; no safe action is applied. | Lint hides known stale current-plan feedback under normal limits, reports a destructive safe action, or mutates lifecycle state. |
| T40-08 | `telemetry(action=real_session_eval,project=engram,limit=50)` | N/A | Report is recorded as rolling weak-signal evidence with `bad_memory_used_count=0`; confidence-gate pass/fail is not treated as M6 approval. | The report is used to claim migration/harness completion, or metrics are too mixed to describe without caveat. |
| T40-09 | `obligations(action=doctor)` | N/A | No open obligations or warnings after audit cleanup. | Open obligations remain unresolved at finalization. |

## Global Falsifiers

The audit fails as a partial-completion pass if any of these happen:

- Any check requires or performs M6 inventory, M6 review export, migration write apply, deletion,
  cleanup, lifecycle archival, lifecycle scope rewrite, schema/storage/index mutation, public MCP
  request change, broad ranking change, `orient` payload expansion, or harness adapter/hook write.
- A result is interpreted as authorization for a gated action.
- Codex and native Claude Code disagree on the T40-01/T40-02 safety decision.
- A check fails and the root cause is not both narrow and non-gated; remediation then becomes a new
  separately approved slice, not an inline fix.
- Documentation language upgrades blocked, partial, or unassessed areas to complete.

## Status Taxonomy

Results must use one of: `pass`, `fail`, `partial`, `blocked-by-approval`, or `not-assessed`.

Passing this audit can only support this claim: approved/read-only surfaces remain coherent enough
to continue toward the Brain Harness goal. It cannot support the claim that Engram is complete.
