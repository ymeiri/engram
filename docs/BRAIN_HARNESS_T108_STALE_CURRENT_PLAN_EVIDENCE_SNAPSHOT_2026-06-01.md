# Brain Harness T108 Stale Current-Plan Evidence Snapshot

Date: 2026-06-01

## Status

T108 is a read-only evidence snapshot for one stale current-plan MemoryItem:
`019e5e0a-86b4-73e3-aa9b-ca350e83e915`.

This snapshot does not authorize or perform archive, apply, deletion, scope correction, lifecycle
cleanup, `lint(action="apply_safe")`, migration, document indexing, harness writes, ranking
changes, `orient` changes, public MCP changes, schema/storage/index behavior changes, or
document-index behavior changes.

All lifecycle gates remain closed. `safe_action=none` is treated as a hard default-deny signal for
this cycle, not as an invitation to act.

## Research Question

After T107 made broad direct `search` next-step prompts return the active project current plan
first, does the repository-scoped current-plan MemoryItem
`019e5e0a-86b4-73e3-aa9b-ca350e83e915` still create enough stale/noisy retrieval pressure to
justify freezing an exact future decision gate?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | `019e5e0a-86b4-73e3-aa9b-ca350e83e915` remains active stale repository-scoped current-plan guidance. It should be documented as one exact target for future user review, with no lifecycle write in this slice. |
| Null | T107 sufficiently reduced stale current-plan noise, so no further target-specific evidence snapshot is needed. |
| Simpler alternative | Rely on existing T48/T52/T73 stale-current-plan notes and avoid another document. |
| Failure | The snapshot is misread as proxy authorization, bundles other lifecycle targets, or implies ranking/`orient` work instead of recording the current evidence. |

## Fresh Evidence

Evidence collection time marker: `2026-06-01T17:57:12Z` UTC.

- Lean `orient` trace `019e8451-e7ce-7171-8c4a-93b80b939e9e` returned current-plan memory
  `019e844f-b038-7f50-b2fc-635771b15a06` first and still included stale repository-scoped
  current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` as a lower-ranked active decision.
- Direct search trace `019e8452-305f-7352-a5dc-3b71a6d71f7a` for
  `current plan next step after T107 broad next-step search calibration Engram Brain Harness`
  returned T107 current-plan memory first and `019e5e0a-86b4-73e3-aa9b-ca350e83e915` second.
- Direct search trace `019e8455-24f4-7c43-8e83-1a200483cf6a` for
  `current plan after T107 stale repository-scoped 019e5e0a` returned T107 current-plan memory
  first and `019e5e0a-86b4-73e3-aa9b-ca350e83e915` second.
- `memory(action="get", id="019e5e0a-86b4-73e3-aa9b-ca350e83e915")` showed the target is still
  active, has kind `decision`, has tag `current-plan`, is scoped to repository
  `/Users/yuval.meiri/projects/engram`, and describes the older Codex document lifecycle
  follow-through plan from 2026-05-25.
- `memory(action="list", status_filter="active", tags=["current-plan"], limit=20)` returned three
  active current-plan tagged items: current project plan
  `019e844f-b038-7f50-b2fc-635771b15a06`, stale repository-scoped target
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915`, and an unrelated `voice-layer` project item.
- `lint(action="run", limit=5)` at `2026-06-01T17:57:13.141028Z` returned
  `feedback-stale-current-plan:019e5e0a-86b4-73e3-aa9b-ca350e83e915` first:
  237 recent stale-feedback records, `safe_action=none`.
- The same lint run also returned
  `feedback-wrong-scope-active-memory:019e5e0a-86b4-73e3-aa9b-ca350e83e915`:
  one recent wrong-scope feedback record, `safe_action=none`.
- `memory(action="changes_since", timestamp="2026-06-01T17:53:40.626429Z")` trace
  `019e8452-e93a-7af1-96e1-6d93945a8bdd` returned zero newer memory items and zero newer commits
  since the startup cursor.
- `handoff(action="get", project="engram")` returned active handoff
  `019e844f-d5cc-7013-a1c7-3ade7351fe94`, which points at T107 current-plan memory and says exact
  gates remain required.
- `telemetry(action="real_session_eval", project="engram", limit=50)` still failed the confidence
  gate: feedback coverage was 34%, feedback covered only one intent, stale-memory judgments remain
  present, and `bad_memory_used_count=0`.
- Git status remained clean except untracked user-owned root `AGENTS.md`.

## AI Review

AI Council recall found prior T48/T52 guidance: a docs-only stale-current-plan packet is acceptable
only if it is pending/default-deny, targets exactly one item, uses fresh get/list/lint/orient
evidence, preserves the active current plan, and runs no lifecycle write.

AI Council broadcast for T108 agreed 3/3 that the next non-gated slice should freeze the exact
target rather than perform broader evidence work, provided no archive/apply/write occurs.

Claude Bridge agreed documentation-only is acceptable but flagged a framing risk: calling the
artifact an approval packet can sound like proxy authorization when lint reports `safe_action=none`.
This document therefore uses "evidence snapshot" language and treats `safe_action=none` as
default-deny evidence for this cycle.

## Completion Matrix Delta

| Area | T108 state | Evidence |
| --- | --- | --- |
| Current-plan retrieval | Partially validated | T107 current-plan memory ranks first in `orient` and direct search for tested prompts. |
| Stale repository current-plan | Risky/noisy | `019e5e0a-86b4-73e3-aa9b-ca350e83e915` remains active and ranks second in tested current-plan searches. |
| Lifecycle safety | Gated/default-deny | Lint reports 237 stale-feedback records but `safe_action=none`; no automatic lifecycle action is safe. |
| Evidence quality | Improved | The exact stale current-plan target is frozen with fresh get/list/search/orient/lint evidence. |
| `orient` hot path | Unchanged | No `orient` code, payload, or ranking change. |
| Search ranking | Unchanged | No additional ranking change beyond committed T107. |
| M6 migration | Still gated | T69 remains required before inspecting the two review-export files. |
| Document indexing | Still gated | T70 exact-file indexing remains pending. |
| Harness readiness | Still gated/risky | T47 exact harness-write repair remains pending. |

## Hard Boundaries

- This snapshot targets only `019e5e0a-86b4-73e3-aa9b-ca350e83e915`.
- Do not archive, apply, delete, scope-correct, or otherwise mutate this MemoryItem from T108.
- Do not run `lint(action="apply_safe")` from T108.
- Do not bundle stale handoffs, other current-plan items, migration candidates, document indexing,
  or harness writes with this target.
- Do not hide this item through ranking, `orient`, public MCP, schema, storage, or index changes.
- Treat stale-feedback count as evidence for review pressure, not proof that an agent may act.

## Future Gate

If the user wants to act on this exact target later, require fresh get/list/lint/orient evidence
immediately before any write. If the evidence still matches this snapshot, the exact future phrase
would be:

`Approve T108: archive stale current-plan memory 019e5e0a-86b4-73e3-aa9b-ca350e83e915 only.`

That phrase would not authorize any other lifecycle action, M6 work, document indexing, harness
write, ranking change, `orient` change, public MCP change, schema/storage/index behavior change, or
document-index behavior change.
