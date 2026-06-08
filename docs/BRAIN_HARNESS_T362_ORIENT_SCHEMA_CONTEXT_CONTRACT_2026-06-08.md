# Brain Harness T362 Orient Schema Context Contract

Date: 2026-06-08
Status: implemented, package-smoked, and installed-runtime validated

## Scope

T362 hardens the public MCP contract for the core Brain Loop entrypoint. Runtime `orient` already
uses `project`, `cwd`, and `response_shape` to resolve project/repository context, select scoped
memory, and choose the full or lean response shape. The schema metadata was still generic, so an
agent reading `tools/list` could miss how those fields affect orientation.

This slice changes only MCP schema metadata and tests:

- makes `OrientRequest.cwd` explicitly describe repository/project resolution and scoped memory
  selection;
- makes `OrientRequest.project` explicitly describe project resolution and project-scoped memory
  selection;
- makes `OrientRequest.response_shape` explicitly describe the lean Brain Loop response contract;
- adds a generated-schema regression for `OrientRequest`;
- adds an HTTP daemon `tools/list` regression for the public `orient` schema surface.

## Research Question

Can Engram make the core `orient` context contract discoverable through the public MCP schema
without changing orientation ranking, memory selection, telemetry, or beta scope?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Explicit `schemars` metadata plus a `tools/list` regression will make the `orient` context contract durable for agent callers. | Supported by focused tests and installed-daemon schema smoke. |
| Null | Existing generic field descriptions are enough. | Rejected because `orient` is the core Brain Loop entrypoint and agents depend on schema text to choose context fields. |
| Simpler alternative | Leave the source comments as-is and document the behavior elsewhere. | Rejected because `tools/list` is the live public contract. |
| Failure | Metadata updates accidentally change orientation behavior. | Avoided; only schema metadata changed. |

## Implementation

`OrientRequest` now exposes these schema descriptions:

```text
cwd = "Current working directory for repository/project resolution and scoped memory selection."
project = "Explicit project name for project resolution and project-scoped memory selection."
response_shape = "Response shape: full (default) or lean for compact trace/cursor/Brain Loop guidance."
```

## Validation

Focused source validation passed:

```text
cargo test -p engram-mcp orient_request_schema_exposes_context_contract
cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list_orient_schema_exposes_context_contract
cargo fmt --all --check
git diff --check
cargo test -p engram-mcp
cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list
./scripts/local-ci.sh
./scripts/package-install-smoke.sh
```

`./scripts/package-install-smoke.sh` rebuilt the release package, verified the tarball checksum,
installed the packaged binary into a temporary prefix, reported `engram 0.2.0-beta.1`, and verified
packaged HTTP `/health`:

```text
{"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

The installed local runtime was refreshed:

```text
old installed hash = 6c278872d2f71a5ce96fba3e1777b3cc2f4690e6d6c9caf74df093fb4fd7e49a
new installed hash = 77a08e895614bea3b02816e67bafd64087ea0634f4b0ca58b8199a9ef7855633
daemon PID = 47577
daemon port = 8765
daemon health = {"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

An installed-daemon MCP `tools/list` smoke confirmed the public `orient` schema descriptions:

```text
cwd = "Current working directory for repository/project resolution and scoped memory selection."
project = "Explicit project name for project resolution and project-scoped memory selection."
response_shape = "Response shape: full (default) or lean for compact trace/cursor/Brain Loop guidance."
```

## Gate Impact

T362 improves public MCP discoverability for the supported local/Codex Brain Loop orientation path.
It does not change orientation ranking, memory selection semantics, telemetry, obligations,
lifecycle state, hosted CI, native Claude, effective-hook visibility, live host labels, or the
supported beta scope.
