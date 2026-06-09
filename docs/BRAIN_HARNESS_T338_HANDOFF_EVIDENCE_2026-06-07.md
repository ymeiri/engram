# Brain Harness T338 Handoff Evidence

Date: 2026-06-07
Status: implemented, installed, and locally validated

## Scope

T338 makes rolling handoff writes evidence-backed. Project-scoped lint after T337 showed the newest
active rolling handoff immediately triggered `missing_evidence`, because `HandoffService::update`
only attached evidence for session-scoped handoffs.

This slice changes `engram-index/src/handoff.rs` so every handoff update carries tool-call
evidence. Session-scoped handoffs keep their existing session-event evidence as additional context.

It does not archive memory, run `lint apply_safe`, mutate obligations, change handoff content
selection, run native Claude, change harness adapters, close hosted CI, or claim lifecycle cleanup.

## Research Question

Can rolling handoff updates satisfy the Memory OS evidence contract without adding caller friction or
changing handoff semantics?

## Hypotheses

| Type | Hypothesis | Evidence |
| --- | --- | --- |
| Preferred | Add automatic tool-call evidence to every handoff update. | This covers project, global, and session handoffs through one service-level path. |
| Null | Leave handoffs unevidenced because their writer provenance is enough. | Rejected because live project-scoped lint flags active handoffs as missing evidence. |
| Broader alternative | Add user-supplied evidence to the MCP/CLI handoff request schema. | Deferred because it adds friction and does not cover hook-triggered handoffs automatically. |
| Failure | Evidence repair changes handoff matching, supersession, or compile behavior. | Guarded by focused handoff tests and preserving the existing session-event evidence. |

## Behavior

`HandoffService::update` now attaches an `EvidenceKind::ToolCall` record to the new handoff item.
The target is scoped when possible:

- `handoff(action=update,scope=global)`
- `handoff(action=update,project=<project>)`
- `handoff(action=update,session_id=<session-id>)`

Session-scoped handoffs still receive their existing `EvidenceKind::SessionEvent` record, so
compiled/session-end handoffs retain session traceability.

## Validation

Focused validation passed:

- `cargo test -p engram-index handoff`
- `cargo check -p engram-index`
- `cargo clippy -p engram-index --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `git diff --check`

The focused test set includes project handoff update, dry-run behavior, supersession behavior,
session compile/write behavior, and harness session-end handoff behavior.

Installed runtime validation passed after:

```bash
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
/Users/yuval.meiri/.local/bin/engram daemon stop
/Users/yuval.meiri/.local/bin/engram daemon start
```

Post-refresh state:

- installed binary hash:
  `e53765568a2232c55c2d17a8a48480e745b2c2fda044a8d087681c20534e3dc5`;
- daemon PID `92750`, spawned by `/Users/yuval.meiri/.local/bin/engram`;
- new rolling handoff `019ea34a-c3ac-74d0-ae42-52cd6adcb610` carries
  `tool_call` evidence with target `handoff(action=update,project=engram)`;
- installed MCP project-scoped lint with `project=engram` and `limit=100` returned
  `new_handoff_flagged=False`.

## Non-Claims

T338 fixes future handoff writes and proves the refreshed local daemon uses that behavior for the
current rolling handoff. It does not retroactively repair every historical unevidenced handoff,
perform broad lifecycle cleanup, or make `lint apply_safe` safe.
