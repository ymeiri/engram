# Brain Harness T61 Continuation Should Gate Fix

Status: Implemented and validated as a narrow direct-search ranking repair.
Date: 2026-05-31
Scope: T60 continuation wording where `what should happen next` was misclassified as gate intent

This slice did not run M6 review export, review apply, deletion, cleanup, lifecycle mutation,
schema/storage/index changes, public MCP changes, `orient` changes, or harness adapter/hook
changes.

## Research Question

Can Engram treat `what should happen next` as continuation/current-plan wording while preserving
gate-first behavior for explicit approval/action prompts?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The T60 direct-search miss is caused by bare `should` triggering gate mode; replacing bare `should` with modal action patterns restores current-plan promotion without weakening migration gates. |
| Null | The rank miss is caused by broader scoring or live-memory noise, so changing `should` detection would not fix the deterministic fixture. |
| Simpler alternative | Document T60 as noise and wait for explicit M6 review-export approval. |
| Failure | The fix lets explicit `should we proceed/apply/run migration_review_export` prompts promote current-plan guidance above gate evidence. |

## Evidence

- T60 continuation `search` placed the current T59 plan only at memory rank 5.
- Source inspection found `asks_for_decision_gate` treated any `should` as gate intent, disabling
  current-plan promotion for `what should happen next`.
- AI Council agreed the slice was justified only as a false-positive repair, with contrast tests
  for explicit action/approval prompts.
- Claude Bridge read-only critique agreed the linguistic boundary is sound: `what should happen
  next` is a status/continuation question, while `should we run/apply/proceed` asks for execution
  permission.
- The Claude Bridge retry repeated the T60 no-write caveat: `write=false` still produced duplicate
  Claude Code rolling handoff MemoryItems. This report records the confound but does not approve
  deleting handoffs or changing hooks/adapters.

## Implementation

- Added unit coverage that `what should happen next` stays out of gate mode.
- Added unit coverage that modal action prompts still trigger gate mode, including
  `Should we run migration_review_export?`.
- Changed decision-gate detection to remove bare `should` and instead match modal action patterns
  such as `should we proceed`, `can we run`, and `whether we should export`.
- Added a deterministic search fixture for the exact T60 continuation wording. The fixture keeps
  current-plan memory first, keeps M6 gate evidence retrievable, and verifies an explicit
  `should we run migration_review_export` contrast prompt does not put the current plan first.

## Validation

- Before implementation, the new unit fixture failed because `what should happen next` triggered
  gate mode.
- Before implementation, the new search fixture failed because the current-plan item did not rank
  first.
- After implementation:
  - `cargo test -p engram-index memory_ranker::tests`
  - `cargo test -p engram-tests --test search_tests test_memory_search`
  - `cargo fmt --all --check`
  - `cargo check -p engram-cli`
  - `git diff --check`

## Verdict

The T60 continuation false-positive is repaired for deterministic direct-search fixtures. This is a
prompt-class ranking fix only. It does not prove broad ranking quality and does not authorize M6
review export/apply, lifecycle writes, schema/storage/index changes, public MCP changes, `orient`
payload expansion, or harness adapter/hook changes.

Future Claude Bridge parity checks still need either a verified no-handoff mode or explicit
acceptance that Claude Code session-end handoff writes may occur.
