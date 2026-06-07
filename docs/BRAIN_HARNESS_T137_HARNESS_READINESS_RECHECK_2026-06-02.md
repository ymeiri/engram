# T137 Harness Readiness Recheck

Date: 2026-06-02
Status: completed read-only audit
Scope: live harness status/doctor recheck after T136

No `harness(action="install")`, adapter write, hook edit, settings edit, binary install, daemon
restart, lifecycle archive/apply, migration action, schema/storage/index change, public MCP change,
ranking change, document-index change, or `orient` payload change was run for T137.

## Research Question

After T136, is there any non-gated evidence that changes the next product-moving step, or does
Engram still require exact T135 approval before installed harness repair?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Read-only status/doctor evidence will show the same installed-harness readiness gap recorded by T135, so the next product-moving step remains exact T135 approval. |
| Null | Installed harness state changed outside this repo work and T135 is stale or no longer necessary. |
| Simpler alternative | Do no live recheck and rely only on the T135 packet. |
| Failure | A read-only recheck is treated as approval for `harness install`, settings/hook edits, user-owned adoption, lifecycle cleanup, M6, or hot-path changes. |

## Measurement

T137 used only read-only evidence:

- lean startup `orient` trace `019e884a-9c05-7410-9d9e-37a7e48990ac`;
- current-plan direct search trace `019e884a-b3e8-7e82-a5a0-c71c5d9a4fa3`;
- architecture/search trace `019e884a-c998-7893-8738-f6759db76740`;
- design/search trace `019e884a-d691-7702-b4fd-8ddac020a4e9`;
- risk/search trace `019e884a-ee62-7f02-bbe4-75b1d9f1af1c`;
- `harness(action="status")` for `generic`, `codex`, `gemini_cli`, `cursor`, and
  `claude_code`;
- `harness(action="doctor")` for the same five harnesses;
- repo docs and git status/log.

T137 intentionally did not run `harness(action="install", write=false)`. T135 requires fresh
matching dry-runs immediately before approved writes; running them early would not satisfy that
pre-write invariant and could blur the approval boundary.

AI Council recall found the recent T136 default-deny insight and T135/T136 broadcast history. No
new broadcast was needed for this narrow status audit. Claude Bridge was not used because the
installed Claude `SessionEnd` hook remains the known drifted surface until T135 is approved and
executed.

## Findings

### 1. Current Plan Recovery Still Works

Lean `orient` returned the active T136 current-plan memory
`019e8849-3d92-7dc2-9dfa-2606ccb0949e`, which says T136 is complete and T135 remains the next exact
gate. Direct current-plan search returned the same memory first.

Risk and design searches still show stale active rolling handoff noise. That matches T136 and does
not change the next action by itself.

### 2. Harness Readiness Is Still False Everywhere

| Harness | Readiness | Read-only evidence |
| --- | --- | --- |
| `generic` | `ready=false` | Required adapter `/Users/yuval.meiri/.engram/harness-policy.md` is missing. |
| `codex` | `ready=false` | `engram-memory-session` and `engram-resume-session` Codex skills are drifted. |
| `gemini_cli` | `ready=false` | Memory-session command, resume-session command, and global `GEMINI.md` are drifted. |
| `cursor` | `ready=false` | Memory-session and resume-session Cursor skills are drifted. |
| `claude_code` | `ready=false` | `SessionEnd` hook is drifted; SessionStart and SessionEnd settings registrations are missing; user-owned settings snippet is skipped. |

All five doctor calls also reported `ready=false` and the generic warning that the harness is not
fully installed. No doctor reported missing Engram MCP tools.

### 3. T135 Remains The Correct Gate

The live readiness evidence still matches the T135 packet's shape:

- the generic policy adapter needs creation;
- Codex, Gemini CLI, and Cursor generated adapters need updates;
- Claude Code still needs the generated `SessionEnd` hook update and settings-local merge;
- user-owned Claude settings snippet remains user-owned and must not be adopted without separate
  approval;
- Claude settings still contain legacy extra permissions that are not part of the current contract,
  but T135 does not authorize cleanup of those existing entries.

Because T137 did not run install dry-runs, it does not refresh or replace T135's planned write
manifest. If T135 is approved later, each harness still needs a fresh matching
`harness(action="install", write=false, adopt_user_owned=false, ...)` dry-run immediately before
its corresponding write.

## Completion Matrix Delta

| Area | T137 status | Evidence |
| --- | --- | --- |
| Current-plan continuity | Validated for this startup | Lean `orient` and direct current-plan search returned T136 current-plan memory first. |
| Installed harness readiness | Missing, gated | All five live status/doctor checks returned `ready=false`. |
| T135 approval packet relevance | Still relevant, not refreshed | Read-only status/doctor evidence matches the broad T135 readiness gap; no install dry-run was run. |
| Cross-harness production readiness | Not ready | Generic, Codex, Gemini CLI, Cursor, and Claude Code all need approved repair. |
| Lifecycle hygiene | Still missing, gated | T136 stale active handoff evidence remains; T137 did not run lint apply/archive. |
| M6 migration completion | Still gated | No migration action was run. |

## Decision

T137 does not justify source changes, lifecycle cleanup, ranking changes, `orient` changes, or
migration work. The live product is still not production-ready across installed harnesses.

The next product-moving step remains exact T135 approval. Without that approval, work should stay
read-only/docs-only or choose another explicitly non-gated validation slice.

## Stop Conditions For Follow-Up

Stop and ask before any follow-up that would:

- run `harness(action="install")` with either `write=false` or `write=true` as part of a repair
  sequence;
- write, edit, replace, adopt, chmod, back up, or delete any installed hook, settings file, adapter,
  command, skill, or project instruction;
- use `adopt_user_owned=true`;
- edit `/Users/yuval.meiri/.claude/settings.json`,
  `/Users/yuval.meiri/.claude/engram-settings-snippet.json`, root `AGENTS.md`, or
  `/Users/yuval.meiri/AGENTS.engram.md`;
- run lifecycle archive/apply, change `handoff(update)` semantics, change ranking or `orient`,
  change public MCP/schema/storage/index/document-index behavior, or run M6 migration actions;
- treat this read-only audit as approval for T135.
