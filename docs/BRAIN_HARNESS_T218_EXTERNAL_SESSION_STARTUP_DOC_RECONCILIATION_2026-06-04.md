# Brain Harness T218 External Session Startup Doc Reconciliation

Date: 2026-06-04
Status: completed docs-only reconciliation

## Scope

This slice updates startup-facing status wording after T217 so the architecture RFC and early
completion-matrix note no longer describe external-session joinability as only a raw caller-label
gap.

It updates only:

- `docs/BRAIN_HARNESS_ARCHITECTURE.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- this report

It does not change source code, refresh the installed runtime, edit hooks/settings/adapters, mutate
lifecycle or migration state, run native Claude, change ranking or `orient` payloads, change public
MCP/schema/storage/index/document-index behavior, delete data, or touch user-owned files.

## Research Question

Do the startup-facing docs still describe external-session joinability accurately after T217?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | The early architecture and matrix wording should mention T217's source-level MCP env fallback while preserving that live labeling remains incomplete until runtime refresh and host env adoption. | Supported. |
| Null | The T217 report and bottom matrix note are sufficient. | Not supported because required startup docs still contained pre-T217 wording. |
| Simpler alternative | Remove external-session caveats from the early checkpoint. | Rejected because T217 is not live runtime validation and does not synthesize host labels. |
| Failure | The docs imply cross-harness live labeling is complete. | Avoided. The updated wording keeps installed-runtime and host-label adoption gaps explicit. |

## Evidence

- T200 implemented direct CLI `--external-session-id` plus `ENGRAM_EXTERNAL_SESSION_ID` fallback for
  direct CLI `orient` and `memory changes-since`.
- T217 implemented source-level MCP fallback to `ENGRAM_EXTERNAL_SESSION_ID` for existing
  telemetry call sites, without public MCP shape changes.
- T217 did not refresh the installed daemon/runtime and did not synthesize host labels.
- The early architecture checkpoint and early completion-matrix reconciliation note still said
  external-session joinability depended on real caller/host labels without naming the new source
  fallback state.

## Change

Updated the early architecture checkpoint and early completion-matrix reconciliation note to say:

- generated adapter readiness remains `ready=true`;
- direct CLI and source-level MCP env fallback support now exist;
- installed runtime has not been refreshed for the MCP fallback;
- hosts still need to provide real labels;
- stale handoffs, native Claude behavior, `/hooks` visibility, and M6 dispositions/deferral remain
  incomplete.

## Decision

T218 improves startup-doc accuracy after T217. It does not refresh runtime or complete live
external-session joinability.
