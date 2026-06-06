# Brain Harness T288 Exact Superseded Handoff Archive - 2026-06-06

## Research Question

Can Engram reduce active lifecycle noise by archiving a small exact batch of superseded rolling
handoffs without using broad `lint apply_safe` or claiming global lifecycle cleanup?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Items with direct incoming `supersedes` graph edges can be archived one by one after exact review, reducing active-memory noise while preserving auditability. |
| Null | The lint findings are only informational; leaving the old handoffs active does not affect current Engram work. |
| Simpler alternative | Keep T284's broad cleanup deferral unchanged and write only another docs-only packet. |
| Failure | Archiving exact targets hides the newest handoff, touches unrelated lifecycle state, or masks the need for a broader cleanup plan. |

## Preflight Evidence

- Fresh `lint(action="run", limit=80, vault_path="/Users/yuval.meiri/.engram/vault")`
  returned only `superseded_item_still_active` findings in the sampled set, with zero safe actions
  applied.
- Each selected target was fetched with `memory(action="get")` and had `status="active"` before
  archive.
- Each selected target had a direct incoming `supersedes` edge in `graph(action="around",
  depth=1)`.

## Archived Exact Targets

| Archived target | Scope | Direct superseding memory |
| --- | --- | --- |
| `019dd5cd-a403-7b53-9010-47bd94bba51a` | `project:ide-mcp-eval-replay-stringification-verification` | `019dd7ff-0041-7e33-b825-cb65d299bfa9` |
| `019dd80d-7466-7061-8417-6d5f085defc6` | `project:dd-source` | `019dd846-0f0f-7271-9e38-34e1ffc4f6d6` |
| `019dd846-0f0f-7271-9e38-34e1ffc4f6d6` | `project:dd-source` | `019dd84c-2812-75c0-bc3d-ab8ec05f9007` |
| `019dd84c-2812-75c0-bc3d-ab8ec05f9007` | `project:dd-source` | `019dd84c-a8d4-7cd3-b1a3-0f910c7050cc` |
| `019dd84c-a8d4-7cd3-b1a3-0f910c7050cc` | `project:dd-source` | `019dd93c-b7f1-7e92-ac27-262e128163cd` |

## Result

T288 archived exactly those five MemoryItems with `archived_by="codex-gpt-5"` and per-item archive
reasons naming the direct superseding memory. The archive action preserved the records and changed
their lifecycle status to `archived`; no records were deleted. The batch audit is recorded as
KnowledgeCommit `019e9d2a-e428-7903-b17d-11468e2644ae`.

Post-archive `lint(action="run", limit=20, vault_path="/Users/yuval.meiri/.engram/vault")`
no longer returned any of the five target IDs. The returned sample advanced to the next unprocessed
superseded-active candidates, beginning with `019dd912-7adc-7860-bd80-95cc681cc061`.

## Non-Claims

T288 is exact lifecycle maintenance only. It does not run broad `lint apply_safe`, complete global
lifecycle cleanup, deprecate or delete direct legacy data, change ranking or `orient`, mutate M6
state, run native Claude, edit harness files, change schema/storage/index behavior, or mark PR #2
ready for review.
