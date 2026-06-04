# Brain Harness T263 T262 Runtime Refresh

Date: 2026-06-04
Status: completed runtime refresh and live validation for T262. This slice installed the current
`engram-cli` source binary, restarted the Engram daemon from the Codex Desktop environment, and
validated that live MCP `orient` telemetry uses the guarded `CODEX_THREAD_ID` fallback. It did not
change source code, edit hooks/settings/adapters, run native Claude, mutate lifecycle or M6 state,
push or set upstream, delete data, change public MCP parameters, change schema/storage/index/
document-index behavior, or touch user-owned files.

## Research Question

Does the installed daemon/runtime now execute T262's guarded Codex Desktop external-session
fallback for the hot `orient` path and feedback inheritance path?

## Hypotheses

- Preferred: installing the current source binary and restarting the daemon from the Codex shell
  makes live MCP `orient` traces record `codex://threads/{CODEX_THREAD_ID}`, and feedback submitted
  without an explicit label inherits that trace label.
- Null: the source fix works only in tests; the installed daemon or MCP proxy still records null
  external-session labels.
- Simpler alternative: leave T262 as source-only. Rejected because host-label adoption remains a
  completion gate and runtime drift is a known Engram risk.
- Failure: daemon restart breaks health/status, loses the pidfile, or labels feedback incorrectly.

## Preflight

- Git HEAD: `3777b74` (`Add guarded Codex thread telemetry fallback`).
- Tracked worktree was clean except untracked root `AGENTS.md`.
- Installed binary before refresh:
  `1059ae2f44bdcddc56ff88f2a1ed441f51459572d24d9b429248e38df1e6e2dc`.
- Source-built debug binary hash:
  `cb75551da472f70efd66f1d3dcb8b92bfd9f4bee2755be350d485a2f3e7a989b`.
- Daemon before refresh: running on port `8765`, PID `14310`.
- Codex host markers in the shell: `CODEX_THREAD_ID=019e683b-1560-7361-b535-53b012e04aa5`,
  `CODEX_SHELL=1`, and `__CFBundleIdentifier=com.openai.codex`.

## Execution

Commands:

```text
cargo install --path engram-cli --root /Users/yuval.meiri/.local
/Users/yuval.meiri/.local/bin/engram daemon stop
/Users/yuval.meiri/.local/bin/engram daemon start
```

Results:

- Install completed and replaced `/Users/yuval.meiri/.local/bin/engram`.
- Installed binary after refresh:
  `186feb4ab1e962733772773af3e1e9ca400cf52c6ebe7f92188e4eb2e17a0339`.
- `engram --version` returned `engram 0.1.0`.
- Daemon after restart: running on port `8765`, PID `70816`.
- Process check: PID `70816` is `/Users/yuval.meiri/.local/bin/engram serve --http --port 8765`.

## Live Validation

Live `orient` validation:

- Request: lean `orient(project="engram", intent="verify_decision")` with prompt
  `T263 live runtime validation for guarded Codex thread external-session fallback after daemon
  refresh.`
- Trace ID: `019e9316-093a-7242-b910-753f672a04b5`.
- `telemetry(get_trace)` returned
  `external_session_id="codex://threads/019e683b-1560-7361-b535-53b012e04aa5"`.

Feedback inheritance validation:

- Submitted feedback for trace `019e9316-093a-7242-b910-753f672a04b5` without an explicit
  `external_session_id`.
- Feedback ID: `019e9316-30b1-7941-a119-77a326d532ab`.
- Returned feedback inherited
  `external_session_id="codex://threads/019e683b-1560-7361-b535-53b012e04aa5"`.

Rolling telemetry check:

- `telemetry(real_session_eval, project="engram", limit=20)` generated at
  `2026-06-04T14:42:52.425434Z` passed the confidence gate with `feedback_coverage=0.75`,
  `distinct_intent_count=3`, `task_failure_count=0`, `bad_memory_used_count=0`,
  `missing_context_count=0`, `wrong_scope_memory_count=0`,
  `external_session_trace_count=1`, and `external_session_feedback_count=1`.

## Decision

T263 closes the installed-runtime validation gate for T262 in Codex Desktop. It proves the current
installed daemon labels live Codex `orient` traces from guarded `CODEX_THREAD_ID` and preserves
feedback trace-label inheritance. It does not prove Claude Code or Gemini host-label adoption,
native Claude prompt-bearing behavior, effective-hook visibility, M6 completion, lifecycle
cleanup, or remote publication/upstream policy.
