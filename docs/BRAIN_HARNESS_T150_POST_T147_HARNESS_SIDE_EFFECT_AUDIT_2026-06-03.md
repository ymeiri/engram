# T150 Post-T147 Harness Side-Effect Audit

Date: 2026-06-03

## Status

Read-only audit complete. No harness install, adapter edit, hook edit, lifecycle mutation, M6 or
migration action, quarantine inspection, schema/storage/index change, document-index change, public
MCP change, payload change, ranking change, `orient` change, PATH/profile/auth change, rollback,
force-kill, deletion, or old-binary reinstall was performed.

## Research Question

After T147 closes the installed-runtime gap for the T146 no-prompt `plan_work` `orient` fix, what is
the next product-moving gate for the Brain Harness goal?

## Hypotheses

Preferred hypothesis: T147 is complete, the old T146 runtime-refresh limitation is now stale but
lifecycle cleanup remains gated, and the next product-moving gate is the exact T135 harness repair
because read-only Claude Bridge consultation can still cause Claude Code session-end handoff writes
while the installed harness is drifted.

Null hypothesis: T147 completion is sufficient for the current Brain Harness goal, and the stale
limitation plus Claude Bridge side effect can be ignored.

Simpler alternative: Prepare only a lifecycle archive packet for the stale T146 runtime-refresh
limitation and defer harness repair.

Failure hypothesis: Treating this audit as authorization would mutate lifecycle or harness state
without exact approval, or would hide a cross-harness side effect that affects production readiness.

## Measurement

- Rechecked active Engram orientation and direct search after T147.
- Read the relevant Brain Harness, Memory OS, research-method, `orient`, T135, T137, T148, and T147
  docs.
- Inspected git status and recent commits.
- Confirmed commit `0059ea4` is reachable from `HEAD`.
- Read memory `019e89f4-7dba-7ae1-a559-85d924af31a3`, ran exact search/orient checks for it, ran
  read-only lint, and traversed its depth-1 graph.
- Recalled AI Council prior decisions before asking for new critique.
- Asked Claude Bridge for read-only critique, then audited the resulting Claude Code harness state
  and side effects.

## Evidence

T147 is complete in source and installed runtime. Commit `0059ea4` recorded the runtime validation,
and live traces showed the current-plan item first for no-prompt and empty-prompt project-scoped
`plan_work` `orient`, while the explicit implementation-prompt guard did not force current-plan
promotion.

The active current plan remained visible after T147. Continuation orient trace
`019e8bbd-e561-7831-86e1-f58946efa703` and direct current-plan search trace
`019e8bbd-ff54-73e0-878b-5e9c9eeb0248` returned current-plan memory
`019e8bbd-0ac8-7e93-85ce-99734808de61` as the relevant plan anchor.

Memory `019e89f4-7dba-7ae1-a559-85d924af31a3` remains active but is now stale. It says the T146
source fix still requires runtime refresh before live no-prompt `orient` changes. T147 contradicted
that limitation by installing the current binary, restarting the daemon, and validating the live
no-prompt and empty-prompt paths. Exact search trace `019e8bc0-a840-7a32-b1e7-3e48176a0514`
returned the stale limitation first for a targeted query, and orient trace
`019e8bc0-c277-7311-8490-f58718cea707` surfaced it second while planning the next slice. Read-only
lint did not flag this exact memory, so the stale assessment is manual and evidence-backed rather
than lint-proven. Depth-1 graph traversal showed direct evidence and scope edges only; no lifecycle
mutation was performed.

AI Council recall reinforced that lifecycle cleanup needs exact target evidence, default-deny
framing, fresh get/search/lint/orient evidence, graph dependency checks, and a hard stop on drift.
Claude Bridge agreed the stale-memory cleanup is a valid class of slice, but warned that archive vs
supersede rationale, graph dependents, lint gaps, and orient/ranking impact must be explicit hard
stops.

Claude Bridge read-only consultation produced a harness side effect: two new Claude Code
session-end stub handoffs appeared in Engram:

- `019e8bc0-59a2-7051-b667-e88a1a4861c0`
- `019e8bc0-59a2-7051-b667-e87463bc9b3b`

This confirms the T137/T135 caution remains live: using Claude Bridge for Engram consultation can
still trigger durable-looking Claude Code handoff writes even when the bridge request itself is
read-only.

Read-only Claude Code harness status and doctor checks still report `ready=false`. The generated
SessionEnd hook template now defaults missing `write_policy` to `nudge`, but the installed Claude
SessionEnd hook is drifted, the settings registrations for SessionStart and SessionEnd are missing,
and the settings files remain user-owned with legacy Engram permission drift. No install or dry-run
repair was executed.

## Decision

T147 closes the installed-runtime gap for the T146 no-prompt `plan_work` `orient` source fix. It
does not close the Brain Harness production-readiness goal.

The next product-moving gate is the exact T135 harness repair approval packet, because cross-harness
consultation is still not production-ready and the read-only Claude Bridge side effect can create
new stale or noisy handoff memory while the installed Claude harness is drifted.

Do not use Claude Bridge again for Engram Brain Harness consultation until T135 is approved and
executed, unless the user explicitly accepts the known side-effect risk for that consultation.

The stale T146 runtime-refresh limitation should be handled by a separate exact lifecycle cleanup
approval after fresh evidence confirms the target is still active, still contradicted by T147, has
no blocking dependent memories, and is safe to archive or supersede. This audit is not that
authorization.

## Completion Matrix

| Area | State | Evidence | Remaining Gate |
| --- | --- | --- | --- |
| T146 no-prompt `plan_work` source fix | Implemented and validated | Commit `d12b2ca`; focused fixtures and regression tests | None for source |
| T147 runtime refresh | Executed and validated | Commit `0059ea4`; live no/empty-prompt traces passed | None for T146 runtime |
| Current-plan/next-step retrieval | Validated for approved prompt class | Live T147 traces and post-T147 current-plan orient/search | Continue monitoring feedback |
| Stale T146 runtime-refresh limitation | Confirmed stale and visible | Memory `019e89f4-7dba-7ae1-a559-85d924af31a3`; search/orient traces | Separate exact lifecycle approval |
| Claude/Codex harness readiness | Not ready | Claude Bridge side effect plus `harness status/doctor ready=false` | T135 exact harness repair approval |
| M6/migration/quarantine | Still review-gated | Existing implementation-plan gates | Separate explicit approval only |
| Lifecycle cleanup | Still gated | AI Council recall and Claude critique | Exact target approval with fresh evidence |

## Next Action

Ask the user to approve the existing T135 harness repair packet before any further Claude Bridge or
cross-harness Engram consultation. If the user prefers to clean up the stale T146 limitation first,
prepare a separate exact lifecycle approval packet that does not bundle harness repair, M6,
migration, ranking, `orient`, schema, storage, index, or document-index changes.
