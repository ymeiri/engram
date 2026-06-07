# T152 T135 Harness Repair Validation Result

Date: 2026-06-03

## Status

Approved T135 harness repair execution complete. The run installed only the five harness writes
authorized by `docs/BRAIN_HARNESS_T135_REFRESHED_HARNESS_REPAIR_APPROVAL_PACKET_2026-06-02.md`,
one harness at a time, after matching fresh dry-runs.

No user-owned adapter was adopted, no root `AGENTS.md` was edited, no
`/Users/yuval.meiri/.claude/settings.json` write was performed, no Claude settings snippet was
overwritten, and no M6, migration, quarantine, lifecycle cleanup, ranking, `orient`,
schema/storage/index, public MCP, or document-index behavior was changed.

## Research Question

After explicit user approval for T135, can the generated Brain Harness adapters and Claude Code hook
registration be repaired without drifting from the approved manifest or touching user-owned files?

## Hypotheses

Preferred hypothesis: fresh dry-runs match T135 exactly, the approved writes install generated
adapters and the Claude Code `settings.local.json` merge, and all five harnesses report
`ready=true`.

Null hypothesis: installed state already changed enough that one or more dry-runs no longer match
T135, so the repair must stop before writes.

Simpler alternative: repair only the Claude Code SessionEnd hook and defer the other adapters.

Failure hypothesis: execution would require adopting user-owned files, editing
`/Users/yuval.meiri/.claude/settings.json`, changing unapproved hook/settings paths, or hiding
post-write readiness warnings.

## Measurement

- Re-oriented with Engram before writing and recovered the active T135 current-plan gate.
- Re-read the T135 approval packet and inspected live harness install source paths before execution.
- Checked git status before writes; the repo had only unrelated untracked root `AGENTS.md`.
- For each harness, ran `harness(action="install", write=false, adopt_user_owned=false, ...)` and
  compared the planned/skipped/warning shape to T135 before running the matching write.
- Ran per-harness `harness(action="status")` and `harness(action="doctor")` after writes.
- Checked `/Users/yuval.meiri/.claude/settings.local.json` JSON validity, SessionEnd hook
  executable bit, and the installed SessionEnd default `write_policy // "nudge"` line.

## Execution Evidence

Fresh dry-runs matched T135:

| Harness | Approved writes applied | Approved skips preserved |
| --- | --- | --- |
| `generic` | Created `/Users/yuval.meiri/.engram/harness-policy.md` | None |
| `codex` | Updated Codex memory-session and resume-session generated skills | `/Users/yuval.meiri/AGENTS.engram.md` already installed |
| `gemini_cli` | Updated memory-session command, resume-session command, and global context | Gemini end-session command already installed |
| `cursor` | Updated Cursor memory-session and resume-session generated skills | Cursor end-session skill already installed |
| `claude_code` | Updated generated SessionEnd hook and merged `settings.local.json` | Existing commands/hooks, user-owned settings snippet, and `AGENTS.engram.md` preserved |

The Claude Code dry-run planned only:

- `/Users/yuval.meiri/.claude/hooks/engram-session-end.sh`
- `/Users/yuval.meiri/.claude/settings.local.json`

It skipped `/Users/yuval.meiri/.claude/engram-settings-snippet.json` as user-owned, and no
`/Users/yuval.meiri/.claude/settings.json` write appeared.

## Validation Evidence

All five harnesses now report `ready=true` in live status/doctor checks:

| Harness | Status | Doctor warnings |
| --- | --- | --- |
| `generic` | `ready=true` | Soft lifecycle compliance reminder |
| `codex` | `ready=true` | Soft lifecycle compliance reminder |
| `gemini_cli` | `ready=true` | Soft lifecycle compliance reminder |
| `cursor` | `ready=true` | Soft lifecycle compliance reminder |
| `claude_code` | `ready=true` | User-owned snippet preserved, extra legacy Engram permissions remain in Claude settings, settings are split across `settings.json` and `settings.local.json`, and lifecycle compliance remains soft |

Additional checks passed:

- `python3 -m json.tool /Users/yuval.meiri/.claude/settings.local.json`
- `test -x /Users/yuval.meiri/.claude/hooks/engram-session-end.sh`
- `rg` confirmed the installed SessionEnd hook default line.

The installed hook contains:

```text
WRITE_POLICY=$(printf '%s' "$INPUT" | jq -r '.write_policy // "nudge"')
```

## Decision

T135 is executed and the local generated harness adapter readiness gap is closed. The prior
`ready=false` evidence from T137/T150 is superseded by the T152 live status/doctor evidence for the
installed local harness state.

This does not prove full Claude Code behavioral parity. Claude Code status still warns that Engram
settings are split across `settings.json` and `settings.local.json`, and effective hook behavior
must be validated in Claude Code. Because running Claude Code can trigger lifecycle hooks, that
behavioral validation should be a separate exact slice.

## Completion Matrix

| Area | State | Evidence | Remaining Gate |
| --- | --- | --- | --- |
| Generated harness adapter install | Implemented | T152 approved writes and status checks | None for local generated adapter files |
| Claude Code SessionEnd hook default | Installed | Hook line uses `write_policy // "nudge"` | Native behavioral validation |
| Claude Code settings merge | Implemented | `settings.local.json` JSON valid; SessionEnd registered there | Verify effective Claude Code hooks |
| Codex/Gemini/Cursor/generic readiness | Validated | `harness status/doctor ready=true` | Ongoing soft lifecycle compliance |
| User-owned file preservation | Validated | Snippet and root `AGENTS.md` were not overwritten | None for T135 |
| Claude Bridge side-effect risk | Reduced but not fully validated | T135 repaired installed SessionEnd hook; no post-repair Claude run yet | Separate cross-harness/live validation |
| M6/migration/quarantine | Still review-gated | Existing gates unchanged | Separate explicit approval |
| Lifecycle cleanup | Still gated | No archive/supersede/apply-safe run | Separate exact lifecycle approval |

## Next Action

Prepare a separate read-only-first post-repair cross-harness validation slice before using Claude
Bridge for substantive Engram consultation again. The slice should verify effective Claude Code hook
configuration and SessionEnd no-prompt behavior while stopping before any unapproved lifecycle state
mutation.
