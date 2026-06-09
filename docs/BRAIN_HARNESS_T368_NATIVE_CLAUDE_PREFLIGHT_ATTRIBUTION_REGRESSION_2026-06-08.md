# Brain Harness T368 Native Claude Preflight Attribution Regression

Date: 2026-06-08
Status: read-only preflight complete; native Claude execution is hard-stopped by a live native
`claude` CLI process.

## Scope

T368 refreshes the native Claude prompt-bearing, effective-hook, and live host-label preflight
after T367. T367 was point-in-time evidence that the earlier native CLI attribution blocker was
absent. T368 checks the current state before any live native proof.

This slice does not launch native Claude, attach to a Claude process, send prompts, run `/hooks`,
signal processes, mutate Claude settings or adapters, run harness install in write mode, mutate
lifecycle state, change source behavior, mark PR #3 ready, merge, tag, publish, or change the
supported beta scope.

## Research Question

Can Engram safely execute a native Claude proof slice now, or has process attribution become
ambiguous again?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Fresh runtime, harness, daemon, vault, obligations, worktree, and process assertions identify whether native Claude execution is currently safe. | Supported. The preflight found a live native `claude` CLI process, so execution is not safe. |
| Null | T367's attribution-clear conclusion remains valid without a new process check. | Rejected. Current process inventory contradicts it. |
| Simpler alternative | Ignore native Claude proof and continue only beta release logistics. | Safe for scoped beta, but it would leave the production/GA native gate stale and could lead to unsafe future execution. |
| Failure | Treat T367 as durable authorization and launch a new native Claude session despite current process ambiguity. | Avoided. |

## Read-Only Evidence

Current branch and worktree:

```text
## yuval.meiri/memory-os-phase1...origin/yuval.meiri/memory-os-phase1
?? AGENTS.md
0	0
```

Current PR state before this docs slice:

```text
PR #3 head: 382e0ecdc37b29ca55094c79d9b5036d5e74f1bb
draft: true
mergeable: MERGEABLE
mergeStateStatus: UNSTABLE
hosted run: 27137818307
hosted jobs: Check, Test, Format, Clippy, Docs all failed with steps=[]
```

Claude runtime:

```text
which claude => /Users/yuval.meiri/.local/bin/claude
readlink /Users/yuval.meiri/.local/bin/claude =>
/Users/yuval.meiri/.local/share/claude/versions/2.1.168
/Users/yuval.meiri/.local/bin/claude --version => 2.1.168 (Claude Code)
sha256(/Users/yuval.meiri/.local/share/claude/versions/2.1.168) =
377f0ecedba8246bdabdf312ce8b7cc8ae1160997b26f5edca352a4a8d61dc78
```

Engram runtime and daemon:

```text
/Users/yuval.meiri/.local/bin/engram --version => engram 0.2.0-beta.1
sha256(/Users/yuval.meiri/.local/bin/engram) =
77a08e895614bea3b02816e67bafd64087ea0634f4b0ca58b8199a9ef7855633

Daemon status: running
Port: 8765
PID: 47577
Spawned by: /Users/yuval.meiri/.local/bin/engram
Spawn version: 0.2.0-beta.1
Current CLI: /Users/yuval.meiri/.local/bin/engram

/health => {"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

Harness readiness:

```text
engram harness status --harness claude-code --root /Users/yuval.meiri --json
engram harness doctor --harness claude-code --root /Users/yuval.meiri --json
```

Both commands report `ready=true`. Expected warnings remain:

- `.claude/engram-settings-snippet.json` is user-owned and is not overwritten;
- Engram permissions are split across `settings.json` and `settings.local.json`;
- legacy Engram permission entries still exist outside the current Claude harness contract;
- effective hook configuration still needs live Claude Code `/hooks` proof;
- lifecycle compliance remains soft and depends on the agent following the policy.

Canonical vault and telemetry:

```text
vault(status, /Users/yuval.meiri/.engram/vault):
total_file_count = 2668
generated_file_count = 2668
user_file_count = 0
memory_item_count = 1859
knowledge_commit_count = 686
expected_generated_file_count = 2670
```

The vault is two generated pages behind expected count after recent memory/project-observation
writes. This is maintenance evidence to refresh during closeout; it does not authorize native
execution or broaden the proof scope.

Telemetry windows after the T368 orientation feedback:

```text
real_session_eval(project=engram, limit=20):
trace_count = 20
feedback_count = 12
feedback_coverage = 0.60
confidence_gate.passed = true

real_session_eval(project=engram, limit=50):
trace_count = 50
feedback_count = 19
feedback_coverage = 0.38
confidence_gate.passed = false
reason = Need feedback coverage of at least 50%; found 38%.
```

The 50-trace confidence miss is evidence to report. It does not authorize broadening the native
Claude scope or running unrelated write paths.

## Process Attribution Result

Fresh process inventory shows a live native `claude` CLI process:

```text
PID   PPID  STAT  TTY      STARTED                    ELAPSED   COMM    ARGS
34797 18673 S+    ttys004  Mon Jun 8 11:49:15 2026    03:53:56  claude  claude
```

This is the same native CLI process class that blocked T363. A new native Claude proof session
would not have clean attribution while this process is live, because transcript, side effect, hook,
or host-label evidence could be confused with the existing session.

The broader process inventory also contains ambient Claude-family helper processes, including
`codex-claude mcp` bridge processes, Claude app `chrome-native-host`, and Claude SDK worker
processes under the local bridge. These helper processes remain reportable context, but the hard
stop is the live native `claude` CLI process on `ttys004`.

## Gate Impact

- T312 prompt-bearing native Claude validation is not executable now.
- T335 effective-hook visibility is not executable now.
- T270 live Claude host-label proof is not executable now.
- The scoped local/Codex MVP beta path remains unaffected.

T367's attribution-clear result should be treated as historical point-in-time evidence, not as
durable authorization. Any future T312, T335, or T270 execution must rerun fresh preflight and stop
again if PID `34797` or any other attribution-confusing native Claude process remains live.

## Next Safe Options

Safe follow-ups that do not require signaling the live process:

- wait for the native `claude` CLI process to exit naturally, then rerun the read-only preflight;
- ask the user for exact approval before any process signal or native Claude execution;
- continue beta release logistics, since this gate is deferred from the scoped local/Codex beta;
- continue unrelated production-hardening slices that do not rely on native Claude attribution.

T368 itself is read-only preflight evidence. It does not prove native Claude prompt-bearing
behavior, effective-hook visibility, live Claude host labels, hosted CI, direct legacy cleanup,
broad lifecycle cleanup, or production/GA completion.
