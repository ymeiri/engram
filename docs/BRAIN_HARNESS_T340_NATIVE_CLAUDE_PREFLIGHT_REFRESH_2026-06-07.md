# Brain Harness T340 Native Claude Preflight Refresh

Date: 2026-06-07
Status: completed read-only preflight; native launch hard-stopped

## Scope

T340 refreshes the native Claude prompt-bearing, effective-hook, and live host-label gate evidence
after T339. It reruns the read-only assertions shared by the T312 prompt-bearing packet, the T335
effective-hook successor packet, and the T270 live host-label packet.

This slice does not launch native Claude, send a prompt, send `/hooks`, signal or kill any process,
use Claude Bridge, mutate settings, adapters, hooks, lifecycle state, M6/migration state, ranking,
`orient`, public MCP/schema/storage/index behavior, branch state, or user-owned files.

## Research Question

Can Engram safely proceed from the native-Claude preflight state to one of the production
validation executions, or does the fresh preflight still require a hard stop before launch?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | The post-T333 adapter and runtime assertions still pass, and the fresh process inventory decides whether native execution can proceed. | Supported. Runtime and harness assertions pass, but process attribution still fails. |
| Null | The T334/T335 evidence is still sufficient and no refresh is useful. | Rejected. Current evidence shows the same blocker persists at the T340 head. |
| Simpler alternative | Defer all native gates without a fresh preflight. | Rejected because a read-only preflight is cheap and clarifies the live blocker. |
| Failure | Launch native Claude despite ambiguous attribution, broaden into `/hooks` or host-label scope, or mutate user Claude state. | Avoided. |

## Preflight Evidence

Repository and PR state:

- Branch `yuval.meiri/memory-os-phase1` is synced with its upstream: `HEAD...@{u}` is `0 0`.
- `HEAD...origin/main` is `35 0`.
- The tracked worktree has no diff; only root `AGENTS.md` is untracked and user-owned.
- PR #3 is open and draft at head `6d0467e933a880f9039fd943b34848c2ca93f069`.
- Hosted CI run `27101033097` still fails all five jobs before workflow steps with empty-step jobs,
  preserving the GitHub Actions account/billing/spending-limit external blocker.

Claude runtime state:

- `which claude` resolves to `/Users/yuval.meiri/.local/bin/claude`.
- `/Users/yuval.meiri/.local/bin/claude` resolves to
  `/Users/yuval.meiri/.local/share/claude/versions/2.1.168`.
- `/Users/yuval.meiri/.local/bin/claude --version` returns `2.1.168 (Claude Code)`.
- Both the symlink path and resolved target hash to
  `377f0ecedba8246bdabdf312ce8b7cc8ae1160997b26f5edca352a4a8d61dc78`.

Harness, daemon, vault, and obligations state:

- Claude Code `harness(action=status)` reports `ready=true`.
- Claude Code `harness(action=doctor)` reports `ready=true`.
- `harness(action=install, harness=claude_code, settings_target=snippet-only, write=false)`
  reports `planned=[]`.
- `engram daemon status` reports port `8765`, PID `92750`, spawned by
  `/Users/yuval.meiri/.local/bin/engram`, spawn version `0.2.0-beta.1`.
- The installed Engram binary hash is
  `e53765568a2232c55c2d17a8a48480e745b2c2fda044a8d087681c20534e3dc5`.
- `vault(action=status, vault_path="/Users/yuval.meiri/.engram/vault")` reports
  `total_file_count=2573`, `generated_file_count=2573`, `user_file_count=0`, and
  `expected_generated_file_count=2573`.
- `obligations(action=doctor, project=engram)` returns no open obligations and no warnings.

Known harness warnings remain bounded:

- `/Users/yuval.meiri/.claude/engram-settings-snippet.json` is user-owned.
- `settings.json` and `settings.local.json` still contain extra legacy Engram permissions.
- Engram Claude settings are split across `settings.json` and `settings.local.json`, so effective
  hook visibility still requires a native `/hooks` measurement.
- Lifecycle compliance remains a soft contract.

Telemetry state:

- `real_session_eval(project=engram, limit=20)` reports `trace_count=20`,
  `feedback_count=6`, `feedback_coverage=0.30000001192092896`, and
  `confidence_gate.passed=false`.
- `real_session_eval(project=engram, limit=50)` reports `trace_count=50`,
  `feedback_count=16`, `feedback_coverage=0.3199999928474426`, and
  `confidence_gate.passed=false`.
- Both reports show `task_failure_count=0` and `bad_memory_used_count=0`, but the confidence gate
  still fails on feedback coverage.

Monitored hashes:

| Path | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `06efbf2a5d84ba62a1fcba0854863579ae23aaabb270e8a7bce7a88368ecf549` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/engram-settings-snippet.json` | `b677c1ed6b915e3186d433f25148d1f7f1e697b0ec0a793e5c3c742833733d60` |
| `/Users/yuval.meiri/.claude/commands/engram-memory-session.md` | `a5075190c01731c82be7b50eb219fe7e467812c3d210e083eec9405e1ff95259` |
| `/Users/yuval.meiri/.claude/commands/engram-resume-session.md` | `90cdf6b33a24c1d8db0f33202dc5cc43dd0c11edb128271d91ad982f48d2a83d` |
| `/Users/yuval.meiri/.claude/commands/engram-end-session.md` | `63c932a02ebd40563be6b7aa90200653d04c8073df61a858a606a7a8dd6482fb` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-start.sh` | `c49c516aa30604cb87841d368e830275aa05355c27a359664876ac742350b27f` |
| `/Users/yuval.meiri/.claude/hooks/engram-stop-nudge.sh` | `3eabbfaf6921cedc5245c18450092747e0c8ba506bb4a47ca04d8b131b33633c` |
| `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `3069926f9b718bf0ec13978827ec2c3eb0d8810d1e01c750a35e8d1b92c652a9` |

## Hard Stop

Fresh process inventory still shows already-running native Claude CLI sessions:

| PID | PGID | TTY | Command |
| --- | --- | --- | --- |
| `60453` | `60453` | `ttys001` | `claude` |
| `311` | `311` | `ttys005` | `claude --plugin-dir /Users/yuval.meiri/go/src/github.com/DataDog/claude-marketplace/ai-developer-workflows` |

Those processes make attribution ambiguous for any new prompt-bearing native Claude, `/hooks`, or
host-label proof run. T312/T335/T270 require stopping before launch when existing native Claude or
Claude-family processes would make attribution ambiguous. Therefore T340 did not launch native
Claude, send input, run `/hooks`, signal a process, or mutate any runtime/configuration state.

## Gate Impact

- T312 prompt-bearing native Claude validation remains unexecuted. Runtime, harness, daemon,
  vault, obligations, and worktree assertions now match; process attribution does not.
- T335 effective-hook visibility remains unexecuted. The current runtime baseline and empty
  snippet-only dry-run match, but `/hooks` cannot be measured under ambiguous attribution.
- T270 live Claude host-label proof remains unexecuted. It still requires one clean native Claude
  MCP trace and postflight telemetry label proof.
- Hosted CI remains externally blocked by GitHub Actions account/billing/spending-limit failures.
- The supported local/Codex MVP beta scope is unchanged.

## Next Closure Condition

Retry T312, T335, or a combined exact packet only after a fresh read-only preflight shows no
attribution-confusing native Claude or Claude-family processes, while the path, target, version,
hash, harness, daemon, obligations, vault, and worktree assertions still match.

## Non-Claims

T340 is read-only preflight evidence. It does not prove native Claude prompt-bearing behavior,
effective-hook visibility, live Claude host labels, hosted CI, direct legacy cleanup, broad
lifecycle cleanup, M6 confidence, or production/GA Brain Harness completion.
