# Brain Harness T166 T157 Lifecycle Archive Result

Date: 2026-06-03
Status: complete as exact-approved single-target lifecycle archive
Scope: Execute only the approved T157 archive for MemoryItem
`019e5e0a-86b4-73e3-aa9b-ca350e83e915`.

## Status

The user approved the exact T157 wording:

```text
Approve T157: after fresh matching read-only get/orient-or-search/lint/graph/git/obligations evidence and no intervening writes, archive exactly MemoryItem 019e5e0a-86b4-73e3-aa9b-ca350e83e915 with the archive payload in docs/BRAIN_HARNESS_T157_STALE_CURRENT_PLAN_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md. Do not run lint apply_safe, archive any other memory, change handoff semantics, ranking, orient, public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude Bridge, Claude hooks, harness installs/settings/hooks/adapters, or user-owned files.
```

This result records execution of only that approved lifecycle write. It did not run
`lint apply_safe`, archive any other MemoryItem, mutate handoff semantics, change ranking or
`orient`, change public MCP/schema/storage/index/document-index behavior, run M6/migration/
quarantine commands, run native Claude or Claude Bridge, edit hooks/settings/adapters, or touch
user-owned files.

Archive means preserved with archive lifecycle metadata. It is not deletion.

## Research Question

Can Engram safely retire the stale repository-scoped current-plan MemoryItem
`019e5e0a-86b4-73e3-aa9b-ca350e83e915` after exact approval, fresh matching evidence, and no
intervening writes?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | A single-target manual archive removes stale current-plan guidance from active retrieval without broad cleanup or hot-path changes. | Supported. Post-archive `orient` no longer returns the target, and lint no longer reports stale/wrong-scope findings for that ID. |
| Null | The item should remain active because fresh evidence no longer matches the packet. | Not supported. The target was unchanged, active, and still flagged stale/wrong-scope with `safe_action=none` immediately before archive. |
| Simpler alternative | Keep rejecting the item through telemetry only. | Rejected by exact user approval for this target. |
| Failure | The slice archives the wrong item, archives more than one item, applies lint broadly, or crosses another approval gate. | Not observed. |

## Fresh Pre-Write Evidence

The final read-only evidence batch was collected immediately before the archive call, with no
intervening writes:

| Check | Result |
| --- | --- |
| Target `memory(get)` | Target existed, was `status=active`, `kind=decision`, title unchanged, repository-scoped to `/Users/yuval.meiri/projects/engram`, tagged `current-plan`, and `updated_at=2026-05-25T07:30:08.716259Z`. |
| Current-plan evidence | Newer active current-plan memory `019e8d49-ef26-74f2-8ff9-3fefa079d7c2` was active and first in lean `orient`; older T156 memory `019e8d05-dce0-7a82-9b23-30ce1405b5bd` was already superseded. The user had supplied the exact T157 approval after T165 was the visible active plan. |
| Lean `orient` | Trace `019e8d57-bff6-7e11-bfe3-8594c13818f5` returned T165 first and the stale T157 target lower, proving current guidance remained recoverable while the target was still active. |
| `lint(run, write=false)` | Reported `feedback_stale_current_plan` for the target with 178 recent stale-feedback records and `feedback_wrong_scope_active_memory` with 32 recent wrong-scope records; both had `safe_action=none`. Counts drifted from the packet payload, but the packet required the target still be stale or wrong-scope with `safe_action=none`, not exact count equality. |
| `graph(around, depth=1)` | Showed evidence, repository scope, capture commit, and the target superseding older memory `019e59f2-524d-76f0-929a-7d2be0cea901`; no new direct dependent MemoryItem appeared. |
| `git status --short` | Only the known user-owned untracked root `AGENTS.md` was present. |
| `obligations(doctor)` | Returned `open=[]` and `warnings=[]`. |

## Archive Executed

The single approved write was:

```text
memory(
  action="archive",
  id="019e5e0a-86b4-73e3-aa9b-ca350e83e915",
  archive_reason="Stale repository-scoped current-plan guidance superseded by active T156 project-scoped current plan 019e8d05-dce0-7a82-9b23-30ce1405b5bd; read-only lint reported feedback_stale_current_plan with 198 recent stale-feedback records and feedback_wrong_scope_active_memory with 23 recent wrong-scope records, both with safe_action=none.",
  archived_by="codex"
)
```

The archive result set:

- `status=archived`
- `archive.archived_by=codex`
- `archive.archived_at=2026-06-03T11:56:54.434906Z`
- `updated_at=2026-06-03T11:56:54.434907Z`

The archive reason intentionally matches the approved packet payload.

## Validation

| Check | Result |
| --- | --- |
| `memory(get)` after archive | Target returned `status=archived` with the approved archive metadata. |
| Lean `orient` after archive | Trace `019e8d58-34df-7413-9a4f-3054a85fd84d` returned T165 first and no longer returned `019e5e0a...`. |
| Direct search after archive | Trace `019e8d58-352b-70d2-901c-dfa82ebf4a4b` returned T165 first and did not return the archived target in the top results; older active handoffs remained noisy lower-rank results. |
| `lint(run, write=false)` after archive | No stale-current-plan or wrong-scope finding for `019e5e0a...`; other lifecycle debt remained. |
| `memory(changes_since)` | Trace `019e8d58-35b0-7621-bfae-75e4c1dafd64` showed exactly the approved archive state change and no knowledge commits. |
| `obligations(doctor)` | Returned `open=[]` and `warnings=[]`. |
| Git status | Repository still showed only the known untracked root `AGENTS.md` before this report was written. |

Telemetry feedback was submitted for the startup, pre-write, post-archive, search, and
`changes_since` traces.

## Completion Matrix

| Area | Current State | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| T157 stale current-plan target | Archived | `memory(get)` shows `status=archived`; post-archive `orient` omits the target; lint no longer flags it | None for this target |
| Active current-plan retrieval | Healthy for current continuation | Lean `orient` and direct search return T165 first | Older active handoffs remain noisy in broad search |
| Lifecycle cleanup | Partially progressed | T157 target closed; T159 and T160 targets remain active | Exact T159/T160 approvals still required; no `lint apply_safe` |
| Native Claude/effective hooks | Missing / exact-gated | T154 packet remains default-deny | Exact T154 approval required before native Claude process |
| M6 migration/quarantine | Missing / high-risk gated | T158/T125 packet remains default-deny | Exact T125 approval required before quarantine reads; apply/deletion require later separate approval |
| Legacy substrate | Preserved | No migration apply, deletion, schema/storage/index change, or legacy simplification occurred | Simplification remains eval- and approval-gated |

## Decision

T157 is complete as a bounded manual lifecycle archive. It reduces active current-plan noise without
changing retrieval code, hot-path payloads, schema/storage/index behavior, document-index behavior,
harness state, M6 state, or user-owned files.

The next product-moving work remains separately exact-gated: T154 native Claude non-session smoke,
T159/T160 lifecycle archives, and T125/T158 M6 quarantine inspection.
