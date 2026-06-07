# Brain Harness T294 Exact Superseded Handoff Archive - 2026-06-06

## Research Question

Can Engram continue reducing active lifecycle noise by archiving the next small exact batch of
superseded rolling handoffs while preserving the T284 default-deny boundary against broad cleanup?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A seventh exact batch can safely archive reviewed superseded handoffs when each target is active and has a direct incoming `supersedes` edge from an active successor. |
| Null | The remaining superseded-active findings should stay active until a broader lifecycle design exists. |
| Simpler alternative | Stop after T288/T289/T290/T291/T292/T293 and leave the lint queue as known pressure only. |
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
| `019dfc5d-b88d-7ba1-8b8f-29369f66ebe3` | `project:dd-source` | `019dfc63-e051-71c2-8bd0-407debdc2cd3` |
| `019dfc63-e051-71c2-8bd0-407debdc2cd3` | `project:dd-source` | `019dfc66-146b-7480-b28d-6a7f960a5c66` |
| `019dfc66-146b-7480-b28d-6a7f960a5c66` | `project:dd-source` | `019dfc87-9317-7fb3-975c-94f5b1647072` |
| `019dfc87-9317-7fb3-975c-94f5b1647072` | `project:dd-source` | `019dfc87-c510-7c80-9159-9fee36315f0d` |
| `019dfc87-c510-7c80-9159-9fee36315f0d` | `project:dd-source` | `019e019c-43a3-7a30-af48-dec8bbfe432f` |

## Result

T294 archived exactly those five MemoryItems with `archived_by="codex"` and per-item archive
reasons naming the direct superseding memory. The archive action preserved the records and changed
their lifecycle status to `archived`; no records were deleted. The batch audit is recorded as
MemoryItem `019e9e4a-20e2-7a13-b1d7-368664aeb605` and KnowledgeCommit
`019e9e4a-57ad-7b51-a787-6b4859421cfa`.

Post-archive `lint(action="list", limit=20, vault_path="/Users/yuval.meiri/.engram/vault")`
no longer returned any of the five target IDs in the sampled findings. The returned sample
advanced to `019dfc97-4f9b-7301-b401-38179a03aeec`.

Canonical vault compile after the batch reported `1,656` MemoryItems, `576` KnowledgeCommits,
`9` repositories, `32` entities, `79` projects, and `2,356` expected/generated files with zero
user files.

## Branch And Pull-Hint Audit

After `git fetch --prune origin`, the feature branch and its upstream were `0 0` apart by
`git rev-list --left-right --count HEAD...origin/yuval.meiri/memory-os-phase0`;
`origin/main...HEAD` was `0 410`; and `origin/main` was an ancestor of `HEAD`. No pull, merge,
rebase, pull-policy config change, or branch rewrite ran. The recurring pull hint is not current
evidence of a local/upstream divergence on `yuval.meiri/memory-os-phase0`.

## Non-Claims

T294 is exact lifecycle maintenance only. It does not run broad `lint apply_safe`, complete global
lifecycle cleanup, deprecate or delete direct legacy data, change ranking or `orient`, mutate M6
state, run native Claude, edit harness files, change schema/storage/index behavior, mark PR #2
ready for review, or change pull/rebase configuration.
