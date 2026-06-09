# Brain Harness T376 Structured Lifecycle Report

Date: 2026-06-08
Status: implemented, installed, and validated in the live local/Codex runtime.

## Research Question

Can harness status and doctor reports expose lifecycle compliance as structured machine-readable
state instead of requiring agents or release checks to parse warning text?

Preferred hypothesis: `harness status` and `harness doctor` include a `lifecycle` object with the
soft-contract flag, enforcement flag, advisory trigger list, and summary message while preserving
existing readiness behavior.

Null hypothesis: lifecycle state remains present only in policy and warning text, leaving release
and agent checks to infer or parse the distinction between adapter readiness and lifecycle
compliance.

Failure hypothesis: adding structured lifecycle state breaks CLI, MCP JSON serialization, harness
tests, or live installed runtime behavior.

## Change

`HarnessStatusReport` now includes:

```json
{
  "lifecycle": {
    "soft_contract": true,
    "enforced": false,
    "advisory_triggers": [
      "task_start_orient",
      "before_major_decision_changes_since",
      "after_discovery_record",
      "before_final_changes_since",
      "before_final_obligations",
      "before_context_compaction_save",
      "session_end_handoff",
      "commit_workflow_consult_memory"
    ],
    "message": "Lifecycle compliance is advisory; agents should follow the listed triggers."
  }
}
```

The CLI text output also prints the lifecycle contract and advisory trigger list before adapter
checks.

## Validation

Focused source validation passed:

- `cargo fmt --all --check`
- `cargo test -p engram-index harness::tests::doctor_names_soft_lifecycle_triggers_when_ready`
- `cargo test -p engram-tests --test harness_tests test_mcp_harness_doctor_returns_structured_lifecycle_report`
- `cargo test -p engram-tests --test harness_tests`
- `cargo check -p engram-cli`
- `cargo clippy -p engram-index -- -D warnings`
- `cargo clippy -p engram-cli -- -D warnings`

Runtime adoption passed:

- `cargo build --release`
- installed `/Users/yuval.meiri/.local/bin/engram` from `./target/release/engram`
- installed and target hashes both:
  `74023c3a5e8050c6d710328d7b4ddab1f0b4f73b35aa61bed35df6245c088c77`
- restarted daemon on PID `77916`
- `/health` returned `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`
- live `engram harness doctor --harness codex --json` returned `ready=true`, structured
  `lifecycle.soft_contract=true`, `lifecycle.enforced=false`, and the full advisory trigger list
- live `engram harness status --harness codex --json` returned the same lifecycle object with no
  warnings
- live text `engram harness doctor --harness codex` printed the lifecycle summary and advisory
  triggers
- `cd dist && shasum -a 256 -c engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz.sha256`
  returned OK

## Gate Impact

T376 improves machine-readable production-gate evidence. Agents, release checks, and future
automation can now inspect lifecycle advisory state directly without scraping warnings or inferring
from the full policy object.

This does not install or mutate hooks/adapters, enforce lifecycle behavior, archive sessions or
memory, run `lint apply_safe`, mutate M6/migration state, change ranking or `orient`, launch
native Claude, run `/hooks`, signal processes, mark PR #3 ready, merge, tag, publish, or change
the beta scope.
