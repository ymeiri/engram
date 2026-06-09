# Brain Harness T392 Decision-Gate Action Boundary Ranking

Date: 2026-06-09
Branch: `yuval.meiri/memory-os-phase1`

## Research Question

Can deterministic decision-gate classification avoid treating incidental substrings such as
`mustache`, `blockchain`, `unblocked`, `allowance`, and `safetybelt` as approval or safety-gate
requests while preserving real gate-action wording?

## Hypotheses

- Preferred: single-word fallback action terms can use ASCII word-boundary matching, reducing false
  gate-mode promotion without weakening explicit modal gate prompts or real action terms.
- Null: existing substring matching is sufficient and the extra boundary handling does not change
  observable routing behavior.
- Failure: boundary matching hides real action or permission prompts such as `must not proceed`,
  `blocked`, `allowed`, `safety`, or `write-apply`.

## Change

`engram-index/src/memory_ranker.rs` now uses `contains_ascii_word` for fallback decision-gate action
terms in `asks_for_decision_gate` and for migration apply permission/action terms in
`asks_for_explicit_migration_apply_gate`.

This keeps prompts such as `current plan next M6 blockchain status` or
`current plan next M6 unblocked status` from entering decision-gate mode merely because they contain
`block` or `blocked` as a substring. Explicit modal phrases such as `should we proceed`, plus real
boundary-delimited terms such as `must`, `blocked`, `allowed`, `safety`, and `write-apply`, still
trigger gate classification.

## Validation

Focused validation passed:

```bash
cargo test -p engram-index memory_ranker::tests
cargo test -p engram-tests --test search_tests \
  test_memory_search_treats_non_gated_next_slice_as_current_plan -- --exact
cargo test -p engram-tests --test search_tests \
  test_memory_search_t40_mixed_query_surfaces_current_plan_and_m6_gate -- --exact
cargo test -p engram-tests --test search_tests \
  test_memory_search_promotes_m6_gate_context_below_current_plan_for_mixed_query -- --exact
cargo fmt --all --check
cargo check -p engram-index
cargo clippy -p engram-index --all-targets -- -D warnings
git diff --check
```

Exact-worktree release validation also passed:

```bash
./scripts/local-ci.sh
./scripts/package-install-smoke.sh
```

The package smoke rebuilt
`dist/engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz`, verified its `.sha256`
file, installed the packaged binary into a temporary prefix, confirmed
`engram 0.2.0-beta.1`, and verified packaged HTTP `/health` returned
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`.

New focused unit tests:

- `decision_gate_action_words_require_boundaries`
- `decision_gate_action_words_still_trigger_at_boundaries`

## Boundary

T392 changes only deterministic MemoryItem ranking query classification. It does not accept the
hosted-CI fallback, mark PR #3 ready, merge, tag, publish, launch native Claude, run `/hooks`,
prove live host labels, mutate M6 state, run lifecycle cleanup, or make the system production/GA
complete.
