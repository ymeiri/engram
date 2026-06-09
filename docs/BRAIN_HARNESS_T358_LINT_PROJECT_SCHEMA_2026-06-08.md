# Brain Harness T358 Lint Project Schema Exposure

Date: 2026-06-08
Status: implemented, source-validated, installed, and live-daemon smoked

## Scope

T358 hardens the public MCP contract for project-scoped Memory OS lint. Project-scoped lint already
worked through the runtime request path, but the live tool metadata observed by Codex did not expose
`project` in the `lint` tool schema. That makes the supported beta path harder for agents to use
reliably, even when the backend behavior is present.

This slice changes only the schema contract and tests:

- adds an explicit `schemars` description for `LintRequest.project`;
- adds a unit regression that the generated `LintRequest` JSON schema contains `project`;
- adds an HTTP-level regression that a source daemon `tools/list` response exposes
  `lint.inputSchema.properties.project`.

## Research Question

Can Engram make project-scoped lint discoverable through the public MCP metadata without changing
lint behavior, storage, daemon semantics, or the beta release scope?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Explicit schema metadata plus an HTTP `tools/list` regression will prevent project-scoped lint from silently disappearing from the public MCP contract. | Supported. |
| Null | Runtime lint behavior tests are enough; schema discoverability does not need its own guard. | Rejected because agent callers depend on tool metadata before they can form valid calls. |
| Failure | The change accidentally alters lint filtering or daemon behavior. | Avoided; the behavior test still passes and the patch is limited to schema/test coverage. |

## Implementation

- Annotated `LintRequest.project` with `#[schemars(description = "Optional project scope to lint.")]`.
- Added `lint_request_schema_exposes_project_filter` in `engram-mcp`.
- Added `test_mcp_tools_list_lint_schema_exposes_project_filter` in the multi-session HTTP daemon
  tests.

## Validation

Focused source validation passed:

```text
cargo fmt --all --check
git diff --check
cargo test -p engram-mcp lint_request_schema_exposes_project_filter
cargo test -p engram-mcp
cargo test -p engram-tests --test lint_tests test_mcp_lint_project_filter_excludes_unrelated_project_memory
cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list_lint_schema_exposes_project_filter
```

Installed runtime refresh:

```text
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
```

This replaced installed hash:

```text
fa91efbd228683dae608881f5828bdc1ffe55b67376e414653f8ac8eb92ba8c9
```

with:

```text
62c9955925f74fba706ad466416033cc0bdbc211cf0443a373d4e5925760589a
```

The live global daemon was restarted onto that binary:

```text
Daemon status: running
Port: 8765
PID: 36562
Spawned by: /Users/yuval.meiri/.local/bin/engram
Spawn version: 0.2.0-beta.1
```

Live installed-daemon MCP smoke passed. A direct `tools/list` JSON-RPC request against
`http://127.0.0.1:8765/mcp` returned a `lint` tool whose input schema includes:

```json
{"project":{"description":"Optional project scope to lint."}}
```

## Gate Impact

T358 improves the supported local/Codex beta tool contract for project-scoped lint. It does not mark
PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove effective-hook
visibility, prove live host labels, mutate lifecycle state, run broad `lint apply_safe`, or change
the supported beta scope.
