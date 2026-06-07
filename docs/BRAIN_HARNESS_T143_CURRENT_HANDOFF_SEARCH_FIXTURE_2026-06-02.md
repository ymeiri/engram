# T143 Current Handoff Search Fixture

Date: 2026-06-02

Scope: source-level regression fixture for the T140 continuation/current-plan approval-gate-context
query class. This slice adds test coverage only. It does not change ranking source, install a
binary, restart the daemon, edit hooks/settings/adapters, run `harness install`, use
`adopt_user_owned`, mutate lifecycle state, run M6, inspect quarantine candidates, or change public
MCP/schema/storage/index/`orient`/document-index behavior.

## Research Question

Does the current source rank active current-plan guidance above a fresh rolling handoff for the
post-T142 live-shaped continuation query, or would T141 fail even after runtime refresh because the
source still has a ranking gap?

## Hypotheses

| Kind | Hypothesis |
| --- | --- |
| Preferred | Current source already promotes the active `decision` tagged `current-plan` above a fresher rolling handoff for the T140 query class; the live handoff-first result is stale installed runtime evidence. |
| Null | Current source also ranks the fresh rolling handoff first, so a new narrow source ranking fix is needed before T141. |
| Simpler alternative | Trust the existing T140 fixture. This is weaker because it covered older handoff distractors, not a newest handoff with strong query overlap. |
| Failure | Treating a passing source fixture as installed runtime proof, or using the fixture to broaden ranking beyond the T140 query class. |

## Fresh Live Evidence

- Lean `orient` trace `019e8884-e4d7-7cb2-bbe2-39b26935c3ce` returned T142 current-plan memory
  `019e8883-ad6f-7063-89ee-529d53e62ef0` first.
- Direct live search trace `019e8884-fedb-73d0-802e-bed68d71f4f3` for
  `current plan next step continue move forward Engram Brain Harness after T142 T141 T140 approval
  gates` returned the fresh rolling handoff `019e8883-ca98-7660-a70d-636c40dfb5c8` first and old
  active rolling handoffs behind it. The current-plan MemoryItem did not lead the result set.
- This preserves the T141 runtime-refresh question: source may be fixed while the running daemon is
  still stale.

## Implementation

Added `test_memory_search_t143_current_handoff_does_not_outrank_current_plan` in
`engram-tests/tests/search_tests.rs`.

The fixture creates:

- a fresh active `handoff` that mirrors the T142 rolling handoff content and has strong query
  overlap;
- a slightly older active project-scoped `decision` tagged `current-plan`;
- the live-shaped T143 query.

It asserts:

- the current-plan MemoryItem ranks first;
- the fresh handoff remains retrievable.

No ranker code changed.

## Validation

- `cargo test -p engram-tests --test search_tests test_memory_search_t143_current_handoff_does_not_outrank_current_plan -- --nocapture`
- `cargo fmt --all --check`
- `cargo test -p engram-index memory_ranker::tests -- --nocapture`
- `cargo test -p engram-tests --test search_tests -- --nocapture`
- `cargo check -p engram-cli`
- `git diff --check`

All checks passed. The full search integration suite now has 33 tests.

## Completion Matrix Delta

| Area | T143 status | Evidence |
| --- | --- | --- |
| T140 source behavior | Hardened | Source fixture proves current-plan guidance outranks a fresh rolling handoff for the live-shaped T143 query. |
| Installed runtime parity | Still gated | Live search still returned handoff-first before runtime refresh; T143 did not install or restart anything. |
| Ranking source | Unchanged | No `engram-index/src/memory_ranker.rs` change was needed. |
| `orient` hot path | Preserved | T143 changed no `orient` code, payload, or contract. |
| Lifecycle cleanup | Still gated | T143 did not archive or supersede stale handoffs/current-plan memories. |
| Harness readiness | Still gated | T143 did not run `harness install` or edit installed hooks/settings/adapters. |
| M6 migration completion | Still gated | T143 did not run migration, quarantine, deletion, cleanup, or legacy simplification. |

## Decision

T143 increases confidence that T141 is the right next runtime-moving gate: current source already
handles the fresh-handoff distractor shape, so the live handoff-first result should be validated by
installing the current binary and restarting the daemon under exact T141 approval.

T143 does not prove live parity. Exact T141 approval is still required before the runtime refresh.
