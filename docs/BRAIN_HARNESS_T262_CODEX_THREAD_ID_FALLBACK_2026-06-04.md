# Brain Harness T262 Codex Thread ID Fallback

Date: 2026-06-04
Status: completed source-level implementation. This slice adds a guarded Codex Desktop
`CODEX_THREAD_ID` fallback for existing external-session telemetry labels in CLI and MCP source. It
does not add public MCP parameters, change response payload shape, change schema/storage/index/
document-index behavior, edit hooks/settings/adapters, refresh the installed runtime, run native
Claude, mutate lifecycle or M6 state, push or set upstream, delete data, or touch user-owned files.

## Research Question

Can Engram reduce Codex Desktop null `external_session_id` traces by adopting a host-native
`CODEX_THREAD_ID` fallback without violating the prior rule against synthesizing labels from
unrelated metadata?

## Hypotheses

- Preferred: `CODEX_THREAD_ID` is acceptable as a host-specific fallback when it is lower
  precedence than explicit labels and `ENGRAM_EXTERNAL_SESSION_ID`, requires a Codex host marker,
  and accepts only short safe tokens before constructing `codex://threads/{id}`.
- Null: host thread labels must be supplied only through `ENGRAM_EXTERNAL_SESSION_ID`; Engram
  should not read `CODEX_THREAD_ID`.
- Simpler alternative: document `ENGRAM_EXTERNAL_SESSION_ID=codex://threads/$CODEX_THREAD_ID`
  as manual setup. Rejected for this slice because it leaves Codex Desktop source behavior unable
  to use an already-present host thread label.
- Failure: ambient environment fallback mislabels non-Codex runs, overrides explicit labels, or
  breaks feedback inheritance from traces.

## Consultation

AI Council recall resurfaced the prior host-label caution: do not auto-fill from unrelated
transport metadata without a host-session contract. A fresh three-model broadcast agreed the slice
is safe if the fallback is host-specific, low-precedence, documented, and tested. The main
guardrails were precedence, whitespace handling, avoiding generic metadata, avoiding malformed
labels, and not expanding public API/schema.

Claude Bridge read-only critique raised two important blind spots: a bare `CODEX_THREAD_ID` could
be injected by non-Codex processes, and constructing `codex://threads/{id}` couples Engram to a
host convention. T262 therefore requires a Codex host marker (`CODEX_SHELL`,
`CODEX_INTERNAL_ORIGINATOR_OVERRIDE`, or `__CFBundleIdentifier=com.openai.codex`) and rejects
thread IDs longer than 128 bytes or containing characters outside ASCII alphanumeric, `-`, and `_`.

## Change

The resolver precedence is now:

1. Explicit MCP request or CLI flag value.
2. `ENGRAM_EXTERNAL_SESSION_ID`.
3. Guarded `CODEX_THREAD_ID` as `codex://threads/{trimmed_id}`.
4. No label.

`CODEX_THREAD_ID` is ignored unless a Codex host marker is present. Whitespace-only values are
ignored. Unsafe token shapes are ignored rather than encoded or stored.

The MCP `telemetry(submit_feedback)` path was corrected during validation: feedback with no
explicit `external_session_id` now leaves the field unset so `TelemetryService::submit_feedback`
inherits the trace label. This preserves trace-feedback joinability and prevents ambient Codex env
fallback from overriding an already-labeled trace.

## Validation

Passed:

- `cargo test -p engram-mcp external_session_id`
- `cargo test -p engram-cli external_session_id`
- `cargo test -p engram-tests --test telemetry_tests`
- `cargo fmt --all --check`
- `cargo check -p engram-cli`
- `cargo clippy --all-targets -- -D warnings`
- `git diff --check`

Additional source smoke:

- `cargo run -p engram-cli -- orient ... --data-dir /private/tmp/engram-t262-codex-thread-smoke
  --json` completed in the live Codex Desktop shell where `CODEX_THREAD_ID` is set.
- The same smoke against the default global store failed because the live daemon already held the
  SurrealDB lock, so the temp-store smoke was used instead. The orientation packet includes the
  trace ID but does not expose `external_session_id`; stored-label behavior is covered by focused
  resolver/runtime tests and telemetry integration.

## Decision

T262 narrows Codex Desktop source-level host-label adoption. It does not refresh the installed
daemon/runtime, prove native Claude or Gemini host labels, add telemetry source attribution, or
complete full cross-harness label adoption.
