# Brain Harness T89 Changes Since Cursor Ergonomics

Status: Implemented and locally validated
Date: 2026-06-01
Scope: Existing `memory(action="changes_since")` cursor error clarity and `orient` contract wording

This slice fixes a small continuity papercut found during startup: `orient` returned a
`memory_cursor` with both `commit_id` and `timestamp`, but a commit-id-only
`memory(action="changes_since")` call failed with the terse error `timestamp required for
changes_since`. The behavior was correct because memory item changes are timestamp-based, but the
error did not explain how to use the cursor.

T89 does not change request parameters, cursor semantics, ranking, `orient` payload shape,
document indexing, migration state, lifecycle state, schema/storage/index behavior, public MCP
surface area, or harness hooks/adapters.

## Research Question

Can Engram make the `orient` to `changes_since` continuity loop harder to misuse without changing
cursor semantics or expanding the hot path?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Keep `timestamp` required, but make the error and contract explicitly say to pass `memory_cursor.timestamp` and optionally `memory_cursor.commit_id`. |
| Null | The existing `timestamp required for changes_since` error is sufficient. |
| Simpler alternative | Only document the requirement, leaving the runtime error unchanged. |
| Failure | The change implies that `commit_id` alone is a valid cursor or changes public MCP parameters. |

## Measurement

Before implementation:

- Lean `orient` returned a cursor with `commit_id` and `timestamp`.
- A live commit-id-only `memory(action="changes_since")` call returned `timestamp required for
  changes_since`.
- Live timestamp-based `changes_since` calls worked and produced traces
  `019e8307-7f16-73b2-bb90-bdb740c49cd9` and
  `019e8307-7f48-7163-8d15-fca9802c7572`.
- Source inspection showed the MCP request already exposes both `commit_id` and `timestamp`, while
  `MemoryService::changes_since_with_options` lists changed memory items and knowledge commits
  after `cursor.timestamp`.

After implementation:

- `changes_since` still requires the timestamp.
- If `commit_id` is supplied without `timestamp`, the MCP error now explains that `commit_id` is not
  a replacement for `memory_cursor.timestamp`.
- `docs/ORIENT_CONTRACT.md` now tells agents to pass `memory_cursor.timestamp` and optionally
  `memory_cursor.commit_id`.
- Targeted test:
  `cargo test -p engram-tests --test memory_tests test_mcp_memory_changes_since_commit_id_error_names_cursor_timestamp`.

## Completion Matrix

| Area | Status | Evidence | Remaining risk |
| --- | --- | --- | --- |
| Cursor semantics | Preserved | `MemoryService` still keys `changes_since` from `cursor.timestamp` | Agents must still preserve the timestamp from `orient` |
| MCP request shape | Unchanged | Existing `timestamp` and `commit_id` fields remain | None for this slice |
| Runtime guidance | Improved | Commit-id-only path now names `memory_cursor.timestamp` and `memory_cursor.commit_id` | Other clients may still omit cursor fields entirely |
| Orient contract | Updated | Feedback expectations document cursor usage | Generated harness text may still be less explicit |
| Validation | Targeted | New MCP test passes | Broader test suite still required before product-completion claims |
| Gated surfaces | Untouched | No archive, migration, document indexing, ranking, `orient`, schema/storage, or harness write | T69/T70/T88 remain exact-approval gated |

## Result

The smallest safe fix is an error-message and contract update. It turns a misleading dead end into
an actionable instruction while preserving the existing cursor model: `timestamp` is the required
clock for memory item freshness, and `commit_id` is optional context for knowledge commits.
