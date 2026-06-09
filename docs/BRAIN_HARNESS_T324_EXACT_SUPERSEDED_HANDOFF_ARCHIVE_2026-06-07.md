# T324 Exact Superseded Handoff Archive Batch

Date: 2026-06-07
Status: completed exact lifecycle archive batch

## Question

Can Engram continue reducing residual lifecycle debt through another reviewed exact-target archive
batch without broad cleanup, deletion, ranking changes, harness writes, or release actions?

## Scope

T324 archives exactly five active rolling handoffs that `lint(action="run", limit=15)` reported as
`superseded_item_still_active` findings after T323. Each target was reviewed with `memory(get)`,
successor `memory(get)`, and `graph(around, depth=1)` before archive.

Targets:

- `019e1837-6d1c-7772-a026-4b2fd41c3490`
- `019e184e-4c03-7531-9c2b-e7374cd58007`
- `019e187c-6314-7d40-a213-c7a94409c80c`
- `019e1b0e-2222-7421-8aed-0b8e01b66561`
- `019e1b3b-0e99-7593-bc91-9191019fcfeb`

All five are `handoff` MemoryItems titled `Rolling handoff`.

## Evidence

The review showed direct active successors:

| Archived target | Direct active successor evidence |
| --- | --- |
| `019e1837-6d1c-7772-a026-4b2fd41c3490` | `019e1837-6ec4-7690-85ae-013314c82dcd` is active and has a `supersedes` edge to it. |
| `019e184e-4c03-7531-9c2b-e7374cd58007` | `019e184e-4db7-70b2-ade4-6f44616c2704` is active and has a `supersedes` edge to it. |
| `019e187c-6314-7d40-a213-c7a94409c80c` | `019e187c-6484-7b61-aa38-1cf799e9ce84` is active and has a `supersedes` edge to it. |
| `019e1b0e-2222-7421-8aed-0b8e01b66561` | `019e1b0e-23a2-74b3-a5eb-53e353a4bc79` is active and has a `supersedes` edge to it. |
| `019e1b3b-0e99-7593-bc91-9191019fcfeb` | `019e1b88-1f4e-7dd3-b187-50853b034819` is active and has a `supersedes` edge to it. |

KnowledgeCommit:

```text
019ea28a-171a-7fc3-bee5-beb62bfc48a6
```

Post-archive lint with `limit=10` no longer returned the five target IDs. The sampled queue now
begins with:

```text
019e1b88-1f4e-7dd3-b187-50853b034819
```

## Interpretation

T324 closes exactly the five archived handoffs. It advances lifecycle hygiene but does not claim
broad lifecycle cleanup, direct legacy deletion/deprecation, retrieval/ranking improvement, or
production-complete Brain Harness parity.

## Boundary

T324 does not run `lint apply_safe`, archive any item beyond the five IDs above, delete memory,
change retrieval/ranking behavior, edit repo code, write harness adapters/settings/hooks, launch
native Claude, run `/hooks`, change CI, mark PR #3 ready, merge, tag, publish, or release.
