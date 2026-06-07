# Brain Harness T132 Post-T129 Startup Gate Audit

Date: 2026-06-02
Status: Completed as docs-only startup and gate audit
Scope: Read-only retrieval, document visibility, harness readiness, lint, telemetry, obligations,
and git state after T129

T132 checked whether a fresh Codex continuation after T129 can recover the current plan and next
approval gate, and whether any non-gated implementation work should proceed before T130 approval.
It made no code, installed hook, settings, adapter, lifecycle, migration, document indexing,
candidate file, ranking, `orient`, public MCP parameter, schema/storage/index behavior, or
document-index behavior change.

## Research Question

After T129, does startup retrieval recover the current plan and T130 approval gate clearly enough
to continue, and what completion-matrix risks remain before hook-template or migration work?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Lean `orient`, exact T130 search, scoped current-plan list, and `handoff(get)` recover T129/T130; broad continuation and document search remain noisy evidence-quality gaps, not implementation approval. |
| Null | T129 current-plan or handoff retrieval fails, making the next safe action ambiguous without user input. |
| Simpler alternative | Rely on the T129 handoff/current-plan write and skip another startup audit. |
| Failure | The audit crosses into T130 hook-template edits, T131 diagnostics, T125 quarantine inspection, T47 harness repair, migration work, lifecycle mutation, ranking changes, `orient` expansion, document indexing, or schema/storage/index changes. |

## Measurement

Read-only evidence collected on 2026-06-02:

- Lean `orient` trace `019e879c-00e1-76b2-ba73-0c980fc4b28c` returned current-plan memory
  `019e879a-61eb-7ae3-8b28-3ca2cd94a220` first. The next candidates were the harness-write gate
  `019e7cde-b517-77d0-aaac-c8638811d4e8`, M6 gate
  `019e7ce5-155d-7a10-85f5-00b9dcc69cd0`, user design preference
  `019e6924-256b-7093-b1c5-286ec4d02461`, and stale repository-scoped current-plan memory
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915`.
- Broad direct current-plan search trace `019e879c-0133-7862-bbb0-d210e2503920` returned the T129
  handoff first and older handoffs below it. It did not provide a clean current-plan-first
  continuation result.
- A narrower current-plan search trace `019e879c-35f7-7d32-b64b-65ec4d14eff8` still returned
  handoffs above current-plan memory, showing that broad current-plan/direct-search wording remains
  handoff-noisy after T129.
- Exact T130 approval search trace `019e879c-363b-7a32-9391-38c79e6cc574` returned current-plan
  memory `019e879a-61eb-7ae3-8b28-3ca2cd94a220` first and T129 handoff
  `019e879a-9b34-7e70-8e91-baab18c11b3e` second, so the exact approval packet is recoverable.
- Architecture/risk searches traces `019e879c-0178-7c73-aba8-6ee80e21cecf` and
  `019e879c-023a-7330-a53f-d341a7eda002` returned the current plan first, then the T129 handoff.
- Design-preference search trace `019e879c-01f5-7c22-bba0-7c57e670401e` returned reviewed user
  preference `019e6924-256b-7093-b1c5-286ec4d02461` first.
- `memory(action="list", project_name="engram", scope_type="project", tags=["current-plan"],
  status_filter="active")` returned exactly one active project current-plan item:
  `019e879a-61eb-7ae3-8b28-3ca2cd94a220`.
- `handoff(action="get", project="engram")` returned T129 handoff
  `019e879a-9b34-7e70-8e91-baab18c11b3e`.
- `docs(action="search", query="Brain Harness T129 Claude Session-End Handoff Root Cause",
  limit=5)` did not return the T129 report in the top five results. Repo docs remain authoritative
  unless exact indexing is approved.
- `harness(action="doctor")` still returned `ready=false` for Claude Code and Codex. Claude Code
  has generated files installed but lacks required `SessionStart` and `SessionEnd` settings
  registrations. Codex generated skills remain drifted from current policy.
- `lint(action="run", write=false, limit=20)` still reports stale and wrong-scope feedback on
  repository-scoped current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915`, with
  `safe_action=none`. It also reports many superseded-active findings whose safe action is
  archival after review; no lifecycle write was run.
- `telemetry(action="real_session_eval", project="engram", limit=50)` passed numerically with
  `feedback_coverage=0.5`, `feedback_trace_count=25`, `bad_memory_used_count=0`, and
  `confidence_gate.requires_user_approval=true`.
- `obligations(action="doctor", project="engram")` returned no open obligations.
- `git status --short` showed only user-owned untracked root `AGENTS.md`; latest commit was
  `149682f Record T129 handoff root cause`.

## Result

Startup recovery is sufficient to preserve the next gate: lean `orient`, exact T130 search, scoped
current-plan listing, and `handoff(get)` all recover the T129 state and T130 approval packet.

This is not a retrieval-quality or readiness pass. Broad direct current-plan searches still rank
rolling handoffs above the current-plan memory, and document search still misses the fresh T129
report by title. The stale repository-scoped current-plan memory remains visible in `orient` and
lint. Claude Code and Codex harnesses still report `ready=false`.

No safe automatic lifecycle or harness action follows from this audit. The next implementation
slice remains the exact T130 hook-template change if approved.

## Completion Matrix Delta

| Area | T132 state | Evidence |
| --- | --- | --- |
| Lean `orient` startup | Healthy for tested Codex prompt | Trace `019e879c-00e1-76b2-ba73-0c980fc4b28c` returned T129 current plan first. |
| Exact T130 recovery | Healthy | Trace `019e879c-363b-7a32-9391-38c79e6cc574` returned T129 current plan first and T129 handoff second. |
| Broad direct current-plan search | Noisy | Traces `019e879c-0133-7862-bbb0-d210e2503920` and `019e879c-35f7-7d32-b64b-65ec4d14eff8` ranked handoffs first. |
| Document visibility | Partial | T129 report was not found in top-five document search by title. |
| Harness readiness | Not ready | Claude Code and Codex `harness(doctor)` still return `ready=false`. |
| Lifecycle quality | Still gated | Lint shows stale/wrong-scope and superseded-active findings, but no safe automatic write was run. |
| M6 migration | Still gated | T125 and all status/prioritize/apply/rerun decisions remain separate approval gates. |

## Next Gate

The recommended next code gate remains:

`Approve T130: change the generated Claude Code SessionEnd hook template so missing hook input write_policy defaults to non-durable/nudge instead of durable; add focused tests proving missing write_policy does not write a handoff, explicit durable still writes, and installed/rendered hook output matches the new default; do not edit installed user hooks or settings, do not run harness install, do not change public MCP parameters, schema/storage/index behavior, ranking, orient, migration, or lifecycle state.`

T132 does not approve T130, T131, T125, T47, migration status/prioritize/apply/rerun, lifecycle
mutation, document indexing, ranking changes, `orient` expansion, public MCP/schema/storage/index
behavior changes, document-index behavior changes, hook/settings/adapter writes, harness install,
or user-owned file adoption.

## Validation

This is a docs-only evidence slice. Validation is limited to:

- read-only Engram MCP evidence from `orient`, `search`, `docs(search)`, `memory(list)`,
  `handoff(get)`, `harness(doctor)`, `lint(run)`, `telemetry(real_session_eval)`, and
  `obligations(doctor)`;
- exact-source documentation updates in the Brain Harness architecture, research method, and
  Memory OS implementation plan;
- `git diff --check` before commit.
