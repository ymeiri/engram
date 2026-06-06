# T299 Brain Harness Beta Scope Contract Fixes

Date: 2026-06-07

## Research Question

After the user rejected the "35% production ready" framing, what can be dropped from the
initial beta and what public-contract fixes still block an honest local/Codex beta?

## Consensus

AI Council and eight read-only subagents converged on the same release boundary: the 35% figure
is telemetry feedback coverage from one T297 sample window, not beta readiness. The initial beta
is the supported local/Codex Brain Loop path. Production parity is a separate, lower-readiness
metric and must not block MVP shipping.

## Dropped From Initial Beta

The following remain explicit limitations or fast-follow work, not beta blockers:

- Native Claude prompt-bearing proof.
- Effective-hook visibility or result proof.
- Live Claude host-label proof.
- Direct legacy deprecation or deletion.
- Exhaustive lifecycle cleanup and broad `lint apply_safe`.
- Rustdoc warning cleanup.
- OIDC/Vault/native-Claude auth debugging.
- Full multi-host harness parity.
- New feature work.

## Beta Blockers Found

The fan-out audit found five initial-beta contract issues:

1. Codex/generic generated guidance paired scoped `obligations(action=detect, ...)` with
   unscoped `obligations(action=doctor)`, allowing unrelated project obligations to leak into
   final-response closeout.
2. MCP `search.project` was documented as telemetry-only even though it also drives scoped
   memory filtering for current-plan retrieval.
3. `engram serve --memory`, `--remote`, credentials, and `--port` were accepted on the default
   stdio/proxy path but only honored by the direct `--http` server path.
4. `docs(action="quarantine_review_export")` wrote generated review pages while returning
   `read_only=true`.
5. README/MCP setup docs overstated common-host support relative to the current beta decision.

The PR body and exact-head CI gate also remain head-specific: any commit after T298 needs its
own green PR CI before the branch can be treated as beta-ready. The T299 head
`37ca96f060293e4b584c4c9490a8205e010d3b6a` later passed exact-head PR CI run
`27076011668`.

## Fixes

- Scoped generated final-response guidance now tells agents to use
  `obligations(action=doctor, project=..., cwd=...)` alongside scoped detection.
- MCP `SearchRequest.project` now describes both telemetry correlation and scoped memory
  filtering.
- The default stdio/proxy `engram serve` path now rejects HTTP-only storage/port flags with a
  clear error instead of silently using persistent storage. Direct `engram serve --http ...`
  still honors those flags.
- Quarantine review export responses now return `read_only=false` with
  `writes_generated_review_pages=true`.
- README and MCP setup docs now state that the current beta is validated for the local/Codex
  Brain Loop path and that broader host parity remains follow-up work.

## Validation

- `cargo test -p engram-cli serve_`
- `cargo test -p engram-index render_codex_adapter_spells_out_document_lifecycle_disposition`
- `cargo test -p engram-tests test_mcp_obligations_doctor_scopes_to_project_and_cwd -- --exact`
- `cargo test -p engram-tests mcp_search_returns_trace_id_when_telemetry_is_initialized -- --exact`
- `cargo fmt --all --check`
- `git diff --check`
- `git diff --cached --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets --jobs 1`
- Exact-head PR CI run `27076011668` on
  `37ca96f060293e4b584c4c9490a8205e010d3b6a`: Check `1m32s`, Format `17s`,
  Docs `1m11s`, Clippy `1m44s`, and Test `40m10s`.

## Non-Claims

T299 does not mark PR #2 ready, merge the PR, tag a release, run native Claude, prove effective
hooks, prove live host labels, delete or deprecate direct legacy data, run broad
`lint apply_safe`, apply M6 decisions, or prove production-complete Brain Harness parity.
