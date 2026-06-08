# Brain Harness T390 Ungated Gate-Context Ranking

Date: 2026-06-09
Branch: `yuval.meiri/memory-os-phase1`

## Research Question

Can continuation prompts that use `ungated`, `un-gated`, or `not gated` near an M6 reference
avoid false contextual migration-gate promotion while explicit gate-action prompts still surface
gate guidance first?

## Hypotheses

- Preferred: the ranker can treat `ungated` variants as continuation vocabulary, matching the
  existing `non-gated` behavior, without weakening explicit `should we proceed/apply/run` gates.
- Null: existing `non-gated` handling is sufficient and `ungated` variants do not affect gate
  classification.
- Failure: stripping the extra wording hides real approval or migration apply prompts.

## Change

`engram-index/src/memory_ranker.rs` now uses one shared
`remove_continuation_gate_negations` helper for gate-language checks. The helper removes:

- `non-gated`
- `non gated`
- `un-gated`
- `ungated`
- `not gated`
- `not a gate`

This keeps continuation wording from matching the `gated`/`gate` substring checks used for
contextual gate promotion. Explicit modal gate actions still trigger gate mode because action words
such as `should we proceed`, `should we run`, `apply`, and `write` remain intact.

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

- `ungated_continuation_words_do_not_create_gate_context`
- `explicit_gate_actions_survive_ungated_wording`

## Boundary

T390 changes only deterministic MemoryItem ranking query classification. It does not accept the
hosted-CI fallback, mark PR #3 ready, merge, tag, publish, launch native Claude, run `/hooks`,
prove live host labels, mutate M6 state, run lifecycle cleanup, or make the system production/GA
complete.
