# Brain Harness T41 Mixed-Query Fixture

Date: 2026-05-31
Status: Completed as validation-only fixture
Scope: Deterministic search fixture only

## Boundary

T41 follows from the T40-04 partial result. It must not run M6 inventory or review export, mutate
memory lifecycle state, change schemas or storage, change public MCP request parameters, expand
`orient`, change harness adapters or hooks, or introduce broad ranking churn.

If source inspection shows the exact mixed-query behavior requires a broad ranking change or
approval-gated lifecycle/M6/harness work, stop and treat that as a new gated slice.

## Research Question

After T40 current-plan capture, can deterministic fixture coverage preserve the live behavior where
the mixed query `current plan next non-gated Brain Harness feedback confidence M6 gate` returns the
latest current plan first while still surfacing active M6 gate context in top memory results?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Existing search/ranker behavior is sufficient after current-plan capture; a fixture can lock the invariant without source behavior changes. |
| Null | The live pass was data-state luck and a deterministic fixture cannot reproduce the intended invariant. |
| Simpler alternative | Document the T40-04 caveat only and wait for explicit approval for broader ranking or lifecycle work. |
| Failure | The fixture requires score/weight changes, lifecycle cleanup, payload expansion, public API changes, M6 work, or harness writes to pass. |

## Measurement

Add an isolated in-memory search fixture with live-shaped records:

- latest project-scoped active `current-plan` decision,
- older repository-scoped active `current-plan` decision as stale noise,
- active M6 approval-gate limitation,
- non-gated calibration/noise records.

Pass criteria:

- the exact T40-04 query returns the latest current plan at rank 1,
- the active M6 approval gate appears in the first five memory results,
- the stale repository current-plan record does not outrank the latest current plan,
- existing explicit migration-apply gate-first regression remains green.

Assertions must avoid exact score checks and avoid over-specifying the M6 gate's rank beyond top
five presence. If the fixture passes without production code changes, record T41 as validation
only. If it fails, inspect root cause before considering any prompt-class local fix.

## Consultation

AI Council recall and broadcast, plus Claude Bridge read-only critique, agreed that this is a safe
non-gated fixture slice if it remains an isolated regression test and avoids score/weight changes,
M6 work, lifecycle writes, `orient` payload changes, public MCP changes, and harness writes. The
main cautions were to avoid exact-rank overfitting, seed live-shaped stale/noisy records, and keep
explicit migration-apply prompts gate-first.

## Result

The fixture `test_memory_search_t40_mixed_query_surfaces_current_plan_and_m6_gate` was added to
`engram-tests/tests/search_tests.rs`. It seeds live-shaped records and verifies:

- exact T40-04 mixed query returns the latest project current plan first,
- active M6 gate memory appears within the first five memory results,
- stale repository current-plan guidance does not outrank the latest current plan,
- an explicit M6 write/apply/deletion query still returns the M6 gate first.

The targeted command passed:

```bash
cargo test -p engram-tests --test search_tests test_memory_search_t40_mixed_query_surfaces_current_plan_and_m6_gate -- --nocapture
```

No production ranking code, public MCP surface, lifecycle state, `orient` payload, M6 migration
flow, schema/storage/index behavior, or harness adapter/hook behavior changed.
