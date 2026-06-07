# T327 Exact Superseded Handoff Archive Batch

Date: 2026-06-07
Status: completed exact lifecycle archive batch

## Question

Can Engram continue reducing residual lifecycle debt through another reviewed exact-target archive
batch without broad cleanup, deletion, ranking changes, harness writes, or release actions?

## Scope

T327 archives exactly five active rolling handoffs that `lint(action="run", limit=15)` reported as
`superseded_item_still_active` findings after T326. Each target was reviewed with `memory(get)`,
successor `memory(get)`, and `graph(around, depth=1)` before archive.

Targets:

- `019e1d56-4fff-7dc3-822c-383132c57a25`
- `019e2088-da67-7d23-9e43-3082d9157208`
- `019e212b-a519-7341-9524-4a028685580b`
- `019e212e-d7d4-75d1-962d-219413f93d4f`
- `019e5daf-f50b-7a22-82a3-5d62ffe9a8bb`

All five are `handoff` MemoryItems titled `Rolling handoff`.

## Evidence

The review showed direct active successors:

| Archived target | Scope | Direct active successor evidence |
| --- | --- | --- |
| `019e1d56-4fff-7dc3-822c-383132c57a25` | `voice-layer` | `019e2088-da67-7d23-9e43-3082d9157208` was active at review time and has a `supersedes` edge to it. |
| `019e2088-da67-7d23-9e43-3082d9157208` | `voice-layer` | `019e211d-c6b9-7190-948a-5bbebc00c604` is active and has a `supersedes` edge to it. |
| `019e212b-a519-7341-9524-4a028685580b` | `code-gen-backend-main.HoaAZV` | `019e212e-d7d4-75d1-962d-219413f93d4f` was active at review time and has a `supersedes` edge to it. |
| `019e212e-d7d4-75d1-962d-219413f93d4f` | `code-gen-backend-main.HoaAZV` | `019e2131-d191-7020-94ac-df06eeecd82a` is active and has a `supersedes` edge to it. |
| `019e5daf-f50b-7a22-82a3-5d62ffe9a8bb` | `engram-dogfood-baf008-claude-orient` | `019e5db0-070d-7211-acb0-a69f5b575c5d` is active and has a `supersedes` edge to it. |

KnowledgeCommit:

```text
019ea2a4-9491-7021-ab93-968f0ef6281d
```

Post-archive lint with `limit=10` no longer returned the five target IDs. The sampled queue now
begins with:

```text
019e5db0-070d-7211-acb0-a69f5b575c5d
```

## Interpretation

T327 closes exactly the five archived handoffs. It advances lifecycle hygiene but does not claim
broad lifecycle cleanup, direct legacy deletion/deprecation, retrieval/ranking improvement, or
production-complete Brain Harness parity.

Some successors were themselves archived in the same batch after review. That is acceptable for
this exact cleanup batch because each target had a direct active successor at review time and the
batch records the successor chain explicitly. The post-batch sampled lint queue begins at the next
unarchived successor candidate.

## Boundary

T327 does not run `lint apply_safe`, archive any item beyond the five IDs above, delete memory,
change retrieval/ranking behavior, edit repo code, write harness adapters/settings/hooks, launch
native Claude, run `/hooks`, change CI, mark PR #3 ready, merge, tag, publish, or release.
