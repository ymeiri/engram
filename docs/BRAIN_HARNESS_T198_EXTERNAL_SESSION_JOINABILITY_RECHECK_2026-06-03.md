# T198 External-Session Joinability Recheck

Date: 2026-06-03
Status: docs-only/read-only audit; no source behavior change

## Scope

T198 rechecks the external-session joinability gap after T197 resolved the live native Claude
process. It does not change source behavior, public MCP parameters, telemetry formulas,
schema/storage/index behavior, document-index behavior, ranking, `orient`, lifecycle state,
harness files/settings/hooks/adapters, M6/migration/quarantine state, deletion, rollback, or
user-owned files.

No new AI Council or Claude Bridge consultation was run because this slice is a read-only
source/test/runtime audit preserving the existing T24 decision rather than a new architecture,
ranking, migration, data-model, eval-design, or irreversible decision. Model critique should be
used before changing host-session contracts or harness integration.

## Research Framing

Question: after T197, is `external_session_trace_count=0` in the current live project telemetry
window evidence of a core telemetry implementation failure, or still a host/caller adoption and
host-session availability gap?

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Core telemetry still supports caller-supplied host labels; the current zero count is caused by callers not supplying `external_session_id`. | Supported. |
| Null | The old T24 audit is sufficient and no new evidence is needed. | Rejected for completion tracking because the live count is currently zero again. |
| Simpler alternative | Record the current zero count as a caveat only. | Too weak; source/tests should be checked before classifying the gap. |
| Failure | Core telemetry lost pass-through, inheritance, counting, or validation behavior. | Not observed. |

Measurement:

- Inspect current runtime telemetry.
- Inspect current source paths for `external_session_id`.
- Run focused telemetry regression tests covering pass-through, feedback inheritance, and report
  counts.
- Do not create synthetic live traces with fake external-session labels; that would make host
  joinability look stronger than the available host evidence.

## Runtime Evidence

Fresh `telemetry(action="real_session_eval", project="engram", limit=50)` at
`2026-06-03T18:05:42Z` returned:

| Metric | Value |
| --- | --- |
| `trace_count` | `50` |
| `feedback_trace_count` | `22` |
| `feedback_coverage` | `0.4399999976158142` |
| `confidence_gate.passed` | `false` |
| `external_session_trace_count` | `0` |
| `distinct_external_session_count` | `0` |
| `unspecified_external_session_trace_count` | `50` |
| `external_session_feedback_count` | `0` |
| `unspecified_external_session_feedback_count` | `22` |
| `task_failure_count` | `0` |
| `bad_memory_used_count` | `0` |

Fresh `telemetry(action="list_traces", project="engram", limit=15)` showed all 15 newest project
traces had `external_session_id=null`, including T198 startup traces:

- `019e8ea9-5082-7c01-bd3e-59aa9037f0e0`
- `019e8ea9-515b-73d2-bb4f-76b0f811b899`
- `019e8ea9-5224-7a11-ae87-fa8a0a550fa0`
- `019e8ea9-52f3-7862-aea6-2b4674252d59`
- `019e8ea9-53bd-7853-ac30-c905667ce714`
- `019e8ea9-7706-7991-8683-3c17308434e8`

This confirms the current rolling live window has no host-session labels. It does not by itself
prove why.

## Source Evidence

Current source inspection supports the T24 implementation claim:

- `engram-core/src/telemetry.rs` stores `external_session_id` on both `BrainHarnessTrace` and
  `AgentFeedback`, provides `BrainHarnessTrace::with_external_session_id`, and exposes report
  fields for trace and feedback external-session counts.
- `engram-index/src/telemetry.rs` copies the trace label onto feedback when feedback omits it,
  validates labels as non-empty and at most 256 characters when provided, aggregates trace and
  feedback external-session counts, and recommends setting `external_session_id` when a known host
  thread/session ID is available.
- `engram-mcp/src/tools.rs` exposes `external_session_id` on `SearchRequest`, `OrientRequest`,
  `TelemetryRequest`, and `MemoryRequest`, and passes it through to `search`, `orient`,
  `telemetry(record_trace/submit_feedback)`, and `memory(changes_since)` trace creation.
- `engram-index/src/memory.rs` passes `OrientInput.external_session_id` and
  `MemoryChangesSinceOptions.external_session_id` into the corresponding trace records.
- `engram-cli/src/main.rs` still sets `external_session_id: None` for the CLI `orient` and
  `memory changes-since` paths, so those local CLI calls cannot contribute host-session labels.

The source evidence points to missing caller-supplied labels in current live use, not missing core
telemetry storage or report plumbing.

## Validation

Commands:

```text
cargo test -p engram-tests --test telemetry_tests mcp_telemetry_tool_records_trace_feedback_and_stats -- --exact
cargo test -p engram-tests --test telemetry_tests
```

Results:

- The focused MCP telemetry test passed and covers `telemetry(record_trace)` with
  `external_session_id`, `submit_feedback` inheritance from the trace, and
  `real_session_eval` external-session counts.
- The full telemetry test target passed: `23 passed; 0 failed`.

Relevant covered surfaces include:

- service-level `orient` pass-through:
  `orient_with_intent_emits_trace_for_agent_feedback`,
- service-level `changes_since` pass-through:
  `changes_since_with_intent_emits_trace_for_agent_feedback`,
- MCP `orient` pass-through:
  `mcp_orient_tags_trace_with_scenario_and_arm`,
- MCP `search` pass-through:
  `mcp_search_returns_trace_id_when_telemetry_is_initialized`,
- telemetry tool record/feedback/report behavior:
  `mcp_telemetry_tool_records_trace_feedback_and_stats`.

## Completion Matrix Delta

| Area | State After T198 | Evidence | Remaining Gate |
| --- | --- | --- | --- |
| Core telemetry label support | Implemented and regression-covered | Source inspection plus `telemetry_tests` | None for core pass-through/counting. |
| Live host-session joinability | Incomplete | Current 50-trace report has `external_session_trace_count=0` | Needs real caller/host labels, not synthetic labels. |
| Confidence gate | Unstable rolling evidence | Current report fails at 44% coverage after new traces | Score assessable traces or use scoped evals before relying on it. |
| `orient` hot path | Preserved | No code or payload change | Do not expand for this gap. |
| Harness/host integration | Not changed | No adapter/hook/settings writes | Requires separate approval and likely model critique before any host-session contract. |
| M6/migration | Still blocked | No migration action | Needs reviewed decisions, dry-run/rollback evidence, telemetry readiness, and explicit approval. |

## Decision

Keep external-session joinability marked incomplete, but classify the current gap as caller/host
adoption and host-session availability rather than a core telemetry implementation failure.

Do not auto-fill `external_session_id` from transport metadata, process IDs, Codex thread guesses,
or synthetic labels. The correct next step, if this gap is prioritized, is an explicit host-session
contract or harness integration slice that makes a real stable host label available to MCP calls.
That would be a separate approval-gated design change, not a T198 follow-up.

T198 does not close broad Brain Harness completion. Remaining separate gates include T172
effective-hook visibility, exact-file document indexing for newer gate docs, lifecycle archive
packets, M6/migration, stale handoff noise, telemetry coverage stability, and cross-harness
host-session labeling.
