# T201 Handoff Supersession Semantics

Date: 2026-06-04
Status: source implementation complete; runtime refresh not run
Scope: Prevent future rolling handoff accumulation by superseding the previous matching handoff on write

## Decision

`HandoffService::update` now mirrors current-plan capture semantics for future writes: when a
non-dry-run handoff update writes a new rolling handoff and links the immediately previous matching
handoff in `supersedes`, it also marks that previous handoff `superseded` and saves it with
tool-call evidence.

The new handoff is saved before the previous handoff status update. If the second save fails, the
system can fall back to the previous pre-T201 state of two active handoffs rather than losing the
new active handoff.

Dry-run behavior remains read-only. The returned planned item still shows the `supersedes` edge
that would be written, but no item is persisted and the previous handoff remains active.

## Research Question

Can Engram reduce future stale active rolling handoff noise at the write source, without archiving
existing memory, changing search ranking, or expanding `orient`?

## Hypotheses

| Type | Result |
| --- | --- |
| Preferred | Marking only the immediately previous matching handoff `superseded` on write prevents future active-handoff chains while preserving the latest handoff. Supported. |
| Null | Handoff noise can only be handled by ranking changes or lifecycle cleanup of existing items. Rejected for future writes; still true for pre-T201 stale handoffs. |
| Simpler alternative | Leave semantics unchanged and rely on agents to call `handoff(get)`. Rejected because the stale active chain repeatedly polluted search and lint evidence. |
| Failure | The change mutates dry-run state, touches unrelated project/session handoffs, or implies existing stale handoffs were cleaned up. Not observed in tests; existing stale handoffs remain out of scope. |

## Implementation

Changed `engram-index/src/handoff.rs` only:

- kept current planned-item behavior that links the previous matching handoff through
  `supersedes`;
- in write mode, saved the new active handoff first;
- then saved the previous matching handoff with `MemoryStatus::Superseded` and tool-call evidence
  naming the new handoff;
- preserved dry-run as zero-write behavior;
- added focused service tests for dry-run, project-scope write behavior, project isolation, and
  `compile(..., dry_run=false)`.

This is preventive only. It does not archive, reject, delete, or otherwise mutate any existing
stored handoff outside the tests.

## AI Review

AI Council recall found prior guidance that lifecycle cleanup, broad ranking, and hot-path changes
should remain separate from narrow handoff work. A three-model AI Council broadcast agreed that
write-time supersession is the smallest source-local repair and recommended saving the new handoff
before marking the previous one superseded, with tests for dry-run, scope isolation, and compile.

Claude Bridge reviewed the plan in read-only isolated mode and agreed the slice is minimal. Claude
called out three caveats that this report preserves:

- the change is preventive, not restorative;
- the authoritative supersession target is the previous item already linked on the new handoff;
- partial failure after saving the new handoff can still leave two active handoffs, which is the
  existing fallback state.

## Validation

Commands run:

```text
cargo test -p engram-index handoff
cargo test -p engram-tests --test harness_tests
cargo test -p engram-tests --test lint_tests
cargo test -p engram-tests --test memory_tests test_mcp_orient_prepare_handoff_lean_surfaces_current_plan_and_gates -- --exact
cargo fmt --all --check
cargo check -p engram-cli
git diff --check
```

All passed.

## Completion Matrix Delta

| Area | State After T201 | Remaining Risk |
| --- | --- | --- |
| Future rolling handoff writes | Previous matching handoff is marked `superseded` in write mode | Runtime-installed binary not refreshed in this slice |
| Dry-run handoff planning | Still read-only and returns the planned supersedes edge | None found |
| Existing stale active handoffs | Unchanged | T187/T191/T193 and broader lifecycle cleanup remain exact-gated |
| Search and `orient` | Unchanged | Old active handoffs can still appear until lifecycle cleanup or installed-runtime refresh happens |
| Hooks/harness behavior | Source callers use the same update path when this binary is installed | Installed hooks/settings/adapters were not edited |
| M6/migration | Unchanged | Candidate decisions, dry-run/apply evidence, rollback plan, and explicit migration approval remain incomplete |

## Non-Actions

T201 did not:

- archive, reject, delete, review, or mutate existing live MemoryItems;
- run `lint(action="apply_safe")`;
- change search ranking, `orient`, public MCP request parameters or payload shape;
- change schema/storage/index/document-index behavior;
- edit hooks, settings, adapters, user-owned files, or installed runtime configuration;
- run native Claude, Claude Bridge write actions, M6/migration/quarantine actions, runtime
  refresh, deletion, rollback, or old-binary reinstall.
