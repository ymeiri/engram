# Brain Harness T380 Stale M6 Gate Memory Archive

Date: 2026-06-08
Status: exact lifecycle archive completed.

## Research Question

Can Engram archive the active 2026-04-28 MemoryItem
`019dd35d-1a48-7103-b0e2-390225f8b418`
(`Memory OS completion is paused at migration review gate`) without broad lifecycle cleanup, given
that T278 later closed the current T68/T209/T210/T250 M6 review-batch disposition/apply gate?

Preferred hypothesis: exact archive is safe because the item is stale as active guidance after T278,
and current active M6 guidance should come from T278 plus the still-active explicit M6 limitation.

Null hypothesis: the item should stay active because it records older migration-review context that
is still useful.

Failure hypothesis: archiving the item could be mistaken for direct legacy deprecation, broad M6
simplification, or production/GA completion.

## Evidence

- `memory(get)` showed the target was active, project-scoped to `engram`, and written on
  2026-04-28 as a checkpoint that said completion was paused at a migration review gate.
- Project-scoped lint reported
  `feedback-stale-active-memory:019dd35d-1a48-7103-b0e2-390225f8b418`.
- A current T278/T380 verification search trace
  `019ea835-fb62-73b0-9640-baaebb06460e` still returned the stale checkpoint first.
- `docs/BRAIN_HARNESS_T278_M6_DISPOSITION_APPLY_RESULT_2026-06-06.md` records that T278
  decided all 12 files in the current M6 review batch, wrote five reviewed active
  `project:engram` MemoryItems, created KnowledgeCommit
  `019e9bd6-7e8e-7611-8326-1811b3b799a2`, verified idempotent post-apply status, and refreshed
  the canonical vault.
- AI Council recall surfaced the T279 lesson: standing authorization permits exact lifecycle
  archives, but archive reasons must be freshly validated and exact-targeted.

## Archive

Archived exactly:

- `019dd35d-1a48-7103-b0e2-390225f8b418` -
  `Memory OS completion is paused at migration review gate`

The archive reason names T278 as the superseding current evidence and explicitly preserves the
remaining boundaries: no direct legacy deprecation/deletion, broad lifecycle cleanup, broad M6
simplification, ranking or `orient` changes, hosted-CI fallback acceptance, native Claude execution,
effective-hook proof, live host-label proof, or beta release mechanics.

## Validation

- Post-archive `memory(get)` reports `status="archived"` for the exact target.
- Post-archive search trace `019ea83c-150b-7011-998f-54f61ba618d4` no longer returns the archived
  checkpoint in active memory results. The top active memory result is the still-current limitation
  `M6 migration approval gate remains explicit`.
- Post-archive project-scoped lint no longer includes
  `feedback-stale-active-memory:019dd35d-1a48-7103-b0e2-390225f8b418`.
- `obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")`
  returned `open=[]` and `warnings=[]`.

## Gate Impact

T380 reduces stale active-memory risk around the M6 gate and improves current retrieval for agents
reasoning about T278. It does not delete data, run broad `lint apply_safe`, run M6 write-apply,
change source behavior, change ranking or `orient`, accept hosted-CI fallback, mark PR #3 ready,
merge, tag, publish, launch native Claude, prove effective hooks, prove live host labels, or change
the scoped beta/production boundary.
