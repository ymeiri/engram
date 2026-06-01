# Brain Harness T110 Telemetry Window Regression

Status: Completed executable measurement regression; no telemetry behavior change
Date: 2026-06-01
Scope: Prove the current default `real_session_eval` sample can disagree with an explicit recent
window without changing confidence formulas, public request parameters, ranking, or hot-path
behavior.

This slice did not generate calibration traces, run M6 inventory/export/apply, inspect migration
candidate files, mutate lifecycle state, run `lint(action="apply_safe")`, index documents, change
ranking, expand `orient`, change public MCP/schema/storage/index behavior, change document-index
behavior, or write harness adapters/hooks.

## Research Question

Can Engram preserve T109's evidence-over-confidence stance by making the default-vs-recent
telemetry confidence divergence executable, without prematurely changing `real_session_eval`
semantics?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A focused regression should prove that an explicit recent window can fail while the larger default sample passes, giving future eval-design work a stable fixture before behavior changes. |
| Null | T109's written audit is enough; no executable coverage is needed. |
| Simpler alternative | Change `DEFAULT_REAL_SESSION_EVAL_LIMIT` from `10_000` to `50` immediately. |
| Failure | The test encodes a misleading product claim, or makes a future intentional default change look like a regression without documenting why. |

## Measurement

Source reads before implementation:

- `engram-index/src/telemetry.rs` shows `DEFAULT_REAL_SESSION_EVAL_LIMIT: usize = 10_000` and
  both scoped and unscoped `real_session_eval_report` select traces first, then fetch feedback for
  those sampled trace IDs.
- `engram-store/src/repos/telemetry.rs` orders sampled traces by `created_at DESC`, so an explicit
  `limit=50` is a recent-trace window.
- `engram-core/src/telemetry.rs` documents `sample_limit` as the maximum recent traces considered.
- Existing telemetry tests already covered trace-linked feedback sampling and scoped filter order,
  but did not preserve the T109 divergence.

AI Council recall returned the prior intent/eval decision: intent is shallow workflow metadata,
and passive coverage alone cannot establish confidence. A fresh AI Council broadcast split on
changing the default immediately:

- Claude Opus and Gemini recommended not changing the global default yet; both favored a
  docs/test or shadow-measurement step first.
- GPT-5.4 considered a default change aligned if tightly validated, but still required caller
  audit and regression coverage.

Claude Bridge timed out before returning a critique, so T110 used the common safe overlap from the
available evidence rather than making a behavior change.

## Implementation

Added
`real_session_eval_default_sample_can_mask_recent_window_failure` in
`engram-tests/tests/telemetry_tests.rs`.

The fixture creates:

- 30 older traces with feedback across three intents and explicit memory attribution;
- 50 newer traces with only 20 feedback records, all in `plan_work`;
- an explicit `real_session_eval_report(Some(50))`, which sees the recent sparse window and fails
  the confidence gate on feedback coverage;
- a default `real_session_eval_report(None)`, which still uses the larger default sample and passes
  because historical feedback dominates.

This intentionally documents current behavior. If a later approved eval-design slice changes the
default window, this test should be revised with that rationale rather than silently removed.

## Validation

Command:

```text
cargo test -p engram-tests real_session_eval_default_sample_can_mask_recent_window_failure
```

Result:

- targeted test passed;
- no production behavior changed.

## Interpretation

T110 strengthens the evidence loop by turning T109's measurement caveat into executable coverage.
It does not prove Brain Harness completion, and it does not authorize M6, lifecycle cleanup,
document indexing, harness writes, ranking changes, `orient` expansion, public MCP changes, or
schema/storage/index changes.

The next behavior-changing telemetry slice, if pursued, should first choose between:

- keeping the current default and requiring explicit recent-window calls for confidence-gate use;
- adding a dual-window or warning-style safeguard;
- changing the default only after caller audit and variance checks.

Those are eval-design decisions, not T110 outcomes.
