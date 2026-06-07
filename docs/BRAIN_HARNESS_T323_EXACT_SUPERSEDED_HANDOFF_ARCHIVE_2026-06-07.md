# T323 Exact Superseded Handoff Archive Batch

Date: 2026-06-07
Status: completed exact lifecycle archive batch

## Question

Can Engram continue reducing residual lifecycle debt through another reviewed exact-target archive
batch without broad cleanup, deletion, ranking changes, harness writes, or release actions?

## Scope

T323 archives exactly five active rolling handoffs that `lint(action="run", limit=15)` reported as
`superseded_item_still_active` findings after T322. Each target was reviewed with `memory(get)` and
`graph(around, depth=1)` before archive. Successor MemoryItems were fetched where they were not
already in the target batch.

Targets:

- `019e17da-5dc1-7b30-a440-f980f16bfefb`
- `019e17dd-0e7c-7773-8fa9-df8196d3c474`
- `019e17ea-e7f6-7c30-8635-1ad43345ee70`
- `019e17eb-00d7-7230-9257-a8188bee6811`
- `019e1825-056e-7ac3-a5f0-053e4703afef`

All five are `handoff` MemoryItems titled `Rolling handoff`.

## Evidence

The review showed direct active successors:

| Archived target | Direct active successor evidence |
| --- | --- |
| `019e17da-5dc1-7b30-a440-f980f16bfefb` | `019e17dd-0e7c-7773-8fa9-df8196d3c474` is active and has a `supersedes` edge to it. |
| `019e17dd-0e7c-7773-8fa9-df8196d3c474` | `019e17ea-e7f6-7c30-8635-1ad43345ee70` is active and has a `supersedes` edge to it. |
| `019e17ea-e7f6-7c30-8635-1ad43345ee70` | `019e17eb-00d7-7230-9257-a8188bee6811` is active and has a `supersedes` edge to it. |
| `019e17eb-00d7-7230-9257-a8188bee6811` | `019e184b-7d2b-7203-9249-f88a40071b18` is active and has a `supersedes` edge to it. |
| `019e1825-056e-7ac3-a5f0-053e4703afef` | `019e1825-071b-70d0-b83d-e65716ab1fb7` is active and has a `supersedes` edge to it. |

KnowledgeCommit:

```text
019ea209-45d3-7ba0-bb5d-330790b4cf99
```

Post-archive lint with `limit=10` no longer returned the five target IDs. The sampled queue now
begins with:

```text
019e1837-6d1c-7772-a026-4b2fd41c3490
```

## Interpretation

T323 closes exactly the five archived handoffs. It advances lifecycle hygiene but does not claim
broad lifecycle cleanup, direct legacy deletion/deprecation, retrieval/ranking improvement, or
production-complete Brain Harness parity.

## Boundary

T323 does not run `lint apply_safe`, archive any item beyond the five IDs above, delete memory,
change retrieval/ranking behavior, edit repo code, write harness adapters/settings/hooks, launch
native Claude, run `/hooks`, change CI, mark PR #3 ready, merge, tag, publish, or release.
