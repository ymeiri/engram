# Engineering Context Evaluation v2

V2 keeps the twelve scenarios, twelve arms, three repetitions, absolute packet limits, and hard
safety gates from frozen v1. It changes only the comparison contract needed to answer the product
question: does a memory treatment add value over the corresponding host baseline?

The v1 calibration showed that current Codex bare-host input already exceeds several absolute v1
token limits. V2 therefore retains those absolute violations in the arm report but adds explicit
within-host treatment-minus-baseline latency and total-token budgets. It never subtracts Codex from
Claude Code or combines unmatched models, versions, fixture revisions, scenarios, or repetitions.

## Preregistered comparisons

Each conceptual comparison has one Codex pair and one Claude Code pair:

| Claim group | Baseline | Treatment |
| --- | --- | --- |
| `native_vs_instructions` | instructions only | native memory |
| `native_skills_vs_native` | native memory | native memory plus repository skills |
| `current_engram_vs_native_skills` | native memory plus skills | current Engram |
| `engram_plus_native_vs_native_skills` | native memory plus skills | Engram plus native memory |
| `lean_engram_vs_native_skills` | native memory plus skills | lean Engram |

The incremental budgets are frozen in `manifest.json`. They are deliberately host-specific and
derived conservatively from the 2026-08-07 smoke calibration: the Codex MCP turn has a much larger
host-reported token increment than Claude Code, while the lean Engram limits remain below the
current/full Engram limits.

## Claim rule

A comparison observes incremental value only when all 36 scenario/repetition pairs are present and:

- host, host version, model, and fixture revision match inside every pair;
- no pair regresses task success, clean acceptance, identity, first action, abstention, scope
  leakage, stale influence, secret leakage, repeated failures, corrections, or evidence quality;
- at least one pair strictly improves a declared non-cost metric;
- every pair stays within the preregistered incremental latency and token budgets;
- every treatment run stays within the scenario's frozen absolute context-packet limit.

Pairwise non-regression prevents aggregate cancellation: a secret leak or wrong-scope regression in
one scenario cannot be hidden by success elsewhere. A claim group is portable only when its Codex
and Claude Code comparisons independently meet the rule. Missing runs produce an incomplete report,
not a positive claim.

Validate without provider execution:

```bash
cargo run -p engram-eval -- validate \
  --manifest evals/engineering_context_v2/manifest.json
```

Score collected JSONL records and optionally enforce the portable claim gate:

```bash
cargo run -p engram-eval -- score \
  --manifest evals/engineering_context_v2/manifest.json \
  --results /path/to/results.jsonl \
  --output /path/to/report.json \
  --require-portable-incremental-value
```

The flag succeeds when at least one preregistered claim group observes incremental value on both
hosts. The report still exposes every arm, pair, missing pair, safety regression, cost delta, and
budget violation so the threshold cannot replace the underlying evidence.
