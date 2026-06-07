# T301 Exact Lifecycle Archive Batch

Date: 2026-06-07

## Research Question

Can the next sampled lifecycle-noise batch be reduced without broad cleanup, ranking changes,
or direct legacy deletion?

## Decision

Yes. T301 archives exactly five active rolling-handoff MemoryItems that were already directly
superseded by active successor handoffs. This is production lifecycle hygiene only. It does not
change beta scope, mark PR #2 ready, run broad `lint apply_safe`, delete records, or deprecate
legacy layers.

## Targets

| Archived target | Scope | Direct active successor |
| --- | --- | --- |
| `019dfd38-fc3d-7352-83a6-c9bbd16349ea` | `project:tmp` | `019dfd39-d183-7d42-bf44-87950acc27ef` |
| `019dfd39-d183-7d42-bf44-87950acc27ef` | `project:tmp` | `019dfd3a-eb89-7bd2-85d1-4420c24c4e5d` |
| `019dfd3a-eb89-7bd2-85d1-4420c24c4e5d` | `project:tmp` | `019dfd3b-7502-7cf2-a097-9ffdf2458729` |
| `019dfd3b-7502-7cf2-a097-9ffdf2458729` | `project:tmp` | `019dfd3d-568f-7653-a145-38d815a0e9ea` |
| `019e019c-43a3-7a30-af48-dec8bbfe432f` | `project:dd-source` | `019e01a0-5d8c-76f3-b537-935a53207cc0` |

## Evidence

- `lint(action=run, limit=20, vault_path=/Users/yuval.meiri/.engram/vault)` returned these five
  items in the sampled `superseded_item_still_active` queue.
- `memory(action=get)` confirmed all five targets were active rolling handoffs before archive.
- `graph(action=around, depth=1)` confirmed each target had a direct incoming `supersedes`
  edge from the successor listed above.
- `memory(action=get)` confirmed the two successors not already in the selected target set,
  `019dfd3d-568f-7653-a145-38d815a0e9ea` and
  `019e01a0-5d8c-76f3-b537-935a53207cc0`, were active rolling handoffs.
- `memory(action=archive)` archived exactly the five target IDs with successor-specific reasons.
- Post-archive `memory(action=get)` confirmed all five target IDs are now `status=archived`.
- Post-archive `lint(action=run, limit=10, vault_path=/Users/yuval.meiri/.engram/vault)`
  advanced to `019e01a0-5d8c-76f3-b537-935a53207cc0` as the first sampled candidate and no
  longer returned the five T301 target IDs.
- Post-archive `vault(action=compile, vault_path=/Users/yuval.meiri/.engram/vault)` refreshed
  the generated vault projection.

## Non-Claims

- T301 does not complete exhaustive lifecycle cleanup.
- T301 does not authorize broad `lint apply_safe`.
- T301 does not delete MemoryItems or direct legacy data.
- T301 does not change ranking, `orient`, schema, storage, MCP behavior, or harness behavior.
- T301 does not mark PR #2 ready or merge it.
