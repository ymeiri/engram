# Brain Harness T92 Lint Superseded Visibility

Status: Implemented and locally validated

Scope: Improve read-only lint visibility for active items that have already been superseded by
another active memory item.

T92 does not archive memory, run `lint(action="apply_safe")`, inspect T69 files, run T70 document
indexing, run M6, change retrieval ranking, expand `orient`, change public MCP request fields,
change schema/storage/index behavior, change document-index behavior, or write harness adapters or
hooks.

## Research Question

After T91, direct search still shows old active handoff items alongside the refreshed handoff, but
`lint(action="run", limit=20)` is dominated by generic stale-feedback rows. Can lint surface
actionable superseded-active findings before generic stale-feedback noise while preserving stale
current-plan feedback as the first signal?

## Hypotheses

| Hypothesis | Prediction |
|---|---|
| Preferred | Move safe-action `superseded_item_still_active` findings ahead of generic stale-feedback rows, but keep `feedback_stale_current_plan` first. |
| Null | Current lint ordering is sufficient; agents can use larger limits or direct search. |
| Simpler alternative | Document the buried finding only and make no code change. |
| Failure | The report ordering implies lifecycle cleanup authority or hides stale current-plan feedback. |

## Measurement

- Pre-change `lint(action="run", limit=20)` returned stale-feedback rows first, led by stale
  repository-scoped current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915`, and did not show
  the superseded active handoffs visible in direct search.
- Direct search trace `019e831a-a496-7091-87fe-f775cd3fd3b3` showed current handoff
  `019e8316-ebd1-7220-b18e-f0d33110131a` plus superseded active handoffs
  `019e82f8-cada-7c31-b073-18ac41986b1e` and
  `019e82f3-53bc-7a83-9e39-cfdb29b06c44`.
- Source inspection showed `LintService::run` uses private priority ordering before truncating to
  the requested limit.

## Change

`engram-index/src/lint.rs` now orders safe-action `superseded_item_still_active` findings after
`feedback_stale_current_plan` and `feedback_wrong_scope_active_memory`, but before generic
`feedback_stale_active_memory`.

The focused regression
`lint_prioritizes_superseded_active_items_before_generic_feedback_noise` constructs stale
current-plan feedback, generic stale-feedback noise, and a superseded active handoff. With
`limit=2`, lint must return stale current-plan feedback first and the superseded active handoff
second.

## Completion Matrix Delta

| Area | State After T92 | Evidence | Remaining Risk |
|---|---|---|---|
| Lint visibility | Improved for superseded active memory | Focused `engram-index` test passes | Report visibility is not lifecycle approval |
| Current-plan feedback | Preserved as highest priority | Existing and new tests keep stale current-plan first | Stale repo current-plan still requires a separate lifecycle gate |
| Handoff cleanup | Still gated | T88 exact archive packet unchanged | Old handoffs remain active until approved lifecycle action |
| M6/document-index/harness | Unchanged and gated | No related tools or writes run | T69/T70/T88 exact approvals still required |

## Validation

- `cargo test -p engram-index lint_prioritizes_superseded_active_items_before_generic_feedback_noise`
- `cargo test -p engram-index lint_prioritizes_feedback_signals_before_duplicate_entity_noise`
- `cargo fmt --all --check`
- `cargo test -p engram-tests --test lint_tests`
- `cargo check -p engram-cli`
- `git diff --check`

Attempted live CLI validation with `cargo run -p engram-cli -- lint run --limit 10` was not used
as proof: inside the sandbox it hit a RocksDB log-rename permission error, and with escalation it
hit the live daemon's `~/.engram/data/LOCK`. The source-level and MCP integration tests are the
authoritative validation for this slice.

## Result

The preferred hypothesis held. Lint can now expose superseded-active cleanup candidates before
generic stale-feedback noise without changing lifecycle state or weakening stale current-plan
visibility.
