# Brain Harness T42 Claude T41 Parity

Date: 2026-05-31
Status: Pre-registered
Scope: Read-only Claude Code search parity for the T41 mixed-query behavior

## Boundary

T42 follows from the T41 fixture and validates only the installed read path from Claude Code. It
must not run M6 inventory or review export, mutate memory lifecycle state, archive or scope-rewrite
memory, change schemas or storage, change public MCP request parameters, expand `orient`, change
ranking behavior, or install or modify harness adapters or hooks.

If the Claude Code surface cannot expose the Engram `search` tool, or if the expected behavior
requires query changes, retries, ranking changes, lifecycle cleanup, M6 work, or harness writes,
stop and record the result as blocked or partial rather than fixing it in this slice.

## Research Question

Does Claude Code, through its own Engram MCP read path, reproduce the exact T41 live search behavior
for the mixed current-plan/M6-gate query and the explicit M6 negative-control query?

## Pinned Records

| Label | Memory ID | Expected role |
| --- | --- | --- |
| Latest current plan | `019e7d02-9f75-7cb2-ae56-a9544a961b25` | Rank 1 for the mixed current-plan query. |
| Active M6 gate | `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` | Top-five for the mixed query; rank 1 for the negative-control query. |
| Stale repository current plan | `019e5e0a-86b4-73e3-aa9b-ca350e83e915` | Must not outrank the latest current plan. |

If a pre-run cursor or direct `search` recheck shows these records changed before the scoreable
Claude Code run, pause and update the pre-registration before scoring.

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Claude Code returns the same material record ordering as Codex for the two exact queries: latest current-plan first for the mixed query, active M6 gate first for the negative-control query. |
| Null | The T41 behavior remains Codex-local or data-state-specific, and Claude Code does not reproduce the expected record IDs/order. |
| Simpler alternative | Keep T41 as deterministic fixture-only evidence and defer cross-harness claims until a later real Claude Code task needs the surface. |
| Failure | The parity check needs query edits, selective retries, lifecycle/M6/harness writes, ranking changes, or manual reinterpretation to pass. |

## Measurement

Before the scoreable Claude Code run:

- capture a Memory OS cursor,
- run the same two exact searches from Codex as a baseline,
- assert the pinned records still match the expected ordering thresholds,
- run `obligations(action=doctor)` and record that no open obligations exist.

Scoreable Claude Code checks use only `mcp__engram__search` and these exact queries:

1. `current plan next non-gated Brain Harness feedback confidence M6 gate`
2. `approved M6 write apply deletion cleanup legacy simplification now`

Pass criteria:

- the mixed query returns latest current-plan memory
  `019e7d02-9f75-7cb2-ae56-a9544a961b25` first,
- the mixed query returns active M6 gate memory
  `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` within the first five memory results,
- stale current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` does not outrank the
  latest current plan,
- the negative-control query returns active M6 gate memory
  `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` first,
- no current-plan memory outranks the M6 gate for the negative-control query,
- the Claude Code result differs from the Codex baseline only outside the asserted record IDs and
  rank thresholds,
- post-run `memory(action=changes_since)` shows no unexpected relevant memory writes during the
  scoreable window, and post-run `obligations(action=doctor)` is clean or any synthetic smoke
  obligations are explicitly resolved/skipped as validation artifacts.

Failure or pause criteria:

- Claude Code cannot call Engram `search`, times out, or returns malformed results,
- expected pinned records are missing or changed before scoring,
- the run needs query edits, selective retries, or cherry-picked output,
- the scoreable window includes unexpected relevant memory writes,
- any result is interpreted as M6 inventory/export/apply/deletion approval,
- any required next action crosses the T42 boundary.

## Consultation

AI Council recall found the T38/T41 boundary guidance: keep the slice intent-local, avoid payload
expansion, lifecycle cleanup, broad ranking, or gated M6/harness work. A T42 AI Council broadcast
agreed the validation should pin exact queries, exact record IDs, top-k thresholds, no special
handling, and narrow conclusion language.

Claude Bridge critique added three controls to this registration: pin the expected records by ID,
define the parity metric as record identity plus asserted rank thresholds, and use a pre/post
cursor check so mid-run memory writes cannot silently shift the ground truth.

## Allowed Conclusion

If T42 passes, it supports only this claim: Claude Code reproduced the installed read-only T41
mixed-query and M6 negative-control behavior for two exact queries in the current data state.

It does not prove broad search quality, broad cross-harness readiness, M6 approval, lifecycle
cleanup safety, hook or adapter readiness, or any need to change `orient`.
