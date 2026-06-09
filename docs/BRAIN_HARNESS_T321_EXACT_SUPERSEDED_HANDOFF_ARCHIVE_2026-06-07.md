# T321 Exact Superseded Handoff Archive Batch

Date: 2026-06-07
Status: completed exact lifecycle archive batch

## Question

Can Engram continue reducing residual lifecycle debt through reviewed exact-target archive batches
without broad cleanup, deletion, ranking changes, harness writes, or release actions?

## Scope

T321 archives exactly five active rolling handoffs that `lint(action="run", limit=15)` reported as
`superseded_item_still_active` findings after T320. Each target was reviewed with `memory(get)`,
successor `memory(get)`, and `graph(around, depth=1)` before archive.

Targets:

- `019e1681-c15b-7642-ab66-3fd846b72cb1`
- `019e168a-eecf-7d42-a52c-80037535fcf2`
- `019e169d-b3c0-7962-b74b-645f1957b7b8`
- `019e176d-f41a-7bb3-b22f-65d7b1bff9e6`
- `019e179d-f906-7063-b00c-3c879ca83e1c`

All five are `handoff` MemoryItems titled `Rolling handoff`.

## Evidence

The review showed direct active successors:

| Archived target | Direct active successor evidence |
| --- | --- |
| `019e1681-c15b-7642-ab66-3fd846b72cb1` | `019e168a-eecf-7d42-a52c-80037535fcf2` is active and has a `supersedes` edge to it. |
| `019e168a-eecf-7d42-a52c-80037535fcf2` | `019e169d-b3c0-7962-b74b-645f1957b7b8` is active and has a `supersedes` edge to it. |
| `019e169d-b3c0-7962-b74b-645f1957b7b8` | `019e179d-f906-7063-b00c-3c879ca83e1c` is active and has a `supersedes` edge to it. |
| `019e176d-f41a-7bb3-b22f-65d7b1bff9e6` | `019e1771-7584-7f61-bfea-304afdd5cf4e` is active and has a `supersedes` edge to it. |
| `019e179d-f906-7063-b00c-3c879ca83e1c` | `019e1c16-e287-72e1-8e69-6d6026cd39bb` is active and has a `supersedes` edge to it. |

KnowledgeCommit:

```text
019ea1fd-1070-7913-bb27-9d703bc58439
```

Post-archive lint with `limit=10` no longer returned the five target IDs. The sampled queue now
begins with:

```text
019e17da-5dc1-7b30-a440-f980f16bfefb
```

## Interpretation

T321 closes exactly the five archived handoffs. It advances lifecycle hygiene but does not claim
broad lifecycle cleanup, direct legacy deletion/deprecation, retrieval/ranking improvement, or
production-complete Brain Harness parity.

## Boundary

T321 does not run `lint apply_safe`, archive any item beyond the five IDs above, delete memory,
change retrieval/ranking behavior, edit repo code, write harness adapters/settings/hooks, launch
native Claude, run `/hooks`, change CI, mark PR #3 ready, merge, tag, publish, or release.
