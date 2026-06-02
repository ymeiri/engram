# Brain Harness T130 Claude Session-End Hook Default

Status: Implemented and locally validated
Date: 2026-06-02
Scope: Change only the generated Claude Code command-style `SessionEnd` hook template so a
missing hook-input `write_policy` is non-durable (`nudge`) instead of durable.

## Research Question

Can the generated Claude Code `SessionEnd` command hook stop turning an absent `write_policy` into
durable handoff write intent while preserving explicit durable `SessionEnd` handoff writes?

## Hypotheses

| Hypothesis | Claim |
| --- | --- |
| Preferred | The daemon already treats missing or non-`durable` policy as non-durable; changing only the generated command hook fallback to `nudge` prevents accidental handoff writes from missing hook input. |
| Null | `SessionEnd` still writes handoffs when `write_policy` is missing, or the generated/rendered hook still emits the old durable fallback. |
| Simpler alternative | Require every hook caller to pass an explicit `write_policy`; this does not fix observed missing-input behavior from generated hooks. |
| Failure | The slice changes installed user hooks/settings, public MCP parameters, schema/storage/index behavior, ranking, `orient`, migration, or lifecycle state. |

## Measurement

- Focused daemon behavior test: missing `write_policy` on a `SessionEnd` hook event must not write a
  rolling handoff.
- Focused daemon behavior test: explicit `write_policy=durable` on `SessionEnd` must still write a
  rolling handoff.
- Render/install output tests: the generated Claude session-end shell hook must contain
  `.write_policy // "nudge"` and must not contain `.write_policy // "durable"`.
- MCP render integration test: `harness(action="render_adapter", harness="claude_code",
  adapter="claude-session-end-hook")` must return the same `nudge` fallback.

## Change

- `engram-index/src/harness.rs` now renders:
  `WRITE_POLICY=$(printf '%s' "$INPUT" | jq -r '.write_policy // "nudge"')`
  in `claude_session_end_hook()`.
- No daemon write semantics changed: `handle_hook_event` still writes durable hook effects only when
  `event.write_policy` equals `durable` case-insensitively.
- No installed user hook or settings file was edited, and no harness install command was run.

## Validation

- `cargo test -p engram-index hook_event_session_end --lib`
- `cargo test -p engram-index render_claude_session_end_hook_defaults_missing_write_policy_to_nudge --lib`
- `cargo test -p engram-index claude_install_merges_settings_and_is_ready --lib`
- `cargo test -p engram-tests --test harness_tests test_mcp_harness_render_claude_session_end_hook_defaults_to_nudge`
- `cargo fmt --all --check`
- `cargo test -p engram-index harness::tests --lib`
- `cargo test -p engram-tests --test harness_tests`
- `cargo check -p engram-cli`
- `git diff --check`

## Result

T130 closes the specific generated-hook default that made missing Claude `SessionEnd` hook input
durable by default. Explicit durable `SessionEnd` handoff writes remain available. This is not a
harness readiness claim: installed user hooks/settings were not edited, Claude Code harness doctor
status may still be `ready=false`, and any real adapter/settings repair remains separately gated.

This slice does not change public MCP parameters, schema/storage/index behavior, ranking, `orient`,
migration, lifecycle state, document indexing, or user-owned files.
