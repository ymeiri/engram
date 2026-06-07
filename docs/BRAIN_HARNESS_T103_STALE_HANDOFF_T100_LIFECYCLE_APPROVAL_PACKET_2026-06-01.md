# Brain Harness T103 Stale Handoff T100 Lifecycle Approval Packet

Status: Pending explicit user approval. No archive or lifecycle write has been run.

Scope: Freeze exactly one superseded rolling handoff target for a future archive decision:
`019e8378-b2f0-7260-a887-4abdf6c0e4e2`.

This packet does not authorize broad stale-handoff cleanup, `lint(action="apply_safe")`, M6
inspection/apply/deletion, T69 inspection, T70 document indexing, lifecycle mutation beyond the
single named target, ranking changes, `orient` expansion, public MCP changes, schema/storage/index
changes, document-index behavior changes, or harness adapter/hook writes.

## Research Question

After T102, is handoff `019e8378-b2f0-7260-a887-4abdf6c0e4e2` a safely identified stale active
handoff target that should be presented to the user as an exact future archive decision, without
archiving it now?

## Hypotheses

| Hypothesis | Prediction |
|---|---|
| Preferred | T102 handoff `019e8381-5e35-78d2-b4f9-7ef949fc6e6b` supersedes `019e8378-b2f0-7260-a887-4abdf6c0e4e2`, while direct search still surfaces both active handoffs. A docs-only exact approval packet improves lifecycle decision quality without mutating memory. |
| Null | `019e8378-b2f0-7260-a887-4abdf6c0e4e2` is not clearly superseded or does not surface as active/noisy enough to justify an approval packet. |
| Simpler alternative | Rely on the T102 current-plan memory and do not create another approval packet. |
| Failure | The packet is mistaken for archive approval, broad handoff cleanup, migration approval, ranking evidence, or permission to apply lint safe actions. |

## Measurement

- Lean `orient` trace `019e8385-c911-70c1-b935-ffba40c0ecd1` returned T102 current-plan memory
  `019e8383-ebbf-7bc2-8e19-7d590a747b49` first and reported no open obligations.
- Direct current-plan search trace `019e8385-f8bc-7d71-8462-88fc6445ca12` returned T102
  current-plan memory first, active handoff `019e8381...` second, and stale active handoff
  `019e8378...` third at the same score as other rolling handoffs.
- Direct architecture/gate search trace `019e8385-f9a6-7af0-9535-bbfecb1c59d8` returned T102
  current-plan memory first, active handoff `019e8381...` second, and stale active handoff
  `019e8378...` third at the same score as the active handoff.
- Direct Memory OS implementation-plan search trace `019e8385-fa57-7b31-9eea-ade864f55955`
  returned T102 current-plan memory first, active M6 gate second, and stale handoff noise below.
- Direct recent-risk search trace `019e8385-fb3f-75a0-b00f-ead5466cb8b4` returned active handoff
  `019e8381...` first and stale active handoff `019e8378...` second.
- `memory(action="changes_since", timestamp="2026-06-01T14:10:43.41204Z")` trace
  `019e8386-4225-7560-8d11-985d08638565` returned zero newer memory items or commits before this
  packet, so no other writer had changed the relevant state.
- `memory(action="get", id="019e8378-b2f0-7260-a887-4abdf6c0e4e2")` confirmed the target remains
  `status=active`, `kind=handoff`, and still describes T99/T100 as the latest implementation
  context.
- `memory(action="get", id="019e8381-5e35-78d2-b4f9-7ef949fc6e6b")` confirmed the active T102
  handoff supersedes `019e8378...`.
- `graph(action="around", node="019e8381-5e35-78d2-b4f9-7ef949fc6e6b", depth=1)` showed a
  `supersedes` edge from `019e8381...` to `019e8378...`.
- Read-only `lint(action="run", limit=30)` returned many `superseded_item_still_active` findings
  with `safe_action=archive_memory_item` and applied zero safe actions. The target did not need to
  appear in that limited page because direct search, `memory(get)`, and graph evidence already
  identify the exact supersession relationship.
- Source inspection confirmed `memory(action="archive")` writes archive metadata for one required
  ID, while `lint(action="apply_safe", write=true)` can archive every matching safe-action finding
  in a report. This packet therefore asks only for a future single-target `memory(action="archive")`,
  not a lint safe-action sweep.
- AI Council recall surfaced prior T88/T38 guidance: strict target-local approval packets must not
  be broadened into payload expansion, lifecycle cleanup, broad ranking, migration, or approval
  inference.
- Claude Bridge read-only critique agreed that a docs-only T103 packet is safe and directly
  anticipated by T102's matrix, with the key caveat that the future archive reason must reference
  T102 handoff `019e8381...` and require its own exact approval phrase.
- Git status before this packet was clean except untracked root `AGENTS.md`, which remains untouched
  and unstaged unless the user explicitly asks.

## Completion Matrix Delta

| Area | State After T103 Packet | Evidence | Remaining Risk Or Gate |
|---|---|---|---|
| Rolling handoff | Current handoff remains `019e8381...` | `handoff(get)`, `memory(get)`, and graph evidence | No issue for normal resume, but stale active handoffs still appear in direct search |
| Stale T100 handoff | Exact future archive target frozen | `019e8381...` supersedes `019e8378...`; direct search returns both active handoffs | Requires exact T103 approval phrase before archive |
| Lifecycle cleanup | Still gated | No `memory(action="archive")`; no `lint(action="apply_safe")`; lint applied `0` safe actions | T88/T95/T97/T99/T101 remain separate exact archive packets |
| M6 migration | Still gated | T69/T70 and M6 gates unchanged | Count drift unresolved; no review apply/delete/simplify action allowed |
| Hot path and retrieval | Unchanged | No ranking, `orient`, public MCP, schema/storage/index, document-index, or harness changes | Broad ranking quality and stale historical noise remain open evidence-quality risks |

## Proposed Approved Archive

If and only if the user approves with the exact phrase below, Codex may run one Memory OS archive
write for this single ID:

```text
memory(
  action="archive",
  id="019e8378-b2f0-7260-a887-4abdf6c0e4e2",
  archive_reason="Superseded by rolling handoff 019e8381-5e35-78d2-b4f9-7ef949fc6e6b after T102.",
  archived_by="codex"
)
```

Validation after an approved archive would be limited to:

- `memory(action="get", id="019e8378-b2f0-7260-a887-4abdf6c0e4e2")` shows archived status and
  archive metadata.
- Direct search no longer returns the target as an active handoff for the tested T102/T103 prompt.
- `handoff(get)` still returns active handoff `019e8381-5e35-78d2-b4f9-7ef949fc6e6b`.
- T69, T70, T88, T95, T97, T99, and T101 gates remain unchanged.

## Stop Conditions

Stop without archiving if any of these occur:

- Approval is missing, conditional, ambiguous, or does not include the exact target ID.
- The target is no longer active or no longer superseded by `019e8381...`.
- The requested action would archive more than the single named target.
- The action would run `lint(action="apply_safe")`, broad handoff cleanup, M6 inspection/apply,
  deletion, lifecycle mutation for other memories, ranking changes, `orient` expansion, public MCP
  changes, schema/storage/index changes, document-index behavior changes, or harness writes.

## Approval Question

Reply exactly:

```text
Approve T103: archive handoff 019e8378-b2f0-7260-a887-4abdf6c0e4e2 only.
```
