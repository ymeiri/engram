# T200 CLI External-Session Label Contract

Date: 2026-06-04
Status: source implementation complete; runtime refresh not run
Scope: Direct CLI caller support for Brain Harness telemetry external-session labels

## Decision

Engram now exposes the existing Brain Harness `external_session_id` telemetry field on the two
direct CLI caller paths that T199 found were still hard-coded to `None`:

- `engram orient --external-session-id <ID>`
- `engram memory changes-since --external-session-id <ID>`

If the flag is omitted, both commands fall back to `ENGRAM_EXTERNAL_SESSION_ID`. Empty or
whitespace-only values normalize to unset. An explicit flag takes precedence over the environment,
including when that explicit value normalizes to unset.

This is a caller contract, not a telemetry-core change. It does not synthesize host transcript IDs,
does not inject MCP transport `mcp-session-id`, does not change MCP request or response shape, and
does not solve hidden Codex Desktop host-thread labeling by itself.

## Research Question

Can Engram make external-session telemetry joinability more complete for one concrete repo-local
caller surface without inventing a wrong session label?

## Hypotheses

| Type | Result |
| --- | --- |
| Preferred | Add explicit CLI label plumbing for direct `orient` and `changes-since`, preserving caller-supplied semantics. Supported. |
| Null | No repo-local source change is useful because only hosts outside the repo can provide meaningful labels. Rejected for CLI callers, still true for hidden host IDs. |
| Simpler alternative | Env-var only or `orient` only would be smaller. Rejected because it is less discoverable or leaves the known `changes-since` hole. |
| Failure | Engram might fabricate labels or conflate MCP transport IDs with host-session IDs. Avoided by explicit caller-only plumbing. |

## Implementation

Changed `engram-cli/src/main.rs` only:

- added optional `--external-session-id` to `orient`;
- added optional `--external-session-id` to `memory changes-since`;
- added `ENGRAM_EXTERNAL_SESSION_ID` fallback at the CLI boundary;
- normalized empty/whitespace labels to `None`;
- passed the resolved label into `OrientInput.external_session_id` and
  `MemoryChangesSinceOptions.external_session_id`;
- added focused unit tests for flag precedence, env fallback, whitespace handling, and command
  parsing.

Existing telemetry validation remains the source of truth for label quality and maximum length.

## AI Review

AI Council recall found no prior stored decision for this exact CLI contract. A three-model
broadcast agreed that the flag plus environment fallback is the smallest safe repo-local slice,
with explicit risks around precedence, PII, high cardinality, and future accidental
`mcp-session-id` substitution.

Claude Bridge was requested for a read-only critique but timed out after 120 seconds. That timeout
is recorded as a caveat, not as supporting evidence.

## Validation

Commands run:

```text
cargo test -p engram-cli external_session_id
cargo test -p engram-tests --test telemetry_tests orient_with_intent_emits_trace_for_agent_feedback -- --exact
cargo test -p engram-tests --test telemetry_tests changes_since_with_intent_emits_trace_for_agent_feedback -- --exact
cargo check -p engram-cli
cargo fmt --all --check
```

All passed. The first attempted focused command,
`cargo test -p engram-cli external_session_id -- --exact`, ran zero tests because `--exact`
filtered by full test name; it was immediately corrected with the substring-filter command above.

## Completion Matrix Delta

| Area | Status After T200 | Remaining Risk |
| --- | --- | --- |
| Core telemetry label storage/reporting | Unchanged, validated by prior tests | None found |
| MCP request pass-through | Unchanged | Callers must still supply labels |
| Direct CLI `orient` | Implemented for caller-supplied labels | Runtime-installed binary not refreshed in this slice |
| Direct CLI `memory changes-since` | Implemented for caller-supplied labels | Runtime-installed binary not refreshed in this slice |
| Codex Desktop live traces | Still incomplete | Host/thread ID is not exposed to Engram CLI by this change |
| Claude ordinary `orient`/`search` traces | Still caller-supplied | Hook/event labels remain separate and unchanged |
| External-session joinability | Improved for direct CLI callers only | Live MCP window still needs host adoption and scoring |

## Non-Actions

T200 did not:

- change public MCP parameters or payload shape;
- change telemetry storage/report formulas;
- change ranking, `orient`, schema/storage/index, or document-index behavior;
- edit hooks, settings, adapters, or user-owned files;
- inject MCP transport session IDs into telemetry;
- run native Claude, Claude Bridge write actions, harness install, document indexing, lifecycle
  archive, M6/migration/quarantine actions, deletion, rollback, runtime refresh, or old-binary
  reinstall.
