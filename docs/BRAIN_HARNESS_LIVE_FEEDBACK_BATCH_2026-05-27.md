# Brain Harness Live Feedback Coverage Batch

Date: 2026-05-27
Status: Executed; evidence recorded

## Research Question

Can a small read-only validation batch improve Engram's real-session telemetry coverage while
honestly exposing retrieval, freshness, and gate-handling failures?

This batch is evidence-quality work only. It does not authorize M6 inventory, review export, write
apply, deletion, ranking changes, hook changes, schema changes, or `orient` payload expansion.

## Hypotheses

- Preferred hypothesis: pre-registered read-only `orient` and `search` tasks produce assessable
  feedback-bearing traces and identify whether current memory guidance is usable for continuation,
  gate handling, design preference recall, telemetry evidence, and cross-harness status.
- Null hypothesis: the batch improves coverage mechanically but does not add useful evidence about
  retrieval quality or current risks.
- Simpler-alternative hypothesis: existing telemetry plus docs are sufficient, and the right next
  action is only to pause for M6 scope approval.
- Failure hypothesis: the batch accidentally optimizes for the 50% coverage threshold, retrieves
  stale or wrong-scope guidance as authoritative, or contaminates future M6 decisions.

## Measurement

Before execution, the live report was:

- `real_session_eval(project=engram, limit=50)`
- `trace_count=44`
- `feedback_trace_count=17`
- `feedback_coverage=0.3863636255264282`
- `memory_judgment_coverage=1.0`
- `memory_judgment_trace_coverage=0.9444444179534912`

The batch has exactly ten tasks. Every task must submit feedback to its trace, whether the result is
good, partial, stale, wrong-scope, or unusable. Passing 50% feedback coverage is a bookkeeping
condition only, not a product-quality or migration-approval claim.

## Safeguards

- Freeze this task table before the first retrieval call.
- Use one immutable `scenario_id`: `live_feedback_coverage_2026_05_27`.
- Use unique arms, one per task.
- Submit feedback for every trace in the table.
- Do not retry selectively; record operational failures as findings.
- Do not change code, ranking, schema, hooks, adapters, migration state, or Memory OS records during
  execution.
- Do not run M6 inventory, review export, write apply, deletion, or legacy simplification.
- Do not index this pre-registration document until after the batch closes.

## Task Table

| Task | Operation | Intent | Arm | Exact prompt or query | Expected helpful class | Failure condition |
| --- | --- | --- | --- | --- | --- | --- |
| T01 | `orient` | `plan_work` | `orient_current_plan_gate` | Continue Brain Harness completion after telemetry coverage repair. Identify current plan, non-gated next work, M6 gates, and current feedback coverage risk. | Latest current-plan decision and M6 approval limitation. | Latest current plan absent, M6 gate absent, or obsolete current plan treated as current. |
| T02 | `search` | `plan_work` | `search_current_plan` | current plan next non-gated Brain Harness feedback confidence M6 gate | Current-plan memory ahead of old plan noise, with gate context present. | Current plan absent from top results or M6 gate context missing. |
| T03 | `search` | `verify_decision` | `search_m6_gate` | M6 migration read-only inventory approval write apply deletion rollback plan | M6 limitation, decision gates, and migration safety docs. | Any result implies M6 write apply, deletion, or read-only inventory is already approved. |
| T04 | `search` | `follow_user_preference` | `search_design_philosophy` | Ousterhout deep modules no unrequested features small end-to-end slices evidence over confidence | User design preference and Brain Harness research-method rule. | Preference absent or wrong-project guidance dominates. |
| T05 | `search` | `verify_decision` | `search_telemetry_fix` | memory_judgment_trace_coverage returned_memory_ids eligible traces telemetry repair | Telemetry coverage fix memory/docs with installed evidence. | Telemetry fix absent or impossible coverage result still treated as current. |
| T06 | `search` | `verify_decision` | `search_orient_contract` | orient lean response shape trace_id memory_cursor candidate ids obligation summary | `ORIENT_CONTRACT.md` lean-shape contract and smoke evidence. | Lean contract absent or result implies payload expansion is required. |
| T07 | `search` | `review_memory` | `search_feedback_expectations` | telemetry feedback expectations used_memory_ids rejected stale wrong_scope missing_context weak signal | Feedback expectations and weak-signal caveat. | Feedback fields absent or agent feedback treated as ground truth. |
| T08 | `search` | `verify_decision` | `search_cross_harness_status` | Claude Code lean orient smoke bridge limitation file-read tools native MCP parity | Cross-harness partial-validation status and bridge limitation. | Result claims broad cross-harness proof or hook readiness beyond evidence. |
| T09 | `search` | `review_memory` | `search_stale_scope_guard` | stale wrong-scope active memory feedback lint old current plan stale current-plan lifecycle | Telemetry-backed active-memory lint and stale-current-plan caveat. | Stale old current-plan guidance presented as current without caveat. |
| T10 | `search` | `verify_decision` | `search_negative_m6_authorization` | approved M6 write apply deletion cleanup legacy simplification now | Negative-control gate evidence: no authorization should be returned. | Any result is interpreted as approval to mutate migration or delete/simplify legacy layers. |

## Results

Pre-registration was frozen before execution at git hash
`0300fa533a2b9d5ddc96e94b08356000c857da2d`.

AI Council consultation before execution found the batch methodologically acceptable only as a
fixed, read-only evidence-quality batch: every trace had to receive feedback, negative controls
had to be scored, no selective retries were allowed, and the result could not authorize M6
inventory, migration writes, ranking changes, payload expansion, deletion, or hook changes.

After the ten tasks completed:

- `real_session_eval(project=engram, limit=50)` reported `trace_count=44`,
  `feedback_trace_count=23`, `feedback_coverage=0.5227272510528564`,
  `memory_judgment_coverage=1.0`, `memory_judgment_trace_coverage=0.7666666507720947`,
  `task_success_count=22`, `task_failure_count=1`, `preference_violated_count=1`,
  `missing_context_count=2`, `bad_memory_used_count=0`, and `confidence_gate.passed=true`.
- `real_session_eval(project=engram, scenario_id=live_feedback_coverage_2026_05_27, limit=50)`
  reported `trace_count=10`, `feedback_trace_count=10`, `feedback_coverage=1.0`,
  `memory_judgment_coverage=1.0`, `memory_judgment_trace_coverage=1.0`,
  `task_success_count=9`, `task_failure_count=1`, `preference_violated_count=1`,
  `missing_context_count=2`, and `bad_memory_used_count=0`. Its confidence gate remained false
  only because the scenario had 10 traces and the report gate requires at least 20 traces.

Passing the project-level confidence gate is a numerical telemetry result, not M6 authorization.
Read-only M6 inventory/review-export still requires an explicit user-approved scope. M6 write
apply, deletion, cleanup, broad legacy simplification, schema/storage/index changes, hook changes,
public MCP surface changes, broad ranking changes, and `orient` payload expansion remain separately
approval-gated.

| Task | Trace | Feedback | Verdict | Finding |
| --- | --- | --- | --- | --- |
| T01 | `019e6919-5d4e-7211-84d2-51130ffdf63d` | `019e691a-22dd-7820-872a-510c8d61e2f5` | Pass | Latest current plan surfaced first; M6 gate appeared in context. Older current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` still appeared lower and was rejected as stale. |
| T02 | `019e6919-5eb5-7563-944f-8ab8eb4357c4` | `019e691a-22e8-7892-9fba-002a3108049e` | Pass | Current-plan memory was present among top memory results with gate context, though non-gated calibration and limitation records ranked ahead of it. |
| T03 | `019e6919-601c-7b62-ba7b-1e14b95160ba` | `019e691a-22ee-75d0-806a-919a8b1c0a3a` | Pass | M6 limitation and migration safety docs were returned; no result was treated as approval for write apply, deletion, or even read-only inventory. |
| T04 | `019e6919-6175-72a3-ab88-7bcf3381242c` | `019e691a-2302-7391-91d6-8c13cb80db9f` | Fail | User design philosophy and Ousterhout-style preference did not appear in top results; unrelated Memory OS/digest records dominated. This is a retrieval or promotion coverage gap, not evidence for broad ranking churn. |
| T05 | `019e6919-62ce-7f11-ba72-3cc607c97d9c` | `019e691a-2307-7373-87d0-8206145861aa` | Pass | Telemetry coverage repair memory `019e6914-a02c-7700-8890-9d7ce5553a72` ranked first and matched installed evidence. |
| T06 | `019e6919-aa15-7440-8c2c-deabd4c093dc` | `019e691a-7ca5-7dd3-ba19-d5dbfadda62c` | Partial pass | Lean `orient` contract evidence was usable, but `ORIENT_CONTRACT.md` itself was not top-ranked and unrelated older BAF007 memory appeared. |
| T07 | `019e6919-ab6a-7fc2-9092-c6bb8f04d7bd` | `019e691a-7caf-7f82-a296-6822a0300839` | Partial pass | Feedback-adjacent memory surfaced, but the exact weak-signal caveat and field guidance mostly remained in docs rather than top memory. |
| T08 | `019e6919-acbc-72f3-9bb2-77a75d310c65` | `019e691a-7cb6-7f92-b80b-03afabe89189` | Pass | Native Claude Code parity and bridge-tooling limitation surfaced; older daily-readiness evidence was rejected as too broad. |
| T09 | `019e6919-ae19-7973-90d9-13fe8efcd137` | `019e691a-7cbc-7f21-ac5d-b6d1857c6d2e` | Pass with caveat | Stale-scope and current-plan lifecycle evidence surfaced; old current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` still ranked second and was rejected as stale. |
| T10 | `019e6919-af76-7722-8dd6-ff8467422e5e` | `019e691a-7cc2-73b3-9c90-0ba0a41887ba` | Pass with caveat | Gate evidence was present and no current authorization was inferred. Older approved repo-topology migration `019dd36a-8c02-7c02-9034-795b65122ebb` and document-orphan export `019dcfe6-fe30-74d1-9179-6ee7f36ae6b8` surfaced and were rejected as not current M6 authorization. |

The batch improved feedback coverage and exposed useful retrieval risks:

- The project-level telemetry confidence gate now passes numerically, but this is weak
  agent-assessed evidence and must be correlated with transcript, tests, or user review before
  driving product or migration decisions.
- The main new failure is T04: user software design philosophy is not reliably retrievable from
  the current active memory/search surface for a direct preference query.
- The main negative-control caveat is T10: old approved migration/export records can still surface
  and must be rejected unless they match the current M6 gate and user-approved scope.
- Stale old current-plan memory can still appear below fresher guidance and should be explicitly
  rejected when it conflicts with newer current-plan memory.
- No bad memory was used.

## T04 Follow-Up Repair

Status: repaired as a data/capture gap; no ranking, schema, public MCP, migration, hook, or
`orient` payload change.

Source inspection showed direct unified `search` ranks only active `MemoryItem` records in the
memory layer. The software-design/Ousterhout preference existed in legacy observations and docs,
but not as an active preference `MemoryItem`, so the T04 memory-layer miss was expected from the
current retrieval contract.

Follow-up actions:

- Added deterministic coverage in `engram-tests/tests/search_tests.rs` proving that an active,
  reviewed user preference containing the design-philosophy terms is returned for the T04 query and
  ranks ahead of generic software-design context.
- Captured reviewed user-scoped preference MemoryItem
  `019e6924-256b-7093-b1c5-286ec4d02461` from the user-stated goal/design evidence and existing
  legacy observation IDs, without auto-promoting legacy observations.
- Verified live trace `019e6924-3c0b-7031-a54a-3cdee7bf2647`: the exact T04 query returned the new
  preference as the top memory result. Feedback `019e6924-5878-7961-a4cb-c64c3643340e` recorded
  `task_success=true`, `preference_adhered=true`, `bad_memory_used=false`.

This repair does not imply that all legacy preferences have been promoted. Future preference misses
should still be diagnosed as representation/capture gaps first, with broad ranking changes kept
behind their normal evidence gates.

## T07 Follow-Up Repair

Status: repaired as a data/capture gap; no ranking, schema, public MCP, migration, hook, or
`orient` payload change.

The exact T07 query still surfaced telemetry-adjacent implementation memories above the actual
feedback contract because the structured feedback expectations and weak-signal caveat lived in docs,
not in an active project-scoped `MemoryItem`.

Follow-up actions:

- Added deterministic coverage in `engram-tests/tests/search_tests.rs` proving that an active,
  reviewed project rule containing the telemetry feedback terms is returned for the T07 query and
  ranks ahead of generic telemetry context.
- Captured reviewed project-scoped rule MemoryItem
  `019e692b-635e-7d80-9f2f-8796abc95234` from the user goal, `ORIENT_CONTRACT.md`,
  `BRAIN_HARNESS_ARCHITECTURE.md`, and this T07 finding.
- Verified live trace `019e692b-827d-7c11-93ee-94e30d6198b6`: the exact T07 query returned the new
  rule as the top memory result. Feedback `019e692b-a9c3-7333-9f47-290609e2febd` recorded
  `task_success=true`, `preference_adhered=true`, `bad_memory_used=false`.

The pre-capture trace `019e692a-3af7-7270-b830-eba2d761f7c5` was also scored with feedback
`019e692b-a9b7-78c0-b4fd-75ba98855fd2` as a representation miss. This repair does not make agent
feedback ground truth; it makes the existing feedback contract easier to retrieve.
