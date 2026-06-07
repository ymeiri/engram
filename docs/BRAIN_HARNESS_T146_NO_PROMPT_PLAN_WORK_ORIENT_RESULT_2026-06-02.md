# T146 Result: No-Prompt PlanWork Orient Current-Plan Boundary

Date: 2026-06-02
Status: source implemented and validated; runtime refresh not performed
Scope: narrow source fix for no-prompt `plan_work` orientation at a task boundary

## Research Question

Should no-prompt `orient(project="engram", cwd="/Users/yuval.meiri/projects/engram",
intent="plan_work", response_shape="lean")` surface the active current-plan item first without
expanding `orient` payload shape or broad ranking behavior?

## Result

Yes, at the source level. T146 extends the existing current-plan orientation path only for
`BrainHarnessIntent::PlanWork` when the prompt is absent or empty and a project/cwd task boundary is
present. It also applies the same narrow condition to the Brain Loop current-plan pin so the compact
hot path leads with the promoted current plan.

The change does not alter public MCP request parameters, response payload shape, direct search
ranking, schema/storage/index behavior, document indexing, lifecycle state, harness files,
migration/M6/quarantine state, user-owned files, PATH/service configuration, rollback, deletion, or
runtime installation.

Runtime refresh remains a separate approval gate. The installed daemon/runtime has not been updated
by T146.

## Source Delta

- `engram-index/src/memory.rs`
  - computes a private task-boundary flag from effective project/cwd;
  - allows latest current-plan promotion for no/empty-prompt `plan_work` only at that boundary;
  - keeps existing `resume_session` and `prepare_handoff` behavior unchanged;
  - pins Brain Loop current-plan only for resume/handoff or no/empty-prompt boundary `plan_work`.
- `engram-tests/tests/memory_tests.rs`
  - adds project-boundary no-prompt full and lean fixture coverage;
  - adds no-boundary and no-current-plan guards;
  - adds an explicit implementation-prompt guard to the prompt-ranking fixture.

## Validation

Passed:

```text
cargo test -p engram-tests test_mcp_orient_no_prompt_plan_work --test memory_tests
cargo test -p engram-tests test_mcp_orient_ranks_reviewed_decisions_by_prompt --test memory_tests
cargo test -p engram-index orient_mission_prompt_diagnostic_distinguishes_intent_from_ranking
cargo test -p engram-index open_ended_plan_work_prompt_detection_stays_narrow
cargo test -p engram-tests --test memory_tests
cargo fmt --all --check
git diff --check
cargo check -p engram-cli
cargo test -p engram-index orient_
cargo test -p engram-tests --test search_tests current
```

Pre-implementation fixture check:

```text
cargo test -p engram-tests test_mcp_orient_no_prompt_plan_work_surfaces_current_plan_at_project_boundary --test memory_tests
```

failed as expected at `active_decisions[0]`, proving the fixture captured the no-prompt current-plan
gap before the source change.

## Completion Matrix Delta

| Area | Status | Evidence |
| --- | --- | --- |
| No-prompt project `plan_work` current-plan retrieval | Source implemented and validated | New MCP fixture asserts `active_decisions[0]` and `brain_loop.top_items[0]` are the current-plan item for full and lean orientation. |
| Explicit implementation prompt guard | Validated | `implement request throttling` continues to rank the prompt-specific decision before the current-plan item. |
| No-boundary/no-current-plan guard | Validated | New fixture proves unscoped no-prompt `plan_work` does not surface project-scoped current plan and project-scoped no-current-plan orientation stays rule-led. |
| Mission/open-ended `plan_work` continuity | Preserved | Existing service diagnostic still passes and does not apply resume/handoff Brain Loop pin to mission-class prompts. |
| Direct search current-plan behavior | Preserved | `search_tests current` subset passed. |
| Runtime parity | Deferred | No install, daemon restart, PATH/service change, or live runtime validation was authorized or performed. |
| M6/migration/quarantine | Still gated | No migration, quarantine, deletion, cleanup, or lifecycle write was performed. |

## Next Action

After this source commit, the next approval-gated step is a runtime refresh/installed-runtime
validation if the user wants live Codex/Claude daemon behavior to reflect T146. M6 migration
completion remains a separate high-risk gate and is not changed by T146.
