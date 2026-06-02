# T133 Post-T130 Live Runtime Gap Audit

Date: 2026-06-02
Scope: read-only post-T130 source-vs-live audit.

## Research Question

After T130 repaired the generated Claude Code `SessionEnd` hook template in source, does the live
Engram MCP runtime and installed user hook now reflect the missing-`write_policy` default of
`nudge`?

## Hypotheses

- Preferred: the committed source, live MCP `harness(render_adapter)`, and installed generated hook
  all default missing hook input `write_policy` to `nudge`.
- Null: only the source tree changed; the live MCP runtime or installed hook still defaults missing
  `write_policy` to `durable`.
- Simpler alternative: no product code change is needed; the next step is a gated binary refresh and
  read-only validation.
- Failure: running install, daemon restart, or hook/settings repair without explicit approval would
  cross the T130 and harness-write gates.

## Measurement

Read-only checks only:

- Compare source template content in `engram-index/src/harness.rs`.
- Inspect the installed generated Claude hook at
  `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh`.
- Render the live MCP `claude-session-end-hook` adapter through `harness(action="render_adapter")`.
- Re-run harness doctors for Claude Code, Codex, Gemini CLI, and Cursor without writes.
- Re-run telemetry, lint, and obligations doctors for current risk context.

No binary install, daemon restart, `harness install`, user hook/settings edit, schema/storage/index
change, ranking change, `orient` change, migration action, lifecycle write, or document-index
behavior change was run.

## Evidence

- Source is T130-correct: `engram-index/src/harness.rs:2085` contains
  `.write_policy // "nudge"`, and the focused source tests assert the absence of
  `.write_policy // "durable"`.
- Installed user hook is not T130-correct: `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh:25`
  still contains `.write_policy // "durable"`.
- Live MCP runtime is not T130-correct: `harness(action="render_adapter", harness="claude_code",
  adapter="claude-session-end-hook")` still rendered `WRITE_POLICY=... .write_policy // "durable"`.
- The current shell resolves `engram` to `/Users/yuval.meiri/.local/bin/engram`.
- `/Users/yuval.meiri/.local/bin/engram` and `/Users/yuval.meiri/.cargo/bin/engram` share binary hash
  `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`.
- The global daemon files point at port `8765` and pid `85557`.
- Claude Code doctor remains `ready=false`: generated adapter files exist, but `SessionStart` and
  `SessionEnd` settings registrations are missing, and the optional settings snippet is user-owned.
- Codex, Gemini CLI, and Cursor doctors remain `ready=false` because required generated adapters are
  drifted.
- `real_session_eval(project="engram", limit=50)` passes numerically with `trace_count=50`,
  `feedback_trace_count=35`, `feedback_coverage=0.70`, `bad_memory_used_count=0`, and
  `external_session_trace_count=0`; it still requires user approval and is weak operational
  evidence.
- `lint(action="run", limit=12)` still reports stale/wrong-scope active memory pressure, led by
  stale current-plan item `019e5e0a-86b4-73e3-aa9b-ca350e83e915` with `safe_action=none`.
- `obligations(action="doctor")` returned no open obligations before this documentation edit.

## Completion Matrix Delta

| Area | Status | Evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| T130 source behavior | Validated | Commit `9dff108`; source/tests show missing `write_policy` defaults to `nudge` and explicit durable still writes | None for source-level repair |
| Live MCP runtime parity | Missing | Live `harness(render_adapter)` still renders `.write_policy // "durable"` | Requires explicit approval for binary refresh and daemon restart before live validation |
| Installed Claude hook parity | Missing | Installed `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` still defaults to `durable` | Requires explicit approval for any user hook/settings repair or harness install |
| Cross-harness readiness | Risky | Claude Code, Codex, Gemini CLI, and Cursor doctors all report `ready=false` | Requires separately approved harness repair scope |
| Migration M6 | Gated | T123/T124 inspected the 9 review candidates; 2 quarantine candidates remain unread | T125 quarantine inspection and M6 apply/delete/status/prioritize remain exact approval gates |
| Memory lifecycle | Risky | Lint still reports stale current-plan and superseded-active pressure | No lifecycle write is safe without explicit review/approval |

## Decision

T130 is complete at the committed source and test level, but the running product is not live-validated
for that fix. The next safe implementation slice is not a code change. It is a gated live-runtime
refresh:

1. install the current `engram-cli` binary,
2. restart the Engram daemon,
3. re-run `harness(render_adapter)` and focused T130 live smoke checks,
4. stop before editing installed hooks/settings or running `harness install`.

Hook/settings repair remains a separate gate because it would write user-owned or installed harness
files.
