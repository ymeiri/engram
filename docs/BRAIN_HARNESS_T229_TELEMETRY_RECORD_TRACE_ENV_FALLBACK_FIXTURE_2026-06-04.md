# T229: Telemetry Record-Trace Env Fallback Fixture

Date: 2026-06-04
Status: source-test hardening pending refreshed runtime gate
Scope: focused MCP tool-level coverage for `telemetry(action="record_trace")` when the request
omits `external_session_id` and the server process provides `ENGRAM_EXTERNAL_SESSION_ID`.

## Research Question

Does source fixture coverage prove that the MCP `telemetry(action="record_trace")` path persists
the resolved runtime `ENGRAM_EXTERNAL_SESSION_ID` when the request omits `external_session_id`,
rather than only proving the private resolver helper in isolation?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The existing T217 resolver is already wired into `telemetry(action="record_trace")`; a focused MCP-level fixture should persist the trimmed runtime env label in the returned trace. |
| Null | The resolver tests pass, but the `record_trace` tool path still omits or drops the runtime env label. |
| Simpler alternative | Rely on the private resolver tests from T217. Rejected because the pending runtime refresh intends to validate the live MCP telemetry tool path, so source coverage should include that exact path. |
| Failure | The fixture changes production behavior, public MCP parameters or payload shape, runtime configuration, schema/storage/index/document-index behavior, ranking, `orient`, lifecycle state, M6/migration/quarantine state, harness files/settings/hooks/adapters, native Claude state, deletion, rollback, or user-owned files. |

## Measurement

Add one fixture:

```text
cargo test -p engram-mcp tools::tests::mcp_telemetry_record_trace_uses_runtime_env_when_request_is_absent -- --exact
```

Fixture shape:

- initialize in-memory telemetry storage;
- initialize a `ToolState` with telemetry enabled;
- acquire the existing env lock and set `ENGRAM_EXTERNAL_SESSION_ID` to a whitespace-padded label;
- call `telemetry_new` with `TelemetryRequest { action: "record_trace", external_session_id: None, ... }`;
- parse the returned JSON and assert `trace.external_session_id` equals the trimmed runtime env
  label.

## Evidence

T217 added the source-level MCP fallback and private resolver tests. It also documented that live
Codex MCP traces would not use the fallback until a separate runtime refresh installed the new
binary and restarted the daemon with a host-provided env label.

Before approving that runtime refresh, the remaining source-test gap was tool-level proof for the
exact live validation path: `telemetry(action="record_trace")` with request label omitted. T229
closes that source-test gap without changing production behavior.

## Change

Added `mcp_telemetry_record_trace_uses_runtime_env_when_request_is_absent` in
`engram-mcp/src/tools.rs`. No production code changed.

## Validation

Passed:

```text
cargo test -p engram-mcp tools::tests::mcp_telemetry_record_trace_uses_runtime_env_when_request_is_absent -- --exact
cargo test -p engram-mcp tools::tests
cargo test -p engram-tests --test telemetry_tests
cargo fmt --all --check
cargo check -p engram-cli
git diff --check
```

The local `tools::tests` group passed with 6 tests. The full telemetry integration target passed
with 24 tests.

## Boundaries

T229 is test-only. It does not change production source behavior, public MCP request parameters or
response shape, ranking, `orient` payload, schema/storage/index/document-index behavior, lifecycle
state, M6/migration/quarantine state, harness files/settings/hooks/adapters, installed runtime,
native Claude state, deletion, rollback, or user-owned files.

Because T229 changes binary-relevant `engram-mcp/src/tools.rs` after T228, T228 is now stale for
exact execution under the packet's deny-by-default invariant. A refreshed runtime approval packet
must supersede T228 before any install/restart/live validation.

## AI Consultation

No AI Council or Claude Bridge consultation was used. This was a narrow fixture-hardening slice for
an already-implemented source path, with no architecture, ranking, migration, data-model, or
irreversible decision.
