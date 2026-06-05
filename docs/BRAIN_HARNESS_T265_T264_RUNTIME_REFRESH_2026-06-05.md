# Brain Harness T265 T264 Runtime Refresh

Date: 2026-06-05
Status: completed runtime refresh and read-only validation for T264. This slice installed the
current `engram-cli` source binary, restarted the Engram daemon from the Codex Desktop environment,
validated live Codex MCP labeling after the refresh, and ran a bounded installed-CLI
simulated-Claude smoke against a temp data dir. It did not run native Claude, edit
hooks/settings/adapters, mutate lifecycle or M6 state, change public MCP parameters, change
schema/storage/index/document-index behavior, change ranking or `orient`, push or set upstream,
delete data, roll back, or touch user-owned files.

## Research Question

Does the installed runtime now include T264's guarded Claude Code fallback source while preserving
the already-live Codex label behavior?

## Hypotheses

- Preferred: installing the current source binary and restarting the daemon brings T264 into the
  installed runtime; live MCP `orient` in Codex still records `codex://threads/{CODEX_THREAD_ID}`;
  installed CLI help advertises the new Claude fallback; and a simulated Claude+inherited-Codex env
  smoke completes without native Claude or hook side effects.
- Null: the installed binary remains at the T263 hash or live Codex label behavior regresses.
- Simpler alternative: leave T264 source-only. Rejected because runtime drift is a recurring Engram
  risk and host-label adoption is a completion gate.
- Failure: daemon restart breaks pidfile/status, Codex label inheritance regresses, or the refresh
  is mistaken for live native Claude/Gemini validation.

## Preflight

- Git HEAD before install: `e7397ed` (`Update external session CLI help`), with T264 behavior in
  prior commit `a012b78` (`Add guarded Claude session telemetry fallback`).
- Tracked worktree was clean except untracked root `AGENTS.md`.
- Installed binary before refresh:
  `186feb4ab1e962733772773af3e1e9ca400cf52c6ebe7f92188e4eb2e17a0339`.
- Daemon before refresh: running on port `8765`, PID `70816`.
- Codex host markers in the shell: `CODEX_THREAD_ID=019e683b-1560-7361-b535-53b012e04aa5`,
  `CODEX_SHELL=1`, and `__CFBundleIdentifier=com.openai.codex`.
- Claude host env was absent in the Codex shell: `CLAUDE_CODE_SESSION_ID` and `CLAUDECODE` were
  unset.

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
  `cb814e3f1a3c55b33d47ce15d4058e054cb7864c2303b94e06e98183f6584ea4`.
- `engram --version` returned `engram 0.1.0`.
- `engram orient --help` now says `--external-session-id` falls back to
  `ENGRAM_EXTERNAL_SESSION_ID`, guarded `CLAUDE_CODE_SESSION_ID`, then guarded
  `CODEX_THREAD_ID`.
- Daemon after restart: running on port `8765`, PID `25189`.
- Process check: PID `25189` is `/Users/yuval.meiri/.local/bin/engram serve --http --port 8765`.

## Live Validation

Codex MCP validation:

- Request: lean `orient(project="engram", intent="verify_decision")` with prompt
  `T265 live Codex validation after T264 runtime refresh. Confirm refreshed daemon labels Codex
  external session and current plan remains visible.`
- Trace ID: `019e964a-1aca-7a63-8549-04c39c491fc0`.
- `telemetry(get_trace)` returned
  `external_session_id="codex://threads/019e683b-1560-7361-b535-53b012e04aa5"`.
- The orient result surfaced `Current plan after T264 Claude session fallback` first.

Feedback inheritance validation:

- Submitted feedback for trace `019e964a-1aca-7a63-8549-04c39c491fc0` without an explicit
  `external_session_id`.
- Feedback ID: `019e964a-3cfb-7de3-9b0d-c1671ebd489b`.
- Returned feedback inherited
  `external_session_id="codex://threads/019e683b-1560-7361-b535-53b012e04aa5"`.

Installed CLI simulated-Claude smoke:

```text
env -u ENGRAM_EXTERNAL_SESSION_ID CLAUDECODE=1 CLAUDE_CODE_SESSION_ID=claude-t265 \
  CODEX_THREAD_ID=codex-t265 CODEX_SHELL=1 \
  /Users/yuval.meiri/.local/bin/engram orient \
  --data-dir /private/tmp/engram-t265-claude-runtime \
  --project engram \
  --cwd /Users/yuval.meiri/projects/engram \
  --prompt 'T265 simulated Claude Code runtime smoke for guarded external session fallback' \
  --limit 1 \
  --json
```

- The installed CLI command completed and returned trace
  `019e964a-9283-7c32-b6db-84d02633a2a7`.
- This proves the refreshed installed CLI can run the T264 path in a guarded Claude+inherited-Codex
  environment without touching the live daemon, native Claude, hooks, or the global Engram store.
- It does not prove the stored `external_session_id`, because the CLI orientation packet exposes
  `trace_id` but not the stored external-session label, and there is no installed CLI telemetry
  read subcommand.

Rolling telemetry check:

- `telemetry(real_session_eval, project="engram", limit=20)` generated at
  `2026-06-05T05:39:00.083284Z` passed the confidence gate with `feedback_coverage=0.55`,
  `distinct_intent_count=4`, `task_failure_count=0`, `bad_memory_used_count=0`,
  `missing_context_count=0`, `wrong_scope_memory_count=0`,
  `external_session_trace_count=7`, and `external_session_feedback_count=3`.

## Decision

T265 closes installed-runtime refresh for the T264 source. The refreshed daemon keeps live Codex
labeling and feedback inheritance working, and the refreshed installed CLI advertises and can run
the guarded Claude fallback path in a simulated env. It does not prove live native Claude Code MCP
labeling, prompt-bearing native Claude behavior, effective-hook visibility, Gemini host-label
adoption, M6 completion, lifecycle cleanup, or remote publication/upstream policy.
