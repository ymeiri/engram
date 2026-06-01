# Brain Harness T86 Rolling Handoff Freshness Audit

Status: Pre-registered; execution pending
Date: 2026-06-01
Scope: Read-only handoff freshness audit plus one rolling-handoff continuity update if stale

This slice checks whether the active rolling handoff preserves the current Brain Harness plan and
approval gates after T85. If the handoff is stale or too thin, Codex may update only the rolling
handoff record with the current plan and next actions.

The user's generic `i approve` reply is authorization to continue safe, non-gated continuity work
only. It is not authorization for T69 count-drift inspection, T70 document indexing, M6 apply,
candidate decisions, deletion, lifecycle mutation, schema/storage/index changes, public MCP changes,
ranking changes, `orient` expansion, or harness adapter/hook writes.

## Research Question

Does the active rolling handoff contain enough current T85/T69/T70 context for a future agent to
resume safely, or is it stale enough that Engram should refresh the handoff while keeping `orient`
as the frictionless task-boundary entrypoint?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The active handoff is low-information or stale relative to current `orient` and repo docs, so a narrow handoff update improves continuity without changing any product behavior. |
| Null | The active handoff already contains current T85/T69/T70 plan context and no update is needed. |
| Simpler alternative | Do not write a handoff and continue relying on `orient` plus current-plan memory as the resume path. |
| Failure | The handoff update is treated as approval for migration, document indexing, lifecycle mutation, ranking, hot-path, schema/storage, public MCP, or harness work. |

## Measurement

1. Compare the active `handoff(action="get", project="engram")` result against:
   - the latest lean `orient` result for the current Brain Harness goal;
   - `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`;
   - `docs/BRAIN_HARNESS_T69_T68_COUNT_DRIFT_DECISION_PACKET_2026-06-01.md`;
   - `docs/BRAIN_HARNESS_T70_T69_DOCUMENT_VISIBILITY_AUDIT_AND_INDEX_PACKET_2026-06-01.md`;
   - `docs/BRAIN_HARNESS_T85_CLAUDE_BRIDGE_PROJECT_TOOL_EXPOSURE_RECHECK_2026-06-01.md`.
2. If the handoff omits the current plan and exact approval gates, update only the rolling handoff
   with:
   - current completed slice: T85;
   - current blocked/gated slices: T69, T70, M6 apply/deletion, lifecycle, harness writes;
   - exact next approval phrases for T69 and T70;
   - explicit instruction that generic approval is insufficient for T69/T70.
3. Re-read the handoff and document the outcome.

## Success Criteria

- The report identifies the prior handoff id and whether it was fresh enough.
- If updated, the new handoff includes current T85/T69/T70 context and concrete next actions.
- `orient` remains unchanged and remains the task-boundary entrypoint.
- No migration inspection, document indexing, lifecycle write, schema/storage/index behavior
  change, public MCP change, ranking change, `orient` change, or harness write occurs.
- Repo docs are committed, and only intended files are staged.

## Stop Conditions

Stop without updating the handoff if:

- the handoff tool requires schema/storage/index, public MCP, harness, hook, or lifecycle changes;
- the operation would inspect T68 export files, index documents, run migration status/prioritize,
  run review apply, decide candidates, delete data, or mutate legacy memory lifecycle;
- the active approval gates cannot be identified from current repo docs and Engram retrieval.
