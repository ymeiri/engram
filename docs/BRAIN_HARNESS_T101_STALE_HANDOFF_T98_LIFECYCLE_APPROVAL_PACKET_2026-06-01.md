# Brain Harness T101 Stale Handoff T98 Lifecycle Approval Packet

Status: Pending explicit user approval. No archive or lifecycle write has been run.

Scope: Freeze exactly one superseded rolling handoff target for a future archive decision:
`019e836a-435a-75e1-8702-ced8eabe85cc`.

This packet does not authorize broad stale-handoff cleanup, `lint(action="apply_safe")`, M6
inspection/apply/deletion, T69 inspection, T70 document indexing, lifecycle mutation beyond the
single named target, ranking changes, `orient` expansion, public MCP changes, schema/storage/index
changes, document-index behavior changes, or harness adapter/hook writes.

## Research Question

After T100, is handoff `019e836a-435a-75e1-8702-ced8eabe85cc` a safely identified stale active
handoff target that should be presented to the user as an exact future archive decision, without
archiving it now?

## Hypotheses

| Hypothesis | Prediction |
|---|---|
| Preferred | T100 handoff `019e8378-b2f0-7260-a887-4abdf6c0e4e2` supersedes `019e836a-435a-75e1-8702-ced8eabe85cc`, while direct search still surfaces both active handoffs. A docs-only exact approval packet improves lifecycle decision quality without mutating memory. |
| Null | `019e836a-435a-75e1-8702-ced8eabe85cc` is not clearly superseded or does not surface as active/noisy enough to justify an approval packet. |
| Simpler alternative | Rely on the T100 current-plan memory and do not create another approval packet. |
| Failure | The packet is mistaken for archive approval, broad handoff cleanup, migration approval, ranking evidence, or permission to apply lint safe actions. |

## Measurement

- Lean `orient` trace `019e837b-d041-71b3-8885-9619a07a562f` returned T100 current-plan
  memory `019e837a-1e38-74b1-8954-d894a1db867f` first and reported no open obligations.
- Direct current-plan search trace `019e837b-f4f6-7c03-93c0-b8ed0e02373f` returned T100
  current-plan memory first, then active handoff `019e8378...`, with stale handoff `019e82f8...`
  also visible lower in the result set.
- Direct architecture/gate search trace `019e837b-f5b8-7881-9469-5e98cf28ce36` returned T100
  current-plan memory first, active handoff `019e8378...` second, and stale active handoff
  `019e836a...` third at the same score as the active handoff.
- Direct Memory OS implementation-plan search trace `019e837b-f66d-7681-843d-40cee091fc60`
  returned T100 first and preserved M6 migration-gate context.
- Direct recent-risk search trace `019e837c-0794-7c60-8801-7f1060b9601d` returned active handoff
  `019e8378...` and stale active handoff `019e836a...` at the same score, above the T100
  current-plan memory.
- Direct review-memory search trace `019e837c-8d8d-73e3-9125-e8d0ae27a42d` returned active
  handoff `019e8378...` second and stale active handoff `019e836a...` third at the same score.
- `memory(action="get", id="019e836a-435a-75e1-8702-ced8eabe85cc")` confirmed the target remains
  `status=active`, `kind=handoff`, and still describes T97/T98 as the latest implementation
  context.
- `memory(action="get", id="019e8378-b2f0-7260-a887-4abdf6c0e4e2")` confirmed the active T100
  handoff supersedes `019e836a...`.
- `graph(action="around", node="019e8378-b2f0-7260-a887-4abdf6c0e4e2", depth=1)` showed a
  `supersedes` edge from `019e8378...` to `019e836a...`.
- `memory(action="changes_since", timestamp="2026-06-01T13:59:49.891182Z")` trace
  `019e837c-07ed-7561-b695-a8fb32fd8c12` returned zero newer memory items or commits before this
  packet, so no other writer had changed the relevant state.
- Read-only `lint(action="run", limit=30)` returned many `superseded_item_still_active` findings
  with `safe_action=archive_memory_item` and applied zero safe actions. The target did not need to
  appear in that limited page because direct search, `memory(get)`, and graph evidence already
  identify the exact supersession relationship.
- Source inspection confirmed `memory(action="archive")` writes archive metadata for one required
  ID, while `lint(action="apply_safe")` can archive every matching safe-action finding in a report.
  This packet therefore asks only for a future single-target `memory(action="archive")`, not a lint
  safe-action sweep.
- AI Council recall surfaced prior T88/T38 guidance: strict target-local approval packets must not
  be broadened into payload expansion, lifecycle cleanup, broad ranking, migration, or approval
  inference.
- Git status before this packet was clean except untracked root `AGENTS.md`, which remains
  untouched and unstaged unless the user explicitly asks.

## Completion Matrix Delta

| Area | State After T101 Packet | Evidence | Remaining Risk Or Gate |
|---|---|---|---|
| Rolling handoff | Current handoff remains `019e8378...` | `handoff(get)`, `memory(get)`, and graph evidence | No issue for normal resume, but stale active handoffs still appear in direct search |
| Stale T98 handoff | Exact future archive target frozen | `019e8378...` supersedes `019e836a...`; direct search returns both active handoffs | Requires exact T101 approval phrase before archive |
| Lifecycle cleanup | Still gated | No `memory(action="archive")`; no `lint(action="apply_safe")`; lint applied `0` safe actions | T88/T95/T97/T99 remain separate exact archive packets |
| M6 migration | Still gated | T69/T70 and M6 gates unchanged | Count drift unresolved; no review apply/delete/simplify action allowed |
| Hot path and retrieval | Unchanged | No ranking, `orient`, public MCP, schema/storage/index, document-index, or harness changes | Broad ranking quality and stale historical noise remain open evidence-quality risks |

## Proposed Approved Archive

If and only if the user approves with the exact phrase below, Codex may run one Memory OS archive
write for this single ID:

```text
memory(
  action="archive",
  id="019e836a-435a-75e1-8702-ced8eabe85cc",
  archive_reason="Superseded by rolling handoff 019e8378-b2f0-7260-a887-4abdf6c0e4e2 after T100.",
  archived_by="codex"
)
```

Validation after an approved archive would be limited to:

- `memory(action="get", id="019e836a-435a-75e1-8702-ced8eabe85cc")` shows archived status and
  archive metadata.
- Direct search no longer returns the target as an active handoff for the tested T100/T101 prompt.
- `handoff(get)` still returns active handoff `019e8378-b2f0-7260-a887-4abdf6c0e4e2`.
- T69, T70, T88, T95, T97, and T99 gates remain unchanged.

## Stop Conditions

Stop without archiving if any of these occur:

- Approval is missing, conditional, ambiguous, or does not include the exact target ID.
- The target is no longer active or no longer superseded by `019e8378...`.
- The requested action would archive more than the single named target.
- The action would run `lint(action="apply_safe")`, broad handoff cleanup, M6 inspection/apply,
  deletion, lifecycle mutation for other memories, ranking changes, `orient` expansion, public MCP
  changes, schema/storage/index changes, document-index behavior changes, or harness writes.

## Approval Question

Reply exactly:

```text
Approve T101: archive handoff 019e836a-435a-75e1-8702-ced8eabe85cc only.
```
