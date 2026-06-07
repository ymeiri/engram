# Brain Harness T281 T255 Native Claude Preflight Drift - 2026-06-06

## Scope

T281 attempted only the read-only preflight for the T255 native Claude prompt-bearing parity packet.
It did not launch native Claude because a T255 hard-stop condition was met before execution.

T281 did not send a native Claude prompt, run `/hooks`, edit hooks/settings/adapters, run harness
install, mutate lifecycle state, run M6, change ranking or `orient`, publish branches, delete data,
rollback, send process signals, or touch user-owned files.

## Research Question

Is the T255 native Claude prompt-bearing validation packet still executable under its recorded
preflight assumptions?

## Hypotheses

| Type | Hypothesis | Evidence |
| --- | --- | --- |
| Preferred | The installed native Claude target still matches the T255 `2.1.161` baseline, so one bounded native session can run. | Rejected. |
| Null | T255 remains docs-only and unexecuted. | Confirmed for this slice. |
| Failure | The binary target/version or process state drifts before launch, so running native Claude would invalidate the packet. | Confirmed by version drift, with live Claude-family processes also visible. |

## Preflight Evidence

Read-only checks showed:

| Check | Result |
| --- | --- |
| Git status | Branch tracks `origin/yuval.meiri/memory-os-phase0`; only root `AGENTS.md` untracked |
| Git diff | no tracked diff |
| Latest commit | `5b5e4bb Record lifecycle archive execution` |
| Claude symlink | `/Users/yuval.meiri/.local/bin/claude -> /Users/yuval.meiri/.local/share/claude/versions/2.1.163` |
| Resolved target | `/Users/yuval.meiri/.local/share/claude/versions/2.1.163` |
| `claude --version` | `2.1.163 (Claude Code)` |
| T255 required baseline | `2.1.161` |
| Resolved target hash | `c7582e926e8fe459dbd9743f19ccb75500e3b455c722902d1aa587a74fb1fa7c` |
| Monitored file snapshot | 43 files across configured user/project Claude paths |
| Harness status | `ready=true`; split settings and user-owned snippet warnings still present |
| Harness doctor | `ready=true`; same warnings plus soft lifecycle-compliance caveat |
| Daemon status | running on port `8765`, PID `25189` |
| Memory cursor | `019e9bec-9949-7341-ab17-4bb9e480a50f` at `2026-06-06T07:56:06.336425Z` |
| Obligations doctor | `open=[]`, `warnings=[]` |
| Telemetry, limit 20 | confidence gate failed: 45% feedback coverage and 2 intents with feedback |
| Telemetry, limit 50 | confidence gate passed: 60% feedback coverage, 5 intents, 0 task failures |

The clean process snapshot after the version probe exited still showed ambient Claude-family
processes:

```text
node /opt/homebrew/bin/codex-claude mcp
node /opt/homebrew/bin/codex-claude mcp
claude
claude --plugin-dir /Users/yuval.meiri/go/src/github.com/DataDog/claude-marketplace/ai-developer-workflows
```

## Decision

T255 is not executable as written because its preflight says to stop if the Claude binary
target/version differs from the recorded `2.1.161` baseline. The current target is `2.1.163`.

The native prompt-bearing Claude session was not launched. The ambient Claude-family processes were
not signaled or modified; after the binary drift hard stop, they were recorded only as additional
launch-confounding evidence.

## Completion Impact

T281 does not close the prompt-bearing native Claude gate. It converts the next safe step from
"execute T255" to "prepare or approve a successor packet for the current Claude `2.1.163` target, or
explicitly defer native Claude prompt-bearing validation with evidence."

Effective-hook visibility and live Claude host-label proof remain separate gates. T269 and T270
should not be inferred from this preflight drift result.
