# Brain Harness T290 Exact Superseded Handoff Archive - 2026-06-06

## Research Question

Can Engram continue reducing active lifecycle noise by archiving the next small exact batch of
superseded rolling handoffs while preserving the T284 default-deny boundary against broad cleanup?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A third exact batch can safely archive reviewed superseded handoffs when each target is active and has a direct incoming `supersedes` edge from an active successor. |
| Null | The remaining superseded-active findings should stay active until a broader lifecycle design exists. |
| Simpler alternative | Stop after T288/T289 and leave the lint queue as known pressure only. |
| Failure | The batch archives a current handoff, relies on inferred supersession, or widens into broad cleanup without per-target evidence. |

## Preflight Evidence

- Fresh `lint(action="list", limit=20, vault_path="/Users/yuval.meiri/.engram/vault")`
  returned only `superseded_item_still_active` findings in the sampled set, with zero safe actions
  applied.
- Each selected target was fetched with `memory(action="get")` and had `status="active"` before
  archive.
- Each selected target had a direct incoming `supersedes` edge in `graph(action="around",
  depth=1)`.
- Each direct superseding memory was fetched or was already in the reviewed batch and was active
  before the archive action.

## Archived Exact Targets

| Archived target | Scope | Direct superseding memory |
| --- | --- | --- |
| `019dd944-8d69-7b81-8659-b0ef8e23c75f` | `project:dd-source` | `019dd946-c602-7ab0-a62d-519944dbd756` |
| `019dd946-c602-7ab0-a62d-519944dbd756` | `project:dd-source` | `019dd947-5d00-71d2-a42a-b6f126a14201` |
| `019dd947-5d00-71d2-a42a-b6f126a14201` | `project:dd-source` | `019dd9b2-0be7-75a2-ac5d-036c0502ee3d` |
| `019dd9b2-0be7-75a2-ac5d-036c0502ee3d` | `project:dd-source` | `019ddd45-11c3-7760-a5e9-6434434689ba` |
| `019ddd45-11c3-7760-a5e9-6434434689ba` | `project:dd-source` | `019ddd46-3320-7bf3-8048-63f09a726c10` |

## Result

T290 archived exactly those five MemoryItems with `archived_by="codex"` and per-item archive
reasons naming the direct superseding memory. The archive action preserved the records and changed
their lifecycle status to `archived`; no records were deleted. The batch audit is recorded as
MemoryItem `019e9d8b-538f-71b2-bc39-74d8d1687005` and KnowledgeCommit
`019e9d8b-649b-7083-9029-78916c813ac1`.

Post-archive `lint(action="list", limit=20, vault_path="/Users/yuval.meiri/.engram/vault")`
no longer returned any of the five target IDs. The returned sample advanced to the next unprocessed
superseded-active candidate, `019ddd46-3320-7bf3-8048-63f09a726c10`.

Canonical vault compile after the batch reported `1,644` MemoryItems, `568` KnowledgeCommits,
`9` repositories, `32` entities, `79` projects, and `2,336` expected/generated files with zero
user files.

## Branch And Pull-Hint Audit

After `git fetch --prune origin`, the feature branch and its upstream were `0 0` apart by
`git rev-list --left-right --count HEAD...origin/yuval.meiri/memory-os-phase0`;
`origin/main...HEAD` was `0 406`; and `origin/main` was an ancestor of `HEAD`. No pull, merge,
rebase, pull-policy config change, or branch rewrite ran. The recurring pull hint is not current
evidence of a local/upstream divergence on `yuval.meiri/memory-os-phase0`.

## Non-Claims

T290 is exact lifecycle maintenance only. It does not run broad `lint apply_safe`, complete global
lifecycle cleanup, deprecate or delete direct legacy data, change ranking or `orient`, mutate M6
state, run native Claude, edit harness files, change schema/storage/index behavior, mark PR #2
ready for review, or change pull/rebase configuration.
