# Brain Harness T91 Rolling Handoff T90 Freshness Repair

Status: Implemented and validated

Scope: Refresh the active rolling handoff after T89/T90 so `handoff(get)` matches the current
Brain Harness plan and approval gates.

T91 is continuity maintenance only. It does not authorize T69 count-drift inspection, T70 document
indexing, T88 archive, M6 review apply, migration deletion, lifecycle cleanup, ranking changes,
`orient` expansion, public MCP changes, schema/storage/index changes, or harness adapter/hook
writes.

## Research Question

After T89/T90, does the active rolling handoff still preserve the current plan and exact gates for
future resume, or is it stale enough to justify one handoff refresh?

## Hypotheses

| Hypothesis | Prediction |
|---|---|
| Preferred | `orient` and direct search recover T90, but `handoff(get)` still stops at T87/T86, so a single rolling handoff update improves continuity without changing product behavior. |
| Null | `handoff(get)` is current enough; no write is needed. |
| Simpler alternative | Document the discrepancy only and keep relying on latest current-plan memory. |
| Failure | A handoff write creates more duplicate handoff noise or is mistaken for approval to archive old handoffs. |

## Measurement

- Lean `orient` trace `019e8313-a277-7912-b72f-02ca68c4b013` returned T90 current-plan memory
  `019e8311-b132-7220-bd55-a2ad8204ce2e` first.
- Direct unified `search` trace `019e8313-c62e-7962-ae4f-561f72048791` returned the same T90
  current-plan memory first for continuation wording.
- `memory(action="changes_since")` trace `019e8314-12f3-7760-b08e-7738ed57279c` found no newer
  memory after the T90 cursor.
- `handoff(action="get", project="engram")` returned active handoff
  `019e82f8-cada-7c31-b073-18ac41986b1e`, whose content still described T86/T87 as the latest
  state and did not include T89/T90.
- Source reading found `HandoffService::update` writes a new active handoff item tagged
  `handoff`/`rolling` and records the previous handoff in `supersedes`; it does not archive or
  delete the previous item. MCP `handoff(action="update")` defaults to dry-run unless
  `dry_run=false`.
- Git status before the slice showed only untracked user-owned root `AGENTS.md`; it was left
  untouched and unstaged.

## Written Handoff

T91 wrote rolling handoff `019e8316-ebd1-7220-b18e-f0d33110131a`, superseding
`019e82f8-cada-7c31-b073-18ac41986b1e`.

The handoff records:

- T90 as the latest completed implementation slice.
- T89/T90 `changes_since` cursor guidance.
- T69, T70, and T88 exact approval gates.
- M6 write/apply/delete/lifecycle, ranking, `orient`, document-index, schema/storage/index, and
  harness-write boundaries.
- The untracked root `AGENTS.md` exclusion.

## Completion Matrix Delta

| Area | State After T91 | Evidence | Remaining Risk |
|---|---|---|---|
| Rolling handoff | Refreshed | `handoff(get)` returns T91 handoff id `019e8316-ebd1-7220-b18e-f0d33110131a` | Older active handoff items may still appear in direct search until explicitly archived |
| Current-plan retrieval | Still validated for observed prompt | T90 remains first in `orient` and direct `search` | Broad ranking quality remains unproven |
| M6 migration | Still gated | T69 exact inspection gate unchanged | Count drift unresolved |
| Document index visibility | Still gated for T70 | T65/T67 completed; T70 exact indexing packet remains pending | T68/T69 docs may remain weak in document search |
| Lifecycle cleanup | Still gated | T88 exact archive target unchanged | No archive was run |

## Validation

- `handoff(action="get", project="engram")` returned written handoff
  `019e8316-ebd1-7220-b18e-f0d33110131a`.
- `git diff --check` passed.
- No code changed, so no Rust test target was required for this documentation and Memory OS
  handoff-maintenance slice.

## Result

The preferred hypothesis held. The live resume entrypoints had drifted: current-plan memory and
docs were at T90, but the rolling handoff was stale at T87/T86. T91 repaired only the rolling
handoff and documented the boundary so future agents do not confuse handoff freshness with approval
for gated migration, archive, document-index, ranking, or hot-path work.
