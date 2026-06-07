# Brain Harness T99 Stale Handoff T96 Lifecycle Approval Packet

Status: Pending explicit user approval. No archive or lifecycle write has been run.

Scope: Freeze exactly one superseded rolling handoff target for a future archive decision:
`019e835e-81c2-7562-897a-e42c0fe8dc08`.

This packet does not authorize broad stale-handoff cleanup, `lint(action="apply_safe")`, M6
inspection/apply/deletion, T69 inspection, T70 document indexing, lifecycle mutation beyond the
single named target, ranking changes, `orient` expansion, public MCP changes, schema/storage/index
changes, document-index behavior changes, or harness adapter/hook writes.

## Research Question

After T98, is handoff `019e835e-81c2-7562-897a-e42c0fe8dc08` a safely identified stale active
handoff target that should be presented to the user as an exact future archive decision, without
archiving it now?

## Hypotheses

| Hypothesis | Prediction |
|---|---|
| Preferred | T98 handoff `019e836a-435a-75e1-8702-ced8eabe85cc` supersedes `019e835e-81c2-7562-897a-e42c0fe8dc08`, while direct search still surfaces both active handoffs. A docs-only exact approval packet improves lifecycle decision quality without mutating memory. |
| Null | `019e835e-81c2-7562-897a-e42c0fe8dc08` is not clearly superseded or does not surface as active/noisy enough to justify an approval packet. |
| Simpler alternative | Rely on the existing T98 current-plan memory and do not create another approval packet. |
| Failure | The packet is mistaken for archive approval, broad handoff cleanup, migration approval, ranking evidence, or permission to apply lint safe actions. |

## Measurement

- Lean `orient` trace `019e836f-27b4-7523-b97b-8be6d21be25a` returned current-plan memory
  `019e836c-f93d-7b12-93aa-0ff67f717b23` first after T98.
- Direct current-plan search trace `019e836f-5d2e-7040-904f-adc694d46f1f` returned the T98
  current-plan memory first, then active rolling handoff `019e836a...`, then stale active handoff
  `019e835e...`.
- Direct architecture/gate search trace `019e836f-68cd-7743-8553-4e8f4378f8c4` also returned the
  active T98 handoff and the stale T96 handoff near the top.
- Direct review-memory search trace `019e8371-25ab-7b63-b7aa-9f8f35d186e4` returned active handoff
  `019e836a-435a-75e1-8702-ced8eabe85cc` and stale active handoff
  `019e835e-81c2-7562-897a-e42c0fe8dc08` at the same score, `0.8894`.
- `graph(action="around", node="019e836a-435a-75e1-8702-ced8eabe85cc", depth=1)` showed
  `019e836a...` has a `supersedes` edge to `019e835e...`.
- `memory(action="get", id="019e835e-81c2-7562-897a-e42c0fe8dc08")` confirmed the target remains
  `status=active`, `kind=handoff`, and still describes T95/T96 as the latest implementation
  context.
- Read-only `lint(action="run", limit=30)` returned many safe-action `superseded_item_still_active`
  rows and applied zero safe actions. The target did not need to appear in that limited page for
  this packet because the graph and direct-search evidence already identify the exact supersession
  relationship.
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

| Area | State After T99 Packet | Evidence | Remaining Risk Or Gate |
|---|---|---|---|
| Rolling handoff | Current handoff remains `019e836a...` | `handoff(get)` and graph evidence | No issue for normal resume, but stale active handoffs still appear in direct search |
| Stale T96 handoff | Exact future archive target frozen | `019e836a...` supersedes `019e835e...`; direct search returns both active handoffs | Requires exact T99 approval phrase before archive |
| Lifecycle cleanup | Still gated | No `memory(action="archive")`; no `lint(action="apply_safe")`; lint applied `0` safe actions | T88/T95/T97 remain separate exact archive packets |
| M6 migration | Still gated | T69/T70 and M6 gates unchanged | Count drift unresolved; no review apply/delete/simplify action allowed |
| Hot path and retrieval | Unchanged | No ranking, `orient`, public MCP, schema/storage/index, document-index, or harness changes | Broad ranking quality and stale historical noise remain open evidence-quality risks |

## Proposed Approved Archive

If and only if the user approves with the exact phrase below, Codex may run one Memory OS archive
write for this single ID:

```text
memory(
  action="archive",
  id="019e835e-81c2-7562-897a-e42c0fe8dc08",
  archive_reason="Superseded by rolling handoff 019e836a-435a-75e1-8702-ced8eabe85cc after T98.",
  archived_by="codex"
)
```

Validation after an approved archive would be limited to:

- `memory(action="get", id="019e835e-81c2-7562-897a-e42c0fe8dc08")` shows archived status and
  archive metadata.
- Direct search no longer returns the target as an active handoff for the tested T98/T99 prompt.
- `handoff(get)` still returns active handoff `019e836a-435a-75e1-8702-ced8eabe85cc`.
- T69, T70, T88, T95, and T97 gates remain unchanged.

## Stop Conditions

Stop without archiving if any of these occur:

- Approval is missing, conditional, ambiguous, or does not include the exact target ID.
- The target is no longer active or no longer superseded by `019e836a...`.
- The requested action would archive more than the single named target.
- The action would run `lint(action="apply_safe")`, broad handoff cleanup, M6 inspection/apply,
  deletion, lifecycle mutation for other memories, ranking changes, `orient` expansion, public MCP
  changes, schema/storage/index changes, document-index behavior changes, or harness writes.

## Approval Question

Reply exactly:

```text
Approve T99: archive handoff 019e835e-81c2-7562-897a-e42c0fe8dc08 only.
```
