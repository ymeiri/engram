# Brain Harness T292 Exact Superseded Handoff Archive - 2026-06-06

## Research Question

Can Engram continue reducing active lifecycle noise by archiving the next small exact batch of
superseded rolling handoffs while preserving the T284 default-deny boundary against broad cleanup?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A fifth exact batch can safely archive reviewed superseded handoffs when each target is active and has a direct incoming `supersedes` edge from an active successor. |
| Null | The remaining superseded-active findings should stay active until a broader lifecycle design exists. |
| Simpler alternative | Stop after T288/T289/T290/T291 and leave the lint queue as known pressure only. |
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
| `019dde56-b3f5-70d3-87b6-ef6ff06751bc` | `project:dd-source` | `019dde57-7139-7fa3-a2e8-94583866c1f2` |
| `019dde57-7139-7fa3-a2e8-94583866c1f2` | `project:dd-source` | `019dde87-a14b-7cc0-9dbd-0a0a84996fbb` |
| `019dde87-a14b-7cc0-9dbd-0a0a84996fbb` | `project:dd-source` | `019dde88-2c90-7860-8ed8-9b14a0273da8` |
| `019dde88-2c90-7860-8ed8-9b14a0273da8` | `project:dd-source` | `019ddea9-a614-7920-badf-ac2e9ae91fcb` |
| `019ddea9-a614-7920-badf-ac2e9ae91fcb` | `project:dd-source` | `019df80f-bb2d-7683-b802-4f4de39469df` |

## Result

T292 archived exactly those five MemoryItems with `archived_by="codex"` and per-item archive
reasons naming the direct superseding memory. The archive action preserved the records and changed
their lifecycle status to `archived`; no records were deleted. The batch audit is recorded as
MemoryItem `019e9de9-78b5-7f13-9eac-ab0276a3d879` and KnowledgeCommit
`019e9de9-c8aa-78c3-8b89-df778e1e41e7`.

Post-archive `lint(action="list", limit=20, vault_path="/Users/yuval.meiri/.engram/vault")`
no longer returned any of the five target IDs in the sampled findings. The fifth target's direct
successor is `019df80f-bb2d-7683-b802-4f4de39469df`, but that successor skipped the next lint row;
the returned sample advanced to the next unprocessed superseded-active candidate,
`019ddebe-5159-71a1-a593-03d5a38ad305`.

Canonical vault compile after the batch reported `1,650` MemoryItems, `572` KnowledgeCommits,
`9` repositories, `32` entities, `79` projects, and `2,346` expected/generated files with zero
user files.

## Branch And Pull-Hint Audit

After `git fetch --prune origin`, the feature branch and its upstream were `0 0` apart by
`git rev-list --left-right --count HEAD...origin/yuval.meiri/memory-os-phase0`;
`origin/main...HEAD` was `0 408`; and `origin/main` was an ancestor of `HEAD`. No pull, merge,
rebase, pull-policy config change, or branch rewrite ran. The recurring pull hint is not current
evidence of a local/upstream divergence on `yuval.meiri/memory-os-phase0`.

## Non-Claims

T292 is exact lifecycle maintenance only. It does not run broad `lint apply_safe`, complete global
lifecycle cleanup, deprecate or delete direct legacy data, change ranking or `orient`, mutate M6
state, run native Claude, edit harness files, change schema/storage/index behavior, mark PR #2
ready for review, or change pull/rebase configuration.
