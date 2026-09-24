# Engineering Context Evaluation v1

This suite measures Engram's incremental value over checked-in instructions and host-native memory. It is deliberately centered on repository identity, scoped decisions, verified procedures, stale/no-result behavior, resume, secret rejection, and deletion—not generic conversational recall.

The manifest freezes twelve Codex/Claude Code configurations and requires three fresh-session repetitions of every scenario. A run is invalid when an Engram arm lacks matched CLI/daemon/schema/adapter attestation; this prevents source, installed binary, live daemon, and generated integration drift from contaminating comparisons.

## Protocol

1. Materialize the same fixture commit for every arm. Replace each `fixture://` URI with an isolated path while preserving its logical identity in the result.
2. Use a fresh host profile and fresh memory store for every arm. Seed only the records named by the scenario fixture. Do not let one arm learn from another.
3. Configure exactly one arm from `manifest.json`. For Engram arms, capture `engram daemon status` and `engram harness status`; do not run when attestation is degraded or drifted.
4. Start a new session, submit the scenario prompt unchanged, and retain the host trace. Do not reveal the expected result to the agent.
5. Record the first externally visible action before judging the final answer. Context keys are stable claim labels shared across host-native and Engram stores; they are not database IDs.
6. Run the scenario's deterministic acceptance check, verify cited evidence against the fixture, and write one JSON object per line using `run-record.example.json` as the shape.
7. Repeat from a clean snapshot. Randomize arm order. Keep model, permissions, tool availability, and fixture revision constant within a comparison.

`observed_markers` must contain only synthetic canaries that were actually found in a durable store, telemetry, logs, or injected context. Never place real secrets in the suite.

The latency limits are end-to-end guardrails, not claims about current performance. Packet and token budgets are intentionally declared before runs. Revise them only by publishing a new suite version after calibration; do not edit this manifest after seeing results.

The first native-host calibration is recorded in
[`CALIBRATION_2026-08-07.md`](CALIBRATION_2026-08-07.md). It leaves v1 frozen and documents that
the current Codex bare-host baseline already exceeds v1's absolute token limits, so a future v2
must preregister paired incremental-over-baseline budgets in addition to absolute cost.

That comparison-only revision is now frozen in
[`../engineering_context_v2/`](../engineering_context_v2/). V1 remains unchanged and valid for
reproducing the original absolute-budget report.

The first paid-run pipeline subset is preregistered separately in
[`../engineering_context_pilot_v1/`](../engineering_context_pilot_v1/). Its preparation command
attests binaries, materializes eight clean isolated fixtures, seeds four isolated `ENGRAM_HOME`
stores, and emits execution commands without calling either provider. It is a pipeline and
flagship-mechanics pilot, not the eventual native-memory incremental-value claim.

## Commands

```bash
cargo run -p engram-eval -- materialize \
  --output /path/to/new-or-empty-fixture

cargo run -p engram-eval -- seed \
  --fixture-map /path/to/new-or-empty-fixture/fixture-map.json \
  --data-dir /path/to/new-or-empty-engram-data

cargo run -p engram-eval -- validate \
  --manifest evals/engineering_context_v1/manifest.json

cargo run -p engram-eval -- score \
  --manifest evals/engineering_context_v1/manifest.json \
  --results /path/to/results.jsonl \
  --output /path/to/report.json
```

`materialize` creates fixed Git history for Atlas main/legacy checkouts, a moved clone, two
registered lookalike checkouts, a deliberately ambiguous non-Git directory, Orbit wrong-scope
evidence, executable acceptance fixtures, verification receipts, `fixture-map.json`, and a
portable semantic `seed-plan.json`. It refuses to overwrite a non-empty target. The seed plan is
an input to the host runner; materialization does not mutate an Engram store. `seed` applies that
plan to a fresh isolated RocksDB store, verifies procedure receipts, and emits the generated IDs
under stable semantic context keys. It also refuses to merge with or overwrite existing data.

The scorer reports task success, exact identity, first-action correctness, repeated failures, scope leakage, stale influence, correction burden, evidence accuracy, latency, tokens, packet size, budget violations, missing runs, and per-run hard-gate failures. It does not combine those into a vanity score.

## First-action labels

Each scenario declares its accepted first-action label. The runner maps the first meaningful host action—not hidden chain of thought—to that label. Examples include `inspect_repository_context`, `ask_scope_clarification`, `run_verified_procedure`, and `reject_secret_capture`. If the first action cannot be established from the trace, record a different explicit label; do not infer the expected one afterward.

## Decision rule

Wrong-scope context, stale influence, or a synthetic secret leak is a release-blocking failure even if the task eventually succeeds. A lean Engram arm is only compelling when it improves task success or first-action correctness and reduces repeated failures/corrections over the corresponding native-memory-plus-skills arm without increasing those safety failures. Report Codex and Claude Code separately.
