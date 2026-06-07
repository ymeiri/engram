# Brain Harness T252 Lifecycle Approval Boundary Recheck

Date: 2026-06-04
Status: completed docs-only approval-boundary recheck. No lifecycle archive, `lint apply_safe`,
M6/migration/quarantine action, ranking/`orient`, public MCP, schema/storage/index,
document-index behavior, harness/runtime/native-Claude action, deletion, rollback, force-kill,
legacy simplification, or user-owned-file change was executed.

## Scope

T252 resolves a decision boundary introduced by the user's latest workflow instruction:

```text
PLease continue. do not stop for my approval for changing anything in the engram project's scope
```

The question is whether that broad instruction authorizes the pending default-deny lifecycle
archive packets T234, T247, and T248. T252 documents the answer only; it does not execute any
archive.

## Research Question

Can Codex treat a broad "continue without stopping for approval" instruction as authorization to
execute exact-target Memory OS lifecycle archive packets that explicitly require exact approval
wording?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Preserve the exact packet boundary: broad workflow permission applies to ordinary Engram repo/docs/code work, not MemoryItem archive writes whose packets say any other reply is non-authorization. | Supported. |
| Null | The latest broad instruction supersedes the packet wording and authorizes T234/T247/T248 archives after fresh prechecks. | Not supported. |
| Simpler alternative | Do nothing because the matrix already says not to infer approvals from broad continuation instructions. | Rejected because the latest user wording is likely to be replayed by future agents and should be explicitly reconciled. |
| Failure | T252 becomes a proxy approval, runs an archive, weakens the default-deny packet contract, or blocks ordinary non-mutation Engram work. | Avoided. |

## Evidence

- T234 approval packet requires exact wording for MemoryItem
  `019dd3fe-ec94-7122-af04-1f35b839387f` and says any other reply is non-authorization.
- T247 approval packet requires exact wording for MemoryItem
  `019e8291-40aa-71a0-b16b-9ba7b6446cc6` and says any other reply is non-authorization.
- T248 approval packet requires exact wording for MemoryItem
  `019e01f2-0a87-7f73-9b0b-7f2443eac7bb` and says any other reply is non-authorization.
- T251 already showed T247 and T248 targets remain active and visible with feedback-stale lint
  findings using `safe_action=none`.
- AI Council recall returned prior default-deny lifecycle guidance: docs-only evidence snapshots
  are acceptable, but archive/apply/scope-correct/ranking changes require exact user approval and
  fresh pre-write evidence.
- AI Council broadcast to `claude-sonnet-4.6`, `gpt-5.4`, and `gemini-3.1-pro` was unanimous:
  broad workflow permission is not authorization for T234/T247/T248 lifecycle archive writes.
- Claude Bridge read-only critique agreed: the broad user instruction is a project-work delegation,
  while exact MemoryItem archive is a separate memory lifecycle operation whose gate is meant to
  force target-specific consent.

## Decision

Preserve the default-deny exact approval boundary.

The latest broad instruction is valid for continuing ordinary Engram project work without pausing
for routine repo/docs/code changes. It does not authorize T234, T247, T248, `lint apply_safe`,
M6 write apply, migration/quarantine action, ranking/`orient` changes, schema/storage/index
changes, harness/native-Claude writes, deletion, or user-owned-file edits.

Future lifecycle archive execution still requires the exact packet wording for the exact target,
fresh matching read-only evidence, and no intervening Engram memory writes as required by that
packet.

## Completion Matrix Delta

| Area | State After T252 | Remaining Gate |
| --- | --- | --- |
| Ordinary Engram repo/docs/code work | Broad user instruction permits continuing without routine approval stops | Stay scoped to requested/necessary project work and validate normally |
| T234/T247/T248 lifecycle archives | Still pending/default-deny | Exact packet wording plus fresh pre-write checks |
| Broad lifecycle cleanup | Still incomplete | Exact-target review only; no broad `lint apply_safe` |
| M6 migration | Still incomplete | Human dispositions or explicit deferral; no write apply from broad permission |
| Current plan | T251 remains the current execution state; T252 clarifies the approval boundary | Capture updated current-plan memory after this docs-only commit |

## Validation

Validation for this docs-only slice:

- read exact T234/T247/T248 packet approval wording
- AI Council `recall_decision` for default-deny lifecycle guidance
- AI Council `broadcast_question` to three models
- Claude Bridge read-only critique
- `git status --short`
- `git diff --check`
- exact document indexing for this report, `docs/BRAIN_HARNESS_ARCHITECTURE.md`, and
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- document-search visibility for T252
- `obligations(action="doctor", project="engram")`
- focused commit with only intended repo docs
