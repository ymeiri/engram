# Brain Harness T293 Exact Superseded Handoff Archive - 2026-06-06

## Research Question

Can Engram continue reducing active lifecycle noise by archiving the next small exact batch of
superseded rolling handoffs while preserving the T284 default-deny boundary against broad cleanup?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A sixth exact batch can safely archive reviewed superseded handoffs when each target is active and has a direct incoming `supersedes` edge from an active successor. |
| Null | The remaining superseded-active findings should stay active until a broader lifecycle design exists. |
| Simpler alternative | Stop after T288/T289/T290/T291/T292 and leave the lint queue as known pressure only. |
| Failure | The batch archives a current handoff, relies on inferred supersession, or widens into broad cleanup without per-target evidence. |

## Preflight Evidence

- Fresh `lint(action="list", limit=20, vault_path="/Users/yuval.meiri/.engram/vault")`
  exposed the next `superseded_item_still_active` findings, with zero safe actions applied.
- Each selected target was fetched with `memory(action="get")` and had `status="active"` before
  archive.
- Each selected target had a direct incoming `supersedes` edge in `graph(action="around",
  depth=1)`.
- Each direct superseding memory was fetched or was already in the reviewed batch and was active
  before the archive action.

## Archived Exact Targets

| Archived target | Scope | Direct superseding memory |
| --- | --- | --- |
| `019ddebe-5159-71a1-a593-03d5a38ad305` | `project:claude-marketplace` | `019ddec0-36a0-7611-a886-60fc2b3d5157` |
| `019ddec0-36a0-7611-a886-60fc2b3d5157` | `project:claude-marketplace` | `019ddec3-78bf-7021-a157-50be5e2b3e2f` |
| `019ddec3-78bf-7021-a157-50be5e2b3e2f` | `project:claude-marketplace` | `019e0319-0638-7272-bfbe-6995539a32d2` |
| `019df80f-bb2d-7683-b802-4f4de39469df` | `project:dd-source` | `019dfc5b-99e4-71b1-aa1b-7d0caf596139` |
| `019dfc5b-99e4-71b1-aa1b-7d0caf596139` | `project:dd-source` | `019dfc5d-b88d-7ba1-8b8f-29369f66ebe3` |

## Result

T293 archived exactly those five MemoryItems with `archived_by="codex"` and per-item archive
reasons naming the direct superseding memory. The archive action preserved the records and changed
their lifecycle status to `archived`; no records were deleted. The batch audit is recorded as
MemoryItem `019e9e19-785d-7010-b831-dcd94ce8bcd9` and KnowledgeCommit
`019e9e19-a913-7111-954d-e95c9a6a9e07`.

Post-archive `lint(action="list", limit=20, vault_path="/Users/yuval.meiri/.engram/vault")`
no longer returned any of the five target IDs in the sampled findings. The third target's direct
successor was `019e0319-0638-7272-bfbe-6995539a32d2`, which skipped later lint rows, so the batch
did not assume one contiguous successor chain. The returned sample advanced to
`019dfc5d-b88d-7ba1-8b8f-29369f66ebe3`.

Canonical vault compile after the batch reported `1,653` MemoryItems, `574` KnowledgeCommits,
`9` repositories, `32` entities, `79` projects, and `2,351` expected/generated files with zero
user files.

## Branch And Pull-Hint Audit

After `git fetch --prune origin`, the feature branch and its upstream were `0 0` apart by
`git rev-list --left-right --count HEAD...origin/yuval.meiri/memory-os-phase0`;
`origin/main...HEAD` was `0 409`; and `origin/main` was an ancestor of `HEAD`. No pull, merge,
rebase, pull-policy config change, or branch rewrite ran. The recurring pull hint is not current
evidence of a local/upstream divergence on `yuval.meiri/memory-os-phase0`.

## Non-Claims

T293 is exact lifecycle maintenance only. It does not run broad `lint apply_safe`, complete global
lifecycle cleanup, deprecate or delete direct legacy data, change ranking or `orient`, mutate M6
state, run native Claude, edit harness files, change schema/storage/index behavior, mark PR #2
ready for review, or change pull/rebase configuration.
