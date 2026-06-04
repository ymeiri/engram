# T232: Project-Name Current-Plan Tag Limit Fixture

Date: 2026-06-04
Status: source-test hardening committed pending refreshed runtime gate
Scope: focused test coverage for the exact stale live MCP `memory(action="list")` request shape
that combines `project_name`, active status, `tags=["current-plan"]`, and `limit`.

## Research Question

Does source fixture coverage prove that
`memory(action="list", project_name="engram", status_filter="active",
tags=["current-plan"], limit=5)` infers project scope, filters tags, and applies the requested
limit after filtering, so startup-style current-plan sampling cannot return another project's
current-plan item or exceed the requested limit after the pending runtime refresh?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The existing T221/T223 source behavior already satisfies the combined live request shape; adding a focused fixture should pass and reduce ambiguity before runtime validation. |
| Null | The combined `project_name` plus `tags` plus `limit` path can still leak wrong-project rows, include untagged rows, or return too many matching rows. |
| Simpler alternative | Rely on T225 `project_name + limit` and T227 `project_name + tags` separately. Rejected because T231 reproduced the stale live leak under the combined startup-style request. |
| Failure | The fixture hides production behavior changes, public MCP request/response changes, ranking or `orient` changes, schema/storage/index/document-index changes, lifecycle or migration work, harness writes, runtime refresh, native Claude, deletion, rollback, or user-owned-file edits. |

## Measurement

Add one fixture:

```text
cargo test -p engram-tests test_mcp_memory_list_project_name_current_plan_tags_preserves_limit --test memory_tests -- --exact
```

Fixture shape:

- add six active Engram project-scoped `decision` MemoryItems tagged `current-plan`;
- add a newer active `voice-layer` project-scoped `decision` MemoryItem tagged `current-plan`;
- add a newer active Engram project-scoped untagged `project_fact`;
- call `memory(action="list", status_filter="active", project_name="engram",
  tags=["current-plan"], limit=5)` with `scope_type` omitted;
- assert the result count is exactly `5`, every returned row is scoped to `engram`, and every
  returned row carries the `current-plan` tag.

This catches three failure modes in one request: missing project-name scope inference, missing tag
filtering, and missing post-filter limit truncation.

## Evidence

T231's read-only live stale-runtime audit reproduced the exact user-facing risk:

```text
memory(action="list", project_name="engram", status_filter="active",
       tags=["current-plan"], limit=5)
```

The live daemon returned the active Engram current plan plus an out-of-scope `voice-layer`
current-plan item. T225 and T227 covered adjacent halves of the source behavior, but no deterministic
fixture combined scope inference, current-plan tag filtering, and a post-filter limit in the same
request shape.

## Change

Added `test_mcp_memory_list_project_name_current_plan_tags_preserves_limit` in
`engram-tests/tests/memory_tests.rs`. No production code changed.

## Validation

Passed:

```text
cargo test -p engram-tests test_mcp_memory_list_project_name_current_plan_tags_preserves_limit --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_project_name_scope_inference_filters_current_plan_tags --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_project_name_scope_inference_preserves_limit --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_project_name_implies_project_scope_before_limit --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_applies_limit_after_scope_filter --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_filters_by_tags_before_limit --test memory_tests -- --exact
cargo test -p engram-tests --test memory_tests
cargo fmt --all --check
cargo check -p engram-cli
git diff --check
```

The full `memory_tests` target passed with 35 tests.

## Boundaries

T232 is test-only. It does not change production source, public MCP request parameters or response
shape, ranking, `orient` payload, schema/storage/index/document-index behavior, lifecycle state,
M6/migration/quarantine state, harness files/settings/hooks/adapters, installed runtime, native
Claude state, deletion, rollback, or user-owned files.

Because T232 changes binary-relevant `engram-tests` after T230, T230 is now stale for exact
execution under its deny-by-default invariant. A refreshed runtime approval packet must supersede
T230 before any install/restart/live validation.

## AI Consultation

No AI Council or Claude Bridge consultation was used. This was a narrow fixture-hardening slice
driven by observed live stale-runtime behavior and already-implemented source logic, with no
architecture, ranking, migration, data-model, or irreversible decision.
