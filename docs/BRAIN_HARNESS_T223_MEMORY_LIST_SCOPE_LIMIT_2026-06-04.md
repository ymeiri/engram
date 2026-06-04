# T223: Memory List Scope-Filter Limit Handling

Date: 2026-06-04
Status: source-level fix committed pending runtime refresh
Scope: narrow MCP `memory(action="list")` behavior for scoped list requests with `limit`.

## Research Question

Does MCP `memory(action="list")` apply `limit` after in-memory scope filtering, so a scoped list
request cannot return more rows than requested when repository-level limiting is intentionally
disabled for correctness?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | When tags or scope filters require fetching without a repository-level limit, the list branch should apply the requested limit once after all in-memory filters. |
| Null | Scope-filtered list requests already obey `limit`; the suspected issue is only a test-fixture artifact. |
| Simpler alternative | Leave current behavior unchanged because T221 already fixed project-name-only scope inference. Rejected because T221 did not cover multiple matching scoped rows after filtering. |
| Failure | Applying the limit after filtering changes public MCP parameters, response shape, ranking/`orient`, schema/storage/index behavior, lifecycle state, migration/quarantine state, or harness files. |

## Measurement

Focused regression:

```text
cargo test -p engram-tests test_mcp_memory_list_applies_limit_after_scope_filter --test memory_tests -- --exact
```

Fixture shape:

- add two active project-scoped Engram MemoryItems;
- add a newer active project-scoped `dd-source` MemoryItem;
- call `memory(action="list", status_filter="active", scope_type="project",
  project_name="engram", limit=1)`;
- assert `count == 1` and the returned item scope remains Engram.

## Evidence

Source inspection showed the `list` branch correctly disabled repository-level limiting when tags
or a scope filter required in-memory filtering:

```text
let fetch_limit = if tags.is_empty() && scope_filter.is_none() {
    request.limit
} else {
    None
};
```

It then applied `items.truncate(limit)` only inside the tag-filter branch. A scope-only filtered
request could therefore return every matching scoped item after filtering.

The new regression failed before the implementation change:

```text
test_mcp_memory_list_applies_limit_after_scope_filter ... FAILED
left: Number(2)
right: 1
```

## Change

The `memory(action="list")` branch now applies the requested limit after any in-memory filtering
path by truncating when `fetch_limit.is_none()`. Unfiltered list requests still pass the limit to
the repository as before.

## Validation

Passed after the source change:

```text
cargo test -p engram-tests test_mcp_memory_list_applies_limit_after_scope_filter --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_filters_by_scope_before_limit --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_project_name_implies_project_scope_before_limit --test memory_tests -- --exact
cargo test -p engram-tests test_mcp_memory_list_filters_by_tags_before_limit --test memory_tests -- --exact
cargo fmt --all --check
cargo test -p engram-tests --test memory_tests
cargo check -p engram-cli
git diff --check
```

The full `memory_tests` target passed with 32 tests.

## Boundaries

T223 does not change public MCP request parameters or response shape, ranking, `orient` payload,
schema/storage/index/document-index behavior, lifecycle state, M6/migration/quarantine state,
harness files/settings/hooks/adapters, installed runtime, native Claude state, deletion, rollback,
or user-owned files.

Because T223 changes binary-relevant `engram-mcp` and `engram-tests` files after T222, T222 is now
stale for execution. A refreshed runtime approval packet must supersede T222 before any runtime
install/restart/live validation.

## AI Consultation

No AI Council or Claude Bridge consultation was used. This was a narrow local invariant fix with a
failing fixture and no architecture, ranking, migration, data-model, or irreversible decision.
