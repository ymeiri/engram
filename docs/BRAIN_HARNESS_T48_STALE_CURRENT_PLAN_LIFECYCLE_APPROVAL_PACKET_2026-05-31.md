# Brain Harness T48 Stale Current-Plan Lifecycle Approval Packet

Status: Pending user approval. No memory lifecycle write is authorized by this document.
Date: 2026-05-31
Scope: Proposal for one exact archive action on stale repository-scoped current-plan guidance

This packet is a request for approval, not approval itself. No `memory(action="archive")`,
supersede, rejection, deletion, migration action, harness write, schema/storage/index change,
public MCP change, ranking change, or `orient` payload change has been run for T48.

## Research Question

Can Engram safely ask for explicit approval to archive one stale repository-scoped current-plan
MemoryItem, using only fresh read-only evidence and a default-deny single-item lifecycle scope?

## Current Evidence

- Final T47 sanity `orient` trace `019e7d3d-3687-7503-a6b7-cbe1511bf94d` and the T48 startup
  `orient` trace `019e7d3e-0ab7-7191-b116-bf2d61cf1d18` both surfaced the new T47 project-scoped
  current plan first, but still included the older repository-scoped current-plan memory
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` lower in the candidate set.
- Direct `search` trace `019e7d3e-1a25-70c3-abfe-ba3d538bb8da` returned the active T47 plan plus
  the same older repository-scoped current-plan memory in a next-step/lifecycle query.
- `memory(action="get", id="019e5e0a-86b4-73e3-aa9b-ca350e83e915")` confirms the target is an
  active repository-scoped `decision` tagged `current-plan`, titled
  `Current plan after Codex document lifecycle follow-through`.
- `memory(action="list", scope_type="project", project_name="engram", tags=["current-plan"],
  status_filter="active")` returned exactly one active project-scoped current-plan item:
  `019e7d3c-afa9-7861-8569-37c2cb68a661`, the T47 current plan.
- `memory(action="list", scope_type="repository", local_path="/Users/yuval.meiri/projects/engram",
  tags=["current-plan"], status_filter="active")` returned exactly one active repository-scoped
  current-plan item: `019e5e0a-86b4-73e3-aa9b-ca350e83e915`.
- Read-only `lint(action="run", limit=20, write=false)` reported
  `feedback_stale_current_plan:019e5e0a-86b4-73e3-aa9b-ca350e83e915` with 129 recent stale
  feedback records and `safe_action=none`.
- Source inspection confirms `memory(action="archive")` requires the exact item `id` and
  `archive_reason`, stores archive metadata, sets status to `archived`, and does not delete the
  MemoryItem. Source also confirms lint safe actions intentionally do not apply to
  `feedback_stale_current_plan`.
- AI Council and Claude Bridge critique agreed this is a defensible non-gated slice only as a
  pending approval packet with one exact lifecycle write, fresh pre-write reads, and hard stop
  conditions for any state drift.

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A single-item archive approval packet can remove stale current-plan guidance from normal retrieval after explicit user approval, without mutating current T47 guidance or broad lifecycle state. |
| Null | The stale item is harmless lower-ranked noise, so archiving is unnecessary until more evidence accumulates. |
| Simpler alternative | Continue rejecting the stale item via feedback and leave lifecycle state unchanged. |
| Failure | The packet implies approval, archives the wrong item, removes needed context, hides a broader lifecycle issue, or bundles unrelated migration, harness, ranking, or hot-path work. |

## Proposed Approval

If the user explicitly approves this packet, the authorized lifecycle write is exactly:

```text
memory(
  action="archive",
  id="019e5e0a-86b4-73e3-aa9b-ca350e83e915",
  archive_reason="Stale repository-scoped current-plan guidance superseded by active T47 project-scoped current plan 019e7d3c-afa9-7861-8569-37c2cb68a661; read-only lint reported feedback_stale_current_plan with 129 recent stale-feedback records and safe_action=none.",
  archived_by="codex"
)
```

Any missing, conditional, partial, or ambiguous approval remains default-deny.

## In Scope After Explicit Approval

| Item | Allowed after explicit approval? | Notes |
| --- | --- | --- |
| Fresh read-only `memory(action="get")` for the target item | Yes | Must confirm the target is still the same active repository-scoped `decision` tagged `current-plan`. |
| Fresh read-only project and repository current-plan `memory(action="list")` checks | Yes | Must confirm the T47 project plan remains active and the target is the only active repository-scoped current-plan item. |
| Fresh read-only `lint(action="run", write=false)` | Yes | Must still report the target as `feedback_stale_current_plan` with `safe_action=none`. |
| Fresh lean `orient` or direct `search` sanity check | Yes | Must show current T47 guidance remains available before the archive. |
| The single `memory(action="archive", ...)` call above | Yes | Only if all fresh pre-write reads match this packet. |
| Post-write read-only validation | Yes | Confirm target is archived, the T47 plan remains active, and the stale-current-plan lint finding for the target is gone. |
| Markdown report, telemetry feedback, and documentation commit | Yes | Evidence annotation only. |

## Out Of Scope

| Item | Authorized by this packet? |
| --- | --- |
| Archiving, superseding, rejecting, editing, or reviewing any other MemoryItem | No |
| Creating a replacement MemoryItem | No |
| Mutating T47 current-plan memory `019e7d3c-afa9-7861-8569-37c2cb68a661` | No |
| Running `lint(action="apply_safe", write=true)` or any automatic lifecycle cleanup | No |
| M6 migration inventory, review export, apply, deletion, cleanup, or legacy simplification | No |
| Harness adapter/settings/hook writes or T47 harness repair execution | No |
| Schema, storage, index, public MCP, ranking, `orient`, graph, lint rule, telemetry formula, or search changes | No |

## Validation Criteria

The approved archive succeeds only if:

- the target item still exists with id `019e5e0a-86b4-73e3-aa9b-ca350e83e915`;
- its status is still `active`;
- its kind is still `decision`;
- its scope is still repository `/Users/yuval.meiri/projects/engram`;
- it is still tagged `current-plan`;
- the active project-scoped current-plan list still includes T47 item
  `019e7d3c-afa9-7861-8569-37c2cb68a661`;
- the repository-scoped active current-plan list still returns only the target item;
- read-only lint still reports `feedback_stale_current_plan` for the target with `safe_action=none`;
- post-write `memory(action="get")` shows the target status is `archived` with the approved archive
  reason;
- post-write project current-plan listing still returns the T47 current plan;
- post-write lint no longer reports `feedback_stale_current_plan` for the archived target;
- `obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")`
  is clean or explicitly resolved/skipped with evidence before final response.

## Stop Conditions

Stop and ask the user again before writing anything if:

- approval is missing, conditional, ambiguous, or changes the allowed scope;
- the target item is already archived, superseded, rejected, missing, or changed in identity;
- the target item is no longer a repository-scoped active `decision` tagged `current-plan`;
- the T47 project-scoped current plan is missing, inactive, changed unexpectedly, or no longer
  first-class current guidance;
- any additional active project-scoped or repository-scoped current-plan item appears and changes
  the single-item analysis;
- lint no longer reports the target as `feedback_stale_current_plan` with `safe_action=none`;
- any step appears to require creating a replacement memory, applying automatic lint cleanup,
  mutating other memories, changing ranking, changing `orient`, running M6, or executing harness
  writes;
- the proposed `archive_reason` or `archived_by` needs to change before execution;
- post-write validation cannot prove the target was archived and the T47 current plan remains
  active.

## Approval Question

Do you approve exactly one lifecycle write to archive MemoryItem
`019e5e0a-86b4-73e3-aa9b-ca350e83e915` with the `memory(action="archive", ...)` payload shown
above, contingent on fresh matching read-only get/list/lint/orient evidence, with no other memory
lifecycle writes, no M6 action, no harness writes, and no schema/storage/index/public-MCP/ranking
or `orient` changes?
