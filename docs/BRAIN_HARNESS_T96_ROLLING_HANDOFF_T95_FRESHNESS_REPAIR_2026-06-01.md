# Brain Harness T96 Rolling Handoff T95 Freshness Repair

Status: Implemented and validated

Scope: Refresh the active rolling handoff after T95 so `handoff(get)` matches the current Brain
Harness plan and exact approval gates.

T96 is continuity maintenance only. It does not authorize T69 count-drift inspection, T70 document
indexing, T88 archive, T95 archive, M6 review apply, migration deletion, lifecycle cleanup,
ranking changes, `orient` expansion, public MCP changes, schema/storage/index changes,
document-index behavior changes, or harness adapter/hook writes.

## Research Question

After T95, does the active rolling handoff still preserve the latest plan and exact gates for
future resume, or is it stale enough to justify one handoff refresh?

## Hypotheses

| Hypothesis | Prediction |
|---|---|
| Preferred | `orient` and direct search recover T95, but `handoff(get)` still stops at T93/T94, so a single rolling handoff update improves continuity without changing product behavior. |
| Null | `handoff(get)` is current enough; no write is needed. |
| Simpler alternative | Document the discrepancy only and keep relying on latest current-plan memory. |
| Failure | A handoff write creates more duplicate handoff noise or is mistaken for approval to archive old handoffs. |

## Measurement

- Lean `orient` trace `019e835c-eeba-7d43-a829-92a2c4af370e` returned T95 current-plan memory
  `019e835a-f55d-7f21-8a65-8b8bf29c5f3e` first.
- Direct current-plan search trace `019e835d-0879-7292-8213-eaeb4eb4882f` returned the same T95
  current-plan memory first for continuation wording.
- Architecture/risk search trace `019e835d-2014-70a1-9907-a0f34dfa8ec8` returned active gates and
  T95, while still showing active and superseded handoff noise below current guidance.
- Implementation-plan search trace `019e835d-2958-72e1-9336-124ecbae2820` returned T95 first and
  confirmed the M6 approval gate remains explicit.
- User design-philosophy search trace `019e835d-3404-7d43-a2da-a7e987dc6f8f` returned reviewed
  preference memory `019e6924-256b-7093-b1c5-286ec4d02461`.
- Recent-risk search trace `019e835d-3e19-7413-8648-050f10c845c5` showed the current T95 context
  together with stale active handoff noise.
- `memory(action="changes_since")` trace `019e835d-c761-7863-8c94-b79a28730497` found no newer
  memory after the T95 cursor.
- `handoff(action="get", project="engram")` returned active handoff
  `019e8352-a610-7f92-859f-f9d74b026ba7`, whose content still described T93 as the latest
  implementation slice and did not include T95.
- Source reading confirmed `HandoffService::update` writes a new active `MemoryKind::Handoff`
  tagged `handoff` and `rolling`, records the previous active handoff in `supersedes`, and does not
  archive or delete the previous item. MCP `handoff(action="update")` defaults to dry-run unless
  `dry_run=false` and requires writer provenance.
- AI Council recall found prior T38/T88 guidance that strict intent-local or exact-target changes
  must not be broadened into payload expansion, lifecycle cleanup, broad ranking, migration, or
  approval inference.
- Git status before the slice showed only untracked user-owned root `AGENTS.md`; it was left
  untouched and unstaged.

## Written Handoff

T96 wrote rolling handoff `019e835e-81c2-7562-897a-e42c0fe8dc08`, superseding
`019e8352-a610-7f92-859f-f9d74b026ba7`.

The handoff records:

- T95 as the latest completed evidence slice.
- T95 git commit `6d736e8`.
- Active current-plan memory `019e835a-f55d-7f21-8a65-8b8bf29c5f3e` and memory commit
  `019e835a-f57f-7e53-9093-24f840f5f9d3`.
- T69, T70, T88, and T95 exact approval gates.
- M6 write/apply/delete/lifecycle, ranking, `orient`, document-index, schema/storage/index, public
  MCP, and harness-write boundaries.
- The untracked root `AGENTS.md` exclusion.

## Completion Matrix Delta

| Area | State After T96 | Evidence | Remaining Risk |
|---|---|---|---|
| Rolling handoff | Refreshed | `handoff(get)` returns T96 handoff id `019e835e-81c2-7562-897a-e42c0fe8dc08` | Older active handoff items may still appear in direct search until explicitly archived |
| Current-plan retrieval | Still validated for observed prompt | T95 remains first in `orient` and direct `search` | Broad ranking quality remains unproven |
| Lifecycle cleanup | Still gated | T88 and T95 exact archive targets unchanged | No archive was run |
| M6 migration | Still gated | T69 exact inspection gate unchanged | Count drift unresolved |
| Document index visibility | Still gated for T70 | T70 exact indexing packet remains pending | T68/T69 docs may remain weak in document search |

## Validation

- `handoff(action="get", project="engram")` returned written handoff
  `019e835e-81c2-7562-897a-e42c0fe8dc08`.
- No code changed, so no Rust test target was required for this documentation and Memory OS
  handoff-maintenance slice.

## Result

The preferred hypothesis held. The live resume entrypoints had drifted again: current-plan memory
and docs were at T95, but the rolling handoff was stale at T93/T94. T96 repaired only the rolling
handoff and documented the boundary so future agents do not confuse handoff freshness with approval
for gated migration, archive, document-index, ranking, or hot-path work.
