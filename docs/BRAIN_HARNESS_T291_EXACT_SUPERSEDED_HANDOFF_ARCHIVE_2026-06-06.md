# Brain Harness T291 Exact Superseded Handoff Archive - 2026-06-06

## Research Question

Can Engram continue reducing active lifecycle noise by archiving the next small exact batch of
superseded rolling handoffs while preserving the T284 default-deny boundary against broad cleanup?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A fourth exact batch can safely archive reviewed superseded handoffs when each target is active and has a direct incoming `supersedes` edge from an active successor. |
| Null | The remaining superseded-active findings should stay active until a broader lifecycle design exists. |
| Simpler alternative | Stop after T288/T289/T290 and leave the lint queue as known pressure only. |
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
| `019ddd46-3320-7bf3-8048-63f09a726c10` | `project:dd-source` | `019dde55-6c94-79c1-8594-035b9ec2e1b3` |
| `019dde55-6c94-79c1-8594-035b9ec2e1b3` | `project:dd-source` | `019dde55-f3f1-7ad1-b9cb-7a6f68b9c416` |
| `019dde55-f3f1-7ad1-b9cb-7a6f68b9c416` | `project:dd-source` | `019dde56-36c9-7bf2-ad38-4914eec2bbdf` |
| `019dde56-36c9-7bf2-ad38-4914eec2bbdf` | `project:dd-source` | `019dde56-7aa2-75a0-b843-a520e39b5935` |
| `019dde56-7aa2-75a0-b843-a520e39b5935` | `project:dd-source` | `019dde56-b3f5-70d3-87b6-ef6ff06751bc` |

## Result

T291 archived exactly those five MemoryItems with `archived_by="codex"` and per-item archive
reasons naming the direct superseding memory. The archive action preserved the records and changed
their lifecycle status to `archived`; no records were deleted. The batch audit is recorded as
MemoryItem `019e9dba-900e-7c53-a502-27d6d5ef7ba6` and KnowledgeCommit
`019e9dba-a27e-7db2-8631-d8a76ec2a571`.

Post-archive `lint(action="list", limit=20, vault_path="/Users/yuval.meiri/.engram/vault")`
no longer returned any of the five target IDs. The returned sample advanced to the next unprocessed
superseded-active candidate, `019dde56-b3f5-70d3-87b6-ef6ff06751bc`.

Canonical vault compile after the batch reported `1,647` MemoryItems, `570` KnowledgeCommits,
`9` repositories, `32` entities, `79` projects, and `2,341` expected/generated files with zero
user files.

## Branch And Pull-Hint Audit

After `git fetch --prune origin`, the feature branch and its upstream were `0 0` apart by
`git rev-list --left-right --count HEAD...origin/yuval.meiri/memory-os-phase0`;
`origin/main...HEAD` was `0 407`; and `origin/main` was an ancestor of `HEAD`. No pull, merge,
rebase, pull-policy config change, or branch rewrite ran. The recurring pull hint is not current
evidence of a local/upstream divergence on `yuval.meiri/memory-os-phase0`.

## Non-Claims

T291 is exact lifecycle maintenance only. It does not run broad `lint apply_safe`, complete global
lifecycle cleanup, deprecate or delete direct legacy data, change ranking or `orient`, mutate M6
state, run native Claude, edit harness files, change schema/storage/index behavior, mark PR #2
ready for review, or change pull/rebase configuration.
