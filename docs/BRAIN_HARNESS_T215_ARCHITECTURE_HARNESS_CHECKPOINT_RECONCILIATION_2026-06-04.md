# Brain Harness T215 Architecture Harness Checkpoint Reconciliation

Date: 2026-06-04
Status: completed docs-only reconciliation

## Scope

This slice updates the architecture RFC so its early Memory OS overview exposes the current
cross-harness and M6 state before older chronological readiness history.

It updates only:

- `docs/BRAIN_HARNESS_ARCHITECTURE.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- this report

It does not edit hooks, settings, adapters, runtime configuration, user-owned files, ranking or
`orient`, public MCP contracts, schema/storage/index/document-index behavior, lifecycle state,
native Claude state, M6/migration/quarantine state, or review workspace files.

## Research Question

Does the architecture RFC still risk misleading startup readers by surfacing old pre-repair
`ready=false` harness checkpoints before the current T214 state?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Add a compact current checkpoint near the Memory OS overview, leaving historical bullets intact but clearly superseded. | Supported. |
| Null | T214 in the implementation plan is enough; architecture does not need a current checkpoint. | Not supported because broad document search still surfaced old architecture readiness text. |
| Simpler alternative | Delete or rewrite old readiness history. | Rejected because older checkpoints remain useful provenance and should not be removed without a stronger cleanup slice. |
| Failure | The architecture update overclaims full harness behavior or M6 completion. | Avoided. The checkpoint preserves unresolved behavior gates and M6 disposition/deferral gates. |

## Evidence

- Fresh read-only `harness(action="doctor")` checks in T214 returned `ready=true` for generic,
  Claude Code, Codex, Gemini CLI, and Cursor.
- T152 records the exact-approved T135 harness repair and its post-write `ready=true` validation.
- T179 records that native Claude startup guidance was visible, but `/hooks` effective-configuration
  output was not obtained.
- T198/T200 keep external-session joinability caller-supplied, not automatically solved for all
  harnesses.
- T209/T210/T213 keep M6 candidate inspection and disposition/apply gates separate.

## Change

Added a short "Harness and migration checkpoint, current through 2026-06-04" to
`docs/BRAIN_HARNESS_ARCHITECTURE.md` after the Memory OS hot-path/specialist-path overview.

The checkpoint says:

- generated local harness adapter readiness is `ready=true` for all supported harnesses;
- older readiness checkpoints below are superseded for local generated adapter readiness;
- lifecycle compliance, Claude settings shape, `/hooks` visibility, prompt-bearing native Claude,
  external-session joinability, and stale handoffs remain incomplete;
- M6 remains review-gated, with inspection complete for 0001-0011 and 0012 requiring explicit
  scope handling.

## Decision

T215 improves startup-doc accuracy without changing runtime behavior. It does not complete the
Brain Harness goal.
