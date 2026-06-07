# T155 Completion Gate Audit

Date: 2026-06-03

## Status

Read-only/docs-only audit complete. No native Claude Code command, Claude Bridge command, lifecycle
hook, harness install, settings edit, M6/migration/quarantine action, lifecycle cleanup, ranking,
`orient`, schema/storage/index, public MCP, document-index, deletion, rollback, force-kill, or
user-owned adoption action was executed.

The goal is not complete. The current highest-signal next gate remains exact T154 approval for a
native Claude non-session smoke. Generic continuation or broad approval language is not sufficient.
Duplicate or late approval for T135 does not reopen harness writes because T135 was already
executed and validated in T152.

## Research Question

After T153/T154 documentation, what is the current evidence-backed completion matrix for the Brain
Harness goal, and what remains before Engram can be called production-quality as a Brain OS for AI
agents?

## Hypotheses

Preferred hypothesis: T153 closes the static post-repair evidence gap but leaves native Claude
execution, lifecycle cleanup, and M6/migration as explicit remaining gates.

Null hypothesis: T153 plus T152 is enough to claim the Brain Harness goal complete.

Simpler alternative: skip the audit and wait for T154 approval.

Failure hypothesis: treating old Claude/narrow-M6 evidence, old handoffs, or `ready=true` adapter
status as full production proof would hide missing behavioral and migration evidence.

## Measurement

- Ran lean Engram `orient` and direct `search` for current plan, architecture/implementation plan,
  user design philosophy, and recent risks.
- Read the current architecture, implementation-plan, T154 approval packet, research-method, and
  `ORIENT_CONTRACT` excerpts.
- Checked current git status and recent commits.
- Re-ran read-only harness status for generic, Codex, Gemini CLI, Cursor, and Claude Code.
- Attempted local CLI `engram lint run --json`; it failed on the SurrealDB lock because the daemon
  owns the database.
- Re-ran lint through MCP (`lint(action="run", write=false)`), which succeeded without applying
  lifecycle actions.

## Evidence Snapshot

Repository:

- Current branch worktree has only unrelated untracked root `AGENTS.md`.
- Latest commit is `6a55ec9 Record T153 Claude static preflight`.
- T135 harness repair was already executed in T152; no additional T135 write is pending.

Harness status:

| Harness | Status | Caveat |
| --- | --- | --- |
| `generic` | `ready=true` | No status warnings |
| `codex` | `ready=true` | No status warnings |
| `gemini_cli` | `ready=true` | No status warnings |
| `cursor` | `ready=true` | No status warnings |
| `claude_code` | `ready=true` | User-owned snippet preserved, extra legacy Engram permissions in Claude settings, split `settings.json` / `settings.local.json`, and effective hooks still require native validation |

Lint:

- Local CLI lint failed because `/Users/yuval.meiri/.engram/data/LOCK` was held by the running
  daemon. This is an execution-path caveat, not a Memory OS finding.
- MCP lint returned current-plan/wrong-scope feedback for old active items and multiple
  superseded-active findings. The stale/current-plan and wrong-scope findings had
  `safe_action=none`; superseded-active findings suggest archive actions but lifecycle mutation
  remains exact-gated.

Retrieval:

- Current lean `orient` returned `019e8cfc-6a59-7f62-900f-9b6cc1ad0572` first: current plan after
  T153 static preflight.
- Direct current-plan search also returned the T153 current-plan memory first.
- Several broad searches still surface old active handoff memories with no evidence. Treat this as
  retrieval noise and lifecycle-debt evidence, not authorization to archive or rewrite memory.

## Completion Matrix

| Area | State | Evidence | Remaining action |
| --- | --- | --- | --- |
| `orient` hot path compactness | Implemented and validated for approved prompt classes | T146/T147 docs and live no/empty-prompt traces; current orient returns T153 plan first | Continue monitoring; do not expand payload without approval |
| Current-plan/next-step retrieval | Validated for current prompt class | T153 current-plan memory first in lean orient and direct search | Old handoff/current-plan noise remains a lifecycle/retrieval-quality caveat |
| Generated harness adapters | Implemented and live status validated | T152 result; fresh T155 status shows all five `ready=true` | Soft lifecycle compliance remains behavioral |
| Claude SessionEnd missing-policy default | Statically validated | T153 report; installed hook defaults omitted `write_policy` to `nudge` | Behavioral verification deferred to exact approval |
| Native Claude behavior | Missing / approval-gated | T153 explicitly did not run native Claude; T154 packet exists | Exact T154 approval for non-session smoke only |
| Claude Bridge side-effect risk | Risky / partially reduced | T150 side-effect evidence; T152 repaired installed hook; no post-repair Bridge run | Do not use Bridge for Engram validation without exact approval |
| Claude settings effective behavior | Partially validated | `settings.local.json` parses; explicit durable policies found in settings | Native/effective hook validation remains gated |
| Codex harness behavior | Partially validated | Generated Codex adapter repaired; prior Codex dogfood/docs lifecycle evidence | Continue dogfood; no immediate approval gate |
| Gemini/Cursor harness behavior | Adapter-ready only | Generated adapters `ready=true` | Native behavior not broadly validated |
| Memory lifecycle cleanup | Missing / approval-gated | MCP lint shows stale/wrong-scope/superseded findings; safe actions require lifecycle writes | Separate exact lifecycle approval, target by target |
| M6 migration completion | Missing / high-risk gated | Prior M6 inventory/export/inspection docs; no review-apply/completion | Separate reviewed-candidate and dry-run/apply approval |
| Legacy layers as substrate | Preserved | No deletion/merge/bypass run | Keep until evals justify simplification |
| Evidence/telemetry loop | Partially validated | Prior telemetry feedback and current trace feedback; lint surfaces stale feedback | More real-session eval and cleanup gates remain |
| Documentation and memory discipline | Implemented for latest slices | T152/T153/T155 docs, commits, obligations, current-plan memory | Keep per-slice discipline |

## Decision

Engram is materially closer to a production Brain OS after T152/T153, but completion remains
unproven. The full definition of done still has unaddressed high-risk unknowns:

- native Claude behavior after repair,
- effective Claude hook configuration,
- lifecycle cleanup for stale/wrong-scope/superseded active memories,
- M6 migration completion or explicit deferral,
- broader cross-harness behavioral validation beyond adapter readiness,
- ongoing retrieval noise from old handoffs/current-plan memories.

The next smallest product-moving step is T154, but only with exact approval:

```text
Approve T154 native Claude non-session smoke.
```

Until that exact approval is present, safe work is limited to read-only evidence gathering,
docs-only audits, or preparation of exact approval packets. Do not run native Claude, Claude Bridge,
Claude `/hooks`, prompt-bearing Claude commands, lifecycle writes, M6/migration actions, harness
installs, or user-owned file adoption under generic continuation wording.
