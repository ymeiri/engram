# Brain Harness T289 Exact Superseded Handoff Archive - 2026-06-06

## Research Question

Can Engram continue reducing active lifecycle noise by archiving the next small exact batch of
superseded rolling handoffs without treating the global lint queue as an automatic cleanup plan?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A second exact batch can safely archive reviewed superseded handoffs when each target is active and has a direct incoming `supersedes` edge from an active successor. |
| Null | The remaining superseded-active findings are harmless enough to leave active until a broader lifecycle plan exists. |
| Simpler alternative | Keep T284's broad cleanup deferral and T288's first exact batch as the last lifecycle action. |
| Failure | The batch archives a current handoff, relies on inferred supersession, or widens into broad cleanup without per-target evidence. |

## Preflight Evidence

- Fresh `lint(action="run", limit=20, vault_path="/Users/yuval.meiri/.engram/vault")`
  returned only `superseded_item_still_active` findings in the sampled set, with zero safe actions
  applied.
- Each selected target was fetched with `memory(action="get")` and had `status="active"` before
  archive.
- Each selected target had a direct incoming `supersedes` edge in `graph(action="around",
  depth=1)`.
- Each direct superseding memory was fetched and was active before the archive action.

## Archived Exact Targets

| Archived target | Scope | Direct superseding memory |
| --- | --- | --- |
| `019dd912-7adc-7860-bd80-95cc681cc061` | `project:codex-claude-bridge` | `019e6993-6d69-78e1-a29d-93a61e2a6413` |
| `019dd93c-b7f1-7e92-ac27-262e128163cd` | `project:dd-source` | `019dd93f-2c18-7bf3-a4a4-038bac9d74fb` |
| `019dd93f-2c18-7bf3-a4a4-038bac9d74fb` | `project:dd-source` | `019dd940-7207-7f51-93ea-533d5f80d6e7` |
| `019dd940-7207-7f51-93ea-533d5f80d6e7` | `project:dd-source` | `019dd941-314b-74d3-a879-4e451c7bd258` |
| `019dd941-314b-74d3-a879-4e451c7bd258` | `project:dd-source` | `019dd944-8d69-7b81-8659-b0ef8e23c75f` |

## Result

T289 archived exactly those five MemoryItems with `archived_by="codex-gpt-5"` and per-item archive
reasons naming the direct superseding memory. The archive action preserved the records and changed
their lifecycle status to `archived`; no records were deleted. The batch audit is recorded as
KnowledgeCommit `019e9d5c-9c39-7c63-89a2-a8d2741c03e0`.

Post-archive `lint(action="run", limit=20, vault_path="/Users/yuval.meiri/.engram/vault")`
no longer returned any of the five target IDs. The returned sample advanced to the next unprocessed
superseded-active candidates, beginning with `019dd944-8d69-7b81-8659-b0ef8e23c75f`.

## Branch And Pull-Hint Audit

T289 also rechecked the recurring divergent-branch/pull hint before any docs commit. After
`git fetch --prune origin`, the feature branch and its upstream were `0 0` apart by
`git rev-list --left-right --count HEAD...origin/yuval.meiri/memory-os-phase0`;
`origin/main...HEAD` was `0 405`; and `origin/main` was an ancestor of `HEAD`. No pull, merge,
rebase, pull-policy config change, or branch rewrite ran. The hint is not current evidence of a
local/upstream divergence on `yuval.meiri/memory-os-phase0`.

## Non-Claims

T289 is exact lifecycle maintenance only. It does not run broad `lint apply_safe`, complete global
lifecycle cleanup, deprecate or delete direct legacy data, change ranking or `orient`, mutate M6
state, run native Claude, edit harness files, change schema/storage/index behavior, mark PR #2
ready for review, or change pull/rebase configuration.
