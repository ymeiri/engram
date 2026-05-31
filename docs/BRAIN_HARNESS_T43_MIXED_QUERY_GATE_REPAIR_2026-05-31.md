# Brain Harness T43 Mixed-Query Gate Repair

Date: 2026-05-31
Status: Pre-registered
Scope: Prompt-specific direct `search` ranking repair for one mixed current-plan/M6-gate query class

## Boundary

T43 is a narrow search-ranker repair following the T42 pre-run baseline failure. It must not run M6
inventory or review export, mutate memory lifecycle state, archive or scope-rewrite memory, change
schemas or storage, change public MCP request parameters, expand `orient`, install or modify
harness adapters or hooks, or change broad ranking weights.

The allowed implementation surface is `engram-index/src/memory_ranker.rs` plus focused regression
tests. If the fix requires query expansion outside the ranker, a new response field, a public MCP
parameter, lifecycle cleanup, M6 work, or broad scoring changes, stop and re-plan.

## Research Question

Can direct unified `search` keep the latest current-plan memory first for the exact mixed query
`current plan next non-gated Brain Harness feedback confidence M6 gate` while also surfacing active
M6 gate context in top-k, without changing explicit M6 apply/gate behavior or broad ranking?

## Evidence From T42

- Codex trace `019e7d08-d297-71b3-b8dd-495078383ce9` returned latest current-plan memory first
  but omitted active M6 gate memory from the top eight memory results.
- Codex diagnostic trace `019e7d09-d6ae-7a83-a9c7-b835c25b9df4` returned active M6 gate memory
  `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` at rank 17 for the exact mixed query.
- Codex negative-control trace `019e7d08-dd64-7830-bd83-5bfb104e5ee1` still returned gate/blocked
  context above current-plan guidance and did not imply M6 approval.
- Source inspection found `SearchService` ranks all active MemoryItems before truncating results,
  so the active M6 gate is in the candidate set. The gap is ranking, not retrieval expansion.

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A local post-ranking promotion can surface already-ranked active migration-gate items below current-plan guidance only for current-plan continuation queries that mention M6/migration and gate context but are not explicit apply/permission gate queries. |
| Null | The T42 live miss is data-state noise or requires broader search/ranking changes that T43 should not make. |
| Simpler alternative | Record the failure and require agents to run a second explicit gate search after current-plan searches. |
| Failure | The repair demotes current-plan guidance, changes explicit apply/gate behavior, promotes M6 gates for pure continuation queries, or needs lifecycle/M6/schema/public-MCP/`orient`/harness changes. |

## Measurement

Add focused regression coverage before or with the code change:

- live-shaped mixed query fixture: latest current-plan rank 1 and active M6 gate within top five,
  despite calibration, stale-plan, handoff, implementation-history, and other gate/noise records;
- pure continuation query without M6/gate context: current-plan remains rank 1 and M6 gate is not
  newly promoted into the asserted top-k;
- explicit M6 apply/permission query and T42 negative-control query: gate/blocked context remains
  above current-plan guidance and no result is treated as M6 approval;
- existing T41 fixture remains green.

Implementation constraints:

- Do not change `rank_score` weights.
- Do not make bare `gate` a decision-gate trigger.
- Do not put M6 gate context above current-plan for continuation queries.
- Reuse existing actionable migration-gate detection where possible.
- Keep the behavior internal to ranked MemoryItem ordering.

## Consultation

AI Council recall surfaced the T12/T38 guidance: bare `gate` should not become a gate-mode trigger,
current-plan continuation should remain current-plan-first, and any repair must avoid payload,
lifecycle, migration, schema, and broad ranking changes.

AI Council broadcast agreed T43 is justified as a conditional post-rank promotion for the
intersection of current-plan continuation intent plus M6/migration gate context, with explicit
apply/permission queries preserved as gate-first.

Claude Bridge warned that hardcoding M6 in generic ranking can become a layering issue, that
classification boundaries are fragile, and that the candidate-set assumption must be checked. The
chosen boundary accepts a narrow migration-gate helper because this ranker already contains
migration-gate classification, `SearchService` ranks all active memory before truncation, and
introducing a new `active_gates` surface or public query expansion would cross a larger boundary
than this prompt-specific repair.

## Allowed Conclusion

If T43 passes, it supports only this claim: for the exact mixed current-plan/M6-gate prompt class,
direct `search` keeps current-plan guidance first while surfacing existing active M6 gate context
within the usable top-k. It does not prove broad ranking quality, cross-harness parity, M6 approval,
lifecycle cleanup safety, or harness readiness.
