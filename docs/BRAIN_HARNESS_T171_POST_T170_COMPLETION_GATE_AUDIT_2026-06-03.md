# T171 Post-T170 Completion Gate Audit

Date: 2026-06-03

## Scope

Docs-only/read-only audit after:

- T169 T125 quarantine inspection report commit `3dfc23d`
- T170 T154 native Claude non-session smoke commit `ebe835d`

This slice did not run native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude,
harness install, settings edits, lifecycle archive, `lint apply_safe`, M6 status/prioritize/apply,
candidate decisions, migration apply, ranking/`orient` changes, public MCP changes,
schema/storage/index changes, document-index behavior changes, deletion, rollback, force-kill,
old-binary reinstall, or user-owned-file edits.

## Research Question

After T125 and T154, what is the current evidence-backed completion matrix for the Brain Harness
goal, and which remaining gate should be treated as the next narrow approval-controlled slice?

## Hypotheses

| Hypothesis | Result |
| --- | --- |
| Preferred | T125/T154 close two bounded gates, but the full goal remains incomplete because effective Claude hook behavior, M6 migration completion/deferral, lifecycle cleanup, telemetry confidence, and broader cross-harness behavior remain unproven or gated. |
| Null | T125/T154 are sufficient to declare the goal complete. |
| Simpler alternative | Skip the audit and wait for another user approval. |
| Failure | Treat non-session Claude metadata output, static harness readiness, or read-only M6 inspection as proof of production completion. |

The preferred hypothesis is supported.

## Current Evidence Snapshot

- Lean `orient` trace `019e8d9c-011c-7200-8917-fcd385f8281a` returned active current-plan
  MemoryItem `019e8d9a-9ba3-7b31-90b5-70d296d6d63a` first.
- Git status remains clean except the pre-existing untracked root `AGENTS.md`.
- Latest commits:
  - `ebe835d Record T154 native Claude smoke`
  - `3dfc23d Record T125 quarantine inspection`
  - `aba70b6 Record T160 lifecycle archive result`
- Harness status is `ready=true` for generic, Codex, Gemini CLI, Cursor, and Claude Code.
- Claude Code still has known warnings: user-owned settings snippet, extra legacy Engram
  permissions, split `settings.json` / `settings.local.json`, and effective hook behavior requiring
  native Claude Code `/hooks` verification.
- Read-only lint still reports wrong-scope active-memory info findings and broad superseded-active
  findings; no safe action was applied.
- Obligations doctor reports `open=[]` and `warnings=[]`.
- Real-session eval over the current 50-trace window fails the confidence gate at 32% feedback
  coverage across two feedback-bearing intents; the gate requires 50% feedback coverage across at
  least three intents.

## Completion Matrix

| Area | Current State | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| `orient` hot path compactness | Implemented and validated for the approved lean/read-only and no-prompt plan-work classes | `docs/ORIENT_CONTRACT.md`; T146/T147 results; T171 orient trace returns active current plan first | Do not expand payload or hot-path responsibilities without a new failure class and explicit approval |
| Current-plan retrieval | Validated on the `orient` hot path; direct memory-list retrieval remains noisy | T171 orient trace first result is active current plan; direct memory-list searches still surfaced superseded/archived items | Direct retrieval noise is a quality risk, but broad ranking churn remains unapproved |
| Generated harness adapters | Installed and status-ready across generic, Codex, Gemini CLI, Cursor, Claude Code | Fresh `engram harness status` for all five harnesses | Readiness is not proof of behavioral lifecycle compliance |
| Claude SessionEnd missing-policy default | Source/render/static validation plus installed static evidence | T130/T133A/T145/T152/T153 evidence; T170 confirms no monitored mutation from metadata commands | Missing `write_policy` behavioral verification remains deferred |
| Native Claude non-session behavior | Validated for metadata/help commands only | T170: exact `claude --version` and `claude --help` completed with no observed monitored mutation or Memory OS writes | Does not prove interactive, prompt-bearing, or `/hooks` behavior |
| Effective Claude hook configuration | Partially validated statically; behavior unproven | T153/T156/T170 note split settings and explicit durable policy entries | Requires exact native `/hooks`/effective-hook validation approval |
| Prompt-bearing native Claude behavior | Missing | T154/T170 explicitly did not run prompt-bearing Claude | Requires exact approval and post-state mutation checks |
| Claude Bridge validation | Risky / not currently usable as proof | Prior reports record Bridge tool-exposure limits and side-effect handoff writes | Do not use for Engram validation without exact approval and side-effect containment |
| M6 candidate inspection | Read-only inspection complete | T123/T124 inspected 0001-0009; T169 inspected quarantine 0010-0011 | Candidate decisions and migration status/dry-run/apply remain separate |
| M6 migration completion | Missing / high-risk gated | Research method 9.3; T169 says no status/prioritize/apply was run | Needs reviewed decisions, dry-run apply report, rollback plan, and explicit approval |
| Lifecycle cleanup | Partially complete | T166/T167/T168 archived three exact-approved targets | Broad superseded-active cleanup remains out of scope; `lint apply_safe` not approved |
| Telemetry confidence | Current sliding 50-trace gate fails | T171 real-session eval: 32% coverage; needs 50% and three intents | Not migration readiness; keep feedback discipline |
| Documentation and memory discipline | Maintained for recent slices | T169/T170 reports and commits; T171 docs-only audit | Must keep capturing current plan and feedback after each meaningful step |
| User-owned files | Preserved | Git status only shows untracked root `AGENTS.md` | Do not stage or edit without user request |

## Decision

The full Brain Harness goal is not complete. T125 and T154 closed two bounded gates, but the
definition of done still lacks:

- effective native Claude Code hook behavior,
- prompt-bearing native Claude behavior,
- missing SessionEnd `write_policy` behavioral verification,
- M6 migration completion or explicit user-approved deferral,
- reviewed candidate decisions and dry-run/apply evidence,
- current sliding-window telemetry confidence,
- broad lifecycle cleanup or explicit deferral.

The next smallest product-moving gate is ambiguous between two approval-controlled paths:

1. Native Claude effective-hook validation, because T170 explicitly left `/hooks`, prompt-bearing,
   and SessionEnd behavior unproven.
2. M6 candidate-decision/dry-run scoping, because T123/T124/T169 finished read-only inspection and
   migration completion remains a definition-of-done item.

Recommended next slice: prepare and request approval for native Claude effective-hook validation
first. Reason: harness behavior is a cross-agent production-quality requirement, and M6 apply should
not proceed while Claude lifecycle side effects remain only partially understood.

## Exact Next Approval Shape To Prepare

Prepare a default-deny approval packet for a narrow native Claude effective-hook validation slice.
It should authorize only preflight snapshots, explicit command inventory, native effective hook
inspection, and post-state comparisons. It must not authorize prompt-bearing model work, hook or
settings edits, harness install, Claude Bridge, lifecycle mutation, M6/migration/quarantine,
ranking/`orient`, public MCP/schema/storage/index/document-index changes, deletion, rollback,
force-kill, old-binary reinstall, or user-owned-file adoption unless those are separately and
explicitly approved.
