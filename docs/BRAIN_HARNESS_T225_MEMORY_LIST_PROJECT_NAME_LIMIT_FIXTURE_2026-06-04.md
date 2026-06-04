# T225: Project-Name Memory List Limit Fixture

Date: 2026-06-04
Status: source-test hardening committed pending runtime refresh
Scope: focused test coverage for the exact `memory(action="list")` live path planned for runtime
refresh validation.

## Research Question

Do source fixtures cover the combined live-path invariant that
`memory(action="list", project_name="engram", limit=1)` both infers project scope and preserves the
requested limit after in-memory scope filtering?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Existing T221 and T223 source changes already satisfy the combined behavior; a focused fixture should pass and make the T224/T226 live validation path deterministic at source level. |
| Null | Project-name-only scope inference and post-filter limit handling interact incorrectly when `scope_type` is omitted. |
| Simpler alternative | Rely on separate T221 and T223 fixtures plus live validation only. Rejected because the runtime packet's read-only validation calls the combined path, and a focused source fixture reduces ambiguity without broadening the MCP surface. |
| Failure | The fixture hides behavior, public MCP, response-shape, ranking, `orient`, schema/storage/index, lifecycle, migration/quarantine, harness, runtime, native Claude, deletion, rollback, or user-owned-file changes. |

## Measurement

Add one fixture:

```text
cargo test -p engram-tests test_mcp_memory_list_project_name_scope_inference_preserves_limit --test memory_tests -- --exact
```

Fixture shape:

- add two active Engram project-scoped MemoryItems;
- add a newer active `dd-source` project-scoped MemoryItem;
- call `memory(action="list", status_filter="active", project_name="engram", limit=1)` with
  `scope_type` omitted;
- assert `count == 1` and the returned item remains Engram project-scoped.

## Evidence

The source already had separate coverage for:

- T221: project-name-only list requests infer `scope_type="project"`; and
- T223: explicit project-scoped list requests reapply `limit` after scope filtering.

The missing evidence was the exact combined path used by the runtime-refresh packet.

## Change

Added `test_mcp_memory_list_project_name_scope_inference_preserves_limit` in
`engram-tests/tests/memory_tests.rs`. No production code changed.

## Validation

Passed:

```text
cargo test -p engram-tests test_mcp_memory_list_project_name_scope_inference_preserves_limit --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_project_name_implies_project_scope_before_limit --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_applies_limit_after_scope_filter --test memory_tests -- --exact
cargo fmt --all --check
cargo test -p engram-tests --test memory_tests
cargo check -p engram-cli
git diff --check
```

The full `memory_tests` target passed with 33 tests.

## Boundaries

T225 is test-only. It does not change production source, public MCP request parameters or response
shape, ranking, `orient` payload, schema/storage/index/document-index behavior, lifecycle state,
M6/migration/quarantine state, harness files/settings/hooks/adapters, installed runtime, native
Claude state, deletion, rollback, or user-owned files.

Because T225 changes binary-relevant `engram-tests` after T224, T224 is now stale for exact
execution under the packet's deny-by-default invariant. A refreshed runtime approval packet must
supersede T224 before any install/restart/live validation.

## AI Consultation

No AI Council or Claude Bridge consultation was used. This was a narrow fixture-hardening slice for
an already-approved source behavior path, with no architecture, ranking, migration, data-model, or
irreversible decision.
