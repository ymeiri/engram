# Brain Harness T140 Approval-Gate Context Search Fix

Date: 2026-06-02
Status: completed source fix and focused validation
Scope: direct unified `search` ranking for continuation prompts that mention approval gates as
context

No lifecycle archive/apply, M6 action, quarantine inspection, harness install, hook/settings/adapter
write, schema/storage/index change, public MCP change, document-index behavior change, `orient`
payload change, or handoff semantic change was run for T140.

## Research Question

Can Engram keep the latest project current-plan MemoryItem first for direct continuation searches
that mention approval gates as context, without weakening explicit gate-action prompts or changing
`orient`?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The live miss is caused by over-broad gate classification: `approval gate` context disables current-plan promotion and allows old active handoffs to lead. A narrow classifier repair can fix this prompt class while preserving explicit gate/action behavior. |
| Null | The miss is only live-corpus noise and should not change source ranking. |
| Simpler alternative | Keep relying on lean `orient`, which already returned the T139 current plan first, and leave direct search noisy. |
| Failure | The repair turns explicit approval/action prompts into current-plan prompts, hides gate context, or expands ranking behavior beyond the tested continuation-with-gate-context class. |

## Baseline Evidence

- Lean `orient` trace `019e8866-0dd0-7e81-b9fe-ae03933876ff` returned T139 current-plan memory
  `019e8864-8711-7641-8b9b-854958e76bd8` first.
- Direct unified `search` trace `019e8866-0e96-7b73-a107-e4a756684bf0` for
  `current plan next step continue move forward Engram Brain Harness after T139 T135 T139 approval
  gate` returned old active rolling handoffs above the T139 current-plan memory.
- Source inspection showed `promote_current_plan_for_continuation_query` already promotes
  current-plan guidance, but `should_promote_current_plan` returned false when
  `asks_for_decision_gate` classified any `approval gate` phrase as gate mode.
- A second post-sort helper, `promote_approval_gate_items_for_gate_query`, also promoted approval
  gate items from raw substring matching instead of the refined gate-intent boundary.

## AI Review

- AI Council recall found prior ranking-slice guidance: keep fixes prompt-class local, fixture-led,
  and separate from lifecycle cleanup or `orient` expansion.
- Fresh AI Council broadcast agreed the safe boundary is to distinguish explicit gate-action intent
  from continuation-with-gate-context, with tests for the live handoff-distractor shape and explicit
  gate/action regressions.
- Claude Bridge was attempted twice with `write=false` and isolated harness. Both attempts timed
  out, so Claude Bridge provided no supporting conclusion for this slice.

## Implementation

T140 changes only `engram-index/src/memory_ranker.rs` and
`engram-tests/tests/search_tests.rs`:

- `asks_for_decision_gate` now treats `approval gate` wording inside a current-plan/continuation
  prompt as context unless the query also has gate-summary or handoff-summary intent.
- Explicit modal/action gate prompts still classify as gate mode, including `should we proceed`,
  `should we run migration_review_export`, and `approved M6 write apply deletion cleanup legacy
  simplification now`.
- `promote_approval_gate_items_for_gate_query` now uses the same gate-intent boundary instead of
  raw `approval gate` substring matching.
- Added a deterministic integration fixture,
  `test_memory_search_t140_continuation_with_approval_gate_context_promotes_current_plan`, with
  old active rolling handoff distractors, a retrievable M6 gate, and a latest project-scoped
  `current-plan` item.
- Added ranker unit coverage for continuation-with-approval-gate context and explicit gate-action
  regressions.

## Validation

The new tests failed before the source repair:

- `cargo test -p engram-index continuation_with_approval_gate_context_promotes_current_plan`
  failed because the classifier disabled current-plan promotion.
- `cargo test -p engram-tests --test search_tests
  test_memory_search_t140_continuation_with_approval_gate_context_promotes_current_plan` failed
  because a non-current-plan item ranked above the current-plan fixture.

Final-tree validation passed:

| Check | Result |
| --- | --- |
| `cargo test -p engram-index memory_ranker::tests -- --nocapture` | Passed, 11 tests |
| `cargo test -p engram-tests --test search_tests -- --nocapture` | Passed, 32 tests |
| `cargo fmt --all --check` | Passed |
| `cargo check -p engram-cli` | Passed |
| `git diff --check` | Passed |

## Completion Matrix Delta

| Area | T140 status | Evidence |
| --- | --- | --- |
| Direct current-plan/next-step retrieval | Improved for one prompt class | The live-shape T140 fixture now ranks the latest project current-plan above old active handoffs even when the query mentions approval gates as context. |
| Explicit gate/action retrieval | Preserved | Existing migration/gate search tests and new classifier assertions keep `should we proceed/run/apply/export` and approved M6 apply/delete/cleanup prompts in gate mode. |
| Approval-gate context visibility | Preserved | The T140 fixture asserts the M6 approval-gate context remains retrievable. |
| `orient` hot path | Unchanged | No `orient` payload, ranking contract, obligation, graph, or migration behavior changed. |
| Lifecycle hygiene | Still gated | No archive/apply or handoff lifecycle cleanup ran; T139 remains pending approval. |
| Installed harness readiness | Still gated | No harness install or local adapter/settings/hook write ran; T135 remains the exact repair gate. |
| Installed runtime parity | Not validated | Source tests passed, but no binary install or daemon restart was run for T140. |

## Decision

T140 is a narrow source-level search ranking repair. It fixes a live continuation-with-approval-gate
context miss without changing `orient`, lifecycle state, installed harnesses, M6, schema/storage/
index, public MCP, or document-index behavior.

The next product-moving gates remain unchanged:

- T139 requires exact approval before archiving stale current-plan MemoryItem
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915`.
- T135 requires exact approval before any harness install/write repair.
- M6 migration completion remains review-gated and requires explicit approval before candidate
  decisions, write-apply, deletion, cleanup, or quarantine inspection.

## Stop Conditions For Follow-Up

Stop and ask before any follow-up that would:

- broaden ranking beyond the tested continuation-with-approval-gate-context prompt class;
- change `orient`, graph, lint, telemetry formula, public MCP, schema/storage/index, or
  document-index behavior;
- archive, supersede, reject, delete, or otherwise mutate MemoryItem lifecycle state;
- run `lint(action="apply_safe")`;
- run M6 migration inventory/export/status/prioritize/apply, inspect quarantine candidates, delete,
  clean up, or simplify legacy layers;
- run `harness(action="install")`, write installed hooks/settings/adapters/commands/skills, use
  `adopt_user_owned=true`, or edit user-owned files.
