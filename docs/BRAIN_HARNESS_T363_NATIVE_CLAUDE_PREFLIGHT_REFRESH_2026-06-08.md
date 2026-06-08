# Brain Harness T363 Native Claude Preflight Refresh

Date: 2026-06-08
Status: read-only preflight complete; native execution remains attribution-blocked

## Scope

T363 refreshes the native Claude prompt-bearing, effective-hook, and live host-label gate evidence
after T362. It reruns the read-only assertions shared by the T312 prompt-bearing packet, the T335
effective-hook successor packet, and the T270 live host-label packet.

This slice does not launch native Claude, send prompts, run `/hooks`, signal processes, mutate
Claude settings or adapters, run harness install in write mode, mutate lifecycle state, change
source behavior, or change the supported local/Codex beta scope.

## Research Question

After the T362 installed-runtime refresh, can Engram safely proceed to a native Claude
prompt-bearing, effective-hook, or host-label proof run?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Runtime, harness, daemon, vault, and obligation assertions still match; process inventory decides whether native execution can proceed. | Supported. Runtime and health assertions match, but process attribution still fails. |
| Null | The T340/T356 evidence is still sufficient and no refresh is useful. | Rejected. Current process inventory changed from earlier PIDs and must be checked before any native run. |
| Simpler alternative | Skip native-Claude gate refresh and continue release logistics only. | Safe for scoped beta, but it would leave a production/GA gate stale. |
| Failure | Launch native Claude despite ambiguous attribution, broaden into `/hooks` or host-label scope, or mutate user Claude state. | Avoided. |

## Read-Only Evidence

Current branch and worktree:

```text
## yuval.meiri/memory-os-phase1...origin/yuval.meiri/memory-os-phase1
?? AGENTS.md
0	0
```

Claude runtime:

```text
which claude => /Users/yuval.meiri/.local/bin/claude
readlink /Users/yuval.meiri/.local/bin/claude => /Users/yuval.meiri/.local/share/claude/versions/2.1.168
/Users/yuval.meiri/.local/bin/claude --version => 2.1.168 (Claude Code)
sha256(/Users/yuval.meiri/.local/bin/claude) =
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
- some legacy Engram permission entries are still present outside the current Claude harness
  contract;
- effective hook configuration still needs live `/hooks` proof.

Snippet-only dry-run:

```text
engram harness install --harness claude-code --root /Users/yuval.meiri --settings-target snippet-only --json
```

The command ran in dry-run mode, planned no writes, and reported all generated adapters already
installed. It skipped the user-owned settings snippet and did not modify Claude settings.

Canonical vault and obligations:

```text
vault(status, /Users/yuval.meiri/.engram/vault):
total_file_count = 2651
generated_file_count = 2651
user_file_count = 0
memory_item_count = 1846
knowledge_commit_count = 681
expected_generated_file_count = 2651

engram obligations doctor --scope-project engram --cwd /Users/yuval.meiri/projects/engram --json
=> {"open":[],"warnings":[]}
```

## Attribution Hard Stop

Fresh process inventory still includes a live native Claude CLI process:

```text
PID   PPID  TTY      STAT  ELAPSED   COMMAND
34797 18673 ttys004  S+    02:41:58  claude
```

The broader process inventory also includes Claude-family bridge and app-helper processes such as
`codex-claude mcp` and `/Applications/Claude.app/.../chrome-native-host`.

The active `claude` process on `ttys004` is enough to make a new single-session native transcript
attribution ambiguous. The T312/T335/T270 packets require stopping before launch when existing
native Claude or Claude-family processes would make attribution ambiguous.

Therefore T363 did not launch native Claude, run `/hooks`, send prompts, signal processes, or
collect prompt-bearing/effective-hook/host-label proof.

## Gate Impact

- T312 prompt-bearing native Claude validation remains unexecuted. Runtime, harness, daemon, vault,
  obligations, and worktree assertions match; process attribution does not.
- T335 effective-hook visibility remains unexecuted. The current runtime baseline and snippet-only
  dry-run match, but `/hooks` cannot be measured under ambiguous attribution.
- T270 live Claude host-label proof remains unexecuted. It still requires one clean native Claude
  run that creates one Engram trace through the live MCP path.
- The scoped local/Codex MVP beta path remains unaffected.

## Next Action

Retry T312, T335, or T270 only after a fresh read-only preflight shows no
attribution-confusing native Claude process while the path, version, hash, harness readiness,
daemon health, vault status, obligations doctor, and worktree checks still match.

T363 is read-only preflight evidence. It does not prove native Claude prompt-bearing behavior,
effective-hook visibility, live Claude host labels, hosted CI, direct legacy cleanup, broad
lifecycle cleanup, or production/GA completion.
