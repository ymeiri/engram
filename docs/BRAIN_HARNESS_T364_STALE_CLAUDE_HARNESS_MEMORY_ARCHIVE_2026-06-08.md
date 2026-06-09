# Brain Harness T364 Stale Claude-Harness Memory Archive

Date: 2026-06-08
Status: completed exact lifecycle cleanup. No source code, harness files, Claude settings,
native Claude sessions, hosted CI, broad `lint apply_safe`, or user-owned files were changed.

## Research Question

Can Engram reduce production-readiness risk by retiring old active Claude-harness claims that are
now stale, evidence-less, and potentially misleading relative to the current native-Claude gate?

## Decision

Yes. Archive exactly these two active MemoryItems:

- `019dd4e3-bcec-7c02-9174-ba0ac0380d45` -
  `Claude Code native hook harness implemented`
- `019dd509-46f2-71c0-aff7-ebe777810825` -
  `Claude Code Engram-native harness activated`

Both records had no durable evidence, were flagged by feedback/lint, and made stronger Claude
harness claims than the current evidenced state supports. Current active evidence now comes from
later records such as T333 generated-adapter repair and T340/T363 native-Claude preflight refreshes:
generated adapter drift is closed, but native Claude prompt-bearing behavior, effective-hook
visibility, and live host labels remain unproved.

## Evidence

Pre-archive direct checks:

- `memory(get)` showed both targets were `status=active` with empty `evidence`.
- `graph(around, depth=1)` for each target showed only `scoped_to project:engram`, with no direct
  dependent MemoryItem edge.
- Project-scoped lint reported both targets as missing evidence and stale-feedback active memory.
- Direct search for Claude harness activation returned newer active evidenced memories first,
  including T333 and T340/T335/T311/T363-family limitations.

Archive execution:

- `memory(action=archive, id=019dd4e3-bcec-7c02-9174-ba0ac0380d45)` archived only the
  evidence-less implementation claim.
- `memory(action=archive, id=019dd509-46f2-71c0-aff7-ebe777810825)` archived only the
  evidence-less activation claim.

Post-archive validation:

- `memory(get)` for both IDs now reports `status=archived` with explicit archive reasons.
- Live daemon project-scoped `lint(action=run, project=engram, limit=30)` returned
  `archived_targets_present=[]`.
- Direct search for the old activation titles now returns active evidenced successor/limitation
  records first, not the archived targets.

## Non-Claims

T364 does not prove native Claude prompt-bearing behavior, effective-hook visibility, live host
labels, hosted CI, production/GA readiness, or broad lifecycle cleanup. It does not delete memory
data, run `lint apply_safe`, mutate Claude settings/hooks, launch native Claude, mark PR #3 ready,
merge, tag, publish, or change the supported beta scope.
