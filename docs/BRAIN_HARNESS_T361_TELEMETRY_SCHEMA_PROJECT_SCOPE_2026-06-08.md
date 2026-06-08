# Brain Harness T361 Telemetry Schema Project Scope

Date: 2026-06-08
Status: implemented, source-validated, installed, restarted, and live-daemon smoked

## Scope

T361 hardens the public MCP contract for project-scoped telemetry reports. Runtime telemetry
already accepted `project` for trace recording and scoped report/list actions, including
`real_session_eval`. The remaining contract gap was schema metadata: `TelemetryRequest.project`
had only a generic doc comment, so an agent reading `tools/list` could miss the supported project
filtering path.

This slice changes only MCP schema metadata and tests:

- adds explicit schema metadata to `TelemetryRequest.project`;
- adds a generated-schema regression for `TelemetryRequest`;
- adds an HTTP daemon `tools/list` regression for the public telemetry schema surface.

## Research Question

Can Engram make project-scoped telemetry reporting discoverable through the public MCP schema
without changing telemetry runtime behavior, storage, ranking, or beta scope?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Explicit `schemars` metadata and a `tools/list` regression will keep project-scoped telemetry reports discoverable to agent callers. | Supported by focused tests. |
| Null | Existing runtime support and generic comments are enough. | Rejected because agents form calls from MCP schema metadata. |
| Simpler alternative | Leave metadata vague and rely on docs. | Rejected because `tools/list` is the live contract surface. |
| Failure | Metadata updates accidentally alter telemetry behavior. | Avoided; only schema metadata changed. |

## Implementation

`TelemetryRequest.project` now exposes this schema description:

```text
Optional project scope for record_trace, list_traces, list_feedback, stats_by_intent, and real_session_eval.
```

## Validation

Focused source validation passed:

```text
cargo test -p engram-mcp telemetry_request_schema_exposes_project_filter
cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list_telemetry_schema_exposes_project_filter
```

Broader validation passed:

```text
cargo fmt --all --check
git diff --check
cargo test -p engram-mcp
cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list
./scripts/local-ci.sh
./scripts/package-install-smoke.sh
```

The local CI-equivalent gate covered `git diff --check`, `cargo fmt --all --check`,
`cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`, serialized
`cargo test --all-targets --jobs 1`, and `cargo doc --no-deps`. The package install smoke verified
the release tarball checksum, temp install, packaged `engram 0.2.0-beta.1`, and packaged HTTP
`/health`.

Installed runtime refresh:

```text
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
```

This replaced installed hash:

```text
ff16b90be46e54d089ce66e5b360630449bffc9f874da031beb10884f994756b
```

with:

```text
6c278872d2f71a5ce96fba3e1777b3cc2f4690e6d6c9caf74df093fb4fd7e49a
```

The live global daemon was restarted onto that binary:

```text
Daemon status: running
Port: 8765
PID: 2865
Spawned by: /Users/yuval.meiri/.local/bin/engram
Spawn version: 0.2.0-beta.1
```

Live daemon `/health` returned:

```json
{"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

Live installed-daemon MCP `tools/list` smoke passed after initialize and initialized notification.
The telemetry tool input schema returned:

```text
Optional project scope for record_trace, list_traces, list_feedback, stats_by_intent, and real_session_eval.
```

## Gate Impact

T361 improves public MCP discoverability for project-scoped telemetry reports in the supported
local/Codex beta path. It does not change telemetry filtering semantics, scoring formulas,
storage, ranking, `orient`, obligations, lifecycle state, hosted CI, native Claude, effective-hook
visibility, live host labels, or the supported beta scope.
