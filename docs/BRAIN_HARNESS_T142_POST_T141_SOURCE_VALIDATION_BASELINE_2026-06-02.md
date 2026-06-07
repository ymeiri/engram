# T142 Post-T141 Source Validation Baseline

Date: 2026-06-02

Scope: non-destructive source validation after T140 and the T141 approval packet. This slice did
not install a binary, restart the Engram daemon, edit installed hooks/settings/adapters, run
`harness install`, use `adopt_user_owned`, mutate lifecycle state, run M6, inspect quarantine
candidates, or change public MCP/schema/storage/index/ranking/`orient`/document-index behavior.

## Research Question

Does the committed source tree still pass critical validation after the narrow T140 direct-search
ranking fix and the docs-only T141 runtime-refresh approval packet?

## Hypotheses

| Kind | Hypothesis |
| --- | --- |
| Preferred | The committed source tree remains clean: formatting, focused T140 ranking/search coverage, CLI check, clippy, full tests, and diff hygiene all pass. |
| Null | Broad validation exposes a regression or a lint/test failure that must be fixed before any runtime refresh should be considered. |
| Simpler alternative | Skip the broad source baseline and rely only on the focused T140 tests; this would leave shared behavior and integration coverage unverified. |
| Failure | Treating a green source baseline as installed runtime parity, harness readiness, lifecycle cleanup, or migration completion. |

## Measurement

- `cargo fmt --all --check`
- `cargo test -p engram-index memory_ranker::tests -- --nocapture`
- `cargo test -p engram-tests --test search_tests -- --nocapture`
- `cargo check -p engram-cli`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
- `git diff --check`

## Results

All validation commands passed.

- Focused ranker coverage passed: 11 tests.
- Focused search integration coverage passed: 32 tests, including
  `test_memory_search_t140_continuation_with_approval_gate_context_promotes_current_plan`.
- `cargo check -p engram-cli` passed.
- `cargo clippy --all-targets -- -D warnings` passed.
- `cargo test --all-targets` passed across unit and integration coverage. Model-download tests
  remained intentionally ignored.
- `git diff --check` passed.

## Completion Matrix Delta

| Area | T142 status | Evidence |
| --- | --- | --- |
| T140 source behavior | Source-validated | Focused ranker and `search_tests` suites passed again. |
| Shared repo correctness | Source-validated | `cargo clippy --all-targets -- -D warnings` and `cargo test --all-targets` passed. |
| T141 runtime refresh | Still gated | T142 did not install the binary or restart the daemon. Exact T141 approval is still required before live T140 validation. |
| T133A SessionEnd live render | Historical success | T133A is already committed as `97ccfe7`; repeating that install/restart at current `HEAD` would also deploy T140 ranking code, so T142 did not rerun it under stale T133A wording. |
| Harness readiness | Still missing | T142 did not edit installed hooks/settings/adapters or run `harness install`. |
| Lifecycle cleanup | Still gated | T142 did not archive stale MemoryItems, run `lint apply_safe`, or mutate handoffs. |
| M6 migration completion | Still gated | T142 did not run migration apply/delete/cleanup or inspect quarantine candidates. |

## Decision

T142 establishes a clean source baseline only. The next runtime-moving step remains the exact T141
approval gate for installing the current binary, restarting the daemon, and read-only validating the
T140 continuation/current-plan approval-gate-context query class.

## Stop Conditions Preserved

Pause before any binary install/daemon restart unless the user approves the exact T141 scope; before
any hook/settings/adapter write; before `harness install`; before `adopt_user_owned`; before
lifecycle archive/apply or handoff semantic change; before M6 migration/quarantine/apply/delete;
and before public MCP/schema/storage/index/document-index/`orient`/ranking-source changes.
