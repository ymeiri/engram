# Brain Harness T43 Mixed-Query Gate Repair

Date: 2026-05-31
Status: Completed
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

## Result

T43 implemented a search-only post-ranking repair in `engram-index/src/memory_ranker.rs`.
For direct `search` queries that already qualify as current-plan continuation prompts and also
mention M6/migration gate context, the ranker can promote an already-ranked active migration-gate
MemoryItem directly below the current-plan MemoryItem. The helper requires
`MemoryRankContext::search`, skips explicit migration apply/permission gate queries, leaves pure
current-plan queries alone, and does not change `rank_score` weights or broad retrieval.

The focused fixture
`test_memory_search_promotes_m6_gate_context_below_current_plan_for_mixed_query` seeds live-shaped
noise, stale current-plan guidance, active M6 approval-gate memory, and explicit-gate controls. It
passes with these claims:

- exact mixed query keeps the latest current plan at rank 1 and places active M6 gate context in
  the first five results;
- pure continuation query without M6/gate context keeps current-plan first and does not promote the
  M6 gate into the asserted top-k;
- explicit `approved M6 write apply deletion cleanup legacy simplification now` query keeps
  gate/blocked context above current-plan guidance.

Validation run:

- `cargo fmt --all --check`
- `cargo test -p engram-index memory_ranker::tests -- --nocapture`
- `cargo test -p engram-tests --test search_tests test_memory_search_t40_mixed_query_surfaces_current_plan_and_m6_gate -- --nocapture`
- `cargo test -p engram-tests --test search_tests test_memory_search_promotes_m6_gate_context_below_current_plan_for_mixed_query -- --nocapture`
- `cargo test -p engram-tests --test search_tests`
- `cargo check -p engram-cli`
- `git diff --check`

Installed-runtime validation used `/Users/yuval.meiri/.local/bin/engram` with SHA-256
`c8b1254ac71f53da80221a2a259014fca89e2e8e8ca1998a4f0128adce01e721` after restarting the daemon
on port 8765 with PID 49169.

- Mixed-query trace `019e7d1c-b20a-7c52-b8af-e6d82439988c` returned current-plan memory
  `019e7d0b-3425-7c00-a395-a69c14cf2a47` at rank 1 and active M6 gate memory
  `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` at rank 2.
- Explicit-M6 negative-control trace `019e7d1c-c100-7721-82ba-8061330aff8f` kept gate/blocked
  context above current-plan guidance.
- Pure continuation trace `019e7d1e-29ad-7540-bcfc-d28131851091` returned the latest current-plan
  memory first and did not promote active M6 gate memory into the top eight.
- Lean `orient` sanity trace `019e7d1e-2a48-7d63-a49d-a7da22bfa68f` stayed compact and did not use
  the search-only contextual M6 promotion.

Telemetry feedback was submitted for all four live traces:
`019e7d1e-91ab-79e1-81ca-d05cc15fd770`, `019e7d1e-91b5-7532-8e90-8087a220e58b`,
`019e7d1e-91d2-7620-9ec3-048682587519`, and
`019e7d1e-91db-76e2-9eb6-47461ef6ed41`.

The remaining high-risk completion gate is unchanged: M6 migration inventory/review-export,
write-apply, deletion, cleanup, and legacy simplification remain approval-gated and were not run.
