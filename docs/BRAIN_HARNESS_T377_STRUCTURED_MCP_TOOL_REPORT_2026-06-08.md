# Brain Harness T377 Structured MCP Tool Report

Date: 2026-06-08
Status: implemented, installed, and validated in the live local/Codex runtime.

## Research Question

Can harness status and doctor reports distinguish "MCP tool availability was not checked" from
"MCP tool availability was checked and required tools are present or missing" without requiring
agents or release checks to infer that state from an empty `missing_mcp_tools` list?

Preferred hypothesis: `harness status` and `harness doctor` include a structured `mcp_tools`
object with checked state, required tools, observed tools, missing tools, and a summary message,
while preserving the compatibility `missing_mcp_tools` field.

Null hypothesis: an empty `missing_mcp_tools` list remains ambiguous between unchecked state and
checked-complete state.

Failure hypothesis: adding structured MCP tool state breaks CLI text output, MCP JSON
serialization, harness readiness, source tests, or live installed runtime behavior.

## Change

`HarnessStatusReport` now includes:

```json
{
  "mcp_tools": {
    "checked": false,
    "required_tools": [
      "orient",
      "memory",
      "harness",
      "lint",
      "graph",
      "handoff",
      "obligations",
      "telemetry",
      "vault"
    ],
    "observed_tools": [],
    "missing_tools": [],
    "message": "MCP tool availability was not checked; provide observed_mcp_tools to verify the required tool set."
  },
  "missing_mcp_tools": []
}
```

When observed tools are supplied, the service sets `mcp_tools.checked=true` and populates
`mcp_tools.missing_tools` plus the compatibility `missing_mcp_tools` list with any missing required
tool names. The CLI text output now prints one of:

- `MCP tools: not checked (9 required)`
- `MCP tools: checked, all required tools observed`
- `MCP tools: checked, missing <tool names>`

## Validation

Focused source validation passed:

- `cargo fmt --all --check`
- `cargo test -p engram-index harness::tests::status_distinguishes_unchecked_from_missing_mcp_tools`
- `cargo test -p engram-tests --test harness_tests test_mcp_harness_status_reports_missing_observed_mcp_tools`
- `cargo test -p engram-tests --test harness_tests`
- `cargo check -p engram-cli`
- `cargo clippy -p engram-index -- -D warnings`
- `cargo clippy -p engram-cli -- -D warnings`

Runtime adoption passed:

- `cargo build --release`
- installed `/Users/yuval.meiri/.local/bin/engram` from `./target/release/engram`
- installed and target hashes both:
  `0810b8600954c4578d025f5d3eff897bed2f0d53a6470a858078202b8f637033`
- restarted daemon on PID `99012`
- `/health` returned `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`
- `cd dist && shasum -a 256 -c engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz.sha256`
  returned OK

Live unchecked state validation passed:

- `engram harness doctor --harness codex --json` returned `ready=true`,
  `mcp_tools.checked=false`, the nine required tool names, `mcp_tools.missing_tools=[]`, and
  `missing_mcp_tools=[]`.
- `engram harness doctor --harness codex` printed `MCP tools: not checked (9 required)`.
- Live MCP `harness(action=status, harness=codex, observed_mcp_tools=[... without telemetry ...])`
  returned `ready=false`, `mcp_tools.checked=true`, `mcp_tools.missing_tools=["telemetry"]`,
  `missing_mcp_tools=["telemetry"]`, and the warning
  `Required MCP tool 'telemetry' was not reported by the client.`

The installed CLI currently does not expose an `--observed-mcp-tool` flag, so the checked path was
not validated through CLI text flags.

## Gate Impact

T377 improves machine-readable release and harness evidence. Agents and release checks can now
tell whether the required MCP tool set was actually checked before interpreting missing-tool
results.

This does not install or mutate hooks/adapters, enforce lifecycle behavior, archive sessions or
memory, run `lint apply_safe`, mutate M6/migration state, change ranking or `orient`, launch
native Claude, run `/hooks`, signal processes, mark PR #3 ready, merge, tag, publish, or change
the beta scope.
