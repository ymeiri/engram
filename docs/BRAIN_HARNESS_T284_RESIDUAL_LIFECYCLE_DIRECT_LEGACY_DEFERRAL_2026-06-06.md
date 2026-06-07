# Brain Harness T284 Residual Lifecycle Direct Legacy Deferral - 2026-06-06

## Scope

T284 is a read-only residual lifecycle and direct-legacy decision checkpoint after T278, T279, and
T283.

T284 does not archive memory, run `lint apply_safe`, delete legacy observations, deprecate direct
legacy paths, run M6, mutate lifecycle state, change ranking or `orient`, edit harness files, run
native Claude, push branches, or touch user-owned files.

## Research Question

Does fresh evidence support a safe residual lifecycle cleanup or direct legacy deprecation action
now?

## Result

No. T284 records a deferral, not a cleanup action.

Fresh lint evidence still shows lifecycle debt, but only as a global, limit-truncated sample. The
first 50 findings were all `superseded_item_still_active` warnings with `safe_action` set to
`archive_memory_item`. That confirms residual cleanup pressure, but it does not define an exact,
reviewed, dependency-checked batch that is safe to mutate.

Direct legacy deprecation/deletion remains separate from the T278 M6 apply result. T278 wrote the
current reviewed MemoryItems and validated idempotence for that batch; it did not prove that legacy
observations can be deleted, hidden, or deprecated without behavioral regression.

## Evidence

| Evidence | Result |
| --- | --- |
| `orient` | Lean project-scoped trace `019e9c07-9ac1-7553-aafd-e1cf5d582898` surfaced the current T283 plan plus the explicit M6 migration approval limitation. |
| `lint(action="run", limit=50)` | Returned 50 `superseded_item_still_active` warnings. Each displayed finding proposed `archive_memory_item`, but the sample was global and limit-truncated. |
| Memory search | Querying active project memory for stale lifecycle/direct legacy terms surfaced the current handoff/current-plan and historical migration/lifecycle guidance, not an exact reviewed batch ready to archive. |
| AI Council recall | Prior T279/T246/T245/T139/T48 guidance consistently required exact target review and rejected broad `lint apply_safe` or bundled lifecycle cleanup. |
| Repo docs | The implementation plan and architecture still say broad lifecycle cleanup and direct legacy deprecation/deletion remain separate, evidence-gated work. |
| PR checks | At the time of this checkpoint, `Check`, `Format`, and `Docs` passed on PR #2; `Test` and `Clippy` were still running, with no failure available to debug. |

## Decision

Defer broad residual lifecycle cleanup and direct legacy deprecation/deletion.

A future lifecycle cleanup may proceed only as a separate exact-target batch with fresh evidence for
each candidate, including:

- `memory(action="get")` or equivalent content retrieval for each exact ID;
- active search visibility and current status;
- supersession/dependency graph review;
- rationale that does not depend on stale pre-T278 or pre-T279 facts;
- dry-run/result docs before any write;
- no use of broad `lint apply_safe` as the approval boundary.

A future direct legacy deprecation/deletion decision must separately prove that active reviewed
MemoryItems preserve the important knowledge and that agent behavior does not regress when legacy
paths are hidden or removed. T278 alone is not that proof.

## Non-Claims

T284 does not claim lifecycle cleanup is complete. It only rejects broad cleanup from the current
truncated lint sample.

T284 does not deprecate, delete, hide, or simplify any legacy data path.
