# Brain Harness T217 MCP External Session Env Fallback

Date: 2026-06-04
Status: completed source-level implementation

## Scope

This slice adds an MCP-side fallback for the existing Brain Harness `external_session_id` telemetry
field. If an MCP request omits `external_session_id` or passes only whitespace, the tool layer now
uses `ENGRAM_EXTERNAL_SESSION_ID` when that environment variable is set to a non-empty value.

It updates only:

- `engram-mcp/src/tools.rs`
- `engram-tests/tests/telemetry_tests.rs`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- this report

It does not add public MCP parameters, change MCP response shape, synthesize labels, change
ranking or `orient` payloads, change schema/storage/index/document-index behavior, mutate
lifecycle or migration state, edit hooks/settings/adapters, refresh the installed runtime, run
native Claude, delete data, or touch user-owned files.

## Research Question

Can Engram improve external-session joinability for ordinary MCP callers without changing public
MCP shape or requiring every tool call to pass `external_session_id` explicitly?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | A private MCP helper can mirror the CLI `ENGRAM_EXTERNAL_SESSION_ID` fallback for telemetry-only fields while keeping explicit request values authoritative. | Supported. |
| Null | MCP should remain explicit-request-only because process environment is too implicit. | Not selected for this source slice; the fallback is telemetry-only, request values win, and tests cover normalization. |
| Simpler alternative | Leave T200 CLI support as the only env fallback and keep documenting MCP caller adoption as incomplete. | Rejected because source evidence showed MCP callers are the common unlabeled path. |
| Failure | The fallback changes public MCP shape, labels non-telemetry data, bypasses validation, or contaminates unrelated runtime behavior. | Avoided in source. The change is private, telemetry-scoped, and downstream validation still rejects overlong labels. |

## Consultation

AI Council recall found the prior T199/T200 caller-label discussion. A fresh Council broadcast to
Claude, GPT, and Gemini supported the narrow fallback with the same cautions: process-level
environment labels can be surprising in shared MCP-server deployments, the request field must win,
whitespace must normalize before fallback, and the resolved label must continue through existing
validation.

Claude Bridge was attempted twice in read-only mode for cross-harness critique. Both attempts timed
out and produced no usable result. This is a consultation confound, not evidence against or for the
change.

## Change

`engram-mcp/src/tools.rs` now has a private resolver:

- trim and normalize a request `external_session_id`;
- if non-empty, use it;
- otherwise trim and normalize `ENGRAM_EXTERNAL_SESSION_ID`;
- otherwise leave the label unset.

The resolver is used only at existing telemetry call sites:

- unified `search` trace recording;
- `orient` trace recording through `OrientInput`;
- `telemetry(action="record_trace")`;
- `telemetry(action="submit_feedback")`;
- `memory(action="changes_since")` trace recording.

## Validation

Commands run:

```text
cargo test -p engram-mcp external_session_id
cargo test -p engram-tests --test telemetry_tests mcp_telemetry_tool_rejects_too_long_external_session_id -- --exact
cargo test -p engram-tests --test telemetry_tests
cargo fmt --all --check
cargo check -p engram-cli
git diff --check
```

Results:

- MCP resolver unit tests passed: request value wins over env, env fallback applies, whitespace
  request falls through to env, whitespace env becomes unset, and the runtime env wrapper reads the
  configured value.
- Telemetry boundary validation passed: overlong `external_session_id` values still fail with the
  existing 256-character validation error.
- Full telemetry integration passed: 24 tests.
- Formatting, CLI check, and diff whitespace checks passed.

## Remaining Gap

This is source-level validation only. The installed daemon/runtime has not been refreshed in this
slice, so live Codex MCP traces will not use the fallback until a separate runtime refresh is run.
Even after refresh, hosts still need to provide a real `ENGRAM_EXTERNAL_SESSION_ID` environment
value for the MCP server process; Engram still does not synthesize host labels.

## Decision

T217 narrows the external-session joinability gap for MCP callers without expanding the public
protocol. It does not complete cross-harness live labeling.
