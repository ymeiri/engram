# Brain Harness T214 Harness Matrix Reconciliation

Date: 2026-06-04
Status: completed docs-only reconciliation

## Scope

This slice reconciles stale cross-harness completion-matrix wording after T152, T170, T179, T198,
T200, T204, and fresh read-only harness doctor checks.

It updates only `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` and this report. It does not edit hooks,
settings, adapters, runtime configuration, user-owned files, ranking or `orient`, public MCP
contracts, schema/storage/index/document-index behavior, lifecycle state, native Claude state,
M6/migration/quarantine state, or review workspace files.

## Research Question

Does the current completion matrix still describe cross-harness readiness accurately after the
approved T135 harness repair and later native-Claude/runtime evidence?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | The older matrix wording still contains pre-repair `ready=false` claims that should be superseded by current live doctor evidence, while preserving remaining behavioral caveats. | Supported. |
| Null | The matrix is already accurate enough because later notes exist. | Not supported. The main cross-harness row still names missing hooks and generated-adapter drift as current risk. |
| Simpler alternative | Add only a current-plan memory item and avoid doc edits. | Rejected because the definition of done depends on an accurate completion matrix. |
| Failure | The reconciliation overclaims full cross-harness behavior or treats adapter readiness as proof of native Claude behavioral parity. | Avoided. The updated note keeps effective-hook visibility, prompt-bearing behavior, host labeling, M6, and lifecycle cleanup open. |

## Evidence Re-Read

- `docs/BRAIN_HARNESS_T152_T135_HARNESS_REPAIR_VALIDATION_RESULT_2026-06-03.md` records the
  exact-approved harness repair and reports `ready=true` for generic, Claude Code, Codex,
  Gemini CLI, and Cursor.
- `docs/BRAIN_HARNESS_T170_T154_NATIVE_CLAUDE_SMOKE_RESULT_2026-06-03.md` records native
  non-session Claude metadata/help commands with no observed monitored-file, Memory OS, obligation,
  or git mutation.
- `docs/BRAIN_HARNESS_T179_T172_NATIVE_CLAUDE_EFFECTIVE_HOOK_RESULT_2026-06-03.md` records the
  hard-stop native PTY result: native startup Engram guidance was visible, but `/hooks` effective
  configuration output was not obtained and the process remained live until later recovery.
- `docs/BRAIN_HARNESS_T198_EXTERNAL_SESSION_JOINABILITY_RECHECK_2026-06-03.md` keeps live
  external-session joinability incomplete and classifies the gap as caller/host adoption rather
  than core telemetry storage failure.
- `docs/BRAIN_HARNESS_T200_CLI_EXTERNAL_SESSION_LABEL_CONTRACT_2026-06-04.md` implements explicit
  CLI caller support for external-session labels, while leaving hidden host/thread labels
  caller-supplied.
- `docs/BRAIN_HARNESS_T204_T203_RUNTIME_REFRESH_VALIDATION_2026-06-04.md` validates installed
  runtime refresh for handoff supersession planning, while leaving live stale handoffs unmutated.

## Fresh Read-Only Harness Checks

All five supported local harnesses plus the generic policy reported `ready=true` today:

| Harness | Current readiness | Remaining warnings |
| --- | --- | --- |
| `generic` | `ready=true` | Lifecycle compliance is soft and depends on the agent following policy. |
| `claude_code` | `ready=true` | User-owned settings snippet preserved; extra legacy Engram permissions remain in settings; settings are split across `settings.json` and `settings.local.json`; lifecycle compliance is soft. |
| `codex` | `ready=true` | Lifecycle compliance is soft and depends on the agent following policy. |
| `gemini_cli` | `ready=true` | Lifecycle compliance is soft and depends on the agent following policy. |
| `cursor` | `ready=true` | Lifecycle compliance is soft and depends on the agent following policy. |

The Claude Code warnings are important. They are not adapter drift or missing required hook
registration, but they are still behavioral and operational caveats.

## Current Cross-Harness State

Implemented and currently validated:

- Generated local harness adapter files are installed for generic, Claude Code, Codex, Gemini CLI,
  and Cursor.
- Claude Code has required Engram permissions and required lifecycle hook registrations in the
  merged settings sources reported by `harness(action="doctor")`.
- Native Claude non-session metadata/help commands completed without observed monitored mutation.
- Native Claude startup guidance was observed in the approved T172 PTY.
- Direct CLI external-session labels are implemented for `engram orient` and
  `engram memory changes-since`.

Still incomplete:

- Claude Code `/hooks` effective-hook visibility did not produce a usable configuration report in
  T179.
- Prompt-bearing native Claude behavior remains unproved.
- Lifecycle compliance is a soft contract, not enforced behavior.
- External-session joinability still depends on real caller/host labels for non-CLI surfaces.
- Live stale handoffs remain active until a future non-dry-run handoff update converges them or a
  separate lifecycle cleanup is explicitly run.
- M6 migration remains gated by human dispositions or explicit deferral.

## Decision

T214 updates the completion matrix state from "harnesses not ready" to "generated adapter readiness
validated, behavioral caveats remain." This is documentation reconciliation only. It does not close
the full Brain Harness goal.
