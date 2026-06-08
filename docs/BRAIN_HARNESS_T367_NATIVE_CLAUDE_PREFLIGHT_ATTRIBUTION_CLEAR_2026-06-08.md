# Brain Harness T367 Native Claude Preflight Attribution Clear

Date: 2026-06-08
Status: read-only preflight complete; native execution is attribution-clear for the native Claude
CLI process class, but still default-deny until exact approval is given for T312, T335, or T270.

## Scope

T367 refreshes the native Claude prompt-bearing, effective-hook, and live host-label preflight
after T366. It checks whether the T363 hard stop still applies.

This slice does not launch native Claude, send prompts, run `/hooks`, signal processes, mutate
Claude settings or adapters, run harness install in write mode, mutate lifecycle state, change
source behavior, mark PR #3 ready, merge, tag, publish, or change the supported beta scope.

## Research Question

After T366, can Engram move the native Claude production gates from attribution-blocked to
approval-ready without executing a native Claude session?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Runtime, harness, daemon, vault, obligations, worktree, and process assertions match; no native `claude` CLI process makes a new session transcript ambiguous. | Supported. The native CLI attribution blocker observed in T363 is absent. |
| Null | T363 still applies unchanged and native Claude remains blocked by an active native `claude` process. | Rejected for the native CLI process class. |
| Simpler alternative | Skip native-Claude gate refresh and continue release logistics only. | Safe for scoped beta, but it would leave a production/GA gate stale. |
| Failure | Treat attribution-clear preflight as approval to launch native Claude, broaden into `/hooks` or host-label scope, or mutate user Claude state. | Avoided. |

## Read-Only Evidence

Current branch and worktree:

```text
## yuval.meiri/memory-os-phase1...origin/yuval.meiri/memory-os-phase1
?? AGENTS.md
0	0
```

Current PR state before this docs slice:

```text
PR #3 head: 49c7c18bc2230b0b25762e7a37701f567a328f8f
draft: true
mergeable: MERGEABLE
mergeStateStatus: UNSTABLE
hosted run: 27137480707
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

Snippet-only dry-run:

```text
engram harness install --harness claude-code --root /Users/yuval.meiri \
  --settings-target snippet-only --json
```

The command ran in dry-run mode, planned no writes, and reported all generated adapters already
installed. It skipped the user-owned settings snippet and did not modify Claude settings.

Canonical vault and obligations:

```text
vault(status, /Users/yuval.meiri/.engram/vault):
total_file_count = 2665
generated_file_count = 2665
user_file_count = 0
memory_item_count = 1856
knowledge_commit_count = 685
expected_generated_file_count = 2665

obligations(action=doctor, project=engram, cwd=/Users/yuval.meiri/projects/engram)
=> {"open":[],"warnings":[]}
```

Telemetry windows were recorded:

```text
real_session_eval(project=engram, limit=20):
trace_count = 20
feedback_count = 10
feedback_coverage = 0.50
confidence_gate.passed = true

real_session_eval(project=engram, limit=50):
trace_count = 50
feedback_count = 18
feedback_coverage = 0.36
confidence_gate.passed = false
reason = Need feedback coverage of at least 50%; found 36%.
```

The 50-trace confidence miss is evidence to report. It does not authorize broadening the native
Claude scope or running unrelated write paths.

## Attribution Result

Fresh process inventory found no active native `claude` CLI process. The previous T363 hard stop
was:

```text
PID 34797 on ttys004: claude
```

That native CLI process is absent in T367.

The broader inventory still contains ambient Claude-family helper processes, including
`codex-claude mcp` bridge processes and the Claude app `chrome-native-host`. These do not by
themselves prove or disprove native Claude Code behavior. They must still be listed in any future
T312, T335, or T270 execution report, and any evidence that could be confused with a bridge,
simulated environment, explicit label, or inherited non-native label must be treated as failed or
inconclusive.

## Gate Impact

- T312 prompt-bearing native Claude validation is now approval-ready from an attribution
  standpoint, assuming a fresh preflight immediately before launch still matches this state.
- T335 effective-hook visibility is now approval-ready from an attribution standpoint, assuming a
  fresh preflight immediately before launch still matches this state.
- T270 live Claude host-label proof is now approval-ready from an attribution standpoint, assuming
  a fresh preflight immediately before launch still matches this state.
- The scoped local/Codex MVP beta path remains unaffected.

## Next Approval Options

Use the existing exact approval packets; do not treat generic continuation as authorization.

- T312: execute the prompt-bearing native Claude MCP-`orient` validation from
  `docs/BRAIN_HARNESS_T312_CLAUDE_2168_SUCCESSOR_PACKET_2026-06-07.md`.
- T335: execute the native Claude effective-hook visibility revalidation from
  `docs/BRAIN_HARNESS_T335_T269_EFFECTIVE_HOOK_2168_SUCCESSOR_PACKET_2026-06-07.md`.
- T270: execute the native Claude host external-session label validation from
  `docs/BRAIN_HARNESS_T270_HOST_LABEL_GATE_APPROVAL_PACKET_2026-06-05.md`.

T367 itself is read-only preflight evidence. It does not prove native Claude prompt-bearing
behavior, effective-hook visibility, live Claude host labels, hosted CI, direct legacy cleanup,
broad lifecycle cleanup, or production/GA completion.
