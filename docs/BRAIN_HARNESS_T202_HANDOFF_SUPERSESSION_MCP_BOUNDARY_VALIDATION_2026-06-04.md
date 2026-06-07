# T202 Handoff Supersession MCP Boundary Validation

Date: 2026-06-04
Status: test-only validation complete
Scope: MCP boundary coverage for the T201 rolling handoff supersession behavior

## Decision

T202 adds an integration test proving the public MCP `handoff(action="update")` path exercises the
T201 semantics: after two non-dry-run updates in the same project, the second response reports the
first handoff as `previous_id`, the new handoff remains active, the new item links the first item in
`supersedes`, and the first item is stored with status `superseded`.

No production behavior changed in this slice.

## Research Question

Does the MCP handoff tool boundary preserve the T201 source-level guarantee, or was the guarantee
only proven at the service unit-test layer?

## Hypotheses

| Type | Result |
| --- | --- |
| Preferred | A focused MCP integration test can prove the public tool path reaches the same `HandoffService::update` semantics. Supported. |
| Null | Service tests are sufficient and MCP coverage adds no useful confidence. Rejected because the public tool path carries request parsing, writer parsing, and shared service state. |
| Simpler alternative | Rely on `harness_tests` hook-event coverage only. Rejected because hook-event tests do not assert previous handoff lifecycle status. |
| Failure | The test reveals a request parsing or writer setup mismatch that prevents the MCP surface from using T201 behavior. Not observed. |

## Implementation

Changed `engram-tests/tests/harness_tests.rs` only:

- exposed the in-memory `MemoryService` from the existing test setup helper so the test can inspect
  the stored previous handoff after driving the public MCP tool path;
- added `handoff_request` test helper for compact `HandoffRequest` construction;
- added `test_mcp_handoff_update_supersedes_previous_handoff`.

## Validation

Commands run:

```text
cargo test -p engram-tests --test harness_tests test_mcp_handoff_update_supersedes_previous_handoff -- --exact
cargo test -p engram-tests --test harness_tests
cargo fmt --all --check
cargo check -p engram-cli
git diff --check
```

All final commands passed.

The first focused test run failed because the test attempted to parse `Id` via `str::parse`, while
Engram IDs use `Id::parse`. The test was corrected and rerun successfully. A later
`cargo fmt --all --check` also caught line wrapping in the new test; `cargo fmt --all` fixed it and
the final check passed.

## Completion Matrix Delta

| Area | State After T202 | Remaining Risk |
| --- | --- | --- |
| T201 service behavior | Covered by unit tests | None found |
| MCP handoff update boundary | Covered by focused integration test | Installed runtime not refreshed |
| Hook-event callers | Existing harness tests still pass | Installed hooks/settings/adapters not edited |
| Existing stale handoffs | Unchanged | Lifecycle cleanup remains separate and gated |
| Broad Brain Harness completion | Still incomplete | External-session labels, M6, document visibility, native-Claude/effective-hook evidence, and existing stale-memory cleanup remain open |

## Non-Actions

T202 did not change production code, public MCP parameters, `orient`, ranking,
schema/storage/index/document-index behavior, lifecycle state, installed runtime, hooks, adapters,
settings, M6/migration/quarantine state, native Claude, deletion, rollback, or user-owned files.
