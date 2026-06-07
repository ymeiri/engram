# T329 Exact Superseded Handoff Archive Batch

Date: 2026-06-07
Status: completed exact lifecycle archive batch

## Question

Can Engram continue reducing residual lifecycle debt through another reviewed exact-target archive
batch without broad cleanup, deletion, ranking changes, harness writes, or release actions?

## Scope

T329 archives exactly five active rolling handoffs that `lint(action="run", limit=10)` reported as
`superseded_item_still_active` findings after T328. Each target was reviewed with `memory(get)`,
successor `memory(get)`, and `graph(around, depth=1)` before archive.

Targets:

- `019e6997-984b-7cd0-9c3f-6b08cf5959d6`
- `019e6a47-b8a9-7382-8f7f-62a3dbd0dce5`
- `019e7cd4-d927-7322-9354-f8b8d054c099`
- `019e7ce0-b1a0-7d63-baac-d04ba7029b43`
- `019e7ce8-1b0b-7922-aa07-3cb161e36601`

All five are `handoff` MemoryItems titled `Rolling handoff`.

## Evidence

The review showed direct active successors:

| Archived target | Scope | Direct active successor evidence |
| --- | --- | --- |
| `019e6997-984b-7cd0-9c3f-6b08cf5959d6` | `codex-claude-bridge` | `019e6998-6c65-7890-aee6-8e2eee12e3ea` is active and has a `supersedes` edge to it. |
| `019e6a47-b8a9-7382-8f7f-62a3dbd0dce5` | `astro-companion` | `019e7cdb-c3f2-7a80-8223-02650c7a83ce` is active and has a `supersedes` edge to it. |
| `019e7cd4-d927-7322-9354-f8b8d054c099` | `dd-source` | `019e7ce0-b1a0-7d63-baac-d04ba7029b43` was active at review time and has a `supersedes` edge to it. |
| `019e7ce0-b1a0-7d63-baac-d04ba7029b43` | `dd-source` | `019e7ce8-1b0b-7922-aa07-3cb161e36601` was active at review time and has a `supersedes` edge to it. |
| `019e7ce8-1b0b-7922-aa07-3cb161e36601` | `dd-source` | `019e7cf7-560c-70e2-bbeb-3448f4637055` is active and has a `supersedes` edge to it. |

KnowledgeCommit:

```text
019ea2b7-45d7-7c00-9adc-8d6387525d20
```

Post-archive lint with `limit=10` no longer returned the five target IDs. The sampled queue now
begins with:

```text
019e7cf7-560c-70e2-bbeb-3448f4637055
```

## Interpretation

T329 closes exactly the five archived handoffs. It advances lifecycle hygiene but does not claim
broad lifecycle cleanup, direct legacy deletion/deprecation, retrieval/ranking improvement, or
production-complete Brain Harness parity.

Some successors were themselves archived in the same batch after review. That is acceptable for
this exact cleanup batch because each target had a direct active successor at review time and the
batch records the successor chain explicitly. The post-batch sampled lint queue begins at the next
unarchived successor candidate.

## Boundary

T329 does not run `lint apply_safe`, archive any item beyond the five IDs above, delete memory,
change retrieval/ranking behavior, edit repo code, write harness adapters/settings/hooks, launch
native Claude, run `/hooks`, change CI, mark PR #3 ready, merge, tag, publish, or release.
