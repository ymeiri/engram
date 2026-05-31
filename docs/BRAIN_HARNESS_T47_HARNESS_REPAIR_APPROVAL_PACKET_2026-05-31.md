# Brain Harness T47 Harness Repair Approval Packet

Status: Pending user approval. No harness write is authorized by this document.
Date: 2026-05-31
Scope: Proposal for exact local harness repair writes derived from read-only dry-runs

This packet is a request for approval, not approval itself. No adapter install, settings edit,
hook registration, migration action, lifecycle mutation, schema/storage/index change, public MCP
change, ranking change, or `orient` payload change has been run for T47.

## Research Question

Can Engram safely ask for explicit approval to repair local harness readiness using only the exact
paths and operations shown by read-only dry-runs, while preserving default-deny boundaries?

## Current Evidence

- T46 read-only `harness(action="doctor")` and `harness(action="status")` checks returned
  `ready=false` for the generic policy, Claude Code, Codex, Gemini CLI, and Cursor.
- Read-only `harness(action="install", write=false, ...)` calls for T47 produced the exact planned
  file effects listed below and no writes.
- Source inspection confirms `harness(action="install")` defaults `write` to `false`, skips
  user-owned files unless `adopt_user_owned=true`, and only merges Claude settings through the
  explicit `settings_target`.
- AI Council and Claude Bridge critique agreed that this can be a safe next slice only if the
  packet is explicitly pending approval, lists the complete allowlist, excludes broad globs and
  user-owned adoption, requires a final matching dry-run before execution, and stops on any drift.

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A docs-only approval packet can let the user approve or reject exact harness repair writes without granting broad local-configuration authority. |
| Null | The proposed repair scope is still too broad, ambiguous, or under-specified to approve safely. |
| Simpler alternative | Defer harness repair and continue only read-only validation, accepting that supported harnesses remain `ready=false`. |
| Failure | The packet implies approval, hides write risks, writes user-owned files, touches hooks/settings outside the allowlist, or creates pressure to bundle unrelated M6/lifecycle/hot-path work. |

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

Any missing, conditional, partial, or ambiguous approval remains default-deny.

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
| `claude_code` | `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` | `already installed` |
| `claude_code` | `/Users/yuval.meiri/.claude/engram-settings-snippet.json` | `skipped user-owned file without Engram marker` |
| `claude_code` | `/Users/yuval.meiri/AGENTS.engram.md` | `already installed` |

## In Scope After Explicit Approval

| Item | Allowed after explicit approval? | Notes |
| --- | --- | --- |
| Fresh `write=false` dry-run for each listed harness | Yes | Must match the manifest before that harness is written. |
| The five `write=true` calls shown above | Yes | Execute one harness at a time; no batching with other work. |
| `harness(action="status")` and `harness(action="doctor")` after each write | Yes | Read-only validation only. |
| Syntax or status checks for the touched generated files/settings | Yes | Only for the listed targets. |
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
| Rewriting hook scripts, command files, or end-session adapters currently reported as skipped | No |
| Globs, recursive sync, inferred companion files, broad directory rewrites, package installs, PATH/env/auth changes, shell profile changes, restarts, or remote fetches | No |
| Schema, storage, index, public MCP, ranking, `orient`, graph, lint, telemetry formula, or memory lifecycle changes | No |

## Validation Criteria

The approved repair succeeds only if:

- each harness gets a fresh `write=false` dry-run immediately before its `write=true` call;
- each fresh dry-run's planned writes match the manifest exactly for that harness;
- `adopt_user_owned=false` is used for every call;
- Claude Code uses `settings_target="settings.local.json"` and no other settings target;
- the Claude settings operation remains a merge into `settings.local.json`, not a replacement or
  edit to `settings.json`;
- no skipped file becomes a planned or written file;
- no unlisted path, symlink surprise, delete, chmod, backup/adoption, hook rewrite, or generated
  companion-file write appears;
- post-write `harness(action="doctor")` and `harness(action="status")` are run for the repaired
  harnesses;
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
- `/Users/yuval.meiri/.claude/settings.json`, `/Users/yuval.meiri/.claude/engram-settings-snippet.json`,
  root `AGENTS.md`, or `/Users/yuval.meiri/AGENTS.engram.md` would be modified;
- a hook file, end-session adapter, command file, shell profile, package, PATH/env/auth file, or
  remote resource would need modification;
- the Claude settings merge would remove or replace existing user configuration instead of merging
  required Engram entries;
- filesystem metadata reveals an unexpected symlink, permission error, missing parent directory,
  or ownership/provenance mismatch;
- installer semantics or source behavior changed since this packet was prepared;
- post-write validation returns a new readiness failure that requires any excluded action.

## Approval Question

Do you approve exactly the five harness `write=true` calls shown in this document, executed one
harness at a time after matching `write=false` dry-runs, with no M6 action, no user-owned adoption,
no `settings.json` edit, no snippet edit, no root `AGENTS.md` edit, no hook rewrite, no globs or
companion-file writes, and no schema/storage/index/public-MCP/ranking/`orient`/lifecycle changes?
