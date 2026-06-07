# Brain Harness T40 Partial Completion Audit

Date: 2026-05-31
Status: Completed partial audit; one mixed-query check remains partial
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

## Scoreable Results

The scoreable audit ran after commit `0322566` pre-registered the fixed checks.

| ID | Result | Evidence |
| --- | --- | --- |
| T40-01 | pass | Codex `prepare_handoff` lean orient trace `019e7cf5-3636-7f33-88f8-86f2f130d539` returned latest T39 current-plan memory `019e7ced-4de2-7860-be61-e5bc6dc1be78`, harness-write gate `019e7cde-b517-77d0-aaac-c8638811d4e8`, M6 gate `019e7ce5-155d-7a10-85f5-00b9dcc69cd0`, and no open obligations. |
| T40-02 | pass | Native Claude Code `prepare_handoff` trace `019e7cf6-0970-7ef0-b9a9-8efc1d448f48` returned the same current-plan and gate IDs as Codex. The synthetic prompt opened three startup obligations, which Codex resolved or skipped with explicit evidence before finalization. |
| T40-03 | pass | Codex `plan_work` lean orient trace `019e7cf5-4787-7202-8acd-ec27d5ed1238` returned the latest current plan first, kept M6/harness constraints visible, and left stale current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` lower as review noise. |
| T40-04 | partial | Direct search trace `019e7cf5-5a61-7943-be95-ecf16c0356de` returned the latest current plan first and stale current-plan memory lower, but the active M6 approval gate did not appear in the top memory results for the mixed non-gated query. This is a retrieval caveat, not authorization. |
| T40-05 | pass | Direct M6 negative-control search trace `019e7cf5-78f0-77a2-bb94-8e248a1a6f92` returned blocked/gate context ahead of old approval-shaped records; no result was treated as current approval. |
| T40-06 | pass | `harness(action=doctor)` returned `ready=false` for Claude Code, Codex, Gemini CLI, and Cursor, with no writes. Claude Code still lacks required session hooks; the other harnesses still have generated adapter drift. |
| T40-07 | pass | `lint(action=run, limit=10)` kept `feedback_stale_current_plan` for `019e5e0a-86b4-73e3-aa9b-ca350e83e915` first, with `safe_action=none` and no safe actions applied. |
| T40-08 | pass with caveat | Project rolling eval after T40 feedback returned `trace_count=50`, `feedback_trace_count=35`, `feedback_coverage=0.70`, `memory_judgment_coverage=1.0`, `external_session_trace_count=22`, `task_success_count=30`, `task_failure_count=5`, `bad_memory_used_count=0`, and `confidence_gate.passed=true`. This remains weak rolling evidence and does not approve M6. |
| T40-09 | pass | After cleanup, `obligations(action=doctor)` returned no open obligations or warnings. |

Scenario-scoped eval for `t40_partial_completion_audit_20260531` returned five scored traces, four
task successes, one task failure for T40-04, and `bad_memory_used_count=0`. Its confidence gate
failed only because the scenario is below the minimum trace/feedback thresholds, which is expected
for this fixed audit batch.

## Outcome

The approved/read-only surfaces remain coherent enough to continue, but the overall Brain Harness
goal is still not complete. T40 strengthened Codex/Claude Code handoff parity and preserved the
explicit M6/harness gates, while exposing a narrow mixed-query retrieval caveat: non-gated current
plan searches can still omit active M6 gate memory from the top memory results.

This audit did not run M6 inventory or review export, write migration data, change lifecycle state,
change schema/storage/index behavior, change public MCP request parameters, expand `orient`, adjust
broad ranking, or install/modify harness adapters or hooks.
