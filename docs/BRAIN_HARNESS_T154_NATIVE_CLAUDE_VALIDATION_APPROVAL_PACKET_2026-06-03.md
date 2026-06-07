# T154 Native Claude Non-Session Smoke Approval Packet

Date: 2026-06-03

## Status

Default-deny approval packet only. This document does not authorize execution by itself.

T153 statically validated the post-T152 harness repair state and found that generated adapters are
installed, the Claude SessionEnd command hook defaults missing `write_policy` to `nudge`, and
Claude Code still has effective-settings warnings that require native validation. Native Claude Code
or Claude Bridge execution remains gated because prior read-only Claude Bridge critique produced
SessionEnd stub handoffs.

This packet narrows the next native step to a non-session CLI smoke only. It does not approve
prompt-bearing Claude execution, Claude Bridge, Claude `/hooks`, or any interactive/native session.

## Approval Requested

Approve T154: after capturing pre-state snapshots, run only these exact native Claude commands:

```bash
/Users/yuval.meiri/.local/bin/claude --version
/Users/yuval.meiri/.local/bin/claude --help
```

After each command, compare post-state snapshots immediately. The slice may not modify
settings/hooks/adapters and must stop before any lifecycle state mutation, prompt-bearing Claude
execution, Claude Bridge execution, Claude `/hooks`, interactive session, harness install,
M6/migration/quarantine action, ranking/`orient` change, schema/storage/index change, public MCP
change, document-index behavior change, deletion, rollback, force-kill, user-owned adoption, or
unlisted command.

## Research Question

After T152 and T153, can the local Claude executable run non-session metadata/help commands without
causing unapproved lifecycle writes or hidden local state changes?

## Hypotheses

Preferred hypothesis: `claude --version` and `claude --help` run without starting a Claude session,
triggering lifecycle hooks, or mutating monitored local state.

Null hypothesis: even non-session native Claude commands cannot be proven read-only or cause any
local mutation, so the slice must stop and report the blocker.

Simpler alternative: skip native execution and treat T153 static validation as the current stopping
point until a sandboxed or hook-disabled approval is prepared.

Failure hypothesis: a supposedly non-session native Claude command triggers SessionStart or
SessionEnd, creates handoff state, changes local settings/cache files, starts background processes,
or normalizes hook behavior without explicit approval.

## Required Preflight Before Any Native Claude Process

The executor must capture and report:

- `git status --short`
- hashes for `/Users/yuval.meiri/.claude/settings.json`,
  `/Users/yuval.meiri/.claude/settings.local.json`, and
  `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh`
- current `engram harness status --json` and `engram harness doctor --json` for Claude Code
- a static inventory of Claude hook declarations and `write_policy` values
- confirmation that `/Users/yuval.meiri/.local/bin/claude` is the target executable
- the two-command table below, copied into the result report before execution

| Command | Purpose | Expected reads | Possible writes | Hook risk | Approved |
| --- | --- | --- | --- | --- | --- |
| `/Users/yuval.meiri/.local/bin/claude --version` | Print local Claude version | Claude binary and supporting runtime files | Unknown until measured | Expected no session hooks; verify by snapshots | Yes, if approved |
| `/Users/yuval.meiri/.local/bin/claude --help` | Print local Claude help/options | Claude binary and supporting runtime files | Unknown until measured | Expected no session hooks; verify by snapshots | Yes, if approved |

If either command cannot be classified as non-session before execution, stop before running it.

## Allowed Execution If Approved

Allowed only after the preflight snapshots are complete:

- direct static file reads
- read-only Engram harness status/doctor checks
- exactly `/Users/yuval.meiri/.local/bin/claude --version`
- exactly `/Users/yuval.meiri/.local/bin/claude --help`
- post-state hash/status comparison immediately after each native command

No Claude Bridge, prompt-bearing Claude command, interactive Claude command, Claude `/hooks`,
config-normalization command, or additional native command is allowed under T154.

## Hard Stops

Stop before execution if:

- either exact native command appears to start a Claude session or require prompt/session context
- hook behavior depends on runtime evaluation that cannot be observed without writes
- a command would start a background daemon, watcher, or long-lived process
- a command would edit or generate files outside a clearly reported scratch area
- a command would require modifying, renaming, chmodding, disabling, or reinstalling hooks/settings
- a command would require `harness install`, `adopt_user_owned=true`, rollback, deletion,
  force-kill, or old-binary reinstall
- any pre-state hash differs from the T153 static preflight evidence before an explanation is
  recorded
- `git status --short` changes unexpectedly

Stop after any command if:

- `git status --short` changes
- any monitored Claude settings or hook hash changes
- a new Engram handoff, lifecycle item, or obligation appears without explicit approval
- Claude output says a lifecycle hook ran, failed, or wrote state
- an unlisted file, config, cache, or lockfile write is detected and cannot be justified as allowed
  scratch output

## Explicit Non-Goals

T154 does not authorize:

- editing `/Users/yuval.meiri/.claude/settings.json`
- editing `/Users/yuval.meiri/.claude/settings.local.json`
- overwriting `/Users/yuval.meiri/.claude/engram-settings-snippet.json`
- editing root `AGENTS.md`
- running Claude Bridge
- running prompt-bearing Claude commands, interactive Claude sessions, or Claude `/hooks`
- running `harness install`
- modifying, disabling, renaming, chmodding, or reinstalling hooks
- adopting user-owned files
- lifecycle archive/supersede/apply-safe writes
- M6/migration/quarantine inventory, export, prioritize, apply, cleanup, or deletion
- ranking, `orient`, public MCP, schema/storage/index, or document-index behavior changes
- treating AI Council or Claude output as proof without source/runtime evidence

## Success Criteria

T154 succeeds only if:

- preflight and postflight snapshots are captured
- only the two approved non-session native commands are executed
- both commands complete or fail without unapproved local mutation
- no lifecycle hook execution, handoff creation, or obligation creation is observed
- effective Claude hook behavior remains explicitly unresolved beyond this non-session smoke
- missing SessionEnd `write_policy` behavioral verification remains explicitly deferred
- no unapproved file, settings, hook, lifecycle, obligation, handoff, M6, migration, quarantine,
  ranking, `orient`, schema/storage/index, public MCP, or document-index mutation occurs
- the result document records failures, hidden-write confounds, and any unresolved uncertainty

## Approval Wording

To approve execution, use:

```text
Approve T154 native Claude non-session smoke.
```

Any broader wording, generic approval, or approval for a different task is not authorization for
T154.
