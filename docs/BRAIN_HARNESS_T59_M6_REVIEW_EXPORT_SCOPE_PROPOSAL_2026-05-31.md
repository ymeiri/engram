# Brain Harness T59 M6 Review Export Scope Proposal

Status: Pending user approval. No review export has been run.
Date: 2026-05-31
Scope: Proposed one-call review-export-only M6 scoping run

This packet asks whether to authorize the next bounded M6 step after the T58 inventory-only run:
one review export call that writes a review batch for human decision. It does not authorize review
apply, candidate acceptance/rejection, deletion, lifecycle mutation, schema/storage/index changes,
public MCP changes, ranking changes, `orient` changes, or harness adapter/hook changes.

## Current Evidence

- T58 ran exactly one approved inventory-only
  `memory(action="migration_inventory", project_name="engram", ...)` call.
- The inventory scanned 115 sources, returned 11 candidates, was not truncated, and wrote no
  Memory OS records.
- The inventory candidate distribution was 9 `review` candidates and 2 `quarantine` candidates.
- The reviewed-source filter skipped 49 candidates already decided in
  `/Users/yuval.meiri/.engram/reviews/2026-04-28-memory-os-completion`.
- The T58 report is
  `docs/BRAIN_HARNESS_T58_T45_M6_INVENTORY_REPORT_2026-05-31.md`.

## Research Question

Can Engram safely export a bounded review batch for the 11 T58 M6 inventory candidates without
making migration decisions or writing active Memory OS records?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | One exact review-export call creates a bounded review workspace for the T58 candidate set while preserving explicit human review and write-apply gates. |
| Null | The export output is ambiguous, stale, unexpectedly broad, or mismatched against the T58 inventory, so no review decisions should be made. |
| Simpler alternative | Defer review export and keep M6 paused, using the T58 inventory report as the current evidence boundary. |
| Failure | The export requires apply/write semantics beyond review-batch creation, reintroduces already-decided sources, or pressures candidate decisions without a separate approval gate. |

## Proposed Approved Call

If the user approves this packet, Codex may first perform a read-only path-existence preflight for
the exact review directory below. If the path already exists, stop before running export.

```text
memory(
  action="migration_review_export",
  project_name="engram",
  limit=200,
  include_entity_observations=true,
  include_session_history=true,
  include_work_observations=true,
  exclude_reviewed_path="/Users/yuval.meiri/.engram/reviews/2026-04-28-memory-os-completion",
  migration_review_path="/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export"
)
```

The `exclude_reviewed_path` must be carried forward from T58 so the 49 already-decided candidates
do not re-enter the review workspace. If this packet is executed on a later date, use the exact
`migration_review_path` above unless the user explicitly approves a different path.

## In Scope After Explicit Approval

- A read-only preflight that checks whether
  `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export` already exists.
- Exactly one `memory(action="migration_review_export", ...)` call using the exact parameters
  shown above.
- A Markdown report summarizing the export result, candidate count, warnings, and stop condition
  if one occurs.
- Telemetry feedback for assessable retrieval/export traces.
- `git diff --check` and a focused documentation commit if documentation changes are made.

## Out of Scope

- `memory(action="migration_review_apply", ...)` or any write apply.
- Candidate acceptance, rejection, ranking, promotion, quarantine decisions, or reading exported
  candidate files to make decisions.
- Deletion, cleanup, archive, supersede, or any other lifecycle mutation.
- Schema, storage, index, public MCP, ranking, or `orient` changes.
- Harness adapter, settings, or hook writes.
- `repo(...)`, `digest(...)`, docs cleanup/reindex, vault compile, or migration status workflows
  unless separately approved.

## Stop Conditions

Stop and report without continuing if any of these occur:

- Approval is missing, conditional, or ambiguous.
- The target review path already exists.
- Any proposed parameter needs to change.
- The export returns zero candidates, more than 11 candidates, or a count mismatch that is not
  explicitly explained by the tool output.
- The output appears to include already-decided candidates or omits the required
  `exclude_reviewed_path`.
- The export errors.
- The tool requires an apply/write mode beyond review-batch export.
- The step appears to require deletion, lifecycle mutation, schema/storage/index work, public MCP
  changes, ranking changes, `orient` changes, harness writes, or candidate decisions.

## Approval Question

Do you approve exactly one review-export-only M6 scoping run using the exact
`memory(action="migration_review_export", ...)` parameters shown above, after a path-existence
preflight, followed only by a Markdown report and documentation commit, with no review apply, no
candidate decisions, no deletion, no lifecycle mutation, no schema/storage/index changes, no public
MCP changes, no ranking or `orient` changes, and no harness adapter/hook changes?
