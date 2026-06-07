# Brain Harness T221 Memory List Project Scope Inference

Date: 2026-06-04
Status: completed source-level implementation

## Scope

This slice fixes a narrow MCP contract pitfall in `memory(action="list")`: when a caller supplied
`project_name` without also supplying `scope_type="project"`, the list handler did not activate
scope filtering. That allowed unrelated project-scoped MemoryItems to appear in a project-intended
list call.

It updates only:

- `engram-mcp/src/tools.rs`
- `engram-tests/tests/memory_tests.rs`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- this report

It does not change public MCP parameters, response shape, schema/storage/index/document-index
behavior, memory ranking, `orient` payloads, lifecycle state, M6/migration/quarantine state,
harness files/settings/hooks/adapters, installed runtime, native Claude state, deletion, rollback,
or user-owned files.

## Research Question

Should `memory(action="list", project_name="engram")` behave as a project-scoped list request even
when the caller omits `scope_type="project"`?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Project-name-only list calls should infer `scope_type="project"` before parsing the scope, matching existing `capture_current_plan` behavior and agent expectations. | Supported. |
| Null | Callers must always pass `scope_type`, and project-name-only list calls should remain unfiltered. | Rejected because the MCP schema describes `project_name` as a list filter and agents naturally pass it for project-scoped retrieval. |
| Simpler alternative | Document that `scope_type` is required for list filtering. | Rejected because the adjacent capture path already infers project scope, and the current behavior can surface wrong-project memory. |
| Failure | The fix changes ranking, broad search behavior, `orient`, storage, or public MCP shape. | Avoided. The change is limited to the MCP list branch scope-filter construction. |

## Evidence

- During post-T220 continuation, `memory(action="list", project_name="engram", status_filter="active",
  query="current plan next step T219 T217 runtime refresh external session approval gate")`
  returned the T220/T219 current-plan item first but also returned unrelated active `dd-source`
  Claude hook MemoryItems.
- Source read showed the list branch only parsed a scope filter when `scope_type` was present,
  while `capture_current_plan` already infers `scope_type="project"` from `project_name`.
- Existing coverage proved explicit `scope_type="project"` filters before limit, but there was no
  adjacent test for `project_name` alone.

## Change

`engram-mcp/src/tools.rs` now clones the request in the `memory(action="list")` branch and infers
`scope_type="project"` when `project_name` is present and `scope_type` is absent. Scope parsing,
filter matching, response shape, and storage behavior remain unchanged.

## Validation

Commands run:

```text
cargo test -p engram-tests test_mcp_memory_list_project_name_implies_project_scope_before_limit --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_filters_by_scope_before_limit --test memory_tests -- --exact
cargo fmt --all --check
cargo test -p engram-tests --test memory_tests
cargo check -p engram-cli
```

Results:

- New regression coverage passed: a project-name-only list request excludes a newer wrong-project
  item before applying the limit.
- Existing explicit-scope list coverage still passed.
- Full `memory_tests` passed: 31 tests.
- Formatting and CLI check passed.

## Decision

T221 closes a concrete project-scope list pitfall without touching `orient`, ranking, or installed
runtime. Installed runtime parity remains separate: this source change is not live until a future
runtime refresh gate covers it.
