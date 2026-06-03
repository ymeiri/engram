# Brain Harness T167 T159 Lifecycle Archive Result

Date: 2026-06-03
Status: complete as exact-approved single-target lifecycle archive
Scope: Execute only the approved T159 archive for MemoryItem
`019e89f4-7dba-7ae1-a559-85d924af31a3`.

## Status

The user approved the exact T159 wording:

```text
Approve T159: after fresh matching read-only get/orient-or-search/T147-evidence/lint/graph/git/obligations evidence and no intervening writes, archive exactly MemoryItem 019e89f4-7dba-7ae1-a559-85d924af31a3 with the archive payload in docs/BRAIN_HARNESS_T159_STALE_T146_LIMITATION_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md. Do not run lint apply_safe, archive any other memory, change handoff semantics, ranking, orient, public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude Bridge, Claude hooks, harness installs/settings/hooks/adapters, or user-owned files.
```

This result records execution of only that approved lifecycle write. It did not run
`lint apply_safe`, archive any other MemoryItem, mutate handoff semantics, change ranking or
`orient`, change public MCP/schema/storage/index/document-index behavior, run M6/migration/
quarantine commands, run native Claude or Claude Bridge, edit hooks/settings/adapters, or touch
user-owned files.

Archive means preserved with archive lifecycle metadata. It is not deletion.

## Research Question

Can Engram safely retire stale T146 runtime-refresh limitation MemoryItem
`019e89f4-7dba-7ae1-a559-85d924af31a3` after exact approval, fresh matching evidence, and no
intervening writes?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | A single-target manual archive removes stale runtime-refresh limitation guidance contradicted by T147 installed-runtime validation without broad cleanup or hot-path changes. | Supported. Post-archive `orient` and targeted memory search no longer return the target as active guidance. |
| Null | The item should remain active because fresh evidence no longer matches the packet. | Not supported. The target was unchanged, active, and still contradicted by T147 evidence immediately before archive. |
| Simpler alternative | Leave the limitation active and rely on agents to remember T147 closed the runtime gap. | Rejected by exact user approval for this target. |
| Failure | The slice archives the wrong item, archives more than one item, applies lint broadly, or crosses another approval gate. | Not observed. |

## Fresh Pre-Write Evidence

The final read-only evidence batch was collected immediately before the archive call, with no
intervening writes:

| Check | Result |
| --- | --- |
| Target `memory(get)` | Target existed, was `status=active`, `kind=limitation`, title unchanged, project-scoped to `engram`, tagged `runtime-refresh-gate` and `t146`, and `updated_at=2026-06-02T20:09:22.10605Z`. |
| Current-plan evidence | Lean `orient` trace `019e8d69-06c7-7de2-bf1d-265307ddd7b8` returned current-plan MemoryItem `019e8d5c-9a5e-7c21-852b-381687e2e7a4` first. |
| T147 contradiction evidence | `docs/BRAIN_HARNESS_T147_T146_RUNTIME_REFRESH_VALIDATION_RESULT_2026-06-03.md` records installed binary hash `0cbbbc82a70f08b52f218369e4c304828037d3615c4bac71c35303957b423f22`, daemon PID `68053`, no-prompt trace `019e8bb8-ba85-7230-aede-84266c5721c6`, and empty-prompt trace `019e8bb8-bb3e-7af2-a765-fcbd5bbc4c50`; both live traces returned the active current-plan item first. |
| `lint(run, write=false)` | The target was not flagged. Other unrelated wrong-scope and superseded-active lifecycle findings remained. This confirms T159 was a human-approved manual archive, not a lint safe action. |
| `graph(around, depth=1)` | Showed only evidence edges, project scope, and writer-session edge; no direct dependent MemoryItem appeared. |
| `git status --short` | Only the known user-owned untracked root `AGENTS.md` was present. |
| `obligations(doctor)` | Returned `open=[]` and `warnings=[]`. |

## Archive Executed

The single approved write was:

```text
memory(
  action="archive",
  id="019e89f4-7dba-7ae1-a559-85d924af31a3",
  archive_reason="Stale T146 runtime-refresh limitation contradicted by T147 installed-runtime validation: after installing binary hash 0cbbbc82a70f08b52f218369e4c304828037d3615c4bac71c35303957b423f22 and restarting the daemon to PID 68053, live no-prompt and empty-prompt project-scoped plan_work orient traces returned the active current-plan item first. Read-only 2026-06-03 search/orient still surfaced this limitation as active guidance; graph depth 1 showed only evidence, project scope, and writer-session edges; lint did not flag this item, so this is a human-approved manual archive, not a lint safe action.",
  archived_by="codex"
)
```

The archive result set:

- `status=archived`
- `archive.archived_by=codex`
- `archive.archived_at=2026-06-03T12:15:36.156053Z`
- `updated_at=2026-06-03T12:15:36.156053Z`

The archive reason intentionally matches the approved packet payload.

## Validation

| Check | Result |
| --- | --- |
| `memory(get)` after archive | Target returned `status=archived` with the approved archive metadata. |
| Lean `orient` after archive | Trace `019e8d69-3e32-7520-9cfa-ab5fee6e245a` returned current-plan MemoryItem `019e8d5c-9a5e-7c21-852b-381687e2e7a4` first and did not return the archived target. |
| Direct search after archive | Trace `019e8d69-3e85-76e3-bb9c-79737a0f1cab` did not return the archived target in the top results; older active handoffs remained noisy. |
| `lint(run, write=false)` after archive | No finding for `019e89f4-7dba-7ae1-a559-85d924af31a3`; unrelated wrong-scope and superseded-active findings remained. |
| `memory(changes_since)` | Trace `019e8d6b-112d-7b20-98cf-a426f5ea8a3d` showed exactly one item change since the pre-archive cursor: this target moved to `status=archived`; no knowledge commits were returned. |
| `obligations(doctor)` | Returned `open=[]` and `warnings=[]`. |
| Git status | Repository still showed only the known untracked root `AGENTS.md` before this report was written. |

## Completion Matrix

| Area | Current State | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| T159 stale T146 limitation target | Archived | `memory(get)` shows `status=archived`; post-archive `orient` and targeted search omit the target | None for this target |
| Active current-plan retrieval | Healthy for current continuation | Lean `orient` returns `019e8d5c-9a5e-7c21-852b-381687e2e7a4` first | Older active handoffs remain noisy in broad search |
| Lifecycle cleanup | Partially progressed | T157 and T159 targets are archived | T160 remains exact-gated; no `lint apply_safe` |
| Native Claude/effective hooks | Missing / exact-gated | T154 packet remains default-deny | Exact T154 approval required before native Claude process |
| M6 migration/quarantine | Missing / high-risk gated | T158/T125 packet remains default-deny | Exact T125 approval required before quarantine reads; apply/deletion require later separate approval |
| Legacy substrate | Preserved | No migration apply, deletion, schema/storage/index change, or legacy simplification occurred | Simplification remains eval- and approval-gated |

## Decision

T159 is complete as a bounded manual lifecycle archive. It removes stale runtime-refresh limitation
guidance contradicted by T147 live installed-runtime validation without changing retrieval code,
hot-path payloads, schema/storage/index behavior, document-index behavior, harness state, M6 state,
or user-owned files.

The next product-moving work remains separately exact-gated: T154 native Claude non-session smoke,
T160 wrong-scope prompt-capture archive, and T125/T158 M6 quarantine inspection.
