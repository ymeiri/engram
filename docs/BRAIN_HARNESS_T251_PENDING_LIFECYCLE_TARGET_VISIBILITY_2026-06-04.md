# Brain Harness T251 Pending Lifecycle Target Visibility

Date: 2026-06-04
Status: completed docs-only read-only lifecycle visibility follow-through. No lifecycle archive,
`lint apply_safe`, M6/migration/quarantine action, ranking/`orient`, public MCP,
schema/storage/index, document-index behavior, harness/runtime/native-Claude action, deletion,
rollback, force-kill, legacy simplification, or user-owned-file change was executed.

## Scope

T251 records fresh evidence that the pending default-deny lifecycle packet targets from T247 and
T248 remain active and visible after T250. It does not refresh either packet's approval wording and
does not execute either archive.

This slice updates only:

- `docs/BRAIN_HARNESS_ARCHITECTURE.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- this report

## Research Question

After T250, do the already-packeted stale lifecycle targets still appear as active MemoryItems in
ways that keep lifecycle cleanup incomplete, and can that be documented without mutating lifecycle
state or changing retrieval behavior?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | T247 and T248 remain real pending lifecycle work: their targets are still active, still visible, and still marked stale by feedback, so the completion matrix should keep lifecycle cleanup incomplete. | Supported. |
| Null | The targets were already archived, superseded, no longer visible, or no longer stale enough to matter. | Not supported by fresh read-only evidence. |
| Simpler alternative | Rely on T247/T248 packet docs only. | Rejected because live `orient`/search/lint evidence after T250 shows the targets still affect retrieval context. |
| Failure | This follow-through is mistaken for approval to archive, runs `lint apply_safe`, changes ranking/`orient`, or treats feedback alone as proof for a lifecycle write. | Avoided. |

## Fresh Evidence

- Lean `orient` trace `019e928c-d7c2-75f1-a6bc-066f6baa4193` returned current-plan MemoryItem
  `019e928b-c8bb-7273-a09c-a1febe170001` first and also returned the T248 target
  `019e01f2-0a87-7f73-9b0b-7f2443eac7bb` as a top item. Obligations were clean.
- Direct search trace `019e928d-0c80-7782-a490-1c2b7df31d13` returned current-plan MemoryItem
  `019e928b-c8bb-7273-a09c-a1febe170001` first, the T248 target
  `019e01f2-0a87-7f73-9b0b-7f2443eac7bb` in the top memory results, and the T247 target
  `019e8291-40aa-71a0-b16b-9ba7b6446cc6` in the top memory results.
- `memory(action="get")` confirmed T247 target `019e8291-40aa-71a0-b16b-9ba7b6446cc6` is still
  `status=active`, kind `custom observation`, scope `project:engram`, with title
  `Post-T76 rolling telemetry gate remains false`.
- `memory(action="get")` confirmed T248 target `019e01f2-0a87-7f73-9b0b-7f2443eac7bb` is still
  `status=active`, kind `decision`, scope `project:engram`, with title
  `Resume continuity probe uses active MemoryItems before ranking changes`.
- `graph(action="around", depth=1)` for both targets showed project scope and evidence edges. No
  new direct supersedes or dependent MemoryItem relation appeared at depth 1.
- Fresh `lint(action="run", write=false, limit=120)` reported feedback-stale findings with
  `safe_action=none` for:
  - `019e01f2-0a87-7f73-9b0b-7f2443eac7bb`, now marked stale by four recent feedback records;
  - `019e8291-40aa-71a0-b16b-9ba7b6446cc6`, now marked stale by nine recent feedback records.
- The same lint sample also surfaced already-packeted T234 target
  `019dd3fe-ec94-7122-af04-1f35b839387f` and global non-Engram lifecycle pressure. This remains
  sampled global evidence, not an exhaustive Engram lifecycle inventory.
- Git status showed only the known user-owned untracked root `AGENTS.md`.

## Completion Matrix Delta

| Area | State After T251 | Evidence | Remaining Gate |
| --- | --- | --- | --- |
| T247 target `019e8291...` | Active and visible | Fresh get/search/graph/lint | Exact T247 approval plus fresh no-intervening-write precheck before archive |
| T248 target `019e01f2...` | Active and visible; can appear in lean `orient` top items | Fresh orient/get/search/graph/lint | Exact T248 approval plus fresh no-intervening-write precheck before archive |
| Broad lifecycle cleanup | Incomplete | Fresh sampled lint still reports wrong-scope, superseded-active, and feedback-stale pressure | Exact-target review; no broad `lint apply_safe` from T251 |
| M6 migration | Unchanged | No M6 commands or workspace edits | Human dispositions or explicit deferral under T210/T250 |
| Hot path/ranking | Unchanged | No source/runtime change | Do not change ranking or `orient` to hide stale targets without a separate approved failure class |

## Decision

T251 does not create a new lifecycle packet. T247, T248, and T234 already cover their exact targets.
The useful update is narrower: the pending packet targets are still live retrieval pressure, so the
goal remains incomplete until those exact lifecycle writes are approved and executed, or explicitly
deferred with evidence.

`lint apply_safe` remains out of scope. Feedback-stale findings have `safe_action=none`, and the
global lint report also contains many unrelated safe-action findings that T251 must not apply.

## Validation

Validation for this docs-only slice:

- lean `orient` trace `019e928c-d7c2-75f1-a6bc-066f6baa4193`
- direct search trace `019e928d-0c80-7782-a490-1c2b7df31d13`
- `memory(action="get")` for T247 and T248 targets
- `graph(action="around", depth=1)` for T247 and T248 targets
- `lint(action="run", write=false, limit=120)`
- `git status --short`
- `git diff --check`
- exact document indexing for this report, `docs/BRAIN_HARNESS_ARCHITECTURE.md`, and
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- document-search visibility for T251
- `obligations(action="doctor", project="engram")`
- focused commit with only intended repo docs
