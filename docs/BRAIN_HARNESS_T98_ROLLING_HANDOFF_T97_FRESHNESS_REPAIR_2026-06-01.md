# Brain Harness T98 Rolling Handoff T97 Freshness Repair

Status: Implemented and validated

Scope: Refresh the active rolling handoff after T97 so `handoff(get)` matches the current Brain
Harness plan and exact approval gates.

T98 is continuity maintenance only. It does not authorize T69 count-drift inspection, T70 document
indexing, T88 archive, T95 archive, T97 archive, M6 review apply, migration deletion, lifecycle
cleanup, ranking changes, `orient` expansion, public MCP changes, schema/storage/index changes,
document-index behavior changes, or harness adapter/hook writes.

## Research Question

After T97, does the active rolling handoff still preserve the latest plan and exact gates for
future resume, or is it stale enough to justify one handoff refresh?

## Hypotheses

| Hypothesis | Prediction |
|---|---|
| Preferred | `orient` and direct search recover T97, but `handoff(get)` still stops at T95/T96, so a single rolling handoff update improves continuity without changing product behavior. |
| Null | `handoff(get)` is current enough; no write is needed. |
| Simpler alternative | Document the discrepancy only and keep relying on latest current-plan memory. |
| Failure | A handoff write creates more duplicate handoff noise or is mistaken for approval to archive old handoffs. |

## Measurement

- Lean `orient` trace `019e8368-fda4-7ea2-93df-502de8a11c3f` returned T97 current-plan memory
  `019e8367-23c7-7413-8604-a5aa9a05083b` first.
- Direct current-plan search trace `019e8369-214e-7293-a94c-518fa4f6cbdc` returned the same T97
  current-plan memory first for continuation wording.
- Architecture/risk search trace `019e8369-2bc4-7bd1-a20a-e586c3490b35` returned T97 first,
  while still showing active and superseded rolling handoffs below it.
- Implementation-plan search trace `019e8369-368c-7fb0-839c-c1198d668ee4` returned the M6 and
  harness-write gates first, then T97 and stale handoff noise; this preserved default-deny gates.
- User design-philosophy search trace `019e8369-40b9-7f92-b63d-700bb269b85f` returned reviewed
  preference memory `019e6924-256b-7093-b1c5-286ec4d02461` first.
- Recent-risk search trace `019e8369-4f21-7c60-a9b9-7a1a6a500a26` returned T97 first and showed
  the active handoff noise below it.
- `memory(action="changes_since")` trace `019e8369-603f-75a0-9ea4-6d17e832d13c` found no newer
  memory after the T97 cursor before T98 work.
- `handoff(action="get", project="engram")` returned active handoff
  `019e835e-81c2-7562-897a-e42c0fe8dc08`, whose content still described T95/T96 as the latest
  implementation context and did not include T97.
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

T98 wrote rolling handoff `019e836a-435a-75e1-8702-ced8eabe85cc`, superseding
`019e835e-81c2-7562-897a-e42c0fe8dc08`.

The handoff records:

- T97 as the latest completed evidence slice.
- T97 git commit `5ef3873`.
- Active current-plan memory `019e8367-23c7-7413-8604-a5aa9a05083b` and memory commit
  `019e8367-23ed-7fb0-84cc-bd25dc858dea`.
- T69, T70, T88, T95, and T97 exact approval gates.
- M6 write/apply/delete/lifecycle, ranking, `orient`, document-index, schema/storage/index, public
  MCP, and harness-write boundaries.
- The untracked root `AGENTS.md` exclusion.

## Completion Matrix Delta

| Area | State After T98 | Evidence | Remaining Risk |
|---|---|---|---|
| Rolling handoff | Refreshed | `handoff(get)` returns T98 handoff id `019e836a-435a-75e1-8702-ced8eabe85cc` | Older active handoff items may still appear in direct search until explicitly archived |
| Current-plan retrieval | Still validated for observed prompt | T97 remains first in `orient` and direct `search` | Broad ranking quality remains unproven |
| Lifecycle cleanup | Still gated | T88, T95, and T97 exact archive targets unchanged | No archive was run |
| M6 migration | Still gated | T69 exact inspection gate unchanged | Count drift unresolved |
| Document index visibility | Still gated for T70 | T70 exact indexing packet remains pending | T68/T69 docs may remain weak in document search |

## Validation

- `handoff(action="get", project="engram")` returned written handoff
  `019e836a-435a-75e1-8702-ced8eabe85cc`.
- `graph(action="around", node="019e836a-435a-75e1-8702-ced8eabe85cc", depth=1)` showed the edge
  `019e836a...` supersedes `019e835e...`.
- No code changed, so no Rust test target was required for this documentation and Memory OS
  handoff-maintenance slice.

## Result

The preferred hypothesis held. The live resume entrypoints had drifted again: current-plan memory
and docs were at T97, but the rolling handoff was stale at T95/T96. T98 repaired only the rolling
handoff and documented the boundary so future agents do not confuse handoff freshness with approval
for gated migration, archive, document-index, ranking, or hot-path work.
