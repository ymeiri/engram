# T182 T181 Direct Search Current-Plan Approval Packet

Date: 2026-06-03
Status: docs-only/default-deny approval packet. No ranking or source behavior has been changed.

## Scope

This packet prepares a narrow future source-and-fixture slice for one observed direct unified
`search` current-plan miss after T181. It does not implement the fix.

It does not run document indexing, send input to the live native Claude PTY, signal or kill PID
`49349`, launch native Claude, run Claude Bridge, edit hooks/settings/adapters, run harness
install, mutate lifecycle state, run M6/migration/quarantine actions, make candidate decisions,
change ranking/`orient`, change public MCP/schema/storage/index/document-index behavior, delete,
roll back, reinstall binaries, or touch user-owned files.

## Current Evidence

- Active current-plan memory is `019e8e54-4595-7931-8b7d-061086f9ddb4`:
  `T181 packet is ready; exact approval needed for T179/T180 document indexing`.
- Lean `orient` trace `019e8e57-7110-7491-b72e-f0377f5a4887` returned that T181 current-plan
  memory first in Brain Loop for a normal continuation prompt.
- Direct unified `search` trace `019e8e57-f73d-7c72-80c0-8b5973a8cd1e` returned the same T181
  current-plan memory first for the simpler query
  `current plan next step Engram Brain Harness T181 T180 T174 exact approval gate`.
- Direct unified `search` trace `019e8e58-2498-7bc0-8520-032122b36920` still missed the active
  current-plan memory for the observed failure query:

```text
current next action after T181 exact approval gate non-gated work Brain Harness T180 T174 document indexing
```

  The top eight memory results were active rolling handoffs from older T145/T144/T143/T142/T140/
  T138/T133A/T134 work.
- Earlier direct unified `search` trace `019e8e55-77f7-7d91-b69a-bb65f6d7c733` showed the same
  prompt-class miss before this packet was drafted.
- Current source inspection shows:
  - `MemoryRankContext::search` requires a text match before guidance scoring.
  - `rank_memory_items` has prompt-specific promotion hooks for current-plan continuation,
    exact approval commands, migration gates, contextual M6 gate queries, and approval-gate
    summaries.
  - `asks_for_current_plan_guidance` recognizes `current plan`, `move forward`, `next step`,
    `next steps`, `resume`, and related continuation phrases, but not `next action`.
  - `asks_for_decision_gate` treats `approval gate` as contextual only when current-plan guidance
    is recognized and no gate-summary intent is present.
- Existing fixture families cover broad next-step prompts, exact approval commands, approval-gate
  continuation context, non-gated next-slice prompts, contextual M6 gate prompts, explicit
  migration apply prompts, and fresh handoff noise. They do not cover the exact
  `current next action after T181 ... exact approval gate ... document indexing` shape.
- AI Council recall for prior ranking work recovered the relevant guidance: keep fixes strict and
  prompt-class local; do not expand `orient`, run lifecycle cleanup, or perform broad ranking churn.

## Research Question

Can Engram make direct unified `search` treat the observed `current next action after T181 ...`
approval-gate-context prompt as current-plan continuation while preserving explicit gate/action
queries and avoiding broad ranking changes?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A focused fixture exposes the missing `next action`/approval-gate-context prompt class and a narrow predicate/ranking adjustment can promote the active project current-plan item above old rolling handoffs. |
| Null | The miss is live-data-specific handoff noise that cannot be reproduced deterministically without broader handoff lifecycle or ranking changes; source should not change under this packet. |
| Simpler alternative | Do not change ranking; rely on lean `orient` and simpler current-plan direct-search wording while documenting this prompt class as a caveat. |
| Failure | The slice expands into broad ranking churn, `orient` payload changes, handoff lifecycle cleanup, document indexing, native Claude recovery, migration work, schema/storage/index changes, or public MCP changes. |

## Proposed Approved Scope

If the user approves this packet, Codex may perform only the following:

1. Read the relevant current source and fixtures before editing:
   - `engram-index/src/memory_ranker.rs`
   - `engram-tests/tests/search_tests.rs`
   - any minimal service code required to understand direct unified search ranking.
2. Add focused deterministic fixture coverage for the observed prompt class:
   - a project-scoped active `decision` tagged `current-plan` representing the T181 gate;
   - old active rolling handoff distractors with strong lexical overlap;
   - the exact observed query shape, or a minimal literal variant preserving
     `current next action`, `T181`, `exact approval gate`, `non-gated`, `T180`, `T174`, and
     `document indexing`;
   - assertion that the active current-plan memory is first.
3. Add guard coverage proving explicit gate/action prompts still do not force current-plan
   promotion, including at least one prompt shaped like `should we proceed`, `run`, `apply`, or
   an exact approval/indexing command where gate/action semantics must remain authoritative.
4. Make the smallest source change needed to satisfy those fixtures, constrained to shared direct
   search ranking behavior for this prompt class.
5. Run targeted tests first, then risk-based regressions:
   - targeted `cargo test -p engram-tests --test search_tests <new_or_nearby_test_name>`;
   - relevant existing search/ranker fixtures for current-plan, exact approval, contextual gate,
     and explicit migration apply prompts;
   - `cargo fmt --all --check`;
   - `cargo check -p engram-cli`;
   - `git diff --check`.
6. Record before/after evidence in a result report and implementation-plan note if source changes
   are made.
7. Commit only intended files, capture current-plan memory, and submit telemetry feedback for the
   T182 traces.

## Success Criteria

- The deterministic fixture fails before the source adjustment and passes after it.
- The observed prompt class ranks the active project current-plan memory first above old rolling
  handoffs.
- Explicit gate/action prompts still prefer gate/action guidance and do not accidentally promote
  current-plan memory.
- Existing current-plan, exact approval, non-gated, contextual M6 gate, and explicit migration
  apply fixtures remain green.
- `orient` request parameters, response shape, and hot-path responsibilities remain unchanged.
- No document indexing, lifecycle mutation, M6/migration/quarantine action, native Claude input,
  Claude Bridge run, harness write, schema/storage/index behavior change, document-index behavior
  change, public MCP change, deletion, rollback, force-kill, old-binary reinstall, or user-owned
  file edit occurs.

## Stop Conditions

Stop and report without continuing if any of these occur:

- Approval is missing, conditional, abbreviated, or ambiguous.
- The fixture cannot reproduce the observed miss without broad synthetic setup unrelated to the
  live trace.
- The minimal source change would change public MCP parameters, `orient` payload shape, schema,
  storage/index behavior, document-index behavior, lifecycle state, M6/migration/quarantine state,
  harness files/settings/hooks/adapters, or installed runtime configuration.
- The implementation requires native Claude, Claude Bridge, process cleanup, T181 document
  indexing, T180 recovery, T174 M6 scoping, lifecycle archive, `lint apply_safe`, deletion,
  rollback, old-binary reinstall, or user-owned-file edits.
- Tests show materially broader ranking movement than the approved prompt class.

## Approval Question

Reply exactly:

```text
Approve T182: implement the narrow direct-search current-plan ranking fixture for the T181 current-next-action approval-gate continuation miss from docs/BRAIN_HARNESS_T182_T181_DIRECT_SEARCH_CURRENT_PLAN_APPROVAL_PACKET_2026-06-03.md. Add focused fixture coverage for the observed query shape so active project current-plan memory outranks older rolling handoffs, preserve explicit gate/action prompts and document-only/search behavior, and run targeted tests. Do not change orient payload/shape, public MCP params, schema/storage/index/document-index behavior, lifecycle/migration state, M6/quarantine, harness files/settings/hooks/adapters, native Claude, Claude Bridge, user-owned files, or broad ranking beyond the approved prompt class.
```
