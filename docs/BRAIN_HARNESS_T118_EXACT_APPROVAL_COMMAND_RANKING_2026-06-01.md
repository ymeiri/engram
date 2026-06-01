# Brain Harness T118 Exact Approval Command Ranking

Status: Completed narrow direct-search ranking calibration
Date: 2026-06-01
Scope: Make exact approval-command memory searches recover the matching active current-plan before
old handoff text noise.

This slice changed only direct MemoryItem search ranking for a narrow command shape. It did not
change `orient`, expand payloads, run document indexing, run document planning, inspect T69 files,
run M6 inventory/export/apply, mutate lifecycle state, run `lint(action="apply_safe")`, change
public MCP parameters or response shape, change schema/storage/index behavior, change document-index
behavior, or write harness adapters/hooks.

## Research Question

When the user or agent searches for an exact approval command such as
`Approve T70: index exact files T59, T68, and T69.`, should direct memory search prefer the active
current-plan memory that contains that exact command over older handoff memories with stronger raw
text overlap?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A search-only detector for exact `Approve T<number>:` commands can promote only matching active current-plan Decision/Rule items and fix the live T70 inversion without broad ranking churn. |
| Null | Existing text scoring is sufficient; older handoff results above current-plan do not materially affect continuity or safety. |
| Simpler alternative | Add `Approve T<number>:` to generic continuation detection and reuse the existing current-plan promotion. |
| Failure | The detector is too broad, treats approval-command retrieval as execution authorization, or disrupts explicit migration-apply gate ranking. |

## Measurement

Startup and live evidence before implementation:

- Lean `orient` trace `019e849d-ffa8-74b1-9f9f-332e83b36495` returned the latest T117
  current-plan memory first, so this was not an `orient` payload issue.
- Direct current-plan search trace `019e849e-27ef-7f50-bb23-0e9f67600368` returned the T117
  current-plan first, so broad continuation retrieval remained healthy.
- Exact T70 direct memory search trace `019e849e-7a30-7c31-9bf1-9192eef0b36d` ranked older T110
  and T109 handoffs above the T117 current-plan for
  `Approve T70: index exact files T59, T68, and T69.`.
- Exact T70 document search still returned older T64/T59/T58 material, confirming that document
  visibility remains a separate indexed-docs gap.

AI consultation:

- AI Council recall found prior ranking decisions that permitted intent-local ranking fixes with
  deterministic fixtures, while rejecting payload expansion, lifecycle cleanup, and broad ranking
  changes.
- AI Council broadcast agreed the slice was justified if it stayed search-only, required a strict
  `approve t<number>:` command shape, promoted only matching active current-plan items, and
  preserved migration-gate precedence.
- Claude Bridge accepted the plan with the same boundary: retrieval only, no write semantics, and
  no generic `approve`/`approval` classifier broadening.
- The simpler alternative of folding the command into generic continuation detection was rejected
  because existing continuation promotion does not require the current-plan item to contain the
  exact command text.

## Implementation

`engram-index/src/memory_ranker.rs` now has a separate search-only promotion pass:

- It runs only when `require_text_match` is true and the caller supplies project or cwd scope.
- It recognizes only normalized commands that start with `approve `, contain a `t<number>` task
  reference before a colon, and include non-empty text after the colon.
- It does not match `approval gate`, modal approval questions, bare `approve`, or approval text
  without the colon-delimited task command.
- It promotes only active current-plan guidance items: active Decision/Rule MemoryItems tagged
  `current-plan`.
- The current-plan item must contain the exact normalized command in title, content, or tags.
- The promotion runs before explicit migration-apply gate promotion so migration-apply gate
  queries can still override it.

`engram-tests/tests/search_tests.rs` adds a T118 live-shaped fixture:

- Older active handoffs contain the exact T70 approval command with stronger raw text placement.
- The active T117-style current-plan contains the exact approval command in content and records
  that T69 inspection and M6 write apply remain separately gated.
- The exact command query must rank the current-plan first.
- A non-command `Approve T70 without colon` query must not trigger the new promotion.

## Validation

Passed:

- `cargo test -p engram-index exact_approval_command_detector_is_narrow`
- `cargo test -p engram-tests test_memory_search_t118_exact_approval_command_promotes_matching_current_plan`
- `cargo test -p engram-index memory_ranker::tests`
- `cargo test -p engram-tests --test search_tests test_memory_search`
- `cargo fmt --all --check`
- `git diff --check`
- `cargo check -p engram-cli`

No installed-runtime MCP search was run for this slice; the changed behavior is covered by
deterministic unit and integration fixtures in the changed crates.

## Completion Matrix Delta

| Area | State | Evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| Exact approval-command memory search | Validated by fixture | T118 fixture ranks matching active current-plan above old handoffs | Needs installed-runtime/live recheck after binary install if this becomes an operational release gate. |
| Generic current-plan search | Preserved | Existing memory-search fixtures still pass | Older stale current-plan memory remains active and noisy in some live results. |
| Explicit migration-apply gate ranking | Preserved | Existing ranker and memory-search migration gate fixtures still pass | M6 write apply remains blocked by confidence and approval gates. |
| `orient` hot path | Unchanged | No `orient` code or fixture changed | Keep `orient` compact; do not move this search-specific behavior into payload expansion. |
| Document index visibility | Unchanged/risky | T117/T118 evidence still shows stale exact T70 doc results | Exact-file document indexing remains a separate approved/gated path. |

## Interpretation

T118 fixes a specific continuity hazard: exact approval-command searches no longer let old handoffs
outrank the active current-plan that defines the current gate state. The change is deliberately
retrieval-only. It helps the agent read the authoritative plan before deciding what an approval
means; it does not itself acknowledge approval, execute a gated action, or broaden any gate.

The next non-gated work should continue improving evidence quality or cross-harness validation. T70
document indexing, T69 inspection, M6 apply/deletion/lifecycle mutation, harness writes, and
public/schema/index behavior changes still require their exact approval gates.
