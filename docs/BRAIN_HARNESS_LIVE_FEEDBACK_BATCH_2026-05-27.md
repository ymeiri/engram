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

- The project-level telemetry confidence gate passed numerically at this checkpoint. A later T18
  pre-feedback re-audit showed the current sample could fail when feedback spans only two intents;
  after scoring T18 traces, the current report passed numerically again. This remains weak
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

## T17 Harness Readiness Drift Audit

Research question: does current read-only `harness(action=doctor)` still support the completion
matrix claim that Claude Code is fully ready while the other harnesses have adapter drift?

Hypotheses:

- Preferred: the readiness implementation is working and the docs were stale; only docs and memory
  need correction.
- Null: Claude Code remains fully ready.
- Simpler alternative: record the raw doctor output without changing status claims.
- Failure: the doctor readiness logic is misleading and needs a code/test fix.

Measurement:

- Run explicit read-only doctor checks for `claude_code`, `codex`, `gemini_cli`, and `cursor` with
  root `/Users/yuval.meiri`, project `engram`, cwd `/Users/yuval.meiri/projects/engram`, and no
  `write` flag.
- Inspect `engram-index/src/harness.rs` readiness logic before interpreting results.

Result:

- All four explicit doctor checks returned `ready=false`.
- Claude Code: required generated adapter files were installed, but required `SessionStart` and
  `SessionEnd` hook registrations were missing from Claude settings; the optional settings snippet
  was user-owned. Extra Engram permission entries were present but are warnings, not the primary
  readiness blocker.
- Codex: required generated `codex-memory-session-skill` and `codex-resume-session-skill` drifted
  from current policy.
- Gemini CLI: required generated memory-session and resume-session commands plus global context
  drifted from current policy.
- Cursor: required generated memory-session and resume-session skills drifted from current policy.
- Source inspection confirmed `ready` requires all required adapters to be installed and, for
  Claude Code, required settings checks to pass; `doctor` then adds the soft lifecycle warning.

Decision:

- Correct the docs claim. Treat current cross-harness readiness as partially validated but not
  fully installed for any supported harness.
- No adapter, hook, settings, schema, ranking, `orient`, migration, or lifecycle-status writes were
  made. Adapter and hook writes remain approval-gated.

## T18 Post-T17 Evidence Gate Audit

Research question: after T17, which remaining Brain Harness quality gaps can be advanced without
crossing an approval gate?

Hypotheses:

- Preferred: read-only evidence will show the next meaningful fixes are lifecycle, hot-path, or
  index-behavior changes that require approval before implementation.
- Null: an existing safe action can be applied without approval.
- Simpler alternative: continue collecting telemetry feedback only.
- Failure: current docs overstate confidence or completion and must be corrected before any fix.

Measurement:

- `real_session_eval(project=engram, limit=50)`.
- `lint(action=apply_safe, write=false, limit=80)`.
- Direct review-memory search for stale current-plan behavior.
- Document search for the newly indexed T17 evidence.

Result:

- Before scoring the T18 retrieval traces, `real_session_eval` reported `trace_count=44`,
  `feedback_trace_count=30`,
  `feedback_coverage=0.6818181872367859`, `memory_judgment_coverage=1.0`, and
  `bad_memory_used_count=0`, but `confidence_gate.passed=false` because feedback spans only two
  intents.
- After scoring the T18 orient/search traces, the 2026-05-27T14:01:00Z recheck returned
  `feedback_trace_count=32`, `feedback_coverage=0.7272727489471436`,
  `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, and
  `confidence_gate.passed=true`.
- `lint(action=apply_safe, write=false)` reported `applied_safe_actions=0`; all observed stale,
  wrong-scope, duplicate-entity, missing-evidence, and handoff findings still have no safe automatic
  action.
- The stale repository-scoped current-plan item
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` now has 42 stale-feedback hits and still appears near the
  latest project current plan in review-memory search. It is visible as a review signal, not an
  automatically safe archival target.
- Document search finds the T17 evidence, but also shows duplicated relative/absolute path chunks
  for some Brain Harness docs. A normalization/idempotency fix would change document-index behavior.

Decision:

- Correct the confidence-gate claim: project telemetry currently passes numerically after scoring
  T18 traces, but this slice proves the result is sample-window sensitive and still weak
  agent-assessed evidence.
- Do not apply lifecycle status changes, hot-path ranking changes, document-index normalization,
  migration inventory/review-export, adapter writes, or hook/settings writes without explicit
  approval.

## T19 Trace-Anchored Real-Session Eval

Research question: does `real_session_eval(limit=N)` measure feedback for the sampled trace set, or
can independent trace and feedback windows distort coverage and confidence?

Hypotheses:

- Preferred: anchor feedback to sampled trace IDs, keeping formulas, output fields, request
  parameters, and confidence-gate constants stable.
- Null: the independent newest-feedback window is intentional enough to keep.
- Simpler alternative: document the window sensitivity without changing code.
- Failure: the slice unbounds feedback, changes public API, or spills into ranking, migration,
  harness adapters, hooks, or `orient`.

Measurement:

- AI Council prior-decision recall, AI Council broadcast, and Claude Bridge critique for eval-design
  blind spots.
- Focused regression test:
  `cargo test -p engram-tests --test telemetry_tests real_session_eval_report_anchors_feedback_to_sampled_traces -- --exact`.
- Broader validation:
  `cargo test -p engram-tests --test telemetry_tests`,
  `cargo test -p engram-tests --test brain_harness_eval_tests`,
  `cargo fmt --all --check`, and `cargo check -p engram-cli`.

Result:

- Added `TelemetryRepo::list_feedback_for_traces`.
- `real_session_eval_report` and scoped real-session eval now fetch feedback linked to the sampled
  trace IDs.
- Output fields, request parameters, formulas, confidence-gate constants, `stats_by_intent`,
  `list_feedback_scoped`, ranking, `orient`, migration, hooks, adapters, and schema/storage/index
  behavior were not changed.
- The focused regression test proves that newer feedback on older traces no longer inflates
  coverage for a smaller recent trace sample.
- Validation passed:
  `cargo test -p engram-tests --test telemetry_tests real_session_eval_report_anchors_feedback_to_sampled_traces -- --exact`,
  `cargo test -p engram-tests --test telemetry_tests`,
  `cargo test -p engram-tests --test brain_harness_eval_tests`,
  `cargo fmt --all --check`, and `cargo check -p engram-cli`.

Decision:

- Treat this as an evidence-quality correctness fix. The confidence gate still relies on weak
  agent-assessed feedback unless checked against transcript evidence, tests, or user review.
- This does not authorize M6 inventory, M6 write apply, deletion, lifecycle status writes, broad
  ranking changes, document-index normalization, hook/settings writes, adapter writes, schema
  changes, or `orient` payload expansion.

## T20 Scoped Real-Session Eval Sampling

Research question: when `real_session_eval` is filtered by project, scenario, or arm, should
`limit=N` mean the newest N traces in that scope, or the scoped subset of the newest N global
traces?

Hypotheses:

- Preferred: scoped eval should apply filters before the limit, then anchor feedback to that scoped
  trace sample. This keeps scoped confidence reports from being starved by newer out-of-scope
  traffic.
- Null: the current newest-global-window semantics are intentional and should stay.
- Simpler alternative: only add a test documenting the current behavior.
- Failure: the slice expands into public parameters, output fields, ranking, `orient`, migration,
  lifecycle writes, document-index normalization, hooks, adapters, or schema changes.

Measurement:

- AI Council recall found the T19 feedback-window consultation; AI Council broadcast and Claude
  Bridge critique agreed this is an eval-starvation bug.
- Focused regression:
  `cargo test -p engram-tests --test telemetry_tests scoped_real_session_eval_applies_limit_after_scope_filters -- --exact`.
- Broader validation:
  `cargo test -p engram-tests --test telemetry_tests`,
  `cargo test -p engram-tests --test brain_harness_eval_tests`,
  `cargo fmt --all --check`, `cargo check -p engram-cli`, and `git diff --check`.

Result:

- Added `TelemetryRepo::list_traces_scoped`, applying project, scenario, and arm predicates before
  the trace limit while preserving newest-first order.
- `TelemetryService::list_traces_scoped` and scoped real-session eval now use that repository query.
- Scoped real-session eval still fetches feedback by the sampled scoped trace IDs, preserving the
  T19 feedback-anchoring behavior.
- Public request parameters, output fields, formulas, confidence-gate constants, ranking, `orient`,
  migration, lifecycle status, document-index behavior, hooks, adapters, schema/storage, and
  `list_feedback_scoped` semantics were not changed.

Decision:

- Treat this as a scoped eval-sampling correctness fix. It improves evidence quality for controlled
  project/scenario/arm reports but remains weak agent-assessed evidence unless checked against
  transcript evidence, tests, or user review.
- `list_feedback_scoped` still has its older scoped-window semantics and should be handled only as a
  separately approved/narrow follow-up if evidence shows operators need drill-down parity.

## T21 Installed-Runtime Validation For T19/T20

Status: passed as a live daemon validation; no source behavior, ranking, public MCP surface,
migration, lifecycle, hook, adapter, schema, storage, or `orient` change.

Research question: after installing the T19/T20 code, does the live daemon apply scope filters
before the trace limit and fetch feedback only for the sampled trace IDs?

Hypotheses:

- Preferred: installing the current binary and restarting the daemon makes the live MCP report match
  the T19/T20 regression tests.
- Null: the daemon was already current or the smoke cannot distinguish old behavior from new
  behavior.
- Simpler alternative: rely on local tests and skip live runtime validation.
- Failure: install/restart fails, the MCP daemon is unreachable, or the scoped report counts newer
  out-of-scope traces or newer feedback attached to older in-scope traces.

Measurement:

- Installed `/Users/yuval.meiri/.local/bin/engram` from the current repo with
  `cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`.
- Restarted the global daemon. New status: port `8765`, PID `11922`.
- Installed binary hash:
  `0192d24d945b7acb8bdfabe129c56d61a5abf0f7ce8223c854139677a93738ab`.
- Scenario: `t21_installed_runtime_eval_20260527_0192d24d`; arm: `memoryitem_orient`.
- Created four in-scope `project=engram` traces, then two newer out-of-scope
  `project=engram-other` traces. Only one of the latest two in-scope traces had feedback. Newer
  feedback was submitted to the older in-scope traces to catch feedback-window drift.

Result:

- `telemetry(action=list_traces, project=engram, scenario_id=..., arm=..., limit=6)` returned the
  four in-scope traces newest-first. The latest two were:
  `019e69e4-6244-7123-a34e-d19e8c44341a` and
  `019e69e4-5582-79a1-8dc4-09411d58aca5`.
- `telemetry(action=real_session_eval, project=engram, scenario_id=..., arm=..., limit=2)` returned
  `trace_count=2`, `feedback_count=1`, `feedback_trace_count=1`, `feedback_coverage=0.5`,
  `feedback_records_per_trace=0.5`, `memory_judgment_trace_coverage=0.5`,
  `task_success_count=1`, `task_failure_count=0`, and applied filters for project, scenario, and
  arm.
- The newer out-of-scope traces
  `019e69e4-9e3e-75f3-96b4-6cf82dce695a` and
  `019e69e4-b6f0-7872-ad41-f7432683e19f` did not starve the scoped sample.
- Newer feedback on older in-scope traces
  `019e69e4-3411-7783-b3a3-f00e0dae3e21` and
  `019e69e4-4640-78c3-a805-6f44283da31b` did not inflate the scoped report.

Decision:

- Treat T19/T20 as installed-runtime validated for this controlled live case.
- Keep the confidence gate caveat: this is evidence-quality plumbing validation, not human-judged
  Brain Harness product proof or migration approval.
- The next non-gated work should stay in targeted validation, evidence quality, or a concrete
  capture/lifecycle gap surfaced by evidence. M6 inventory/export/apply/deletion, lifecycle writes,
  hook/adapter writes, public MCP changes, schema/storage changes, broad ranking changes, and
  `orient` payload expansion remain approval-gated.

## T22 Claude Code Cross-Harness Read-Only Smoke

Status: native Claude Code reproduced the T21 report; Claude Bridge remains tool-limited for this
request.

Research question: can a separate Claude Code harness read the same installed T21 telemetry report
through its own Engram MCP connection?

Measurement:

- AI Council recall before consultation found no directly relevant prior decision for T19/T20
  cross-harness validation.
- Claude Bridge was asked to run the read-only report with `mcp__engram__telemetry` allowed, but
  the bridge project harness exposed only file-read tools. This is a bridge tool-exposure
  limitation, not an Engram runtime failure.
- Native Claude Code `2.1.152` was run from `/Users/yuval.meiri/projects/engram` with
  `--allowedTools mcp__engram__telemetry` and no write permission.

Result:

- Native Claude Code reported `tool_available=true`.
- `real_session_eval(project=engram,
  scenario_id=t21_installed_runtime_eval_20260527_0192d24d, arm=memoryitem_orient, limit=2)`
  returned `trace_count=2`, `feedback_count=1`, `feedback_trace_count=1`,
  `feedback_coverage=0.5`, `task_success_count=1`, and `task_failure_count=0`, with the expected
  project/scenario/arm filters.
- `list_traces` returned the same newest two in-scope trace IDs:
  `019e69e4-6244-7123-a34e-d19e8c44341a` and
  `019e69e4-5582-79a1-8dc4-09411d58aca5`.
- Claude's final explanation incorrectly inferred that the report was scoped to orient-operation
  traces. That interpretation is rejected: the current source applies project/scenario/arm filters
  before `limit=2`, and the two newest in-scope traces happen to be orient traces.

Decision:

- Treat this as cross-harness validation for the read-only MCP telemetry report shape used in T21.
- Do not treat Claude's model explanation as proof; the evidence is the matching tool result plus
  source inspection.
- This does not validate hooks, adapter installs, bridge tool parity, broad report quality, M6
  migration, ranking, lifecycle writes, schema/storage changes, public MCP changes, or `orient`
  payload changes.

## T24 External-Session Telemetry Coverage Audit

Status: read-only evidence audit; no source behavior, ranking, public MCP surface, migration,
lifecycle, hook, adapter, schema, storage, or `orient` change.

Research question: is sparse `external_session_id` coverage in the live feedback loop a core
telemetry implementation gap, a harness-guidance/adoption gap, or a host-session availability limit?

Hypotheses:

- Preferred: the core telemetry path already supports caller-supplied host labels, but current Codex
  startup calls do not have a stable host-thread label to pass, so the gap is joinability/adoption
  rather than missing storage or eval plumbing.
- Null: existing docs and tests fully cover the issue, so no new recorded evidence is useful.
- Simpler alternative: record only the current completion-matrix caveat and defer all action.
- Failure: attempting to auto-fill a label would require hook, adapter, host integration, or public
  behavior changes and would hit an approval gate.

Measurement:

- `telemetry(action=real_session_eval, project=engram, limit=50)` generated at
  `2026-05-27T14:58:28Z` returned `trace_count=50`, `feedback_count=38`,
  `feedback_trace_count=38`, `feedback_coverage=0.7599999904632568`,
  `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, and
  `confidence_gate.passed=true`.
- The same report returned `external_session_trace_count=5`,
  `distinct_external_session_count=1`, `unspecified_external_session_trace_count=45`,
  `external_session_feedback_count=5`, and `unspecified_external_session_feedback_count=33`, with
  the recommendation to set `external_session_id` when a host thread/session ID is known.
- `telemetry(action=list_traces, project=engram, limit=12)` showed the latest startup/audit traces
  all had `external_session_id=null`, including T24 startup searches
  `019e69f1-81a4-7b00-b93d-4af8bf9da741`,
  `019e69f1-824f-7f31-b5a6-4e85a62e851e`,
  `019e69f1-82f9-7e71-8412-3f080b1862ca`,
  `019e69f1-83a9-7272-900c-55c7d61106ad`, and
  `019e69f1-8456-7c30-a4ab-1ca07abc29fa`.
- Source inspection found the storage/model fields in `BrainHarnessTrace` and `AgentFeedback`, MCP
  request fields on `orient`, `search`, `memory(action=changes_since)`, and `telemetry`, service-side
  feedback inheritance from the trace label, eval aggregation of external-session counts, and
  validation that non-empty labels are at most 256 characters.
- Focused tests already cover service-level `orient` and `changes_since` pass-through, MCP
  `orient` and `search` pass-through, `telemetry(action=record_trace)` plus
  `submit_feedback` inheritance, and real-session eval external-session count reporting.

Result:

- Treat this as a host/harness attribution gap, not a core telemetry implementation gap. The core
  path stores, preserves, validates, reports, and tests caller-supplied labels.
- Current Codex Desktop/native MCP calls in this thread did not expose a stable host-thread label to
  the agent. Supplying a synthetic label would make trace-to-transcript joins look stronger than the
  evidence supports.
- Generated harness guidance currently emphasizes keeping `trace_id` values and submitting
  feedback; it does not force an `external_session_id` because the host label may be unavailable.
  Updating hooks, adapters, or host integration to provide one is a separate approved slice.

Decision:

- Keep the feedback loop marked partially validated: the current confidence gate passes numerically,
  but most live traces still cannot be joined back to host transcripts through
  `external_session_id`.
- Do not infer or auto-fill `external_session_id` from unrelated transport/session metadata without
  an explicit host-session contract.
- This does not authorize M6 inventory/export/apply/deletion, lifecycle writes, hook/adapter writes,
  public MCP changes, schema/storage changes, broad ranking changes, or `orient` payload expansion.

## T25 Rolling Evidence Window Re-Audit

Status: read-only evidence audit only. No source behavior, ranking formula, migration flow, harness
adapter, hook, schema, or `orient` payload changed.

Research question:

- After T24 feedback scoring and the next T25 startup traces, does the rolling
  `real_session_eval(project=engram, limit=50)` report still support the completion-matrix evidence
  claims without overstating confidence?

Hypotheses:

- Preferred: the report remains useful operational evidence, but it is sample-window-sensitive; new
  unscored startup traces can lower feedback coverage even when the underlying feedback loop is
  working and `bad_memory_used_count` remains zero.
- Null: the existing T24 audit is enough and the latest report adds no meaningful completion-matrix
  information.
- Simpler alternative: submit feedback for new traces only, without recording the rolling-window
  interpretation.
- Failure: treating the rolling report as migration approval, changing confidence formulas, or
  expanding hot-path behavior without a user-approved slice.

Measurement:

- After T24 trace feedback was submitted, the project report reached
  `feedback_trace_count=44/50`, `feedback_coverage=0.88`, `external_session_trace_count=5/50`,
  and `bad_memory_used_count=0`.
- The T25 startup added fresh orient/search traces. A read-only report generated at
  `2026-05-27T15:06:04Z` returned `trace_count=50`, `feedback_trace_count=38`,
  `feedback_coverage=0.7599999904632568`, `memory_judgment_coverage=1.0`,
  `bad_memory_used_count=0`, `confidence_gate.passed=true`,
  `external_session_trace_count=5`, and `unspecified_external_session_trace_count=45`.
- After the T25 startup traces were scored, the same rolling report generated at
  `2026-05-27T15:10:44Z` returned `trace_count=50`, `feedback_trace_count=44`,
  `feedback_coverage=0.8799999952316284`, `memory_judgment_coverage=1.0`,
  `bad_memory_used_count=0`, `confidence_gate.passed=true`,
  `external_session_trace_count=5`, and `unspecified_external_session_trace_count=45`.
- T25 startup retrieval still returned the active current-plan memory first for `orient` and the
  direct current-plan search. The stale repository-scoped current-plan memory still appeared as
  lower-ranked noise in broad startup results.
- Read-only lint reported specialized stale-current-plan feedback for
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` with 61 recent stale feedback records and
  `applied_safe_actions=0`.
- Read-only harness doctor still reported all four supported harnesses as `ready=false`: Claude
  Code is missing settings hook registrations, while Codex, Gemini CLI, and Cursor have generated
  adapter or context drift. No harness files were written.

Result:

- The confidence gate is useful as a rolling operational signal, not a proof of completion. Its
  numerator can move down when new unscored traces enter the latest 50-trace window.
- Scoring the fresh T25 startup traces restored the report to the same `44/50` feedback-trace level
  observed after T24 scoring, which supports the window-sensitivity interpretation rather than a
  telemetry regression.
- T24's external-session conclusion still holds: sparse joinability is a caller/harness adoption or
  host-availability gap, while the core storage/pass-through/reporting path is already covered.
- The stale repository-scoped current-plan memory remains a visible lifecycle review issue. The
  latest evidence does not justify automatic archive/scope writes.

Decision:

- Keep evidence and feedback loop status at "partially validated" until live traces are routinely
  scored and corroborated against transcripts, tests, or user review.
- Do not use the rolling confidence gate as approval for M6 migration inventory, review export,
  write apply, deletion, or legacy simplification.
- Score new startup traces when assessable, but do not change telemetry semantics or hook/adapter
  behavior without a separate approved slice.

## T26 Obligation Noise Suppression

Status: implemented as a narrow obligation-detection fix. No `orient` payload, ranking formula,
telemetry formula, migration flow, lifecycle status, hook, adapter, schema, or storage behavior
changed.

Research question:

- Can agent-native obligation detection reduce false follow-through pressure from safety-gate
  wording and local instruction files without weakening explicit failed-tool or document-disposition
  detection?

Hypotheses:

- Preferred: bare `schema` mentions in research/safety text should not create failed-tool recovery
  obligations, and untracked root instruction files such as `AGENTS.md` should not create document
  disposition candidates. Explicit failed-tool wording and ordinary durable document edits should
  still be detected.
- Null: the existing detector is noisy but acceptable because agents can manually skip the false
  positives.
- Simpler alternative: keep code unchanged and document the skip pattern.
- Failure: suppressing too broadly would hide real failed tool calls or real durable documents that
  need indexing, memory capture, or explicit skip evidence.

Measurement:

- Source inspection found `detect_prompt_obligations` treated bare `schema` as a tool-failure cue,
  which matched T25/T26 research text even when no tool call failed.
- Source inspection also found `detect_document_obligations` created document candidates for
  untracked root instruction files before later context filtering or same-content skip logic.
- The code now routes tool-failure detection through a narrower `has_tool_failure_cue` helper and
  skips untracked root instruction files before creating document-disposition candidates.
- Focused unit validation passed:
  `cargo test -p engram-index obligation::tests` (`6` passed), covering both new regression cases
  plus existing prompt/document/idempotency behavior.
- MCP boundary validation passed:
  `cargo test -p engram-tests --test obligation_tests` (`10` passed).
- Additional checks passed: `cargo fmt --all --check`, `cargo check -p engram-cli`, and
  `git diff --check`.

Result:

- Obligation detection is quieter for ordinary Brain Harness research prompts that mention schema or
  failure hypotheses without an actual tool failure.
- The persistent user-owned root `AGENTS.md` no longer becomes a document-disposition candidate just
  because it is untracked in the checkout.
- Existing explicit failed-tool and ordinary durable-document coverage remains tested.

Decision:

- Treat this as an obligation signal-quality fix only. It does not authorize M6 work, lifecycle
  archive/scope writes, hook/adapter writes, ranking changes, or `orient` expansion.

## T27 Installed-Runtime Validation For T26

Status: passed as a live daemon validation. No source behavior, `orient` payload, ranking formula,
telemetry formula, migration flow, lifecycle status, hook, adapter, schema, or storage behavior
changed in this slice.

Research question:

- After installing the T26 code and restarting the daemon, does live MCP obligation detection apply
  the same noise suppression that passed source and MCP-boundary tests?

Hypotheses:

- Preferred: the refreshed installed binary removes the two observed false positives in live MCP:
  bare `schema` / `failure hypothesis` wording does not create `tool_failure_recovery`, and the
  untracked user-owned root `AGENTS.md` file does not create a document-disposition candidate.
  Explicit failed-tool wording still creates `tool_failure_recovery`.
- Null: source tests pass, but the installed daemon remains stale or live MCP behavior differs.
- Simpler alternative: rely on source tests only and leave the installed daemon unchanged.
- Failure: install/restart breaks MCP access or suppresses explicit failed-tool recovery too broadly.

Measurement:

- Pre-install live MCP dry-run against daemon PID `11922`, installed binary hash
  `0192d24d945b7acb8bdfabe129c56d61a5abf0f7ce8223c854139677a93738ab`, reproduced the stale
  behavior: prompt `Failure hypothesis: avoid schema changes unless evidence justifies them.`
  produced `document_disposition` for `AGENTS.md`, `source_reading`, and
  `tool_failure_recovery`.
- Refreshed `/Users/yuval.meiri/.local/bin/engram` with
  `cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`, installed binary hash
  `7d9256dc2ca9fcefaaa54bf620c15989fa20926c929d9e6beca27012b6afc9cf`, and restarted the daemon on
  port `8765`, PID `50257`.
- Post-install live MCP dry-run for the same prompt returned only `source_reading`; it did not return
  `tool_failure_recovery`, `document_disposition`, or any `AGENTS.md` candidate.
- Post-install live MCP dry-run for `A tool call failed because of wrong parameters.` still returned
  `tool_failure_recovery`.

Result:

- The T26 obligation-noise fix is now validated in the installed live runtime used by Codex MCP.
- The known installed-binary drift caveat was addressed for this slice by installing the current
  source and restarting the global daemon.

Decision:

- Treat this as installed-runtime evidence for obligation signal quality only. It does not authorize
  M6 inventory/export/apply/deletion, lifecycle writes, ranking changes, hook/adapter writes,
  schema/storage changes, public MCP surface changes, telemetry formula changes, or `orient`
  expansion.

## T28 Claude Code Cross-Harness Obligation Smoke

Status: passed with a harness-write caveat through Claude Bridge. No source behavior, ranking,
`orient` payload, migration flow, lifecycle status, hook, adapter, schema, storage, public MCP
surface, or telemetry formula changed.

Research question:

- Does native Claude Code, through its own Engram MCP connection exposed by Claude Bridge, observe the
  same T27 installed-runtime obligation behavior as Codex?

Hypotheses:

- Preferred: Claude Code receives the same dry-run obligation results as Codex: the bare `schema` /
  `failure hypothesis` prompt does not produce `tool_failure_recovery` or an `AGENTS.md`
  document-disposition candidate, while explicit failed-tool wording still produces
  `tool_failure_recovery`.
- Null: Codex MCP sees the fixed daemon but Claude Code's MCP path is stale or unavailable.
- Simpler alternative: rely on Codex installed-runtime validation only.
- Failure: Claude Bridge cannot expose the Engram MCP obligations tool, or Claude Code receives a
  divergent result that needs a separate harness investigation.

Measurement:

- AI Council recall found no directly relevant prior decision for this exact obligation parity smoke.
- Claude Bridge foreground read-only task used `harness=personal`, `write=false`, and allowed only
  `mcp__engram__obligations`.
- Claude Code dry-run for prompt `Failure hypothesis: avoid schema changes unless evidence justifies
  them.` returned only `source_reading` with id `019e6a13-8410-7be3-8c5a-38614d40e9d1`. It did not
  return `tool_failure_recovery` and did not return any `AGENTS.md` document-disposition candidate.
- Claude Code dry-run for prompt `A tool call failed because of wrong parameters.` returned
  `tool_failure_recovery` with id `019e6a13-890d-7781-9e1e-cc50c5793c75`.
- Both requested Claude Code validation calls reported dry-run results: `written=[]`,
  `skipped_existing=[]`, and `warnings=[]`.
- Follow-up Codex `obligations(action=doctor)` found that the Claude Code harness itself had opened
  two prompt-derived obligations from the synthetic smoke prompt: `tool_failure_recovery`
  `019e6a13-6a3f-7d12-92b6-a89cc7d91b37` and `source_reading`
  `019e6a13-6a3f-7d12-92b6-a884a00d46c9`. Codex skipped both with explicit synthetic-smoke
  evidence because there was no real failed tool recovery or Claude source-reading task to complete.

Result:

- The T26/T27 obligation-noise behavior is now replicated through Claude Code for this exact MCP
  request shape.
- This is cross-harness evidence for the shared `obligations` surface only. It does not validate
  hooks, generated adapter settings, broad Claude Code readiness, ranking, migration, or `orient`.
- The caveat is material for future smokes: prompts that intentionally contain obligation trigger
  phrases can cause harness-start obligation writes even when the requested validation calls are
  dry-run. Always run `obligations(action=doctor)` afterward and resolve or skip synthetic artifacts.

Decision:

- Treat this as narrow Claude Code parity evidence for obligation signal quality. It does not
  authorize M6 inventory/export/apply/deletion, lifecycle writes, ranking changes, hook/adapter
  writes, schema/storage changes, public MCP changes, telemetry formula changes, or `orient`
  expansion.

## T29 Read-Only Completion Gate Audit

Status: completed as a documentation/evidence audit. No source behavior, migration flow, lifecycle
status, hook, adapter, schema, storage, public MCP surface, telemetry formula, ranking, or `orient`
payload changed.

Research question:

- After T27/T28, does the completion matrix still identify a concrete non-gated implementation
  slice, or is the remaining Brain OS definition of done blocked by approval-gated migration and
  harness-configuration work?

Hypotheses:

- Preferred: the matrix is current enough to show meaningful progress while preserving the remaining
  approval gates.
- Null: T27/T28 do not change any completion status.
- Simpler alternative: rely on the prior T23/T24 matrix without another audit.
- Failure: the audit implies approval for migration, lifecycle mutation, hook writes, or hot-path
  changes that the user has not granted.

Measurement:

- T29 startup lean `orient` trace `019e6a18-38dc-7b01-9f0f-802c995e4830` returned active
  current-plan memory `019e6a16-e428-7ee0-9959-78af745a72ae` first.
- Direct startup searches for current plan, architecture, implementation plan, user philosophy, and
  risks surfaced the active current plan plus relevant gate/caveat memory.
- `git status --short` showed only the user-owned untracked root `AGENTS.md`.
- The live daemon was running on port `8765`, PID `50257`, with installed binary hash
  `7d9256dc2ca9fcefaaa54bf620c15989fa20926c929d9e6beca27012b6afc9cf`.
- `real_session_eval(project=engram, limit=50)` at `2026-05-27T15:42:45Z` returned
  `trace_count=50`, `feedback_trace_count=37`, `feedback_coverage=0.7400000095367432`,
  `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, `confidence_gate.passed=true`, and
  `external_session_trace_count=0`.
- `obligations(action=doctor)` returned no open obligations and no warnings.
- `harness(action=doctor)` returned `ready=false` for `claude_code`, `codex`, `gemini_cli`, and
  `cursor`. Claude Code still lacks required `SessionStart` and `SessionEnd` settings registrations;
  Codex, Gemini CLI, and Cursor still have required generated adapter drift.

Result:

- Current-plan/next-step retrieval remains validated for the current continuation prompt class.
- Obligation signal quality is now validated in both Codex and Claude Code for the observed request
  shape.
- The evidence loop remains only partially validated because the rolling feedback window is
  sample-sensitive and the latest sampled traces have no external session labels.
- Cross-harness behavior remains partial: shared MCP request shapes work, but harness readiness is
  still false.

Decision:

- The remaining high-risk completion gate is still M6 migration. Even another read-only
  inventory/review-export pass requires explicit user-approved scope.
- Adapter or hook writes also remain explicitly gated.
- The next step should be user-approved M6 scope, user-approved harness adapter/hook repair, or a
  new evidence-backed narrow slice if the user wants to keep avoiding those gates.

## T30/T31 Documentation And Live-State Audit

Status: completed as a documentation/evidence audit. No source behavior, migration flow, lifecycle
status, hook, adapter, schema, storage, public MCP surface, telemetry formula, ranking, or `orient`
payload changed.

Research question:

- After the T30 architecture/research-method doc sync commits and T31 startup retrieval, does the
  completion matrix change, or do the same approval gates still define the next major product work?

Hypotheses:

- Preferred: the matrix remains stable, with better synchronized docs and fresher evidence for the
  same gates.
- Null: T30/T31 add no new completion evidence.
- Simpler alternative: rely on the T29 audit only.
- Failure: the audit implies approval for migration inventory, lifecycle mutation, hook/adapter
  writes, ranking changes, schema/storage changes, or `orient` expansion.

Measurement:

- T30 committed two non-gated doc syncs: `cb39282` for
  `docs/BRAIN_HARNESS_ARCHITECTURE.md` and `42ed92c` for
  `docs/BRAIN_HARNESS_RESEARCH_METHOD.md`.
- T31 startup lean `orient` trace `019e6a25-a81f-7a00-807f-4b5c30c91432` returned current-plan
  memory `019e6a24-99c8-7043-89a5-b363ca755460` first, while stale repository current-plan memory
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` still appeared as lower-ranked noise.
- Direct T31 searches again surfaced historical no-evidence architecture/council memories and stale
  migration-completion memory below current guidance.
- `git status --short` showed only the user-owned untracked root `AGENTS.md`.
- The daemon was still running on port `8765`, PID `50257`, with installed binary hash
  `7d9256dc2ca9fcefaaa54bf620c15989fa20926c929d9e6beca27012b6afc9cf`.
- Read-only `harness(action=doctor)` still returned `ready=false` for Claude Code, Codex, Gemini
  CLI, and Cursor.
- Read-only `lint(action=run, limit=80)` reported `feedback_stale_current_plan` for
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` with 79 recent stale feedback records and
  `safe_action=none`.
- Before scoring the new T31 traces, `real_session_eval(project=engram, limit=50)` returned
  `trace_count=50`, `feedback_trace_count=38`, `feedback_coverage=0.7599999904632568`,
  `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, `confidence_gate.passed=true`, and
  `external_session_trace_count=0`.
- After scoring the T31 startup traces, the same report returned `feedback_trace_count=44`,
  `feedback_coverage=0.8799999952316284`, `bad_memory_used_count=0`,
  `confidence_gate.passed=true`, and `external_session_trace_count=0`.

Result:

- T30 improved documentation synchronization without changing product behavior.
- T31 reconfirmed current-plan retrieval for the current continuation prompt.
- Evidence-loop coverage remains sample-window sensitive and the latest sampled traces still have no
  external session labels.
- Stale historical memories remain visible as review signals, not automatic lifecycle actions.
- Harness readiness remains false for all supported harnesses.

Decision:

- M6 remains approval-gated before even read-only inventory/review-export.
- Adapter or hook writes remain separately gated.
- The next non-gated work should be another narrow evidence-quality, validation, or documentation
  synchronization slice unless the user explicitly approves one of the gated paths.

## T32 Lint Evidence-Prioritization Slice

Status: completed as a narrow lint report-ordering change. No source data, lifecycle status,
migration flow, hook, adapter, schema, storage, public MCP request shape, telemetry formula,
ranking, or `orient` payload changed.

Research question:

- Can lint report ordering make review-critical feedback signals visible under normal limits
  without changing any memory lifecycle authority?

Hypotheses:

- Preferred: a deterministic private priority sort can surface stale current-plan and wrong-scope
  feedback before duplicate-entity, unresolved-obligation, and archive-safe lifecycle noise.
- Null: agents already scan enough findings, so order has little practical value.
- Simpler alternative: document the lint-noise caveat only.
- Failure: the ordering implies cleanup authority or changes which findings are generated.

Measurement:

- T32 startup lean `orient` trace `019e6a2b-cfd9-7ca2-8c6c-a19adced80e1` returned current-plan
  memory `019e6a2a-45db-7110-8000-a86cc4970a76` first. Direct T32 current-plan search trace
  `019e6a2b-f4a1-7b10-ab0c-d2c6491b021e` also returned it first, with stale repository-scoped
  current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` still second.
- Live pre-change `lint(action=run, limit=80)` showed `duplicate_entity_candidate` findings at the
  top while the stale current-plan finding appeared lower in the report.
- Source inspection found `LintService::run` sorted findings lexicographically by ID before
  applying `limit`.
- The patch changes only private finding priority before truncation. The generated finding IDs,
  rule names, messages, safe actions, and MCP schema stay unchanged.
- Focused validation passed:
  - `cargo test -p engram-index lint`
  - `cargo test -p engram-tests --test lint_tests`
  - `cargo fmt --all --check`
  - `cargo check -p engram-cli`
  - `git diff --check`
- A direct read-only CLI smoke against the live DB failed because the running daemon held the
  RocksDB lock, so installed-runtime validation used MCP after install/restart.
- After installing binary `62db1e301ef7913ad685caa39d96ce0c479fc160fff3e8002df66401f619fce9`
  and restarting the daemon on port `8765`, PID `85531`, live
  `lint(action=run, limit=10)` returned `feedback_stale_current_plan` for
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` first, followed by wrong-scope feedback findings. The
  finding still has `safe_action=none`.
- After scoring the T32 startup retrieval traces, `real_session_eval(project=engram, limit=50)`
  returned `feedback_trace_count=49`, `feedback_coverage=0.9800000190734863`,
  `bad_memory_used_count=0`, `confidence_gate.passed=true`, and
  `external_session_trace_count=0`.

Result:

- The stale current-plan review signal is no longer buried behind duplicate-entity,
  unresolved-obligation, or archive-safe lifecycle noise under small lint limits.
- This is evidence-quality work only. It does not archive, supersede, scope-correct, delete,
  migrate, or authorize any lifecycle write.

Decision:

- Treat T32 as a lint usability improvement, not lifecycle cleanup authority.
- M6 inventory/export/apply/deletion and harness adapter or hook writes remain explicit approval
  gates.

## T33 Claude Code Lint-Ordering Parity Smoke

Status: completed as read-only cross-harness validation. No source behavior, lifecycle status,
migration flow, hook, adapter, schema, storage, public MCP request shape, telemetry formula,
ranking, or `orient` payload changed.

Research question:

- Does Claude Code, through its own Engram MCP path, observe the same T32
  `lint(action=run, limit=10)` priority ordering that Codex observes?

Hypotheses:

- Preferred: Claude Code returns `feedback_stale_current_plan` for
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` first with `safe_action=none`.
- Null: Codex-only installed-runtime validation is sufficient for this report-ordering surface.
- Simpler alternative: document T32 without another parity smoke.
- Failure: Claude Bridge tool exposure or prompt-created synthetic obligations make the result
  unusable.

Measurement:

- AI Council prior-decision recall found no directly matching T32 lint-ordering consultation.
- Claude Bridge ran a read-only `harness=personal` task with `write=false`, allowing only
  `mcp__engram__lint` and `mcp__engram__obligations`.
- Claude Code reported `lint(action="run", limit=10)` was available and returned ten findings with
  `applied_safe_actions=0`.
- The first finding matched the expected T32 result exactly:
  - rule: `feedback_stale_current_plan`
  - id: `feedback-stale-current-plan:019e5e0a-86b4-73e3-aa9b-ca350e83e915`
  - item id: `019e5e0a-86b4-73e3-aa9b-ca350e83e915`
  - title: `Current-plan guidance has stale feedback`
  - safe action: `none`
- Claude Code's follow-up obligations doctor reported one prompt-created
  `design_context_reading` obligation. Codex resolved it as `design_context_read` using the actual
  T33 startup design-context reads already completed before selecting this slice. A follow-up Codex
  `obligations(action=doctor, project=engram, cwd=/Users/yuval.meiri/projects/engram)` returned
  `open=[]` and `warnings=[]`.
- After scoring the T33 startup retrieval traces, `real_session_eval(project=engram, limit=50)`
  returned `feedback_trace_count=47`, `feedback_coverage=0.9399999976158142`,
  `bad_memory_used_count=0`, `confidence_gate.passed=true`, and
  `external_session_trace_count=0`.

Result:

- The T32 lint ordering result is now validated through Claude Code's MCP path, not only Codex.
- The validation remains narrow: it covers this read-only `lint` report shape and does not prove
  hook readiness, adapter/settings readiness, lifecycle cleanup safety, migration authority, broad
  ranking quality, or `orient` behavior.
- Synthetic cross-harness prompts can still create startup obligations, so future smokes must run
  doctor and close artifacts.

Decision:

- Treat T33 as cross-harness evidence for the shared MCP `lint` surface only.
- Keep M6 and harness writes approval-gated.
