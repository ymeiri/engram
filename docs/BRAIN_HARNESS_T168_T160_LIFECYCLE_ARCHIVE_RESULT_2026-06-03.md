# Brain Harness T168 T160 Lifecycle Archive Result

Date: 2026-06-03
Status: complete as exact-approved single-target lifecycle archive
Scope: Execute only the approved T160 archive for MemoryItem
`019e7f52-4fc2-7f61-93b4-9a741aba966e`.

## Status

The user approved the exact T160 wording:

```text
Approve T160: after fresh matching read-only get/orient-or-search/target-visibility/lint/graph/git/obligations evidence and no intervening writes, archive exactly MemoryItem 019e7f52-4fc2-7f61-93b4-9a741aba966e with the archive payload in docs/BRAIN_HARNESS_T160_WRONG_SCOPE_CLAUDE_PROMPT_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md. Do not run lint apply_safe, archive any other memory, change handoff semantics, ranking, orient, public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude Bridge, Claude hooks, harness installs/settings/hooks/adapters, or user-owned files.
```

This result records execution of only that approved lifecycle write. It did not run
`lint apply_safe`, archive any other MemoryItem, change handoff semantics, change ranking or
`orient`, change public MCP/schema/storage/index/document-index behavior, run M6/migration/
quarantine actions, run native Claude or Claude Bridge, edit hooks/settings/adapters, or touch
user-owned files.

Archive means preserved with archive lifecycle metadata. It is not deletion.

## Research Question

Can Engram safely retire wrong-scope Claude Code prompt-capture MemoryItem
`019e7f52-4fc2-7f61-93b4-9a741aba966e` after exact approval, fresh matching evidence, and no
intervening writes?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | A single-target manual archive removes one-time prompt-capture guidance without broad cleanup or hot-path changes. | Supported. Post-archive search and lint no longer return the target as active guidance/finding. |
| Null | The target should remain active because fresh evidence no longer matches the packet. | Not supported. The target was unchanged, active, exact-query visible, and lint-flagged immediately before archive. |
| Simpler alternative | Leave the wrong-scope prompt capture active and rely on agents to reject it. | Rejected by exact user approval for this target. |
| Failure | The slice archives the wrong item, archives more than one item, applies lint broadly, or crosses another approval gate. | Not observed. |

## Fresh Pre-Write Evidence

The final read-only evidence batch was collected immediately before the archive call, with no
intervening writes:

| Check | Result |
| --- | --- |
| Target `memory(get)` | Target existed, was `status=active`, `kind=rule`, title `Claude Code user-stated instruction`, project-scoped to `engram`, tagged `claude-code`, `hook-event`, and `user-stated`, and `updated_at=2026-05-31T18:36:01.346882Z`. |
| Current-plan evidence | Lean `orient` trace `019e8d85-d20d-7ba2-8fad-9d8caf2f1619` returned current-plan MemoryItem `019e8d6d-3b32-7681-82da-1af0cf5b89b0` first. |
| Target visibility | Exact prompt-class search trace `019e8d85-d25e-79d3-a7a5-430b021a0bdc` returned the target first as active `rule` guidance. |
| `lint(run, write=false)` | Reported `feedback_wrong_scope_active_memory` for the target with 9 recent wrong-scope feedback records and `safe_action=none`; no safe action was applied. |
| `graph(around, depth=1)` | Showed only manual-review prompt evidence and project scope; no direct dependent MemoryItem appeared. |
| `git status --short --branch` | Only the known user-owned untracked root `AGENTS.md` was present. |
| `obligations(doctor)` | Returned `open=[]` and `warnings=[]`. |

The lint count increased from the packet's original 4 wrong-scope feedback records to 9, which
strengthened the wrong-scope evidence. The archive payload intentionally used the exact approved
packet text.

## Archive Executed

The single approved write was:

```text
memory(
  action="archive",
  id="019e7f52-4fc2-7f61-93b4-9a741aba966e",
  archive_reason="Active Claude Code prompt capture from 2026-05-31 telemetry evidence-loop work is wrong-scope durable guidance: it is a one-time critique request about real_session_eval_report_scoped/list_feedback_scoped, lint reported feedback_wrong_scope_active_memory with 4 recent wrong-scope records and safe_action=none, exact search still returned it as active guidance, and graph depth 1 showed only manual-review prompt evidence and project scope. Human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)
```

The archive result set:

- `status=archived`
- `archive.archived_by=codex`
- `archive.archived_at=2026-06-03T12:47:03.446401Z`
- `updated_at=2026-06-03T12:47:03.446401Z`

## Validation

| Check | Result |
| --- | --- |
| `memory(get)` after archive | Target returned `status=archived` with the approved archive metadata. |
| Lean `orient` after archive | Trace `019e8d86-34c5-74c2-948b-e29589eaea29` returned current-plan MemoryItem `019e8d6d-3b32-7681-82da-1af0cf5b89b0` first and did not return the archived target. |
| Direct search after archive | Trace `019e8d86-3516-7d73-828d-2e6ae2d6c5a0` did not return the archived target in the top results. |
| `lint(run, write=false)` after archive | No finding for `019e7f52-4fc2-7f61-93b4-9a741aba966e`; unrelated wrong-scope and superseded-active findings remained. |
| `memory(changes_since)` | Trace `019e8d86-3594-7c50-ba66-1f16467b7dde` showed exactly one item change since the pre-archive cursor: this target moved to `status=archived`; no knowledge commits were returned. |
| `obligations(doctor)` | Returned `open=[]` and `warnings=[]`. |
| Git status | Repository still showed only the known untracked root `AGENTS.md` before this report was written. |

## Completion Matrix

| Area | Current State | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| T160 wrong-scope prompt-capture target | Archived | `memory(get)` shows `status=archived`; post-archive search and lint omit the target | None for this target |
| Active current-plan retrieval | Healthy for current continuation | Lean `orient` returns `019e8d6d-3b32-7681-82da-1af0cf5b89b0` first | Older active handoffs remain noisy in broad search |
| Lifecycle cleanup | Partially progressed | T157, T159, and T160 targets are archived | Broad superseded-active cleanup remains out of scope; no `lint apply_safe` |
| Native Claude/effective hooks | Missing / exact-gated | T154 packet remains default-deny | Exact T154 approval required before native Claude process |
| M6 migration/quarantine | Missing / high-risk gated | T158/T125 packet remains default-deny | Exact T125 approval required before quarantine reads; apply/deletion require later separate approval |
| Legacy substrate | Preserved | No migration apply, deletion, schema/storage/index change, or legacy simplification occurred | Simplification remains eval- and approval-gated |

## Decision

T160 is complete as a bounded manual lifecycle archive. It removes one wrong-scope Claude Code
prompt capture from active guidance without changing retrieval code, hot-path payloads,
schema/storage/index behavior, document-index behavior, handoff semantics, harness state, M6 state,
or user-owned files.

The next product-moving work remains separately exact-gated: T154 native Claude non-session smoke
and T125/T158 M6 quarantine inspection. Broad `lint apply_safe` and superseded-active cleanup remain
out of scope.
