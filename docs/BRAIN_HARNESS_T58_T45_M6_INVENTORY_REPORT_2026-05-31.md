# Brain Harness T58 T45 M6 Inventory Report

Status: Completed approved inventory-only scoping run. No migration decisions accepted.
Date: 2026-05-31
Scope: One read-only `memory(action="migration_inventory", ...)` call approved by the user

This report records the result of the T45 inventory-only M6 scoping run. It did not run review
export, apply, deletion, lifecycle mutation, schema/storage/index changes, public MCP changes,
ranking changes, `orient` changes, or harness adapter/hook changes.

## Approved Call

The user explicitly approved the T45 inventory-only M6 scoping run. Codex ran exactly one inventory
call with the approved parameters:

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

## Research Question

Can Engram safely gather a bounded, read-only inventory of current legacy migration candidates so
the user can decide whether a later review-export phase is justified?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A bounded inventory-only run identifies candidate volume, source-layer mix, reviewed-source filtering behavior, and immediate risk without writing MemoryItems or review batches. |
| Null | The inventory output is too broad, stale, noisy, or ambiguous to justify a review-export phase. |
| Simpler alternative | Defer review export and continue with retrieval, feedback, and harness evidence only. |
| Failure | The inventory requires writes, returns unbounded output, or creates pressure to apply/delete/archive without reviewed candidates and explicit approval. |

## Result

The inventory completed at `2026-05-31T11:41:52.591551Z`.

| Metric | Value |
| --- | ---: |
| Sources scanned | 115 |
| Total candidates | 11 |
| Returned candidates | 11 |
| Truncated | false |
| Limit | 200 |

## Distribution

| Source kind | Count |
| --- | ---: |
| Project observation | 9 |
| Entity observation | 2 |

| Proposed disposition | Count |
| --- | ---: |
| Review | 9 |
| Quarantine | 2 |

| Proposed memory kind | Count |
| --- | ---: |
| Decision | 6 |
| Project fact | 4 |
| Limitation | 1 |

| Confidence bucket | Count |
| --- | ---: |
| High | 9 |
| Medium | 2 |

## Warnings

- Dry run only: no Memory OS records were written.
- Only explicitly accepted review candidates are eligible for migration writes.
- Skipped 55 candidates whose source was already migrated.
- Skipped 49 candidates already decided in the review workspace.

## Candidate Summary

| Source key | Source kind | Proposed kind | Disposition | Scope note | Staleness |
| --- | --- | --- | --- | --- | ---: |
| `dogfood.baf008-accepted-live-2026-05-24` | Project observation | Project fact | Review | Engram project | 7 days |
| `dogfood.baf008-prearm-setup-2026-05-24` | Project observation | Decision | Review | Engram project | 7 days |
| `dogfood.claude-code-scoped-obligation-smoke-2026-05-24` | Project observation | Decision | Review | Engram project | 7 days |
| `dogfood.claude-code-obligation-list-scope-fix-2026-05-24` | Project observation | Project fact | Review | Engram project | 7 days |
| `dogfood.claude-code-2026-05-24-review` | Project observation | Decision | Review | Engram project | 7 days |
| `decisions.claude-hook-reenable-prompt-2026-05-24` | Project observation | Decision | Review | Engram project | 7 days |
| `maintenance.disk-cleanup-2026-05-24` | Project observation | Project fact | Review | Engram project | 7 days |
| `decisions.orient-recent-git-context` | Project observation | Decision | Review | Engram project | 24 days |
| `testing.dogfood-pilot-2026-05-07` | Project observation | Decision | Review | Engram project | 24 days |
| `telemetry.recall.432971` | Entity observation | Project fact | Quarantine | `review-all-system` entity, broad cross-project scope | 24 days |
| `gotchas.shared-worktree-branch-loss` | Entity observation | Limitation | Quarantine | `review-all-system` entity, broad cross-project scope | 31 days |

## Interpretation

The T45 inventory scope is small enough for a later human review batch: 11 candidates rather than
the earlier broad inventory's thousands of candidates. The reviewed-source filter is active: 49
candidates were skipped because they were already decided in the review workspace, and 55 were
skipped because their sources were already migrated.

The nine review candidates are all Engram project observations. They mostly describe May 2026
dogfood, Claude Code obligation behavior, hook re-enable planning, recent git context in `orient`,
and the first dogfood pilot. These may be useful as durable Memory OS evidence, but the inventory
does not decide that.

The two quarantine candidates are entity observations for `review-all-system`, not Engram project
observations. Their quarantine disposition is appropriate because entity observations can be linked
broadly across projects and need scope confirmation before any migration.

## Caveats

- This report accepts no migration candidates.
- This report does not create a review batch.
- This report does not prove candidate correctness, freshness, or final scope.
- The inventory is dry-run evidence only; it is not write-apply approval.
- Old migration/export approval-shaped memories remain stale unless they match a current
  user-approved scope.

## Next Gate

The next M6 step is a separate approval decision: whether to authorize a bounded
`memory(action="migration_review_export", ...)` scope for these 11 inventory candidates. Any review
export must be explicitly approved before it runs.

M6 write apply, deletion, cleanup, broad legacy simplification, lifecycle mutation, schema/storage
or index changes, public MCP changes, ranking changes, `orient` payload changes, and harness
adapter or hook changes remain out of scope and still require separate explicit approval.
