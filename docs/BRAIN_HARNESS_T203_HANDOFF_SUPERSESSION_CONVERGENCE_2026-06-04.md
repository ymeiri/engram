# T203 Handoff Supersession Convergence

Date: 2026-06-04
Status: source implementation complete; runtime refresh not run
Scope: Converge all active same-scope rolling handoff predecessors on the next write

## Decision

`HandoffService::update` now treats duplicate active rolling handoffs for one scope as a write-path
invariant breach to converge, not as a chain to consume one item at a time. The update path gathers
all active handoff MemoryItems matching the target project, session, or global scope, links every
matching predecessor in the new handoff's `supersedes` vector, saves the new active handoff first,
and then marks each predecessor `superseded` with tool-call evidence pointing at the new handoff.

`previous_id` remains the newest previous matching handoff ID, preserving the existing response
shape for callers that only need the immediate predecessor. Dry-run remains read-only: it returns
the planned supersession links but does not write the new handoff or mutate predecessors.

This is source-level convergence only. It does not mutate existing live data until the refreshed
binary is used for a future non-dry-run handoff update.

## Research Question

Should Engram keep T201's single-previous supersession semantics, or should a future handoff write
self-heal all same-scope active handoff predecessors without running a separate lifecycle cleanup?

## Hypotheses

| Type | Result |
| --- | --- |
| Preferred | Superseding all active same-scope predecessors on the next handoff write restores the one-active-handoff invariant without archive/delete, ranking, or hot-path changes. Supported by source tests. |
| Null | T201 single-previous semantics are sufficient and older active handoffs should wait for explicit lifecycle archive packets. Rejected for the future write path because it can leave pre-T201 duplicate actives indefinitely. |
| Simpler alternative | Keep read paths defensive and leave duplicate active handoffs in storage. Rejected because it pushes a writer invariant into every reader of active memory. |
| Failure | The change mutates dry-run state, over-supersedes other scopes, changes public MCP shape, or requires schema/model churn. Not observed; `supersedes` is already a vector and focused tests cover dry-run plus scope isolation. |

## AI Review

AI Council recall surfaced prior guidance to keep handoff work separate from broad lifecycle
cleanup and hot-path expansion. A three-model AI Council broadcast agreed that write-time
convergence is the smallest source-local repair for duplicate active same-scope handoffs, with
three caveats preserved here:

- save the new handoff before marking old ones superseded, so failure falls back to the previous
  duplicate-active state rather than losing the new handoff;
- do not introduce schema or ontology churn;
- keep race-condition handling and live historical cleanup separate.

Claude Bridge read-only critique was attempted in isolated mode and timed out after 120 seconds.
That timeout is a caveat, not supporting evidence.

## Implementation

Changed `engram-index/src/handoff.rs` only:

- replaced the update path's single `latest_handoff` lookup with an internal
  `active_handoffs` helper that returns all active matching handoffs in repository order;
- kept `latest_handoff` as a read helper by taking the first active matching item;
- recorded every active matching predecessor ID in the new handoff's `supersedes` vector;
- saved the new active handoff first in write mode;
- marked every previous matching active handoff `superseded` with tool-call evidence;
- added tests for dry-run purity, all-predecessor project convergence, other-project isolation,
  and existing session compile behavior.

## Validation

Commands run:

```text
cargo test -p engram-index handoff
cargo test -p engram-tests --test harness_tests
cargo fmt --all
cargo fmt --all --check
cargo check -p engram-cli
git diff --check
```

All final validation commands passed.

## Completion Matrix Delta

| Area | State After T203 | Remaining Risk |
| --- | --- | --- |
| Future rolling handoff writes | One successful write now converges all active same-scope predecessor handoffs to `superseded` | Runtime-installed binary not refreshed in this slice |
| Dry-run handoff planning | Still zero-write; planned item links all matching active predecessors | None found |
| Existing live stale active handoffs | Unchanged until a refreshed runtime performs a future write, or separately approved lifecycle cleanup runs | T187/T191/T193 and broader cleanup remain separate |
| Search and `orient` | Unchanged | Old active handoffs can still appear until live data converges or is explicitly cleaned |
| Public MCP shape | Unchanged; `previous_id` remains the newest predecessor | Existing MCP boundary test covers the service path but does not synthesize a duplicate-active live fixture |
| M6/migration and harness writes | Unchanged | Candidate decisions, dry-run/apply evidence, rollback plan, runtime refresh, and hook/settings work remain separate |

## Non-Actions

T203 did not:

- archive, reject, delete, review, or mutate existing live MemoryItems;
- run `lint(action="apply_safe")`;
- change search ranking, `orient`, public MCP request parameters, or payload shape;
- change schema/storage/index/document-index behavior;
- edit hooks, settings, adapters, user-owned files, installed runtime, or daemon configuration;
- run native Claude, Claude Bridge write actions, M6/migration/quarantine actions, runtime refresh,
  deletion, rollback, or old-binary reinstall.
