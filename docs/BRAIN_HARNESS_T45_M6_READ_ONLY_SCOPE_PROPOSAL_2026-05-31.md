# Brain Harness T45 M6 Read-Only Scope Proposal

Status: Pending user approval. No M6 action is authorized by this document.
Date: 2026-05-31
Scope: Proposal for one inventory-only M6 scoping run

This packet is a request for approval, not approval itself. No M6 inventory, review export, apply,
deletion, lifecycle mutation, schema/storage/index change, public MCP change, or harness/hook
change has been run for T45.

## Research Question

Can Engram safely gather a bounded, read-only inventory of current legacy migration candidates so
the user can decide whether a later review-export phase is justified?

## Current Evidence

- T44 closed the immediate direct-search current-plan/M6-gate parity gap in Codex and Claude Code.
- The active M6 gate says even read-only M6 inventory or review export requires explicit
  user-approved scope.
- `docs/BRAIN_HARNESS_RESEARCH_METHOD.md` section 9.2 requires a documented reason or controlled
  dogfood evidence, no unresolved bad-memory-use finding in current high-stakes scenarios, and
  explicit user approval for the inventory scope before read-only M6 inventory.
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` still marks migration completion as approval-gated.
- Earlier broad inventory evidence reported thousands of sources/candidates, so the first proposed
  scope is inventory-only with a hard result limit before any review-export request.

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A bounded inventory-only M6 scoping run can identify candidate volume, source-layer mix, duplicate/reviewed-source filtering behavior, and immediate risk without writing MemoryItems or review batches. |
| Null | The inventory output is too broad, stale, noisy, or ambiguous to justify a review-export phase. |
| Simpler alternative | Defer all M6 work and continue improving retrieval, feedback, and harness evidence without touching migration data. |
| Failure | The inventory requires writes, implies migration approval, returns unbounded output, hits unexpected state, or creates pressure to apply/delete/archive without reviewed candidates and explicit approval. |

## Consultation

AI Council recall surfaced prior guidance that M6 work remains approval-gated and that migration
ranking/calibration slices must not be treated as migration authorization. A fresh AI Council
broadcast agreed that an approval packet is a valid non-gated step only if it is explicitly pending
approval, lists exact actions, isolates read-only inventory from review export and apply phases,
and ends with a binary scoped approval question. Claude Bridge gave the same critique and added two
constraints reflected here: list exact MCP parameters, and treat missing, conditional, or ambiguous
approval as default-deny.

## Proposed Approval

If the user explicitly approves this packet, the authorized action is exactly:

```text
memory(
  action="migration_inventory",
  project_name="engram",
  limit=200,
  include_entity_observations=true,
  include_session_history=true,
  include_work_observations=true,
  exclude_reviewed_path="/Users/yuval.meiri/.engram/reviews/2026-04-28-memory-os-completion"
)
```

After that single MCP call, the agent may write one Markdown report summarizing the returned counts,
warnings, candidate source-kind distribution, obvious stale/reviewed-source caveats, and the next
approval gate. The report must not contain accepted migration decisions.

## In Scope

| Item | Allowed after explicit approval? | Notes |
| --- | --- | --- |
| `memory(action="migration_inventory", ...)` with the exact parameters above | Yes | One bounded inventory call only. |
| Summarize the inventory result in a Markdown report | Yes | Report only; no candidate decisions. |
| Submit telemetry/feedback for the inventory trace if available | Yes | Evidence annotation only. |
| Run `git diff --check` and commit the report | Yes | Documentation commit only. |

## Out Of Scope

| Item | Authorized by this packet? |
| --- | --- |
| `memory(action="migration_review_export", ...)` | No |
| `repo(action="migration_inventory", ...)` or `repo(action="migration_review_export", ...)` | No |
| `memory(action="migration_review_apply", ...)` or any dry-run/write apply | No |
| `memory(action="digest_extraction_apply", ...)` | No |
| `docs(action="reindex_execute")`, `docs(action="cleanup_execute")`, or cleanup apply flows | No |
| `memory(action="archive")`, lifecycle promotion/rejection/supersede, deletion, or scope rewrite | No |
| Schema, storage, index, public MCP, ranking, `orient`, harness adapter, or hook changes | No |
| Treating old migration/export approvals as current approval | No |

## Validation Criteria

The approved inventory-only run succeeds only if:

- exactly one `memory(action="migration_inventory", ...)` call is run with the parameters above;
- no MemoryItems, review batches, lifecycle state, schema/storage/index state, public MCP surface,
  ranking logic, `orient` payload, hooks, or harness configuration are changed;
- the report states candidate counts, source-layer distribution, warnings, and uncertainty without
  accepting or applying any candidate;
- the report ends with the next explicit gate, normally whether to authorize a separate
  `migration_review_export` scope.

## Stop Conditions

Do not run the proposed inventory if:

- the user does not explicitly approve the exact action above;
- the user gives a conditional or ambiguous approval;
- the requested parameters need to change before execution;
- the tool requires any write/apply mode to proceed;
- the output indicates unexpected schema/storage/index state;
- the result limit hides material risk that needs a different scope;
- any step appears to require review export, apply, deletion, lifecycle mutation, cleanup, or
  harness changes.

If any stop condition appears, stop and ask the user before proceeding.

## Approval Question

Do you approve exactly one inventory-only M6 scoping run using the `memory(action="migration_inventory", ...)`
parameters shown in this document, followed only by a Markdown report and documentation commit, with
no review export, no apply, no deletion, no lifecycle mutation, no schema/storage/index changes, no
public MCP changes, no ranking or `orient` changes, and no harness adapter or hook changes?
