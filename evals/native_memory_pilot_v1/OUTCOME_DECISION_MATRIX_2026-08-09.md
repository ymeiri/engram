# Native-memory pilot v1 outcome decision matrix — 2026-08-09

This matrix was written after teaching completed but before activation or evaluation. It does not
change `protocol.json`, the frozen run plan, acceptance metrics, or provider execution. Its purpose
is to prevent post-result reinterpretation and select the next smallest experiment from the
preregistered report.

## Integrity first

| Final state | Interpretation | Required action |
| --- | --- | --- |
| `invalid=true` | No outcome comparison is admissible. | Preserve every trace, diagnose the exact integrity failure, and prepare a new immutable plan only if the failure is in the evaluator or host boundary. Do not repair or replay a completed lane. |
| `complete=false` | The lifecycle is unfinished. | Continue only the frozen missing phase after its gates pass. Do not classify incremental value. |
| `complete=true`, valid | The six matched outcomes may be compared. | Read lane-level failures and raw matched deltas before using the aggregate signal. |

`all_acceptance_passed` is not a prerequisite for a valid comparison: a failed native comparator is
an outcome. For the flagship journey, however, the Engram and combined treatments must work on both
hosts; comparator failure alone cannot prove the product outcome.

## Aggregate signal

| `incremental_value_signal` | What it supports | Next smallest slice |
| --- | --- | --- |
| `not_observed` | This case did not show a strict, non-regressive Engram improvement over native memory. | Inspect which lanes failed. If every layer failed, repair the task or boundary before another comparison. If native memory matched or beat Engram, do not expand Engram retrieval or graph features; select a safety case only if Engram still has a plausible condition/scope advantage. |
| `partial` | At least one treatment improved on at least one host, but portability across both hosts is unproven. | Freeze one new case targeted at the missing host or metric. Do not repeat this case merely to seek a favorable aggregate. |
| `engram_across_hosts` | Lean Engram strictly and non-regressively improved on native memory for both hosts in this case. | Keep lean Engram as the default candidate and test wrong-scope plus stale-prerequisite abstention next. |
| `combined_across_hosts` | Engram plus native memory improved across both hosts, while lean Engram did not. | Preserve native-memory interoperation and test whether the advantage survives wrong-scope and stale-prerequisite cases before changing the default harness. |
| `engram_and_combined_across_hosts` | Both Engram treatments improved across hosts. | Prefer the lean treatment unless a preregistered metric favors the combined boundary; then test safety and correction propagation. |

The report's Pareto signal is descriptive. One case and one repetition cannot establish a product
claim even when portable incremental value is observed.

## Follow-up order

If the lifecycle is valid and an Engram treatment works on both hosts, add cases in this order:

1. Repository/project ambiguity and wrong-scope guidance: the correct outcome is explicit
   abstention with no foreign procedure execution.
2. Stale or prerequisite-mismatched procedure: the correct outcome is abstention, with the failed
   condition explained.
3. No-result behavior: the agent asks for the missing material information instead of inventing or
   broadening scope.
4. User correction and deletion propagation: a corrected record replaces the old guidance and a
   confirmed forget removes every Engram-owned projection without affecting unrelated state.

Each follow-up must be a new frozen protocol. Thresholds and current-pilot outcomes remain
immutable. Broader connectors, graph expansion, or retrieval redesign remain deferred unless one
of these experiments identifies a specific failure they can address.

