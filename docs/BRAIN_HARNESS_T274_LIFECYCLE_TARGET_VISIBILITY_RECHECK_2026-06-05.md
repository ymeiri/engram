# Brain Harness T274 Lifecycle Target Visibility Recheck

Date: 2026-06-05
Status: completed docs-only read-only lifecycle target visibility recheck. No lifecycle archive,
`lint apply_safe`, M6/migration/quarantine action, canonical vault write, native Claude or bridge
execution, harness install/settings/hook/adapter edit, ranking/`orient`, public MCP,
schema/storage/index, document-index behavior, branch publication, deletion, rollback,
force-kill, legacy simplification, or user-owned-file change was executed.

## Scope

T274 refreshes the T251/T252 lifecycle evidence after T273. It answers whether the three pending
default-deny exact lifecycle packet targets still remain active and visible, and whether the user's
broad continue instruction changes the lifecycle approval boundary.

This slice updates only:

- `docs/BRAIN_HARNESS_ARCHITECTURE.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- this report

## Research Question

After T273, are T234/T247/T248 still concrete active lifecycle cleanup targets, and can the
completion matrix make that clear without mutating Memory OS lifecycle state?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | The three exact targets remain active and visible; lifecycle cleanup remains incomplete until exact packet execution or explicit deferral. | Supported. |
| Null | One or more targets were already archived, superseded, or no longer relevant enough to keep lifecycle cleanup open. | Not supported by fresh read-only evidence. |
| Simpler alternative | Rely on T251/T252 without another report. | Rejected because T273 changed current-plan state and the latest broad continue instruction can be replayed by future agents. |
| Failure | Treat T274 as archive approval, run broad `lint apply_safe`, or change ranking/`orient` to hide stale targets. | Avoided. |

## Measurement

Before editing docs, T274 required:

- fresh lean `orient`;
- direct exact-target lifecycle search;
- `memory(get)` for the T234, T247, and T248 target IDs;
- `graph(around)` for the same IDs;
- read-only `lint(action="run", write=false)`;
- recent telemetry feedback and rolling eval evidence;
- source confirmation for lint/global safe-action boundaries;
- clean git status except known user-owned root `AGENTS.md`.

## Fresh Evidence

Lean `orient` trace `019e9927-b247-7342-91dd-e4e92e96aa72` returned current-plan MemoryItem
`019e9922-fc4c-7e73-82e7-c6bd0b7a225b` first, followed by the harness write gate, M6 explicit
scope gate, commit preference, and document-lifecycle dogfood context. No open obligations were
returned.

Direct memory search trace `019e9927-e7b5-7ad1-814b-5cdda6a7faed` for pending lifecycle targets
returned the T273 current plan first and the M6 gate second. The stale targets still appeared in
the top memory results:

- T248 target `019e01f2-0a87-7f73-9b0b-7f2443eac7bb` at score `0.8000737`;
- T247 target `019e8291-40aa-71a0-b16b-9ba7b6446cc6` at score `0.74817896`;
- T234 target `019dd3fe-ec94-7122-af04-1f35b839387f` at score `0.7376737`.

`memory(action="get")` confirmed all three target IDs remain active:

| Packet | Target | Fresh status | Fresh title |
| --- | --- | --- | --- |
| T234 | `019dd3fe-ec94-7122-af04-1f35b839387f` | active | `Memory OS migration completion run finished` |
| T247 | `019e8291-40aa-71a0-b16b-9ba7b6446cc6` | active | `Post-T76 rolling telemetry gate remains false` |
| T248 | `019e01f2-0a87-7f73-9b0b-7f2443eac7bb` | active | `Resume continuity probe uses active MemoryItems before ranking changes` |

`graph(action="around", depth=1)` for all three targets showed project scope and evidence edges.
No direct supersedes or replacement MemoryItem edge appeared at depth 1. The T247 graph still
shows the known mismatched telemetry evidence label; target content should therefore be read from
`memory(get)` plus current telemetry/docs, not from that graph label.

Fresh `lint(action="run", write=false, limit=160)` generated findings at
`2026-06-05T19:00:03Z`. The visible sample was dominated by globally sorted
`superseded_item_still_active` safe-action findings and older open-obligation findings. The sample
did not provide a target-specific cleanup basis for T234/T247/T248.

Recent `telemetry(action="list_feedback", project="engram", limit=80)` still included feedback
marking T247 and T248 stale in current runs. For example, T273/T271-era feedback marked
`019e8291-40aa-71a0-b16b-9ba7b6446cc6` stale for current risk/design searches, and startup/search
feedback marked `019e01f2-0a87-7f73-9b0b-7f2443eac7bb` stale/historical for current planning.
T234 did not appear in the viewed recent feedback window, but it remains active and directly
visible, and current M6 docs still contradict its completion title.

Fresh `telemetry(action="real_session_eval", project="engram", limit=50)` at
`2026-06-05T19:00:12.153762Z` returned:

- `feedback_coverage=0.5799999833106995`;
- `distinct_intent_count=5`;
- `confidence_gate.passed=true`;
- `task_failure_count=0`;
- `bad_memory_used_count=0`;
- `wrong_scope_memory_count=0`;
- `missing_context_count=0`;
- `stale_memory_count=25`.

This supports the evidence loop as operationally healthy in the current sample, while also showing
that stale-memory pressure remains live. It is not proof that lifecycle cleanup is complete.

Source inspection reconfirmed the safety boundary:

- `engram-index/src/lint.rs` `LintService::run` loads all memory with
  `list_memory_items(None, None)`, sorts globally, and only then truncates by `limit`.
- `lint_feedback_stale_active_memory` emits review findings with no safe action.
- `lint_feedback_wrong_scope_active_memory` emits review findings with no safe action.
- `lint_superseded_active_items` is the path that adds `safe_action=archive_memory_item`.
- `LintService::apply_safe` iterates every returned `ArchiveMemoryItem` safe action and archives
  each still-active item.
- `MemoryService::archive_memory` archives exactly one requested item by ID, and
  `MemoryItem::with_archive` sets status to `Archived`, records archive metadata, and updates
  `updated_at`.

Git status before editing showed only the known user-owned untracked root `AGENTS.md`.

## Decision

Lifecycle cleanup remains incomplete and exact-target-gated.

The useful T274 update is not a new lifecycle packet. T234, T247, and T248 already define the
default-deny exact archive paths. The fresh evidence shows those targets remain active and visible,
while the current plan and M6 gate still outrank them for the tested lifecycle query.

Broad `lint apply_safe` remains the wrong operation for this class. It can archive unrelated
safe-action findings from the global sample and it is not the safe-action path for the
stale-feedback T247/T248 targets or the T234 human-approved stale migration-completion target.

The user's broad instruction to continue ordinary Engram project work still does not authorize
T234/T247/T248 archive writes. T252's approval-boundary decision remains current.

## Completion Matrix Delta

| Area | State After T274 | Evidence | Remaining Gate |
| --- | --- | --- | --- |
| T234 target `019dd3fe...` | Active and directly visible; still contradicted by current M6 gate docs | Fresh get/search/graph plus M6 matrix docs | Exact T234 approval and fresh pre-write checks, or explicit lifecycle deferral |
| T247 target `019e8291...` | Active, visible, and recently marked stale in feedback | Fresh get/search/graph/feedback plus current telemetry | Exact T247 approval and fresh pre-write checks, or explicit lifecycle deferral |
| T248 target `019e01f2...` | Active, visible, and recently marked stale in feedback | Fresh get/search/graph/feedback | Exact T248 approval and fresh pre-write checks, or explicit lifecycle deferral |
| Broad lifecycle cleanup | Still incomplete | Fresh global lint sample has unrelated safe-action findings and stale-memory telemetry pressure | Exact-target review; no broad `lint apply_safe` |
| Current-plan retrieval | Healthy for this slice | Lean `orient` and direct search returned T273 current plan first | Keep `orient` hot path unchanged |
| Telemetry evidence | Passing current 50-trace sample with stale-memory pressure | `real_session_eval` passed with 58% feedback coverage, five intents, and 25 stale marks | Continue scoring material traces; do not treat as completion proof |

## Validation

Validation for this docs-only slice:

- lean `orient` trace `019e9927-b247-7342-91dd-e4e92e96aa72`;
- direct memory search trace `019e9927-e7b5-7ad1-814b-5cdda6a7faed`;
- `memory(action="get")` for T234, T247, and T248 target IDs;
- `graph(action="around", depth=1)` for the three target IDs;
- `lint(action="run", write=false, limit=160)`;
- `telemetry(action="list_feedback", project="engram", limit=80)`;
- `telemetry(action="real_session_eval", project="engram", limit=50)`;
- source reads of `engram-index/src/lint.rs`, `engram-index/src/memory.rs`, and
  `engram-core/src/memory.rs`;
- `git status --short --branch`;
- `git diff --check`;
- exact document indexing for this report, `docs/BRAIN_HARNESS_ARCHITECTURE.md`, and
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`;
- document-search visibility for T274.
