# T148 Completion Gate Audit

Status: read-only audit complete; no runtime, migration, lifecycle, or harness writes performed

Scope: completion-distance audit after T146 source completion and T147 approval-packet creation

## Research Question

After T146 fixed no-prompt `plan_work` current-plan orientation at the source level and T147
prepared a default-deny runtime-refresh packet, how far is Engram from the Brain Harness definition
of done, and what exact gate blocks the next product-moving step?

## Hypotheses

| Hypothesis | Statement |
| --- | --- |
| Preferred | Source-level current-plan retrieval is fixed and documented, but installed runtime parity, harness readiness, lifecycle cleanup, and M6 migration remain approval-gated. |
| Null | Existing live runtime and harness evidence are already sufficient to call the goal complete. |
| Simpler alternative | Stop after T147 and ask only for runtime-refresh approval, without updating completion evidence. |
| Failure | This audit accidentally runs a runtime refresh, migration/quarantine command, lifecycle apply/archive, harness install/write, schema/storage/index change, document-index change, or broad ranking work. |

## Measurement

This audit used only read-only checks:

- startup lean `orient(project="engram", cwd="/Users/yuval.meiri/projects/engram",
  intent="plan_work", response_shape="lean")`;
- direct current-plan search;
- `git status`, recent commits, binary hashes, and daemon status from already-approved read-only
  inspection;
- `telemetry(action="real_session_eval", project="engram", limit=50)`;
- `obligations(action="doctor")`;
- `lint(action="run", write=false)`;
- `harness(action="doctor")` for Claude Code, Codex, Gemini CLI, Cursor, and generic harnesses;
- repo docs for T146, T147, T121-T126, and the latest M6/T125 gate.

No `cargo install`, daemon restart, harness install, lifecycle apply/archive, migration inventory,
migration review export/status/prioritize/apply, quarantine inspection, deletion, schema/storage/
index change, document-index change, or user-owned-file write was run.

## Evidence

| Area | Current evidence | Status |
| --- | --- | --- |
| T146 source behavior | Commit `d12b2ca` changed only the narrow `orient` source path: no/empty-prompt project/cwd-boundary `plan_work` promotes the latest current-plan decision into `active_decisions` and pins it first in Brain Loop, with implementation-prompt and no-boundary/no-current-plan guards. | Implemented and source-validated |
| T146 source validation | Targeted and regression checks passed: focused `memory_tests` no-prompt fixture, reviewed-decision prompt regression, service `orient_` sweep, `search_tests current`, full `memory_tests`, `cargo fmt --all --check`, `cargo check -p engram-cli`, and `git diff --check`. | Validated at source level |
| Live no-prompt `orient` | Startup trace `019e89f9-a1a9-7072-ab36-bbbef3f55fa1` still returned generic Brain Loop items from the installed runtime, not the active current-plan first. | Live runtime stale |
| Direct current-plan search | Direct search trace `019e89f9-a3bd-7c42-9a8a-cc282fa1` returned current-plan memory `019e89f9-2778-71e1-abf3-7a24d0fe714b` first and runtime limitation memory `019e89f4-7dba-7ae1-a559-85d924af31a3` second. | Retrieval working outside stale `orient` runtime path |
| Runtime identity | `/Users/yuval.meiri/.local/bin/engram` hash remains `3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b`; `/Users/yuval.meiri/.cargo/bin/engram` hash is `ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`; daemon remains on port `8765`, PID `10768`. | T147 pre-state still stale and gated |
| Git state | Recent commits are `4a628c1` docs sync, `06091aa` T147 packet, and `d12b2ca` T146 source fix. `git status --short` still shows only user-owned untracked `AGENTS.md`. | Clean for intended work, with user-owned file untouched |
| Telemetry | Rolling project eval returned `trace_count=50`, `feedback_trace_count=26`, `feedback_coverage=0.52`, `memory_judgment_coverage=1.0`, `task_success_count=16`, `task_failure_count=10`, `bad_memory_used_count=0`; `plan_work` intent had `feedback_coverage=0.4893617033958435`, `task_success_count=13`, `task_failure_count=10`. | Useful weak signal, not completion proof |
| Obligations | `obligations(action="doctor")` returned no open obligations or warnings before this report. | Clean before report write |
| Lint | `lint(action="run", write=false)` still reports telemetry-derived stale/wrong-scope current-plan findings and many `superseded-active:*` lifecycle findings with safe actions available only through gated lifecycle apply. | Lifecycle cleanup still gated |
| Harness readiness | `harness(action="doctor")` returns `ready=false` for Claude Code, Codex, Gemini CLI, Cursor, and generic. Claude Code has a drifted SessionEnd hook, missing SessionStart/SessionEnd settings registrations, and user-owned settings snippet. Codex, Gemini, and Cursor adapters are drifted; generic policy is missing. | Cross-harness readiness not complete |
| M6 migration | T121-T124 document that read-only candidate inspection is not candidate review completion. T127/T128 keep the next M6 gate as exact approval for T125 quarantine-candidate inspection only; migration apply, deletion, lifecycle mutation, and candidate decisions remain separate gates. | High-risk gate open |

## Completion Matrix

| Category | State | Evidence |
| --- | --- | --- |
| Implemented | Narrow no-prompt `plan_work` current-plan source fix. | T146 source commit `d12b2ca` and result doc. |
| Implemented | T147 runtime-refresh approval packet. | Commit `06091aa` and packet exact approval phrase. |
| Implemented | Central docs reflect T146 source complete and T147 pending. | Commit `4a628c1`. |
| Validated | Source-level no/empty-prompt project-scoped `plan_work` fixture behavior. | Focused `memory_tests` and full `memory_tests` passed. |
| Validated | Prompt-specific implementation guard and no-boundary/no-current-plan guard. | T146 fixture coverage passed. |
| Validated | Direct current-plan search can recover the active plan. | Trace `019e89f9-a3bd-7c42-9a8a-cc282fa1`. |
| Partially validated | Telemetry confidence and memory-quality feedback. | Rolling eval has no bad-memory-used count, but only weak agent-assessed evidence and mixed `plan_work` outcomes. |
| Partially validated | Harness contract design and renderable policy. | Doctor can compare adapters/settings and reports concrete drift, but all checked harnesses are `ready=false`. |
| Missing | Installed-runtime parity for T146. | T147 has not been approved/executed; live no-prompt `orient` remains stale. |
| Missing | Claude Code/Codex/Gemini/Cursor harness readiness. | Doctor shows drifted/missing adapters or hooks; no install/write approval. |
| Missing | M6 migration completion or explicit deferral. | T125 quarantine inspection and later candidate decisions/apply remain unexecuted and separately gated. |
| Missing | Lifecycle cleanup of stale/superseded active memory. | Lint reports safe actions, but lifecycle mutation requires approval. |
| Risky | Broad continuation/no-prompt hot path depends on installed daemon freshness. | Source and direct search disagree with live no-prompt `orient` until runtime refresh. |
| Risky | Telemetry is useful but self-assessed. | `feedback_coverage=0.52` and mixed `plan_work` failures cannot prove product quality. |
| Risky | Harness drift can make agents follow different lifecycle contracts. | All checked harnesses are not ready. |
| Blocked/gated | T147 runtime refresh and live validation. | Exact approval phrase from T147 is required before install/restart/read-only live validation. |
| Blocked/gated | M6 migration/quarantine and lifecycle cleanup. | Separate exact approvals required before any write/apply/archive/delete/decision step. |

## Distance From Goal

Engram is not complete enough yet. The source-level current-plan gap that motivated T146 is fixed,
but the installed runtime still serves stale no-prompt `plan_work` behavior. The next product-moving
step is T147: install the current `engram-cli` binary to `/Users/yuval.meiri/.local`, restart the
Engram daemon, and run the packet's read-only live validation. Without T147, the hot-path `orient`
definition-of-done item remains unverified in the actual daemon used by Codex and Claude Code.

After T147 passes, the remaining high-risk distance is not a mystery:

- repair or explicitly defer harness readiness under its own approval gate;
- complete or explicitly defer M6 migration/quarantine under exact review-gated approvals;
- resolve or explicitly defer lifecycle cleanup of stale/superseded active memory;
- rerun critical orient/search/harness/telemetry validation and update docs/memory with final
  evidence.

## Next Gate

Do not continue with runtime-changing work until the exact T147 approval phrase is provided:

```text
Approve T147: execute the T146 no-prompt PlanWork runtime refresh from
docs/BRAIN_HARNESS_T147_T146_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-02.md. Install the current
engram-cli binary to /Users/yuval.meiri/.local, restart the Engram daemon, and run read-only live
validation that no-prompt and empty-prompt project-scoped plan_work lean orient return the active
current-plan item first while explicit implementation-prompt plan_work does not force current-plan
promotion. Do not edit installed hooks/settings/adapters, run harness install, use
adopt_user_owned, change public MCP params or payload shape, schema/storage/index,
document-index behavior, lifecycle state, ranking beyond the already-committed T146 source,
M6/migration/quarantine, user-owned files, PATH/profile/auth configuration, rollback, force-kill,
deletion, or old-binary reinstall.
```
