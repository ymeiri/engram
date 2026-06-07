# T325 Exact Superseded Handoff Archive Batch

Date: 2026-06-07
Status: completed exact lifecycle archive batch

## Question

Can Engram continue reducing residual lifecycle debt through another reviewed exact-target archive
batch without broad cleanup, deletion, ranking changes, harness writes, or release actions?

## Scope

T325 archives exactly five active rolling handoffs that `lint(action="run", limit=15)` reported as
`superseded_item_still_active` findings after T324. Each target was reviewed with `memory(get)`,
successor `memory(get)`, and `graph(around, depth=1)` before archive.

Targets:

- `019e1b88-1f4e-7dd3-b187-50853b034819`
- `019e1c16-e287-72e1-8e69-6d6026cd39bb`
- `019e1c1b-7ce7-7e41-b436-57825899f151`
- `019e1c47-6680-7651-abbd-83060f3126ef`
- `019e1c51-d266-7fd3-a327-d89f544967cb`

All five are `handoff` MemoryItems titled `Rolling handoff`.

## Evidence

The review showed direct active successors:

| Archived target | Direct active successor evidence |
| --- | --- |
| `019e1b88-1f4e-7dd3-b187-50853b034819` | `019e1c1b-7ce7-7e41-b436-57825899f151` is active and has a `supersedes` edge to it. |
| `019e1c16-e287-72e1-8e69-6d6026cd39bb` | `019e1d0a-bcb2-79b0-8eca-a624c0229de2` is active and has a `supersedes` edge to it. |
| `019e1c1b-7ce7-7e41-b436-57825899f151` | `019e1c47-6680-7651-abbd-83060f3126ef` is active and has a `supersedes` edge to it. |
| `019e1c47-6680-7651-abbd-83060f3126ef` | `019e1c51-d266-7fd3-a327-d89f544967cb` is active and has a `supersedes` edge to it. |
| `019e1c51-d266-7fd3-a327-d89f544967cb` | `019e1d3c-eb00-7eb0-90f9-fa7944557b90` is active and has a `supersedes` edge to it. |

KnowledgeCommit:

```text
019ea292-d32f-7901-9183-d99527315d22
```

Post-archive lint with `limit=10` no longer returned the five target IDs. The sampled queue now
begins with:

```text
019e1d0a-bcb2-79b0-8eca-a624c0229de2
```

## Interpretation

T325 closes exactly the five archived handoffs. It advances lifecycle hygiene but does not claim
broad lifecycle cleanup, direct legacy deletion/deprecation, retrieval/ranking improvement, or
production-complete Brain Harness parity.

## Boundary

T325 does not run `lint apply_safe`, archive any item beyond the five IDs above, delete memory,
change retrieval/ranking behavior, edit repo code, write harness adapters/settings/hooks, launch
native Claude, run `/hooks`, change CI, mark PR #3 ready, merge, tag, publish, or release.
