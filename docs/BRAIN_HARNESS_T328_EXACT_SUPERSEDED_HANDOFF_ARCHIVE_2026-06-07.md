# T328 Exact Superseded Handoff Archive Batch

Date: 2026-06-07
Status: completed exact lifecycle archive batch

## Question

Can Engram continue reducing residual lifecycle debt through another reviewed exact-target archive
batch without broad cleanup, deletion, ranking changes, harness writes, or release actions?

## Scope

T328 archives exactly five active rolling handoffs that `lint(action="run", limit=10)` reported as
`superseded_item_still_active` findings after T327. Each target was reviewed with `memory(get)`,
successor `memory(get)`, and `graph(around, depth=1)` before archive.

Targets:

- `019e5db0-070d-7211-acb0-a69f5b575c5d`
- `019e6993-6d69-78e1-a29d-93a61e2a6413`
- `019e6994-a7b5-7530-9b74-483d48709d13`
- `019e6995-67c0-7221-8922-1cad83d54229`
- `019e6995-ff6d-7db0-84bc-06475ffe4fa1`

All five are `handoff` MemoryItems titled `Rolling handoff`.

## Evidence

The review showed direct active successors:

| Archived target | Scope | Direct active successor evidence |
| --- | --- | --- |
| `019e5db0-070d-7211-acb0-a69f5b575c5d` | `engram-dogfood-baf008-claude-orient` | `019e5dba-1b29-7d50-bead-d0daf419f66e` is active and has a `supersedes` edge to it. |
| `019e6993-6d69-78e1-a29d-93a61e2a6413` | `codex-claude-bridge` | `019e6994-a7b5-7530-9b74-483d48709d13` was active at review time and has a `supersedes` edge to it. |
| `019e6994-a7b5-7530-9b74-483d48709d13` | `codex-claude-bridge` | `019e6995-67c0-7221-8922-1cad83d54229` was active at review time and has a `supersedes` edge to it. |
| `019e6995-67c0-7221-8922-1cad83d54229` | `codex-claude-bridge` | `019e6995-ff6d-7db0-84bc-06475ffe4fa1` was active at review time and has a `supersedes` edge to it. |
| `019e6995-ff6d-7db0-84bc-06475ffe4fa1` | `codex-claude-bridge` | `019e6997-984b-7cd0-9c3f-6b08cf5959d6` is active and has a `supersedes` edge to it. |

KnowledgeCommit:

```text
019ea2ae-78ee-7900-86ff-ffc19b8bc33c
```

Post-archive lint with `limit=10` no longer returned the five target IDs. The sampled queue now
begins with:

```text
019e6997-984b-7cd0-9c3f-6b08cf5959d6
```

## Interpretation

T328 closes exactly the five archived handoffs. It advances lifecycle hygiene but does not claim
broad lifecycle cleanup, direct legacy deletion/deprecation, retrieval/ranking improvement, or
production-complete Brain Harness parity.

Some successors were themselves archived in the same batch after review. That is acceptable for
this exact cleanup batch because each target had a direct active successor at review time and the
batch records the successor chain explicitly. The post-batch sampled lint queue begins at the next
unarchived successor candidate.

## Boundary

T328 does not run `lint apply_safe`, archive any item beyond the five IDs above, delete memory,
change retrieval/ranking behavior, edit repo code, write harness adapters/settings/hooks, launch
native Claude, run `/hooks`, change CI, mark PR #3 ready, merge, tag, publish, or release.
