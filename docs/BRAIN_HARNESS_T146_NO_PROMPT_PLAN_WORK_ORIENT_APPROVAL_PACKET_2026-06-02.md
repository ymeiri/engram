# T146 Approval Packet: No-Prompt PlanWork Orient Current-Plan Boundary

Date: 2026-06-02
Status: pending user approval
Scope: approval request for one narrow `orient` source fixture and implementation slice

This packet is a request for approval, not approval itself. No source change, runtime refresh,
`orient` behavior change, ranking-source broadening, public MCP parameter change, schema/storage/
index change, document-index behavior change, lifecycle mutation, harness install or hook/settings/
adapter write, M6/migration/quarantine action, user-owned file edit, shell/PATH/service change,
rollback, force-kill, deletion, or old-binary reinstall has been run for T146.

## Research Question

Should no-prompt `orient(project="engram", cwd="/Users/yuval.meiri/projects/engram",
intent="plan_work", response_shape="lean")` at a resolved project task boundary surface the latest
active project current-plan guidance first, without changing `orient` payload shape or broad
ranking behavior?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The T145 failure is a boundary-classification gap: no-prompt `plan_work` has no query text, so it bypasses the existing current-plan promotion path even though it is semantically a task-boundary planning request. A narrow predicate extension plus deterministic fixture can make the latest active current plan lead only for no-prompt project-scoped `plan_work`. |
| Null | No-prompt `plan_work` should remain generic because callers that need continuity should use `resume_session` or provide a prompt. |
| Simpler alternative | Treat the T145 result as operator error and require callers to always provide prompt text. This preserves source behavior but weakens `orient` as a frictionless task-boundary entrypoint. |
| Failure | The fix promotes current plans for ordinary implementation/debug prompts, expands the `orient` payload or public API, mutates lifecycle state, changes direct `search` ranking, or hides broader ranking churn inside a hot-path repair. |

## Current Evidence

- T145 installed the current runtime and restarted the daemon. The active binary hash is
  `3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b`, and the daemon is running
  on port 8765 with PID `10768`.
- Direct T140/T143 live search traces `019e89b6-6ff0-72a1-bc53-96aa4d1b5819`,
  `019e89b6-7037-7271-933d-71f1ba12cfb3`, and
  `019e89b6-7081-7dd0-9b5f-988c1e838c4f` returned current-plan guidance first.
- Exact no-prompt `plan_work` lean orient trace `019e89b6-6fa0-71f2-977a-f9046eaabbdf` returned
  compact output but generic guidance, not the active T145 current plan.
- Fresh reproduction trace `019e89ba-e9e6-7ef2-9904-b4d648074d83` still misses current-plan
  guidance for no-prompt `plan_work`.
- Fresh positive-control trace `019e89ba-e8a0-7b71-bfad-23dd08bca7fd` shows `plan_work` with an
  explicit continuation/current-plan prompt returns the post-T145 current plan first.
- Fresh positive-control trace `019e89ba-e945-73e1-9115-94d0217bd0e7` shows `resume_session` with
  no prompt returns the post-T145 current plan first.
- Active current-plan listing returns exactly one Engram project current-plan item:
  `019e89b8-74f0-7b72-be03-042967637f43`.

## Source Finding

Read-only source inspection found the likely root cause:

- `engram-index/src/memory.rs` promotes current-plan guidance for `resume_session` and
  `prepare_handoff` without query text.
- `plan_work` current-plan promotion calls
  `should_prioritize_current_plan_for_plan_work(intent, query)`, which currently requires
  `query.is_some_and(is_open_ended_plan_work_prompt)`.
- Brain Loop group scoring is a second affected site: without query text, groups score `0.0` and
  sort by group order, while the continuity current-plan pin currently applies only to
  `resume_session` and `prepare_handoff`.

The failure is therefore specific to no-prompt `plan_work`, not to current-plan capture,
runtime freshness, direct `search`, or explicit continuation prompts.

## AI Review

AI Council recall found prior guidance for `orient` current-plan repairs: keep the change
intent-local, do not expand payloads, do not run lifecycle cleanup, and do not introduce broad
ranking churn.

Fresh AI Council broadcast agreed that a narrow fix is reasonable if it is constrained to
no-prompt `plan_work` at a resolved project/cwd task boundary. The models recommended fixtures for:

- no-prompt `plan_work` at an Engram project boundary returns the latest active project current
  plan first in both decisions ordering and Brain Loop top-items;
- explicit implementation-oriented `plan_work` prompts do not receive forced current-plan
  promotion;
- no-prompt `plan_work` without a resolved project/current-plan boundary does not synthesize a
  promotion.

Claude Bridge foreground review timed out after 120 seconds. Read-only background Claude Bridge job
`ccb_20260602191029_1752e3af` then completed and found a material implementation caveat: fixing
only the decision-list promotion would be a half-fix because Brain Loop group ordering would still
put rules/preferences/limitations before decisions when the prompt is absent. T146 therefore must
cover both the decision-list promotion site and the Brain Loop current-plan pin site, with tests
asserting both `active_decisions.first()` and `brain_loop.top_items.first()`.

## Proposed Implementation Boundary

If approved, T146 should make only the smallest source change needed to treat no-prompt
`plan_work` at a resolved project/cwd task boundary as eligible for latest active current-plan
promotion.

Allowed implementation shape:

- extend the current-plan orientation promotion predicate for `BrainHarnessIntent::PlanWork` so
  empty or absent query text can opt into latest-current-plan promotion;
- rely on the existing post-filtered active-memory set as the implicit scope guard, so no current
  plan can be promoted unless one survived project/cwd relevance filtering;
- extend the Brain Loop continuity current-plan pin only as far as needed for the same no-prompt
  `plan_work` boundary, so the promoted current plan can lead visible `brain_loop.top_items`;
- reuse existing `prioritize_latest_current_plan` behavior;
- keep older current-plan suppression behavior unchanged for `resume_session` and
  `prepare_handoff`;
- keep `response_shape="lean"` as a presentation option only;
- add focused deterministic fixture coverage.

Disallowed implementation shape:

- no public MCP parameter, enum, or response payload change;
- no broad direct `search` ranking change;
- no new semantic/vector query for empty prompts;
- no lifecycle archive/apply/supersession beyond normal test fixtures;
- no migration, M6, quarantine, document-index, schema/storage/index, harness, settings, hook,
  adapter, user-owned file, PATH, service, rollback, force-kill, deletion, or old-binary changes.

## Required Fixture Coverage

Add or update focused tests before or with implementation:

1. `orient` no-prompt `plan_work` at project boundary:
   - seed a latest active project-scoped `decision` with `current-plan` tag;
   - seed generic rule/preference/limitation noise;
   - call `orient(project="engram", cwd="/Users/yuval.meiri/projects/engram",
     intent=PlanWork, prompt=None, response_shape=lean)`;
   - assert the current-plan item is first in `active_decisions`;
   - assert the current-plan item is first in `brain_loop.top_items`;
   - assert `used_memory_candidate_ids` includes the current-plan item;
   - assert lean shape still omits raw buckets, `context_pack`, and trust payloads.
2. Explicit implementation prompt guard:
   - same fixture state;
   - call `orient(intent=PlanWork, prompt="implement request throttling")`;
   - assert current-plan promotion is not forced above prompt-specific implementation guidance by
     the no-prompt rule.
3. Non-project/no-boundary guard:
   - no prompt and `intent=PlanWork`, but no resolved project/cwd boundary or no active current
     plan;
   - assert no synthetic current-plan promotion occurs and shape remains unchanged.
4. Existing continuity controls:
   - existing `resume_session`, `prepare_handoff`, explicit mission/open-ended `plan_work`, and
     specific reviewed-gate fixtures still pass.

## Validation Commands

Run at minimum:

```text
cargo fmt --all --check
cargo test -p engram-tests --test memory_tests orient
cargo test -p engram-tests --test search_tests current
cargo check -p engram-cli
git diff --check
```

If implementation touches shared ranking helpers or broad orientation behavior, broaden validation
to:

```text
cargo test -p engram-tests --test brain_harness_eval_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Runtime refresh is not authorized by this packet. If source changes pass and installed-runtime
validation is needed later, create a separate runtime-refresh approval packet.

## Pass Criteria

T146 implementation passes only if all of the following are true:

- no-prompt project-scoped `plan_work` returns the latest active project current-plan item first in
  Brain Loop;
- explicit continuation/current-plan `plan_work` behavior remains green;
- `resume_session` and `prepare_handoff` current-plan behavior remains green;
- explicit implementation prompts do not get current-plan first merely because a project current
  plan exists;
- no public MCP/payload/schema/storage/index/document-index/lifecycle/harness/M6/user-owned-file
  behavior changes are present;
- focused and relevant regression tests pass;
- docs, telemetry feedback, Engram current-plan memory, obligations doctor, and a focused commit
  record the result.

## Stop Conditions

Stop before implementation or commit if:

- the fixture requires broad Brain Loop group-scoring changes instead of a local current-plan
  predicate;
- any existing tested prompt class regresses;
- source changes require public MCP, schema/storage/index, lifecycle, harness, migration, M6,
  document-index, user-owned-file, PATH/service, rollback, force-kill, deletion, or runtime refresh
  work;
- evidence is ambiguous about whether a no-prompt request has a resolved project/cwd boundary.

## Completion Matrix Delta

| Area | T146 packet status | Evidence |
| --- | --- | --- |
| No-prompt `plan_work` root cause | Identified, read-only | Source and live traces show missing query text bypasses current-plan promotion. |
| Implementation approval | Pending | This packet is not approval. |
| `orient` hot path | Preserved | No source behavior changed by this packet. |
| Direct search | Validated by T145 | T145 direct search traces return current-plan first. |
| Runtime parity | Validated by T145 | Installed hash `3d801be9...`, daemon PID `10768`. |
| Harness readiness | Still gated | No harness install or hook/settings/adapter write. |
| Lifecycle cleanup | Still gated | No archive/apply or `lint apply_safe`. |
| M6 migration completion | Still gated | No M6/migration/quarantine/apply/delete/cleanup. |

## Exact Approval Phrase

```text
Approve T146: implement the narrow no-prompt PlanWork orient current-plan fix from
docs/BRAIN_HARNESS_T146_NO_PROMPT_PLAN_WORK_ORIENT_APPROVAL_PACKET_2026-06-02.md. Add focused
fixture coverage for no-prompt project-scoped plan_work asserting both active_decisions and
brain_loop top_items, explicit implementation-prompt guard, and non-project/no-current-plan guard.
Do not change public MCP params or payload shape, direct search ranking beyond shared behavior
required by this exact orient path, schema/storage/index, document-index behavior, lifecycle state,
harness files, M6/migration/quarantine, user-owned files, PATH/service configuration, rollback,
force-kill, deletion, old-binary reinstall, or runtime refresh.
```
