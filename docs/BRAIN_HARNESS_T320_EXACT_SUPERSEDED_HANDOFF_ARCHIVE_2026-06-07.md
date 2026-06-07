# T320 Exact Superseded Handoff Archive Batch

Date: 2026-06-07
Status: completed exact lifecycle archive batch

## Question

Can Engram reduce residual lifecycle debt without using broad `lint apply_safe`, changing
ranking, deleting memory, or touching harness/runtime state?

## Scope

T320 archives exactly five active rolling handoffs that `lint(action="run", limit=20)` reported as
`superseded_item_still_active` findings. Each target was reviewed with `memory(get)` and
`graph(around, depth=1)` before archive.

Targets:

- `019e1612-f863-7f63-bacb-a6d03ddf1f7c`
- `019e1614-5134-7f32-9ffc-a6d7567f6f7a`
- `019e1618-d7b9-77c1-b795-d2ded5233a7c`
- `019e162b-a94f-7c53-87c0-969e35c8cc6a`
- `019e162e-7f15-7da0-9450-ac98f63062c0`

All five are `handoff` MemoryItems titled `Rolling handoff`, scoped to project `dd-source`, with
`tags=["handoff", "rolling"]`.

## Evidence

The review showed a direct supersession chain:

| Archived target | Direct successor evidence |
| --- | --- |
| `019e1612-f863-7f63-bacb-a6d03ddf1f7c` | `019e1614-5134-7f32-9ffc-a6d7567f6f7a` has a `supersedes` edge to it. |
| `019e1614-5134-7f32-9ffc-a6d7567f6f7a` | `019e1618-d7b9-77c1-b795-d2ded5233a7c` has a `supersedes` edge to it. |
| `019e1618-d7b9-77c1-b795-d2ded5233a7c` | `019e162b-a94f-7c53-87c0-969e35c8cc6a` has a `supersedes` edge to it. |
| `019e162b-a94f-7c53-87c0-969e35c8cc6a` | `019e162e-7f15-7da0-9450-ac98f63062c0` has a `supersedes` edge to it. |
| `019e162e-7f15-7da0-9450-ac98f63062c0` | `019e1681-c15b-7642-ab66-3fd846b72cb1` has a `supersedes` edge to it. |

Each archive used an ID-specific archive reason naming the direct successor and the review tools.

KnowledgeCommit:

```text
019ea1f5-c385-7111-8a1b-6133a87b0c01
```

Post-archive lint with `limit=10` no longer returned the five target IDs. The sampled queue now
begins with:

```text
019e1681-c15b-7642-ab66-3fd846b72cb1
```

## Interpretation

T320 closes exactly the five archived handoffs. It advances lifecycle hygiene but does not claim
broad lifecycle cleanup, direct legacy deletion/deprecation, ranking changes, or production-complete
Brain Harness parity.

## Boundary

T320 does not run `lint apply_safe`, archive any item beyond the five IDs above, delete memory,
change retrieval/ranking behavior, edit repo code, write harness adapters/settings/hooks, launch
native Claude, run `/hooks`, change CI, mark PR #3 ready, merge, tag, publish, or release.
