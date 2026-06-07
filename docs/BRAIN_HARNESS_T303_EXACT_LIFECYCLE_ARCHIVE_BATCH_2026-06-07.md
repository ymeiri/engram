# T303 Exact Lifecycle Archive Batch

Date: 2026-06-07

## Research Question

Can the next sampled lifecycle-noise batch be reduced without broad cleanup, ranking changes,
native Claude execution, or direct legacy deletion?

## Decision

Yes. T303 archives exactly five active `project:dd-source` rolling-handoff MemoryItems that were
already directly superseded by active successor handoffs. This is exact lifecycle hygiene only.
It does not run broad `lint apply_safe`, delete records, deprecate legacy layers, execute native
Claude, validate hooks or host labels, or change code behavior.

## Targets

| Archived target | Scope | Direct active successor |
| --- | --- | --- |
| `019e01a0-5d8c-76f3-b537-935a53207cc0` | `project:dd-source` | `019e01d6-adc4-7971-aca3-c663b2be52c5` |
| `019e01d6-adc4-7971-aca3-c663b2be52c5` | `project:dd-source` | `019e01db-1e53-7c23-b6c0-b4ba8d58b0bc` |
| `019e01db-1e53-7c23-b6c0-b4ba8d58b0bc` | `project:dd-source` | `019e01f2-cfa4-7de0-b073-3bc1926e5c3c` |
| `019e01f2-cfa4-7de0-b073-3bc1926e5c3c` | `project:dd-source` | `019e01f4-5fd7-77c2-8491-2f66a2eebda1` |
| `019e01f4-5fd7-77c2-8491-2f66a2eebda1` | `project:dd-source` | `019e02b0-22ab-72c0-8105-1e7909dd4279` |

## Evidence

- `lint(action=run, limit=25, vault_path=/Users/yuval.meiri/.engram/vault)` returned these five
  items as the first sampled `superseded_item_still_active` findings.
- `memory(action=get)` confirmed all five targets were active rolling handoffs before archive.
- `graph(action=around, depth=1)` confirmed each target had a direct incoming `supersedes` edge
  from the successor listed above.
- `memory(action=get)` confirmed the successor outside the selected target set,
  `019e02b0-22ab-72c0-8105-1e7909dd4279`, was an active rolling handoff.
- `memory(action=archive)` archived exactly the five target IDs with successor-specific reasons.
- Post-archive `memory(action=get)` confirmed all five target IDs are now `status=archived`.
- Post-archive `lint(action=run, limit=12, vault_path=/Users/yuval.meiri/.engram/vault)` advanced
  to `019e02b0-22ab-72c0-8105-1e7909dd4279` as the first sampled candidate and no longer returned
  the five T303 target IDs.

## Non-Claims

- T303 does not complete exhaustive lifecycle cleanup.
- T303 does not authorize broad `lint apply_safe`.
- T303 does not delete MemoryItems or direct legacy data.
- T303 does not change ranking, `orient`, schema, storage, MCP behavior, or harness behavior.
- T303 does not execute native Claude, validate effective hooks, or prove live host labels.
