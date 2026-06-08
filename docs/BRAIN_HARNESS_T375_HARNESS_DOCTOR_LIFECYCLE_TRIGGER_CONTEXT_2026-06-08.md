# Brain Harness T375 Harness Doctor Lifecycle Trigger Context

Date: 2026-06-08
Status: implemented, installed, and validated in the live local/Codex runtime.

## Research Question

Can `engram harness doctor` distinguish "adapters are installed" from "lifecycle compliance is
still agent-advisory" by naming the exact soft lifecycle triggers that remain outside hard runtime
enforcement?

Preferred hypothesis: when a harness is ready, `doctor` keeps `ready=true` and adds the canonical
policy trigger names to the advisory warning.

Null hypothesis: `doctor` continues to emit only a generic soft-lifecycle warning, making release
and production-gate reports less actionable.

Failure hypothesis: the change makes status/doctor readiness noisy, changes install behavior, or
breaks MCP harness integration.

## Change

`engram-index/src/harness.rs` now formats the ready-state `doctor` warning from
`report.policy.lifecycle_triggers`. The report still uses the existing policy data and does not add
new lifecycle enforcement or new install behavior.

Ready `doctor` output now includes:

```text
Advisory triggers: task_start_orient, before_major_decision_changes_since,
after_discovery_record, before_final_changes_since, before_final_obligations,
before_context_compaction_save, session_end_handoff, commit_workflow_consult_memory.
```

## Validation

Focused source validation passed:

- `cargo test -p engram-index harness::tests::doctor_names_soft_lifecycle_triggers_when_ready`
- `cargo fmt --all --check`
- `cargo check -p engram-cli`
- `cargo test -p engram-tests --test harness_tests`
- `cargo clippy -p engram-index -- -D warnings`

Runtime adoption passed:

- `cargo build --release`
- installed `/Users/yuval.meiri/.local/bin/engram` from `./target/release/engram`
- installed and target hashes both:
  `8eb6ce4b0789fb23d49d8e1d0bead6f4b22409f5301fd5c5be01394d949bbcfe`
- restarted daemon on PID `54091`
- `/health` returned `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`
- live `engram harness doctor --harness codex --json` returned `ready=true` with the advisory
  trigger list
- live `engram harness status --harness codex --json` remained `ready=true` with no warnings
- `cd dist && shasum -a 256 -c engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz.sha256`
  returned OK

## Gate Impact

T375 improves production-gate evidence quality for lifecycle compliance. A ready harness report
now shows exactly which lifecycle steps remain soft agent obligations instead of presenting a vague
soft-compliance warning.

This does not install or mutate hooks/adapters, enforce lifecycle behavior, archive sessions or
memory, run `lint apply_safe`, mutate M6/migration state, change ranking or `orient`, launch
native Claude, run `/hooks`, signal processes, mark PR #3 ready, merge, tag, publish, or change
the beta scope.
