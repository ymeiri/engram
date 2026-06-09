# Brain Harness T378 CLI Observed MCP Tool Flags

Date: 2026-06-08
Status: implemented, installed, and validated in the live local/Codex runtime.

## Research Question

Can the CLI exercise the structured MCP tool availability checks added in T377, so release and
operator scripts can verify checked-complete or checked-missing tool state without calling the MCP
API directly?

Preferred hypothesis: `engram harness status` and `engram harness doctor` accept repeatable
`--observed-mcp-tool <TOOL>` flags and pass those names into the existing harness status/doctor
service path.

Null hypothesis: checked MCP tool state remains available only through the MCP request API, leaving
the installed CLI limited to unchecked reports.

Failure hypothesis: adding the flags breaks existing CLI harness status/doctor defaults, Clap
parsing, JSON output, or text output.

## Change

`engram harness status` and `engram harness doctor` now accept:

```bash
engram harness status --harness codex \
  --observed-mcp-tool orient \
  --observed-mcp-tool telemetry \
  --json
```

The flags are repeatable. When omitted, the existing unchecked behavior is preserved:
`mcp_tools.checked=false` and no required tools are treated as missing.

## Validation

Focused source validation passed:

- `cargo fmt --all --check`
- `cargo test -p engram-cli harness_status_parses_observed_mcp_tool_flags`
- `cargo test -p engram-cli`
- `cargo check -p engram-cli`
- `cargo clippy -p engram-cli -- -D warnings`

The broader T377 service and MCP tests already cover checked missing-tool semantics. T378 adds the
CLI route into that existing service path.

Runtime adoption passed:

- `cargo build --release`
- installed `/Users/yuval.meiri/.local/bin/engram` from `./target/release/engram`
- installed and target hashes both:
  `d7e17ae33bdfd48c84fd24070b1d10b17a284c0e31993e7a9b190c7450180b34`
- restarted daemon on PID `39185`
- `/health` returned `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`

Installed CLI checked-state validation passed:

- `engram harness status --harness codex` with all nine required `--observed-mcp-tool` flags
  returned `ready=true`, `mcp_tools.checked=true`, `mcp_tools.missing_tools=[]`, and
  `missing_mcp_tools=[]`.
- `engram harness doctor --harness codex` with observed tools missing `telemetry` returned
  `ready=false`, `mcp_tools.checked=true`, `mcp_tools.missing_tools=["telemetry"]`,
  `missing_mcp_tools=["telemetry"]`, and the missing telemetry warning.
- Text `engram harness status --harness codex` with all nine observed tools printed
  `MCP tools: checked, all required tools observed`.

## Gate Impact

T378 closes the T377 CLI caveat. Release checks can now verify `mcp_tools.checked=true` from the
installed CLI.

This does not install or mutate hooks/adapters, enforce lifecycle behavior, archive sessions or
memory, run `lint apply_safe`, mutate M6/migration state, change ranking or `orient`, launch
native Claude, run `/hooks`, signal processes, mark PR #3 ready, merge, tag, publish, or change
the beta scope.
