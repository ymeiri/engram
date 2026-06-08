# Brain Harness T365 Stale Orient-Contract Memory Archive

Date: 2026-06-08
Status: completed exact lifecycle cleanup. No source code, harness files, hosted CI, broad
`lint apply_safe`, native Claude sessions, or user-owned files were changed.

## Research Question

Can Engram reduce active-memory risk by retiring old May 6 orient-contract/architecture checkpoint
records that are now stale, evidence-less, and duplicated by newer evidenced Brain Harness state?

## Decision

Yes. Archive exactly these two active MemoryItems:

- `019dfed3-519d-7f01-8c46-c9245ba0045b` -
  `AI Council and Claude next-step synthesis after orient contract`
- `019dfed5-1875-7110-b355-8d1060e6d04a` -
  `Brain Harness Architecture synced after orient contract checkpoint`

Both records had no durable evidence, were marked by feedback as stale, and were also flagged by
project-scoped lint as missing evidence. Their May 6 next-step and architecture-checkpoint guidance
is historical, not current. Newer evidenced records now carry the current state: T295/T343/T364
beta scope/current-plan records, T333/T340/T363 harness-gate records, and current architecture and
release notes.

## Evidence

Pre-archive direct checks:

- `memory(get)` showed both targets were `status=active` with empty `evidence`.
- `graph(around, depth=1)` for each target showed only `scoped_to project:engram`, with no direct
  dependent MemoryItem edge.
- Project-scoped lint reported both targets as stale-feedback active memory and missing-evidence
  active memory.
- Direct search for the old orient-contract/architecture checkpoint titles returned the current
  evidenced T364 current-plan memory first, followed by newer evidenced beta-scope/lifecycle records.

Archive execution:

- `memory(action=archive, id=019dfed3-519d-7f01-8c46-c9245ba0045b)` archived only the stale
  AI Council/Claude next-step synthesis.
- `memory(action=archive, id=019dfed5-1875-7110-b355-8d1060e6d04a)` archived only the stale
  architecture checkpoint record.

Post-archive validation:

- `memory(get)` for both IDs now reports `status=archived` with explicit archive reasons.
- Live daemon project-scoped `lint(action=run, project=engram, limit=40)` returned
  `archived_targets_present=[]` and reduced the returned finding count from `20` to `16`.
- Direct search for the old titles still returns active evidenced current-plan/beta-scope records
  first.

## Non-Claims

T365 does not prove production/GA readiness, native Claude prompt-bearing behavior, effective-hook
visibility, live host labels, hosted CI, direct legacy deletion, or broad lifecycle cleanup. It does
not delete memory data, run `lint apply_safe`, mutate source code, mark PR #3 ready, merge, tag,
publish, or change the supported beta scope.
