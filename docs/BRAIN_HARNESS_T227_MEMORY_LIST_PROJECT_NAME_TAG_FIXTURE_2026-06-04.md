# T227: Project-Name Current-Plan Tag Fixture

Date: 2026-06-04
Status: source-test hardening committed pending runtime refresh
Scope: focused test coverage for startup-style MCP `memory(action="list")` calls that combine
`project_name` with `tags=["current-plan"]`.

## Research Question

Does source fixture coverage prove that `memory(action="list", project_name="engram",
tags=["current-plan"])` infers project scope before tag filtering, so startup-style current-plan
sampling cannot return another project's current-plan item after the pending runtime refresh?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The T221 source fix already covers this path because scope inference runs before tag filtering; adding a focused fixture should pass and preserve startup-style evidence quality. |
| Null | Project-name-only scope inference and tag filtering interact incorrectly, allowing out-of-project current-plan items to remain visible. |
| Simpler alternative | Rely on separate project-name-only, tag-only, and combined project-name-plus-limit fixtures. Rejected because the live startup-style call uses tags and just returned out-of-scope evidence from the stale installed runtime. |
| Failure | The fixture hides behavior, public MCP, response-shape, ranking, `orient`, schema/storage/index, lifecycle, migration/quarantine, harness, runtime, native Claude, deletion, rollback, or user-owned-file changes. |

## Measurement

Add one fixture:

```text
cargo test -p engram-tests test_mcp_memory_list_project_name_scope_inference_filters_current_plan_tags --test memory_tests -- --exact
```

Fixture shape:

- add an Engram project-scoped MemoryItem tagged `current-plan`;
- add an untagged Engram project-scoped MemoryItem;
- add a newer `voice-layer` project-scoped MemoryItem tagged `current-plan`;
- call `memory(action="list", status_filter="active", project_name="engram",
  tags=["current-plan"])` with `scope_type` omitted;
- assert `count == 1`, title `Engram tagged current plan`, and Engram project scope.

## Evidence

During T227 startup, a read-only live MCP call against the stale installed runtime:

```text
memory(action="list", project_name="engram", status_filter="active",
       tags=["current-plan"], limit=5)
```

returned the active Engram current-plan item plus an out-of-scope `voice-layer` current-plan item.
That runtime symptom is expected before the T221/T223/T225/T227 refresh because the installed
binary still predates project-name-only scope inference. The missing source evidence was a fixture
for the exact startup-style tag path.

## Change

Added `test_mcp_memory_list_project_name_scope_inference_filters_current_plan_tags` in
`engram-tests/tests/memory_tests.rs`. No production code changed.

## Validation

Passed:

```text
cargo test -p engram-tests test_mcp_memory_list_project_name_scope_inference_filters_current_plan_tags --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_project_name_implies_project_scope_before_limit --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_project_name_scope_inference_preserves_limit --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_filters_by_tags_before_limit --test memory_tests -- --exact
cargo fmt --all --check
cargo test -p engram-tests --test memory_tests
cargo check -p engram-cli
git diff --check
```

The full `memory_tests` target passed with 34 tests.

## Boundaries

T227 is test-only. It does not change production source, public MCP request parameters or response
shape, ranking, `orient` payload, schema/storage/index/document-index behavior, lifecycle state,
M6/migration/quarantine state, harness files/settings/hooks/adapters, installed runtime, native
Claude state, deletion, rollback, or user-owned files.

Because T227 changes binary-relevant `engram-tests` after T226, T226 is now stale for exact
execution under the packet's deny-by-default invariant. A refreshed runtime approval packet must
supersede T226 before any install/restart/live validation.

## AI Consultation

No AI Council or Claude Bridge consultation was used. This was a narrow fixture-hardening slice
driven by observed live stale-runtime behavior and existing source logic, with no architecture,
ranking, migration, data-model, or irreversible decision.
