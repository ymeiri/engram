# Brain Harness T373 Stale Session Lint Context

Date: 2026-06-08
Status: implemented a small production-hardening improvement for Memory OS lifecycle review.

## Scope

T373 improves the actionability of stale active-session lint findings. The previous finding named
only the session ID and said that the session had been active for more than one day. That was enough
to detect lifecycle debt, but not enough to safely review cleanup without extra lookups.

This slice changes only lint finding text. It does not end or abandon sessions, archive memory,
run `lint apply_safe`, mutate obligations, change ranking or `orient`, mutate M6/migration state,
launch native Claude, run `/hooks`, signal processes, mutate settings/adapters, mark PR #3 ready,
merge, tag, publish, or change beta scope.

## Change

`engram-index/src/lint.rs` now includes concrete review context in each
`stale_active_session` finding message:

- project, or `unknown`;
- agent, or `unknown`;
- RFC3339 `started_at`;
- `age_hours`.

The finding still uses `safe_action=none`. This keeps stale-session cleanup as a reviewed operator
decision rather than an automatic lifecycle mutation.

## Validation

Focused source validation passed:

- `cargo test -p engram-index lint_project_scope_filters_memory_obligations_and_sessions`
- `cargo test -p engram-index lint`
- `cargo test -p engram-tests --test lint_tests`
- `cargo fmt --all --check`
- `cargo check -p engram-index`
- `cargo clippy -p engram-index --all-targets -- -D warnings`

Final candidate validation after this note is present:

- `./scripts/local-ci.sh`
- `./scripts/package-install-smoke.sh`

## Gate Impact

T373 reduces production lifecycle review risk by making stale-session findings self-contained
enough for safe human or agent review. It does not close broad lifecycle cleanup, native Claude
prompt-bearing proof, effective-hook visibility, live host-label proof, hosted CI, M6 write-apply,
or production/GA readiness.
