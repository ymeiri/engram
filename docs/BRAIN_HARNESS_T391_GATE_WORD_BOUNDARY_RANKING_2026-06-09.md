# Brain Harness T391 Gate Word-Boundary Ranking

Date: 2026-06-09
Branch: `yuval.meiri/memory-os-phase1`

## Research Question

Can deterministic gate-language classification avoid treating unrelated words such as `gateway`,
`gatekeeper`, and `gatedness` as gate context while preserving real gate terms such as `M6 gate`,
`gated state`, and `review-gated`?

## Hypotheses

- Preferred: bare gate terms can use ASCII word-boundary matching while multi-word gate phrases
  stay substring-based, reducing false contextual M6 gate promotion without weakening real gate
  guidance.
- Null: existing substring matching is sufficient and the extra boundary handling does not change
  observable ranking behavior.
- Failure: word-boundary handling hides real `gate`, `gated`, or safety vocabulary needed for
  approval-gate guidance.

## Change

`engram-index/src/memory_ranker.rs` now uses `contains_ascii_word` for short gate-language terms
such as `gate`, `gated`, `must`, `blocked`, `cannot`, and `never`. Multi-word or hyphenated
phrases such as `approval gate`, `review-gated`, `do not`, `should not`, and `requires approval`
remain exact substring checks.

This keeps continuation prompts like `current plan next M6 gateway routing confidence` from
creating contextual M6 gate promotion just because `gateway` starts with `gate`.

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

- `gateway_words_do_not_create_gate_context`
- `gate_boundary_words_still_trigger_gate_context`

## Boundary

T391 changes only deterministic MemoryItem ranking query classification. It does not accept the
hosted-CI fallback, mark PR #3 ready, merge, tag, publish, launch native Claude, run `/hooks`,
prove live host labels, mutate M6 state, run lifecycle cleanup, or make the system production/GA
complete.
