# T161 Duplicate T135 Completion Gate Audit

Date: 2026-06-03

## Status

Read-only/docs-only audit complete. The user supplied another approval for T135 harness repair, but
T135 was already executed and validated in T152. This audit does not perform harness writes,
`harness install`, native Claude execution, Claude Bridge execution, lifecycle mutation, M6 work,
ranking or `orient` changes, schema/storage/index changes, public MCP changes, document-index
changes, deletion, rollback, force-kill, old-binary reinstall, or user-owned-file edits.

## Research Question

After the late duplicate T135 approval, what is the current completion state and what approved work
remains available without crossing a pause gate?

## Hypotheses

Preferred hypothesis: T135 remains consumed by T152; fresh read-only evidence still shows harness
adapter readiness, while full Brain Harness completion remains blocked on separate exact approval
gates.

Null hypothesis: the duplicate T135 approval reopens the harness write scope or exposes remaining
T135 repair work.

Simpler alternative: stop after telling the user T135 is already complete.

Failure hypothesis: treating duplicate approval as fresh permission would rerun harness writes,
hide lifecycle debt, or conflate adapter readiness with native Claude behavior.

## Measurement

- Lean `orient` for the duplicate T135 prompt returned current T160 plan memory
  `019e8d20-e858-7840-8f11-a2885be661f8` first in trace
  `019e8d23-7a5b-7243-8059-93375796aa23`.
- Direct search for T135/T152 evidence returned current T160 plan memory first, then prior native
  Claude search parity evidence and T135/T152 handoff context, in trace
  `019e8d23-9f3f-7b10-9e31-6e4126fb6b0c`.
- Repo docs were re-read around T135, T152, T153/T154, T155, T156, T157, T158, T159, and T160.
- Git status before docs edits showed only unrelated untracked root `AGENTS.md`; recent HEAD was
  `98d38f5` (`Record T160 wrong-scope prompt packet`).
- Fresh `harness(status)` and `harness(doctor)` checks reported `ready=true` for generic, Codex,
  Gemini CLI, Cursor, and Claude Code.
- Claude Code status/doctor still warned about a user-owned settings snippet, extra legacy Engram
  permissions, split `settings.json`/`settings.local.json`, and soft lifecycle compliance.
- Read-only `lint(action="run", write=false)` still reported stale/wrong-scope active memory for
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915`, wrong-scope active memory for
  `019e7f52-4fc2-7f61-93b4-9a741aba966e`, and many superseded-active items. No write was applied.
- `telemetry(action="real_session_eval", project="engram", limit=50)` reported feedback coverage
  of 42%, below the 50% confidence-gate threshold; its recommendation kept M6 write-apply blocked.
- `obligations(action="doctor")` and dry-run `obligations(action="detect")` returned no open
  obligations before this document was created.

No AI Council or Claude Bridge consultation was run for this audit. The slice makes no new
architecture, ranking, migration, or data-model decision, and Claude Bridge/native Claude execution
remain separately gated by the current docs.

## Completion Matrix

| Area | State | Evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| T135 harness repair | Implemented and validated | T152 validation report; fresh T161 status/doctor `ready=true` for all five harnesses | Duplicate approval does not reopen writes |
| Generated harness adapters | Implemented and live-ready | Generic, Codex, Gemini CLI, Cursor, and Claude Code status/doctor all `ready=true` | Lifecycle compliance is soft and depends on agents following the contract |
| Claude Code effective behavior | Partially validated | Static T153/T156 evidence; generated hooks installed; SessionEnd default uses `nudge` | Native Claude behavior and effective `/hooks` behavior require exact T154 or later approval |
| Current-plan retrieval | Implemented and currently healthy for this prompt class | T161 orient and direct search returned the current T160 plan first | Stale lower-rank current-plan/lifecycle items still need exact lifecycle cleanup |
| Lifecycle cleanup | Missing / gated | Read-only lint still flags stale/wrong-scope/superseded active memory | T157, T159, and T160 exact approvals are required before target-local archives; no `lint apply_safe` |
| M6 migration/quarantine | Missing / high-risk gated | Telemetry confidence gate failed at 42% coverage; T158 packet preserves T125 quarantine-inspection gate | Exact T125 approval is required for remaining read-only quarantine inspection; write apply/deletion remain separate future gates |
| Cross-harness behavior | Partially validated | Prior Codex/native Claude smokes plus current adapter readiness | Broader Codex/Claude/Gemini/Cursor behavior is not fully proven by adapter status alone |
| Legacy substrate | Preserved | No deletion, merge, bypass, or schema/storage/index change in this slice | Legacy simplification remains gated by eval evidence and explicit user approval |
| User-owned files | Preserved | Root `AGENTS.md` remains untracked and untouched; Claude user-owned snippet not adopted | Any adoption/edit remains explicitly gated |

## Decision

T135 is complete and consumed by T152. The late duplicate T135 approval should be recorded as a
non-actionable duplicate, not used as permission to rerun harness writes.

The Brain Harness goal is closer than before T152 because generated adapter readiness is now live
across all five harnesses, but it is not complete. The remaining high-risk unknowns are native
Claude/effective-hook behavior, lifecycle cleanup of stale and wrong-scope active memory, M6
migration/quarantine completion or explicit deferral, and broader cross-harness behavioral evidence.

## Next Action

The next product-moving options remain exact approval-gated:

- T160 single-target archive for wrong-scope Claude prompt capture.
- T159 single-target archive for stale T146 runtime-refresh limitation.
- T157 single-target archive for stale repository-scoped current-plan guidance.
- T154 native Claude non-session smoke.
- T125 remaining M6 quarantine inspection from T158.

Without one of those exact approvals, continue only with read-only evidence refreshes, docs-only
audits, telemetry feedback, and obligation follow-through.
