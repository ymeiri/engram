# Brain Harness T126 Harness Readiness Recheck

Date: 2026-06-02
Status: Completed
Scope: Read-only harness readiness evidence refresh

T126 rechecked harness readiness after T124 without attempting any harness repair. All checked
harness targets still report `ready=false`, so the completion matrix remains unchanged:
cross-harness behavior is partially validated, but harness readiness is not complete.

No `harness(action="install")`, adapter write, settings edit, hook registration, user-owned file
adoption, M6 action, lifecycle mutation, document indexing, ranking change, `orient` expansion,
public MCP change, schema/storage/index behavior change, or document-index behavior change was run.

## Research Question

After T124, has external harness configuration drift changed the read-only readiness state for the
generic policy, Claude Code, Codex, Gemini CLI, or Cursor enough to update the completion matrix?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | All checked harnesses remain `ready=false`; this only refreshes evidence and keeps T47 as the exact harness-write gate. |
| Null | One or more harnesses was externally repaired and now reports ready, creating a narrower follow-up validation slice. |
| Simpler alternative | Rely on T71/T106 and avoid another read-only recheck. |
| Failure | The recheck crosses into install, settings mutation, hook changes, migration, lifecycle, ranking, or `orient` changes. |

## Measurement

Read-only evidence collected on 2026-06-02:

- `orient(project="engram", intent="plan_work", response_shape="lean")` returned current-plan
  memory `019e8777-8eb5-73d3-a9c0-86074a06069f` first and preserved the M6 and harness-write
  approval gates as relevant context.
- `harness(action="doctor", root="/Users/yuval.meiri", write=false)` returned `ready=false` for
  the default generic policy.
- `harness(action="status", harness=..., root="/Users/yuval.meiri", write=false)` was run for
  `generic`, `claude_code`, `codex`, `gemini_cli`, and `cursor`.
- `git status --short` showed only the user-owned untracked root `AGENTS.md` before this report
  was written.

## Result

| Harness | Ready | Read-only finding |
| --- | --- | --- |
| `generic` | false | Required policy document is missing at `/Users/yuval.meiri/.engram/harness-policy.md`. |
| `claude_code` | false | Required generated command and hook files are installed, but Claude settings still lack required `SessionStart:startup|resume|compact` and `SessionEnd` registrations. The settings snippet is user-owned, and settings still contain extra legacy Engram permission entries outside the current contract. |
| `codex` | false | Required `codex-memory-session-skill` and `codex-resume-session-skill` are drifted; the project agents snippet is installed. |
| `gemini_cli` | false | Required `gemini-memory-session-command`, `gemini-resume-session-command`, and `gemini-global-context` are drifted; `gemini-end-session-command` is installed. |
| `cursor` | false | Required `cursor-memory-session-skill` and `cursor-resume-session-skill` are drifted; `cursor-end-session-skill` is installed. |

No status result reported missing MCP tools for this Codex session.

## Completion Matrix Delta

| Area | T126 state | Evidence |
| --- | --- | --- |
| Cross-harness behavior | Partially validated, not ready | All five checked harness targets returned `ready=false`. |
| Harness repair | Still gated | T47 remains the pending exact harness-write approval packet. |
| M6 migration | Still gated | T125 quarantine inspection and all status/prioritize/apply decisions remain separate explicit gates. |
| `orient` hot path | Unchanged | T126 used lean `orient` only for startup context; no payload or ranking change was made. |

## Validation

This is a docs-only evidence slice. Validation is limited to:

- direct read-only tool evidence from `orient` and `harness`;
- exact-source documentation updates in the Brain Harness architecture, research method, and Memory
  OS implementation plan;
- `git diff --check` before commit.

## Next Gate

The next M6 gate remains exact approval for T125:

`Approve T125: read-only inspect quarantine candidate files 0010-0011 from the written T68 M6 review-export snapshot; no review files, no status/prioritize/apply, no candidate decisions, and no writes except the inspection report.`

Harness repair remains a separate T47 approval packet. Generic approval does not authorize harness
writes, lifecycle changes, indexing, or M6 migration.
