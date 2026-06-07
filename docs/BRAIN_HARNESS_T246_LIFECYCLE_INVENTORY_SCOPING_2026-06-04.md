# Brain Harness T246 Lifecycle Inventory Scoping

Date: 2026-06-04
Status: completed docs-only read-only lifecycle inventory scoping. No lifecycle archive,
`lint apply_safe`, migration, M6/quarantine action, harness write, native Claude action, runtime
refresh, ranking, `orient`, public MCP, schema/storage/index, document-index behavior, deletion,
rollback, force-kill, legacy simplification, or user-owned-file change was executed.

## Scope

T246 follows T245's caveat that sampled global lint output is not an exhaustive Engram-scoped
lifecycle inventory. This slice asks whether the current read-only tools can identify remaining
Engram-scoped lifecycle candidates without mutating memory state.

This slice updates only:

- `docs/BRAIN_HARNESS_ARCHITECTURE.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- this report

## Research Question

Can Engram identify remaining Engram-scoped lifecycle candidates read-only, without relying on
global lint ordering or mutating lifecycle state?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Read-only evidence can identify at least one concrete Engram-scoped candidate, but cannot prove exhaustive lifecycle cleanup. | Supported. |
| Null | Current tools cannot distinguish Engram-scoped lifecycle pressure from global lint pressure at all. | Rejected for exact targeted candidates. |
| Simpler alternative | Leave T245 caveat as-is and do no further scoping. | Rejected because a known stale-feedback candidate is visible with exact evidence. |
| Failure | Treat a global lint sample or one candidate as proof that Engram lifecycle cleanup is complete, or as approval for archive/apply actions. | Avoided. |

## Measurement

Before documenting any candidate, the slice required:

- source confirmation of whether `lint` is scoped or global;
- fresh read-only `lint(action="run", write=false, limit=30)`;
- project-scoped active telemetry memory listing;
- exact `search`, `memory(get)`, and `graph(around)` evidence for any named candidate;
- AI Council and Claude Bridge critique for overclaim and gate risks;
- no lifecycle mutation.

## Source Evidence

`engram-index/src/lint.rs` shows `LintService::run` loads all MemoryItems with
`list_memory_items(None, None)`, runs all lint rules, sorts by global priority, and only then
truncates findings by `options.limit`. `lint_feedback_flagged_active_memory` reads the most recent
500 telemetry feedback rows and emits stale or wrong-scope findings only for active MemoryItems.
Feedback-stale and wrong-scope findings intentionally have `safe_action=none`; only
`superseded-active` findings add `safe_action=archive_memory_item`.

`engram-mcp/src/tools.rs` supports project-scoped filtering for `memory(action="list")`, but that
scoping is separate from `lint(action="run")`. Therefore, current lint output is useful lifecycle
pressure, not a scoped Engram inventory.

## Read-Only Findings

Fresh `lint(action="run", write=false, limit=30)` returned three leading wrong-scope active-memory
findings for non-Engram `dd-source` session-insight items, followed by globally sorted
`superseded-active` findings. This confirms that absence from a small global lint sample is not
evidence of absence of Engram-scoped lifecycle debt, and that lint priority order is not
Engram-scoped severity order.

Fresh project-scoped `memory(action="list", scope_type="project", project_name="engram",
status_filter="active", tags=["telemetry"], limit=30)` returned 9 active telemetry-tagged
Engram items. This is a focused telemetry-memory list, not an all-lifecycle inventory.

Exact read-only evidence identified one unranked candidate for future exact-target lifecycle
review:

- MemoryItem `019e8291-40aa-71a0-b16b-9ba7b6446cc6`
- Title: `Post-T76 rolling telemetry gate remains false`
- Scope: `project:engram`
- Status: `active`
- Tags: `telemetry`, `t76`, `confidence-gate`, `brain-harness`
- `memory(get)` content records a 2026-06-01 T76 point-in-time failing gate caused by feedback
  spanning only two intents.
- Recent feedback rows include repeated stale marks for this exact item, including T243 and T244
  follow-through feedback.
- T244 later recorded `telemetry(action="real_session_eval", project="engram", limit=50)` at
  `2026-06-04T11:14:07.108605Z` with `feedback_coverage=0.5199999809265137`,
  `confidence_gate.passed=true`, `task_failure_count=0`, `bad_memory_used_count=0`,
  `wrong_scope_memory_count=0`, and `missing_context_count=0`.
- `graph(action="around", node="019e8291-40aa-71a0-b16b-9ba7b6446cc6", depth=1)` showed the item
  scoped to `project:engram` with two evidence edges and no supersedes edge in the depth-1 graph.

The graph evidence node for the telemetry tool call displayed a mismatched older label, so the
candidate content should be read from `memory(get)` and the later contradiction from T244 docs and
feedback rows, not from that graph label.

## AI Review

AI Council recall found prior T48/T139 lifecycle guidance: lifecycle packets should be
default-deny, exact-target, fresh-evidence-gated, and must not bundle M6, harness, ranking,
`orient`, schema/storage/index, or broader cleanup.

AI Council broadcast and Claude Bridge critique agreed on these corrections:

- Do not call T246 an exhaustive Engram lifecycle inventory.
- Attribute the candidate discovery to project-scoped list plus exact search/get/graph, not to
  scoped lint.
- Treat `019e8291...` as one unranked candidate, not the highest-priority or only Engram debt.
- Do not conflate stale feedback, obsolete content, and lifecycle disposition.
- Require fresh re-verification before any future archive packet.
- Make the no-mutation claim auditable by naming the read-only operations used.

## Result

T246 narrows the lifecycle gate by identifying one concrete, Engram-scoped, active MemoryItem whose
T76 point-in-time telemetry claim now appears stale relative to later T244 telemetry evidence and
recent feedback marks. It does not prove that all Engram-scoped lifecycle debt has been
inventoried, does not rank this candidate against other lifecycle work, and does not authorize any
archive or `lint apply_safe` operation.

Any future exact-target lifecycle packet for `019e8291-40aa-71a0-b16b-9ba7b6446cc6` must first
rerun fresh read-only evidence with no intervening writes:

- `memory(get)` confirms the item is still active and has the same title, scope, tags, content, and
  status.
- `graph(around)` confirms current supersession/lifecycle relationships.
- current telemetry or feedback evidence still names the item as stale, wrong-scope, or obsolete.
- current project `orient` still surfaces a healthy current plan and no higher-priority blocker is
  bundled into the packet.
- user-visible packet text remains default-deny and exact-target.

## Completion-Matrix Effect

Lifecycle cleanup remains incomplete. The closed T157/T159/T160 exact targets stay closed, and
T246 adds one known candidate for future exact-target review: `019e8291-40aa-71a0-b16b-9ba7b6446cc6`.
Because lint is global and the focused telemetry list is not all lifecycle memory, Engram still
lacks an exhaustive scoped lifecycle inventory.

## Validation

Validation for this docs-only slice:

- source reading of `engram-index/src/lint.rs` and `engram-mcp/src/tools.rs`
- `lint(action="run", write=false, limit=30)`
- project-scoped active telemetry memory listing
- exact `search`, `memory(get)`, and `graph(around)` for `019e8291-40aa-71a0-b16b-9ba7b6446cc6`
- T243/T244 document evidence for the telemetry gate transition
- AI Council recall and broadcast critique
- Claude Bridge isolated read-only critique
