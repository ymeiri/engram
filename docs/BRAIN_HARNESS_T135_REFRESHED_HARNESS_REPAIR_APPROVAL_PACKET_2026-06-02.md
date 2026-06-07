# T135 Refreshed Harness Repair Approval Packet

Date: 2026-06-02
Status: pending user approval
Scope: refreshed proposal for exact local harness repair writes derived from read-only dry-runs

This packet is a request for approval, not approval itself. No adapter install, settings edit,
hook registration, migration action, lifecycle mutation, schema/storage/index change, public MCP
change, ranking change, document-index change, or `orient` payload change has been run for T135.

## Research Question

Can Engram safely refresh the stale T47 local harness repair packet after T130/T133A using fresh
read-only status and dry-run evidence, without writing local harness state?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A docs-only refreshed packet can let the user approve or reject exact harness repair writes without granting broad local-configuration authority. |
| Null | The proposed repair scope is too broad, stale, or ambiguous to approve safely. |
| Simpler alternative | Defer harness repair and continue only read-only validation, accepting that supported harnesses remain `ready=false`. |
| Failure | The packet implies approval, hides new write effects, writes user-owned files, touches unlisted hooks/settings, or bundles unrelated M6/lifecycle/hot-path work. |

## Why T47 Is Stale

T47 was a useful approval packet, but it is no longer exact. It listed the installed Claude
`SessionEnd` hook as skipped/already installed. After T130 and the approved T133A live-runtime
refresh, source and live render now default missing `write_policy` to `nudge`, while the installed
Claude hook still defaults to `durable` and is reported as drifted. Fresh dry-run evidence now
plans a generated-adapter update for:

`/Users/yuval.meiri/.claude/hooks/engram-session-end.sh`

That file effect was not in the T47 planned write manifest, so T47 should not be reused as approval
for repair writes.

## Current Evidence

- Lean `orient` for the resumed work returned the active current plan, harness write gate, M6 gate,
  commit preference, and research-method rule.
- Direct current-plan search recovered the active current-plan memory first, but still returned old
  handoffs below it, confirming stale handoff noise remains an evidence-quality issue.
- Source inspection confirms `harness(action="install")` still defaults `write=false`,
  `adopt_user_owned=false`, and writes only after explicit `write=true`.
- T133A validated live `harness(render_adapter, claude_code, claude-session-end-hook)` output:
  missing hook-input `write_policy` now defaults to `nudge` in the running MCP runtime.
- Fresh read-only `harness(action="status")` checks still report `ready=false` for generic,
  Claude Code, Codex, Gemini CLI, and Cursor.
- Fresh read-only `harness(action="install", write=false, adopt_user_owned=false, ...)` dry-runs
  produced the planned file effects listed below and no writes.
- AI Council recall found prior T47 decisions. A new T135 broadcast agreed that the next
  non-destructive slice should be a docs-only refreshed approval packet, with exact file effects,
  fresh matching dry-runs before each write, and stop conditions for any drift.
- Claude Bridge was not consulted for this packet because the installed Claude `SessionEnd` hook is
  part of the known stale durable-handoff write path until repaired.

## Current Readiness Matrix

| Harness | Current status | Evidence |
| --- | --- | --- |
| `generic` | `ready=false` | Missing `/Users/yuval.meiri/.engram/harness-policy.md`. |
| `codex` | `ready=false` | Drifted memory-session and resume-session skills. |
| `gemini_cli` | `ready=false` | Drifted memory-session command, resume-session command, and `GEMINI.md`. |
| `cursor` | `ready=false` | Drifted memory-session and resume-session skills. |
| `claude_code` | `ready=false` | Drifted `SessionEnd` hook, user-owned settings snippet skipped, missing settings registrations, and legacy extra permissions. |

## Proposed Approval

If the user explicitly approves this packet, the authorized write sequence is exactly the following
five MCP calls, executed one harness at a time and only after a fresh matching dry-run for that
harness:

```text
harness(
  action="install",
  harness="generic",
  root="/Users/yuval.meiri",
  write=true,
  adopt_user_owned=false
)

harness(
  action="install",
  harness="codex",
  root="/Users/yuval.meiri",
  write=true,
  adopt_user_owned=false
)

harness(
  action="install",
  harness="gemini_cli",
  root="/Users/yuval.meiri",
  write=true,
  adopt_user_owned=false
)

harness(
  action="install",
  harness="cursor",
  root="/Users/yuval.meiri",
  write=true,
  adopt_user_owned=false
)

harness(
  action="install",
  harness="claude_code",
  root="/Users/yuval.meiri",
  write=true,
  adopt_user_owned=false,
  settings_target="settings.local.json"
)
```

Any missing, conditional, partial, broad, or ambiguous approval remains default-deny.

## Planned File Effects

The final pre-write dry-run must match this manifest exactly.

| Harness | Operation | Path | Dry-run message |
| --- | --- | --- | --- |
| `generic` | Create generated adapter | `/Users/yuval.meiri/.engram/harness-policy.md` | `will create generated adapter` |
| `codex` | Update generated adapter | `/Users/yuval.meiri/.codex/skills/engram-memory-session/SKILL.md` | `will update generated adapter` |
| `codex` | Update generated adapter | `/Users/yuval.meiri/.codex/skills/engram-resume-session/SKILL.md` | `will update generated adapter` |
| `gemini_cli` | Update generated adapter | `/Users/yuval.meiri/.gemini/commands/engram/memory-session.toml` | `will update generated adapter` |
| `gemini_cli` | Update generated adapter | `/Users/yuval.meiri/.gemini/commands/engram/resume-session.toml` | `will update generated adapter` |
| `gemini_cli` | Update generated adapter | `/Users/yuval.meiri/.gemini/GEMINI.md` | `will update generated adapter` |
| `cursor` | Update generated adapter | `/Users/yuval.meiri/.cursor/skills/engram-memory-session/SKILL.md` | `will update generated adapter` |
| `cursor` | Update generated adapter | `/Users/yuval.meiri/.cursor/skills/engram-resume-session/SKILL.md` | `will update generated adapter` |
| `claude_code` | Update generated adapter | `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `will update generated adapter` |
| `claude_code` | Merge settings only | `/Users/yuval.meiri/.claude/settings.local.json` | `will merge Engram MCP permissions and lifecycle hooks into settings.local.json` |

The current dry-runs also skipped these already-installed or user-owned files. This packet does not
authorize writing them:

| Harness | Skipped path | Current dry-run reason |
| --- | --- | --- |
| `codex` | `/Users/yuval.meiri/AGENTS.engram.md` | `already installed` |
| `gemini_cli` | `/Users/yuval.meiri/.gemini/commands/engram/end-session.toml` | `already installed` |
| `cursor` | `/Users/yuval.meiri/.cursor/skills/engram-end-session/SKILL.md` | `already installed` |
| `claude_code` | `/Users/yuval.meiri/.claude/commands/engram-memory-session.md` | `already installed` |
| `claude_code` | `/Users/yuval.meiri/.claude/commands/engram-resume-session.md` | `already installed` |
| `claude_code` | `/Users/yuval.meiri/.claude/commands/engram-end-session.md` | `already installed` |
| `claude_code` | `/Users/yuval.meiri/.claude/hooks/engram-session-start.sh` | `already installed` |
| `claude_code` | `/Users/yuval.meiri/.claude/hooks/engram-stop-nudge.sh` | `already installed` |
| `claude_code` | `/Users/yuval.meiri/.claude/engram-settings-snippet.json` | `skipped user-owned file without Engram marker` |
| `claude_code` | `/Users/yuval.meiri/AGENTS.engram.md` | `already installed` |

## In Scope After Explicit Approval

| Item | Allowed after explicit approval? | Notes |
| --- | --- | --- |
| Fresh `write=false` dry-run for each listed harness | Yes | Must match the manifest before that harness is written. |
| The five `write=true` calls shown above | Yes | Execute one harness at a time; no batching with other work. |
| `harness(action="status")` and `harness(action="doctor")` after each write | Yes | Read-only validation only. |
| Syntax or status checks for the listed touched files/settings | Yes | Only for the listed targets. |
| A Markdown report and documentation commit | Yes | Report the exact outcome and any remaining readiness caveat. |
| Telemetry feedback and current-plan memory capture | Yes | Evidence annotation only. |

## Out Of Scope

| Item | Authorized by this packet? |
| --- | --- |
| M6 migration inventory, review export, apply, deletion, cleanup, or lifecycle mutation | No |
| `adopt_user_owned=true` or backup/replacement of user-owned files | No |
| Editing `/Users/yuval.meiri/.claude/settings.json` | No |
| Editing `/Users/yuval.meiri/.claude/engram-settings-snippet.json` | No |
| Editing root `AGENTS.md` or `/Users/yuval.meiri/AGENTS.engram.md` | No |
| Rewriting hook scripts, command files, or end-session adapters outside the manifest | No |
| Globs, recursive sync, inferred companion files, broad directory rewrites, package installs, PATH/env/auth changes, shell profile changes, restarts, or remote fetches | No |
| Schema, storage, index, public MCP, ranking, `orient`, graph, lint, telemetry formula, document-index behavior, or memory lifecycle changes | No |

## Validation Criteria

The approved repair succeeds only if:

- each harness gets a fresh `write=false` dry-run immediately before its `write=true` call;
- each fresh dry-run's planned writes match the manifest exactly for that harness;
- `adopt_user_owned=false` is used for every call;
- Claude Code uses `settings_target="settings.local.json"` and no other settings target;
- the Claude settings operation remains a merge into `settings.local.json`, not a replacement or
  edit to `settings.json`;
- no skipped file becomes a planned or written file;
- no unlisted path, symlink surprise, delete, chmod, backup/adoption, unlisted hook rewrite, or
  generated companion-file write appears;
- post-write `harness(action="doctor")` and `harness(action="status")` are run for each repaired
  harness;
- any remaining `ready=false` status or warning is reported instead of hidden;
- `obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")`
  is clean or explicitly resolved/skipped with evidence before final response.

## Stop Conditions

Stop and ask the user again before writing anything if:

- approval is missing, conditional, ambiguous, or changes the allowed scope;
- a fresh dry-run differs from the manifest by path, operation, count, warning, settings target, or
  skipped/planned classification;
- any path outside the manifest appears in `planned` or `written`;
- any user-owned file would be adopted, backed up, overwritten, or changed;
- `/Users/yuval.meiri/.claude/settings.json`,
  `/Users/yuval.meiri/.claude/engram-settings-snippet.json`, root `AGENTS.md`, or
  `/Users/yuval.meiri/AGENTS.engram.md` would be modified;
- a hook file outside the manifest, end-session adapter, command file, shell profile, package,
  PATH/env/auth file, or remote resource would need modification;
- the Claude settings merge would remove or replace existing user configuration instead of merging
  required Engram entries;
- filesystem metadata reveals an unexpected symlink, permission error, missing parent directory,
  or ownership/provenance mismatch;
- installer semantics or source behavior changed since this packet was prepared;
- post-write validation returns a new readiness failure that requires any excluded action.

## Exact Approval Wording

Approve T135: execute the refreshed harness repair sequence from
`docs/BRAIN_HARNESS_T135_REFRESHED_HARNESS_REPAIR_APPROVAL_PACKET_2026-06-02.md`: exactly five
harness install writes, one harness at a time after matching fresh dry-run, with
`adopt_user_owned=false` for all calls and Claude Code `settings_target=settings.local.json`; allow
only the listed generated adapter updates and the listed Claude `settings.local.json` merge; do not
edit `settings.json`, the user-owned Claude settings snippet, root `AGENTS.md`,
`/Users/yuval.meiri/AGENTS.engram.md`, unlisted hooks/commands, M6/migration/lifecycle/ranking/
`orient`/schema/storage/index/public MCP/document-index behavior, or use `adopt_user_owned=true`.
