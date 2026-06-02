# Brain Harness T128 Claude Post-T127 Parity Check

Date: 2026-06-02
Status: Completed with handoff-continuity failure
Scope: Read-only Claude Code parity check for post-T127 startup retrieval

T128 asked Claude Code, through Claude Bridge, to run the same post-T127 startup retrieval checks
that Codex ran in T127. The retrieval portion partially passed: Claude Code recovered the active
current-plan memory first for lean `orient` and broad continuation search. The handoff portion
failed: Claude Code session-end automation wrote stub handoffs despite the bridge task being
`write=false`, superseding the detailed T127 Codex handoff.

No candidate files were inspected, and no migration status/prioritize/apply/rerun, lifecycle
mutation, document indexing, ranking change, `orient` expansion, public MCP/schema/storage/index
change, document-index behavior change, hook/settings/adapter install, or user-owned file adoption
was intentionally run by Codex.

## Research Question

After T127, does Claude Code recover the same current-plan and gate context as Codex without
degrading continuity?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Claude Code recovers current-plan memory `019e8786-9b04-74d1-bf03-2a01357f7aea` first in lean `orient` and broad continuation search, and `handoff(get)` returns the T127 handoff. |
| Null | Claude Code retrieval differs from Codex enough that the next action is ambiguous. |
| Simpler alternative | Rely on the Codex-only T127 evidence and skip Claude Code parity. |
| Failure | Claude Code cannot access the read-only tools, times out, creates synthetic obligations, or writes/supersedes handoff state during a read-only parity check. |

## Measurement

Claude Bridge evidence collected on 2026-06-02:

- Foreground `claude_ask` timed out after 120 seconds without a result.
- Background job `ccb_20260602085426_4157e438` succeeded under `write=false` with only read-only
  Engram tools allowed.
- Claude Code lean `orient` trace `019e878b-5758-7cf3-9591-e5c80a5c65d8` returned current-plan
  memory `019e8786-9b04-74d1-bf03-2a01357f7aea` first. The next items were the harness-write
  gate `019e7cde-b517-77d0-aaac-c8638811d4e8`, the document-ingestion limitation
  `019dd404-1783-7790-bee1-b64cd1b2b691`, the commit preference
  `019e03be-a9a5-7db2-848d-eb26ef78bcb5`, and stale repository-scoped current-plan memory
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915`.
- Claude Code broad continuation search trace `019e878b-5941-7512-8054-2fd61a11cc06` returned
  current-plan memory `019e8786-9b04-74d1-bf03-2a01357f7aea` first. Older handoff and historical
  architecture memories appeared below it, with the then-current T127 handoff
  `019e8786-c69e-7bc2-a6c2-abc6db937c8e` ranked fourth.
- Claude Code exact T125 search trace `019e878b-5b11-7f50-b863-67ad201fde13` returned the T127
  handoff first, older T126/T124/T70 handoffs second through fourth, and current-plan memory fifth.
  T125 was recoverable, but the result was noisy and not a clean approval-audit surface.
- Claude Code `memory(list)` returned exactly one active Engram project current-plan item:
  `019e8786-9b04-74d1-bf03-2a01357f7aea`.
- Claude Code `handoff(get)` did not return the expected T127 Codex handoff
  `019e8786-c69e-7bc2-a6c2-abc6db937c8e`. It returned Claude Code stub handoff
  `019e878a-6c8e-77f2-86c1-8ed8c1bb9b70`, which superseded the T127 handoff.
- After the bridge job finished, Codex rechecked `handoff(get)` and found another Claude Code
  session-end stub handoff, `019e878c-8be4-7363-9d37-bbe3d7070441`, superseding
  `019e878a-6c8e-77f2-86c1-8ed8c1bb9b70`.
- Claude Code generated two open obligations, `019e8788-9bae-7350-84c1-5e64737deb24` and
  `019e8788-9bae-7350-84c1-5e537a102289`, from the prompt framing. Codex skipped them as
  inapplicable to the read-only retrieval parity task.

## Result

Claude Code passes current-plan retrieval parity for the tested prompt class: the active T127
current plan is first in lean `orient` and broad continuation search, and the scoped current-plan
list has exactly one active item.

Claude Code fails handoff continuity for this run. The bridge job's read-only mode did not prevent
Claude Code session-end automation from writing stub handoffs. Those stubs superseded the rich T127
Codex handoff and do not carry the T125/T47 gate context. Current-plan memory therefore remained
the only reliable authoritative carrier of the next gate until Codex restored the handoff after
this report.

The exact T125 lookup remains noisy in both harnesses: handoffs tie above current-plan memory, and
there is no dedicated T125 gate item. The safe operational rule remains to recover T125 through the
active current plan and repository docs, not to treat exact approval search as a clean audit tool.

## Completion Matrix Delta

| Area | T128 state | Evidence |
| --- | --- | --- |
| Claude Code current-plan retrieval | Validated for the tested startup prompt | Claude orient trace `019e878b-5758-7cf3-9591-e5c80a5c65d8` and broad search trace `019e878b-5941-7512-8054-2fd61a11cc06` returned current-plan memory first. |
| Claude Code exact T125 lookup | Recoverable but noisy | Trace `019e878b-5b11-7f50-b863-67ad201fde13` ranked the T127 handoff first, older handoffs second through fourth, and current-plan fifth. |
| Rolling handoff continuity | Failed | Claude Code stub handoffs `019e878a-6c8e-77f2-86c1-8ed8c1bb9b70` and `019e878c-8be4-7363-9d37-bbe3d7070441` superseded the rich T127 handoff during a read-only bridge run. |
| Obligation quality | Noisy | Claude Code created two prompt-derived obligations that Codex skipped as inapplicable. |

## Validation

This is a docs-only evidence slice. Validation is limited to:

- Claude Bridge job result `ccb_20260602085426_4157e438`;
- local Codex recheck of `handoff(get)`, `obligations(doctor)`, `memory(list)`, and `git status`;
- exact-source documentation updates in the Brain Harness architecture, research method, and Memory
  OS implementation plan;
- `git diff --check` before commit.

## Next Gate

T128 does not approve hook, settings, adapter, or harness repair work. The handoff-write failure
should be carried as evidence for the existing harness-readiness and hook/configuration gap until
an exact harness-write scope is approved.

The next M6 gate remains exact approval for T125:

`Approve T125: read-only inspect quarantine candidate files 0010-0011 from the written T68 M6 review-export snapshot; no review files, no status/prioritize/apply, no candidate decisions, and no writes except the inspection report.`
