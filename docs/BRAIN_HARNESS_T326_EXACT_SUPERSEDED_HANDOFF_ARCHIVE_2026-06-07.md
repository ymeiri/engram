# T326 Exact Superseded Handoff Archive Batch

Date: 2026-06-07
Status: completed exact lifecycle archive batch

## Question

Can Engram continue reducing residual lifecycle debt through another reviewed exact-target archive
batch without broad cleanup, deletion, ranking changes, harness writes, or release actions?

## Scope

T326 archives exactly five active rolling handoffs that `lint(action="run", limit=15)` reported as
`superseded_item_still_active` findings after T325. Each target was reviewed with `memory(get)`,
successor `memory(get)`, and `graph(around, depth=1)` before archive.

Targets:

- `019e1d0a-bcb2-79b0-8eca-a624c0229de2`
- `019e1d29-a03c-7110-a58b-0aea4a6b7f05`
- `019e1d3c-eb00-7eb0-90f9-fa7944557b90`
- `019e1d3f-51c7-7050-8108-b667120b7514`
- `019e1d51-b545-7d71-9584-b008c448ad2e`

All five are `handoff` MemoryItems titled `Rolling handoff`.

## Evidence

The review showed direct active successors:

| Archived target | Scope | Direct active successor evidence |
| --- | --- | --- |
| `019e1d0a-bcb2-79b0-8eca-a624c0229de2` | `dd-source` | `019e1d29-a03c-7110-a58b-0aea4a6b7f05` was active at review time and has a `supersedes` edge to it. |
| `019e1d29-a03c-7110-a58b-0aea4a6b7f05` | `dd-source` | `019e7cd4-d927-7322-9354-f8b8d054c099` is active and has a `supersedes` edge to it. |
| `019e1d3c-eb00-7eb0-90f9-fa7944557b90` | `voice-layer` | `019e1d3f-51c7-7050-8108-b667120b7514` was active at review time and has a `supersedes` edge to it. |
| `019e1d3f-51c7-7050-8108-b667120b7514` | `voice-layer` | `019e1d51-b545-7d71-9584-b008c448ad2e` was active at review time and has a `supersedes` edge to it. |
| `019e1d51-b545-7d71-9584-b008c448ad2e` | `voice-layer` | `019e1d56-4fff-7dc3-822c-383132c57a25` is active and has a `supersedes` edge to it. |

KnowledgeCommit:

```text
019ea29c-85ae-7ff3-bba9-d8ea85453f75
```

Post-archive lint with `limit=10` no longer returned the five target IDs. The sampled queue now
begins with:

```text
019e1d56-4fff-7dc3-822c-383132c57a25
```

## Interpretation

T326 closes exactly the five archived handoffs. It advances lifecycle hygiene but does not claim
broad lifecycle cleanup, direct legacy deletion/deprecation, retrieval/ranking improvement, or
production-complete Brain Harness parity.

Some successors were themselves archived in the same batch after review. That is acceptable for
this exact cleanup batch because each target had a direct active successor at review time and the
batch records the successor chain explicitly. The post-batch sampled lint queue begins at the next
unarchived successor candidate.

## Boundary

T326 does not run `lint apply_safe`, archive any item beyond the five IDs above, delete memory,
change retrieval/ranking behavior, edit repo code, write harness adapters/settings/hooks, launch
native Claude, run `/hooks`, change CI, mark PR #3 ready, merge, tag, publish, or release.
