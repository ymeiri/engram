# Brain Harness T400 Native Claude Gate Preflight Script

Date: 2026-06-09
Status: completed read-only gate hardening and canonical vault resync.

## Scope

T400 makes the native Claude prompt-bearing, effective-hook, and live host-label production gate
repeatable with `scripts/native-claude-gate-preflight.sh`.

The script collects read-only evidence for:

- branch/upstream sync and source tree state,
- current Claude Code path, target, version, and SHA-256,
- installed Engram daemon provenance,
- Claude Code harness status and doctor readiness,
- snippet-only harness install dry-run drift,
- project-scoped obligations doctor,
- canonical vault alignment,
- native Claude CLI process attribution blockers.

It does not launch Claude, send prompts, run `/hooks`, signal processes, mutate settings,
adapters, hooks, lifecycle state, M6 state, memory state, release state, PR state, tags, or
published artifacts.

## Research Question

Can Engram replace one-off native-Claude preflight notes with a repeatable fail-closed script while
preserving the hard stop for contaminated native Claude execution?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | A small shell script can classify the production gate as ready or blocked using the same read-only evidence as T340/T372. | Supported. The script reports `gate_state: blocked` with the current native Claude process as the blocker. |
| Null | Manual preflight docs are enough. | Rejected. Claude Code advanced from `2.1.168` to `2.1.169`, showing why a repeatable current-state check is needed. |
| Simpler alternative | Defer native Claude proof without a script. | Rejected. That would preserve ambiguity and invite stale packet execution. |
| Failure | The script launches Claude, sends `/hooks`, mutates settings, or treats static readiness as behavioral proof. | Avoided by read-only implementation and explicit non-actions in output. |

## Current Evidence

The fresh preflight baseline is:

```text
branch: yuval.meiri/memory-os-phase1
upstream: origin/yuval.meiri/memory-os-phase1 (ahead=0 behind=0)
head: b233f6601fbbaacaf1169ced93a31459f0f6f039
claude_bin: /Users/yuval.meiri/.local/bin/claude
claude_target: /Users/yuval.meiri/.local/share/claude/versions/2.1.169
claude_version: 2.1.169 (Claude Code)
claude_sha256: 86d8b820ad7eed50e50a130706d3dc5ef70696f91194de1b3897a842182afe3a
engram_bin: /Users/yuval.meiri/.local/bin/engram
daemon_spawn_version: 0.2.0-beta.1
harness_status_ready: true
harness_doctor_ready: true
snippet_only_dry_run_planned: []
obligations: open=[], warnings=[]
vault: initialized=true, generated=2814, expected=2814, user=0
```

Before the repo change, the canonical vault had drifted to `2813` generated files versus `2814`
expected after a new MemoryItem write. T400 recompiled `/Users/yuval.meiri/.engram/vault`; status
then returned `total_file_count=2814`, `generated_file_count=2814`,
`expected_generated_file_count=2814`, and `user_file_count=0`.

The current hard stop is still live native Claude attribution:

```text
native_claude_processes_present: true
native_claude_processes:
  34797 18673 ttys004 S+ ... claude
```

That process keeps T312 prompt-bearing proof, T335 `/hooks` effective-hook proof, and T270 live
host-label proof blocked. T400 records current Claude Code `2.1.169` as the observed preflight
baseline, but it is still not execution evidence.

## Script Contract

Normal report mode exits successfully if evidence was collected and classified:

```bash
scripts/native-claude-gate-preflight.sh
```

Strict mode fails closed unless every check is ready and no native Claude CLI process is already
running:

```bash
scripts/native-claude-gate-preflight.sh --require-ready
```

During development, `--allow-worktree-changes` permits tracked or extra untracked source changes
without turning them into gate blockers. Final release/GA checks should omit that flag.

## Validation

Validation performed for this slice:

- `bash -n scripts/native-claude-gate-preflight.sh`
- `scripts/native-claude-gate-preflight.sh --allow-worktree-changes`
- `scripts/native-claude-gate-preflight.sh --allow-worktree-changes --require-ready`

The normal report returned `gate_state: blocked` and exited `0`. Strict mode returned the same
evidence and exited `2`, proving the gate fails closed while PID `34797` is live.

## Gate Impact

For the scoped local/Codex MVP beta, T400 does not change the ship path. The remaining beta gate is
still release-owner acceptance of the exact-head local/package evidence as hosted-CI fallback, or
restored exact-head hosted CI green, followed by ready/merge/tag/publish mechanics.

For production/GA, T400 improves the native-Claude gate from a manual note to a repeatable
preflight command. It does not close native Claude prompt-bearing proof, effective-hook
visibility, live host labels, multi-host parity, hosted CI, M6, lifecycle cleanup, release, or
production/GA readiness.
