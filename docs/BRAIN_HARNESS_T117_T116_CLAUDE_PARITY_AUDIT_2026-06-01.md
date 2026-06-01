# Brain Harness T117 T116 Claude Parity Audit

Status: Completed docs-only cross-harness read-path audit; no gated write run
Date: 2026-06-01
Scope: Verify whether Claude Code can recover the T116 current plan and gate boundary after the
approval-scope audit.

This slice did not run document indexing, document planning, reindex, cleanup, orphan recovery, M6
inspection, migration review export/apply, deletion, lifecycle mutation, `lint(action="apply_safe")`,
ranking changes, `orient` changes, public MCP changes, schema/storage/index behavior changes,
document-index behavior changes, MemoryItem creation for document packets, or harness adapter/hook
changes.

## Research Question

After T116, can Claude Code recover the active current-plan memory and pending approval gates
through Engram, and does that reduce any cross-harness completion-matrix risk without authorizing a
write?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Claude Bridge project harness still lacks Engram tool exposure, but the personal harness can recover the T116 current plan first; T70 remains unapproved. |
| Null | Claude Code cannot recover the T116 current plan through either harness, so cross-harness read-path parity remains unvalidated. |
| Simpler alternative | Skip Claude parity until the next exact write approval and rely only on Codex read-path evidence. |
| Failure | The parity probe is misread as approval for T70, T69, T111, M6, lifecycle cleanup, ranking, `orient`, or harness writes. |

## Measurement

Startup evidence was refreshed before writing this report:

- Lean `orient` trace `019e8499-2b43-7421-9d19-e6b24149b781` returned T116 current-plan
  memory `019e8494-b328-7fe1-9cf3-609fc9894f5e` first, with the T47 harness gate, M6 gate, and
  commit preference also present.
- Direct current-plan search trace `019e8499-b01b-7f42-af81-8b5526520eff` returned T116 current
  plan first and the T116 rolling handoff second.
- `git status --short` returned only the pre-existing user-owned untracked root `AGENTS.md`; latest
  commit was `c38c93d` (`Record T116 approval scope audit`).
- `obligations(action="doctor")` returned no open obligations.
- `telemetry(action="real_session_eval", project="engram", limit=50)` still failed the confidence
  gate. The latest run improved ordinary feedback coverage to `0.5`, but it still had feedback
  across only two intents and the gate requires at least three.

Claude Bridge parity evidence:

- Project-harness Claude Bridge read-only probe could not call Engram: both
  `mcp__engram__orient` and `mcp__engram__search` returned `No such tool available`. No trace IDs
  were produced. Treat this as bridge/project tool exposure drift, not as proof that native Claude
  Code cannot use Engram.
- Personal-harness Claude Bridge read-only probe succeeded. Orient trace
  `019e8497-685d-7461-9f27-e2b3218d3e09` returned T116 current-plan memory
  `019e8494-b328-7fe1-9cf3-609fc9894f5e` as the first returned memory ID. Search trace
  `019e8497-6bf0-7fd1-abdc-f7587a87e48c` returned the same T116 current plan first for
  `current plan after T116 approval scope audit next action T70 exact phrase`.
- The personal-harness result recovered the T70 gate only as current-plan context. It did not
  produce or imply approval, and it noted that the T70 exact phrase was only partially visible in
  the returned summary.

Remaining retrieval caveats:

- Fresh exact-phrase direct search trace `019e8499-f566-7652-8410-5ac68d22265d` for
  `Approve T70: index exact files T59, T68, and T69.` still ranked older T110/T109 handoffs above
  the T116 current plan. T116 current plan appeared fourth.
- Fresh document search for the same exact phrase returned older T64/T59/T58 material in the top
  five, not T70 or T116. Repo files and current-plan memory remain authoritative for recent gate
  evidence until exact indexing is approved.

## Completion Matrix Delta

| Area | State | Evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| Codex current-plan retrieval | Validated for this startup | `orient` and current-plan search returned T116 first | Older repository-scoped current-plan memory remains active/noisy. |
| Claude personal-harness read path | Partially validated | Personal Claude Bridge `orient` and `search` returned T116 current plan first | Read-path parity only; no write-path or hook readiness proof. |
| Claude project-harness read path | Missing | Project harness lacked Engram MCP tools | Requires harness/tool exposure repair under the existing T47 gate before treating project Bridge parity as ready. |
| Exact T70 phrase retrieval | Risky | Exact phrase search still ranked older handoffs above T116; document search did not recover T70/T116 | Do not run T70 unless the exact user phrase appears and repo docs are read. |
| Document index visibility | Risky | Document search still relies on older indexed packets | T70 exact-file indexing remains gated. |
| Real-session confidence | Not complete | Latest `limit=50` report fails intent-diversity gate | M6 write-apply remains blocked by evidence and approval gates. |

## Interpretation

T117 improves cross-harness evidence but does not close the harness completion risk. Claude
personal-harness read-path retrieval can recover the active T116 current plan first, while the
project harness still lacks Engram tool exposure. Exact T70 phrase retrieval remains noisy and the
document index is still stale for recent packets.

No pending gate is authorized. The next write gate remains the exact phrase
`Approve T70: index exact files T59, T68, and T69.` Generic approval text is still insufficient for
T70, T69, T111, M6, lifecycle cleanup, ranking, `orient`, public MCP/schema/storage/index behavior,
document-index behavior, or harness writes.
