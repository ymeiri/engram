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

## T06 Follow-Up Repair

Status: repaired as a data/capture gap; no ranking, schema, public MCP, migration, hook, or
`orient` payload change.

The exact T06 query still surfaced an unreviewed candidate-ID implementation memory above the
actual lean-`orient` response-shape and hot-path contract because that contract lived primarily in
`ORIENT_CONTRACT.md`, not in an active reviewed project `MemoryItem`.

Follow-up actions:

- Added deterministic coverage in `engram-tests/tests/search_tests.rs` proving that an active,
  reviewed project rule containing the lean-`orient` contract terms is returned for the T06 query
  and ranks ahead of generic orient implementation context.
- Captured reviewed project-scoped rule MemoryItem
  `019e6931-bd2d-7281-b9f6-952eaa2a20e4` from `ORIENT_CONTRACT.md`,
  `BRAIN_HARNESS_ARCHITECTURE.md`, and the T06 pre-capture trace.
- Verified live trace `019e6931-d088-7493-a0d7-7795485ac944`: the exact T06 query returned the new
  rule as the top memory result. Feedback `019e6931-f385-7563-a634-16db587f695e` recorded
  `task_success=true`, `preference_adhered=true`, `bad_memory_used=false`.

The pre-capture trace `019e6930-5d44-71a2-8438-e311340e7a8d` was also scored with feedback
`019e6931-f37f-79e1-b8b8-b96927d19724` as a representation miss. This repair keeps lean
`orient` as a presentation option and does not expand the hot path.

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

## T09 Follow-Up Repair

Status: improved as read-only lint visibility; no automatic cleanup, ranking, schema, migration,
hook, or `orient` payload change.

The original T09 caveat was that stale old current-plan guidance can still appear below fresher
guidance and must be explicitly rejected. The post-2d1fdcd startup reproduced that caveat:
repository-scoped memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` still appeared in lean `orient`
even though newer project current-plan memory supersedes the actual work state. Existing
telemetry-backed lint already reported it as stale active memory with many feedback hits, but the
finding was generic.

Follow-up actions:

- Added a read-only lint rule `feedback_stale_current_plan` for active `decision`/`rule`
  MemoryItems tagged `current-plan` when recent feedback marks them stale.
- Kept `safe_action=none`; stale feedback is a review signal, not proof or permission to archive,
  rewrite, or delete memory.
- Added focused service and MCP tests proving current-plan stale feedback uses the specific rule
  and no duplicate generic stale-active-memory finding is emitted for the same item.

## T10 Follow-Up Clarification

Status: covered by existing read-only lint visibility; no new classifier, automatic cleanup,
ranking, schema, migration, hook, or `orient` payload change.

The original T10 caveat was that older migration/export approval records can still surface for
authorization-shaped queries and must be rejected unless they match a current user-approved M6
scope. AI Council feedback supported a specialized lint possibility, but Claude Bridge identified a
material design risk: unlike `current-plan`, Engram has no explicit migration-authorization tag or
lifecycle predicate. Adding a new classifier would rely on brittle title/content heuristics or
invent M6-shaped classification work outside the approval gate.

Follow-up actions:

- Kept old migration/export approval-shaped records on the existing generic
  `feedback_stale_active_memory` lint path when telemetry marks them stale.
- Added focused service and MCP tests proving an old repo-topology migration approval memory marked
  stale emits `feedback_stale_active_memory` with `safe_action=none`, and does not get classified as
  `feedback_stale_current_plan`.
- Documented that this generic finding is a review signal only: it does not invalidate historical
  approvals, authorize current M6 work, archive/delete memory, or change retrieval behavior.

## T11 Startup Feedback Stabilization

Status: evidence update only; no ranking, schema, migration, hook, lifecycle, or `orient` payload
change.

Research question: after the T10 clarification, does the normal startup sequence preserve the current
plan, feedback contract, and M6 gates while keeping stale historical memory as rejected evidence?

Measurement:

- Startup `orient` trace `019e694a-1369-7a62-b414-afd428f96a8b` returned current-plan memory
  `019e6948-d9b1-7d52-8f62-539a6db583a7`, the non-gated limitation
  `019e68d8-50b3-7212-8499-0f4361aae70c`, and the M6 read-only approval gate
  `019e6857-1059-7e41-b69f-eb26ef78bcb5`.
- Exact T07 recheck trace `019e694c-57fd-7702-9824-ccb7932a92f6` returned telemetry-feedback rule
  `019e692b-635e-7d80-9f2f-8796abc95234` first for the target `review_memory` query.
- Implementation-plan and risk searches still surfaced stale migration-completion memory
  `019dd3fe-ec94-7122-af04-1f35b839387f`; feedback marked it stale rather than using it.
- `lint(action=run, limit=80)` reported that same memory as
  `feedback_stale_active_memory` after four recent stale-feedback records, with `safe_action=none`.
- After scoring the T11 startup/search traces, `real_session_eval(project=engram, limit=50)` reported
  `trace_count=44`, `feedback_trace_count=22`, `feedback_coverage=0.5`,
  `memory_judgment_coverage=0.9545`, `bad_memory_used_count=0`, and
  `confidence_gate.passed=true`.

The result supports the existing direction: continue using direct feedback plus read-only lint to
make stale active memory visible, but do not infer M6 approval, automatic archival, deletion, or broad
ranking changes from agent feedback alone.

## T12 Gate-Context Ranking Calibration

Status: repaired as a narrow query-classification bug; no broad ranking weights, schema, public MCP,
migration, lifecycle, hook, or `orient` payload change.

Research question: when a continuation query asks for the current plan and mentions the M6 gate only
as context, should direct unified `search` still promote active current-plan guidance?

Before evidence:

- Trace `019e6954-4bf3-7432-9122-057cb9ab5b9b` for
  `current plan next step non-gated Brain Harness completion T11 feedback stabilization M6 gate`
  returned the fresh current-plan MemoryItem `019e6952-49b7-7a80-b53b-7dd0790e0ce9`, but behind
  non-gated calibration notes.
- Source inspection showed `asks_for_decision_gate` still treated bare `gate` as a decision-gate
  query after stripping `non-gated`, which disabled current-plan promotion.

Hypotheses:

- Preferred: bare `gate` in a current-plan/next-step prompt is often milestone context, not an
  approval request; removing it as an unconditional query trigger fixes the observed miss.
- Null: the behavior is acceptable ranking noise and should remain documented only.
- Simpler alternative: only document a prompt-writing caveat.
- Failure: migration apply/approval prompts lose gate-first behavior.

Decision and validation:

- AI Council consensus and Claude Bridge critique favored implementation within this narrow
  classification boundary.
- `asks_for_decision_gate` now keeps strong action or permission terms such as `should`, `proceed`,
  `allowed`, `allow`, `apply`, `safety`, `block`, `blocked`, and `must`, but no longer treats bare
  `gate` as sufficient.
- `test_memory_search_treats_non_gated_next_slice_as_current_plan` now covers the observed
  `current plan next step ... M6 gate` wording and a competing calibration fact.
- Existing `test_memory_search_keeps_gate_guidance_above_current_plan` and the mixed
  `should/proceed/migration apply` assertion preserve gate-first behavior for actual approval
  prompts.
- After scoring T12 startup/search traces, `real_session_eval(project=engram, limit=50)` reported
  `trace_count=44`, `feedback_trace_count=27`, `feedback_coverage=0.6136363744735718`,
  `memory_judgment_coverage=0.9629629850387573`, `bad_memory_used_count=0`, and
  `confidence_gate.passed=true`.

This result does not justify broad ranking churn. It only fixes a documented false positive in the
query classifier for current-plan promotion.

## T13 Installed-Runtime Validation

Status: T12 live runtime verified; a separate explicit gate-query live gap remains.

Research question: after installing the T12 code and restarting the daemon, does native MCP `search`
return the fresh current-plan memory first for the exact `current plan next step ... M6 gate`
continuation query, while preserving useful gate context for explicit migration-apply prompts?

Measurement:

- Before install, `/Users/yuval.meiri/.local/bin/engram` was hash
  `5b989d898ff033505c584c27d483ea9b3b433e679cc5bbf16befb59c48d1325c`, daemon PID `10065`.
- Pre-install exact T12 trace `019e6964-34ea-7222-b01d-b5414b161d2c` returned the fresh current-plan
  memory second, behind older non-gated calibration memory.
- Installed binary hash
  `62272400960eaaeb2fd7aa44aa13bf6f93abdbc81b5d11bc9106b0bcc82df29b`, restarted daemon on port
  `8765`, PID `79904`.
- Post-install exact T12 trace `019e6969-a674-7631-8ffa-b532b8638262` returned current-plan memory
  `019e6960-7ead-7001-9a4f-d8adce7c8264` first.
- After scoring the T13 traces, `real_session_eval(project=engram, limit=50)` reported
  `trace_count=46`, `feedback_trace_count=34`, `feedback_coverage=0.739130437374115`,
  `memory_judgment_coverage=0.970588207244873`, `bad_memory_used_count=0`, and
  `confidence_gate.passed=true`. The `postinstall_live_mcp` arm recorded one pass and three
  failures, matching the split result below.
- Focused repository validation before install passed:
  `cargo test -p engram-tests --test search_tests test_memory_search_treats_non_gated_next_slice_as_current_plan -- --exact`,
  `cargo test -p engram-tests --test search_tests test_memory_search_keeps_gate_guidance_above_current_plan -- --exact`,
  and `cargo check -p engram-cli`.

Result:

T12 is now verified in the installed live runtime for the exact gate-context current-plan prompt.
However, post-install gate-control traces `019e696a-0698-7e20-940a-b0ad23a29994` and
`019e696a-2540-7172-a473-33f13538d54d` still ranked calibration or current-plan memory above M6 gate
context for explicit migration-apply prompts. That is a separate narrow live-data/ranking gap. It
does not authorize read-only M6 inventory, migration write apply, deletion, cleanup, legacy
simplification, schema changes, hook changes, public MCP surface changes, broad ranking churn, or
`orient` payload expansion.

## T14 Explicit Migration-Apply Gate Calibration

Status: fixed and live verified; M6 remains approval-gated.

Research question: when a direct unified `search` query explicitly asks whether to proceed with
migration apply, should Engram rank the current migration review stop condition above current-plan,
calibration, and broad implementation-history records?

Consultation and hypotheses:

- AI Council and Claude Bridge both rejected docs-only repair and broad ranking churn. They
  recommended a prompt-class-specific fix with a live-shaped fixture.
- Preferred: explicit migration/M6 apply-permission prompts should promote actionable migration gate
  evidence first.
- Null: existing lexical ranking is sufficient once data is scored.
- Simpler alternative: capture a new gate MemoryItem only; this would not protect the current live
  distractors.
- Failure: old migration approvals or historical dry-run summaries outrank the unresolved gate, or
  the fix disturbs current-plan continuation prompts.

Implementation:

- Added `promote_migration_gate_for_explicit_apply_query` in the shared memory ranker. It only runs
  when the query mentions migration/M6, asks for apply/proceed/approval/write-style permission, and
  passes the decision-gate classifier.
- Candidate items must be active, not current-plan guidance, not calibration noise, and must contain
  migration/apply detail plus actionable blocking language. The final tie-breaker favors stronger
  stop signals such as `must not proceed`, `do not mark`, `pending/undecided`, and explicit write
  approval over merely reviewed historical batch status.
- Added deterministic coverage in
  `test_memory_search_promotes_live_like_migration_gate_over_calibration_noise`, including
  live-shaped distractors for non-gated calibration, current-plan guidance, broad implementation
  history, a reviewed dry-run batch, and old approval history.

Validation:

- `cargo test -p engram-tests --test search_tests test_memory_search_promotes_live_like_migration_gate_over_calibration_noise -- --exact`
- `cargo test -p engram-tests --test search_tests`
- `cargo fmt --all --check`
- `cargo check -p engram-cli`
- `git diff --check`
- Installed binary hash
  `fea91cc46549c138a425389394af9c4cdd9d8727eb39137f8afc179a976968eb` and restarted the daemon on
  port `8765`, PID `9969`.

Live result:

- Final trace `019e698d-b766-7e71-a4da-a8c593f1b191` for
  `Should we proceed with migration apply?` ranked
  `019dd35d-1a48-7103-b0e2-390225f8b418` (`Memory OS completion is paused at migration review
  gate`) first.
- Final trace `019e698d-b791-7d93-a0d6-542219e3eb6c` for
  `next non-gated step, should we proceed with migration apply?` ranked the same gate memory first.
- Regression trace `019e698d-b7ae-7a13-b2c5-d58a9898deab` kept the T13 current-plan memory
  `019e696e-83ce-73c2-9650-8591575d5bb7` first for the exact current-plan/M6-gate context query.
- Intermediate traces recorded the live misses that shaped the fixture: broad implementation
  history `019dd0ae-e56a-75a2-9181-a172c22460e4` and reviewed batch status
  `019dd368-5484-7d80-9de5-a6245b7e4a05` each outranked the unresolved gate before the final
  signal ordering.
- After scoring T14 traces, `real_session_eval(project=engram, limit=50)` reported
  `trace_count=44`, `feedback_trace_count=35`, `feedback_coverage=0.7954545617103577`,
  `memory_judgment_coverage=0.9714285731315613`, `bad_memory_used_count=0`, and
  `confidence_gate.passed=true`.

This is still a narrow prompt-class calibration. It does not authorize read-only M6 inventory,
migration write apply, deletion, cleanup, legacy simplification, schema/storage/index changes, hook
changes, public MCP surface changes, broad ranking churn, or `orient` payload expansion.

## T15 Claude Code Cross-Harness Smoke

Status: passed as read-only validation; no code or ranking changes.

Research question: does Claude Code, using its own connected Engram MCP server, observe the same T14
ranking boundary as Codex for explicit migration-apply prompts and the current-plan/M6 context
regression?

Measurement:

- Claude Code version: `2.1.152`.
- `claude mcp list` reported `engram: /Users/yuval.meiri/.local/bin/engram serve - Connected`.
- Claude Code was invoked with `--permission-mode dontAsk`, `--allowedTools mcp__engram__search`,
  and edit/shell tools disallowed.

Result:

- Trace `019e6993-d4da-70a1-b5eb-9185eeb23339` ranked
  `019dd35d-1a48-7103-b0e2-390225f8b418` first for
  `Should we proceed with migration apply?`.
- Trace `019e6993-d891-7ff3-93ef-4bd8ad14d9c7` ranked the same paused gate memory first for
  `next non-gated step, should we proceed with migration apply?`.
- Trace `019e6994-8ec9-7343-9198-9298867b9ceb` ranked current-plan memory
  `019e6992-e937-73e3-a165-a706d5f15a7d` first for the contextual
  `current plan next step ... M6 gate` regression query.
- After scoring these traces, `real_session_eval(project=engram, limit=50)` reported
  `trace_count=42`, `feedback_trace_count=36`, `feedback_coverage=0.8571428656578064`,
  `memory_judgment_coverage=0.9722222089767456`, `bad_memory_used_count=0`, and
  `confidence_gate.passed=true`.

This validates the shared MCP search behavior in Claude Code for the observed T14 prompt class. It
does not validate hooks, adapter installation, M6 inventory/write apply, or broad ranking quality.

## T16 Scoped Memory List Filtering

Status: fixed and live verified; evidence-quality surface only.

Research question: when agents ask `memory(action=list)` for active current-plan records in an
explicit project scope, can Engram exclude wrong-project guidance before applying `limit`?

Hypotheses:

- Preferred: explicit `scope_type` fields on `memory(action=list)` should filter matching
  `MemoryItem` scopes before `limit`, just like tag filtering already does.
- Null: scope fields are add-only on this MCP action and callers should not expect list filtering.
- Simpler alternative: document the limitation and require callers to filter client-side.
- Failure: scope filtering hides useful cross-scope records or changes `orient`/ranking behavior.

Observed failure:

- Before the fix, live MCP
  `memory(action=list, scope_type=project, project_name=engram, tags=[current-plan], status_filter=active)`
  returned three items: the current Engram project plan, an older repository-scoped Engram plan, and
  a `voice-layer` project current-plan item. That made evidence sampling noisy and wrong-project.

Implementation:

- `memory(action=list)` now parses an explicit `scope_type` as a scope filter, fetches before
  limiting when a scope or tag filter is present, retains matching scopes, then applies `limit`.
- The change is limited to the MCP memory-list surface. It does not change storage queries,
  unified `search`, `orient`, ranking, migration, schema, hooks, adapters, or lifecycle status.
- Added focused MCP regression `test_mcp_memory_list_filters_by_scope_before_limit`, where a newer
  `voice-layer` current-plan item must not outrank an older requested Engram current-plan when
  `scope_type=project`, `project_name=engram`, and `limit=1`.

Validation:

- `cargo test -p engram-tests --test memory_tests test_mcp_memory_list_filters_by_scope_before_limit -- --exact`
- `cargo test -p engram-tests --test memory_tests`
- `cargo fmt --all --check`
- `cargo check -p engram-cli`
- `cargo clippy -p engram-mcp --all-targets -- -D warnings`
- `git diff --check`
- Installed binary hash
  `0d4581c1cffdd17af0d4d8f0911812a05a2c3ce3f9ff8766d455e043ed73a211` and restarted the daemon on
  port `8765`, PID `36805`.

Live result:

- The same scoped current-plan list call returned exactly one item:
  `019e6997-96d0-76a0-ac67-c7655df0958f`, the Engram project current plan after T15.
- The older repository-scoped current plan and wrong-project `voice-layer` current plan were no
  longer returned for the explicit project-scope request.
- Native Claude Code `2.1.152`, with `engram: /Users/yuval.meiri/.local/bin/engram serve`
  connected and only `mcp__engram__memory` allowed, reproduced the same behavior after the T16
  current-plan capture: count `1`, memory `019e69af-011f-7450-9f8c-1ff067f0f183`, title
  `Current plan after T16 scoped memory-list filtering`, scope `project / engram`.
- After scoring the startup traces that led to this slice,
  `real_session_eval(project=engram, limit=50)` reported `trace_count=43`,
  `feedback_trace_count=30`, `feedback_coverage=0.6976743936538696`,
  `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, and
  `confidence_gate.passed=true`.

This closes a concrete evidence-sampling gap. It does not archive stale current-plan memory, change
ranking, or authorize M6 inventory/write apply, deletion, cleanup, legacy simplification,
schema/storage/index changes, hook changes, public request-parameter changes, or `orient` payload
expansion.
