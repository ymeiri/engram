# Brain Harness T360 Obligations Schema Scope

Date: 2026-06-08
Status: implemented, source-validated, installed, restarted, and live-daemon smoked

## Scope

T360 hardens the public MCP contract for scoped obligation health checks. Runtime MCP behavior
already accepted `project` and `cwd` for `obligations(action="list")`,
`obligations(action="open")`, and `obligations(action="doctor")`, and T359 added matching CLI
flags. The remaining contract gap was schema metadata: `ObligationRequest.cwd` still described
itself as detect-only, so an agent reading `tools/list` could miss the supported scoped list/doctor
path.

This slice changes only MCP schema metadata and tests:

- updates `ObligationRequest.cwd` schema description to mention detect/list/open/doctor scoping;
- updates `ObligationRequest.project` schema description to mention detect/add/list/open/doctor;
- adds a generated-schema regression for `ObligationRequest`;
- adds an HTTP daemon `tools/list` regression for the live public schema surface.

## Research Question

Can Engram make scoped obligation list/doctor behavior discoverable through the public MCP schema
without changing obligation runtime behavior, storage, or beta scope?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Explicit `schemars` metadata and `tools/list` regression will keep scoped obligation health checks discoverable to agent callers. | Supported. |
| Null | Runtime support and CLI flags are enough; MCP metadata can stay generic or stale. | Rejected because agents form tool calls from MCP schema metadata. |
| Simpler alternative | Change doc comments only. | Rejected because explicit schema descriptions are easier to assert and harder to regress. |
| Failure | Metadata updates accidentally alter obligation behavior. | Avoided; only schema metadata changed, and obligation behavior tests still pass. |

## Implementation

- Annotated `ObligationRequest.cwd` with:
  `Current working directory for detect/list/open/doctor scoping.`
- Annotated `ObligationRequest.project` with:
  `Optional project scope for detect, add, list, open, and doctor.`
- Added `obligation_request_schema_exposes_scope_filters` in `engram-mcp`.
- Added `test_mcp_tools_list_obligations_schema_exposes_scope_filters` in the HTTP daemon
  multi-session tests.

## Validation

Focused source validation passed:

```text
cargo test -p engram-mcp obligation_request_schema_exposes_scope_filters
cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list_obligations_schema_exposes_scope_filters
cargo fmt --all --check
git diff --check
cargo test -p engram-mcp
cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list
```

Installed runtime refresh:

```text
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
```

This replaced installed hash:

```text
ae45c01ab2a4c5046508e916a7c381655a71611f223fd8fc7989392cd3879f79
```

with:

```text
ff16b90be46e54d089ce66e5b360630449bffc9f874da031beb10884f994756b
```

The installed binary reports:

```text
engram 0.2.0-beta.1
```

The live global daemon was restarted onto that binary:

```text
Daemon status: running
Port: 8765
PID: 48118
Spawned by: /Users/yuval.meiri/.local/bin/engram
Spawn version: 0.2.0-beta.1
```

Live daemon `/health` returned:

```json
{"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

Live installed-daemon MCP smoke passed after initialize and initialized notification. A direct
`tools/list` request returned an `obligations` tool whose input schema includes:

```json
{
  "project": "Optional project scope for detect, add, list, open, and doctor.",
  "cwd": "Current working directory for detect/list/open/doctor scoping."
}
```

Final exact-head validation passed:

```text
./scripts/local-ci.sh
./scripts/package-install-smoke.sh
```

The local CI-equivalent gate covered `git diff --check`, `cargo fmt --all --check`,
`cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`, serialized
`cargo test --all-targets --jobs 1`, and `cargo doc --no-deps`. The package install smoke created
and verified:

```text
dist/engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz
dist/engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz.sha256
```

and confirmed packaged HTTP `/health` returned:

```json
{"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

## Gate Impact

T360 improves public MCP discoverability for scoped obligation health checks in the supported
local/Codex beta path. It does not mark PR #3 ready, merge, tag, publish, close hosted CI, run
native Claude, prove effective-hook visibility, prove live host labels, mutate lifecycle state, run
broad `lint apply_safe`, or change the supported beta scope.
