# Brain Harness T107 Broad Next-Step Search Calibration

Date: 2026-06-01

## Status

T107 completed a narrow direct unified `search` current-plan calibration for broad next-step
wording. It does not change `orient`, score weights, migration, lifecycle state, document indexing,
harness adapters/hooks/settings, public MCP request/response shape, schema, or storage behavior.

## Research Question

When an agent asks a broad continuation question such as `what should happen next Engram Brain
Harness`, should direct `search` treat that as current-plan guidance and return the active
current-plan MemoryItem first?

## Hypotheses

- Preferred: this is a lexical current-plan guidance gap. Adding exact broad next-step phrases to
  `asks_for_current_plan_guidance` fixes the observed query without affecting explicit migration or
  approval-gate prompts.
- Null: the observed result is acceptable because old synthesis and research-method rules are also
  relevant, so no ranking code should change.
- Simpler alternative: document the gap and keep relying on explicit `current plan` / `next step`
  wording.
- Failure: broad wording over-promotes current-plan guidance for explicit migration/apply questions
  or hides gate evidence.

## Before Evidence

Live MCP direct search before the patch:

- Trace `019e83a1-67e7-79e3-bfdb-3cfc54159dec`,
  `what should happen next Engram Brain Harness`: old
  `AI Council and Claude next-step synthesis after orient contract` ranked first, the generic
  research-method rule ranked second, and active T106 current-plan memory
  `019e839e-b951-7fe2-a80f-4834622b04d3` ranked fourth.
- Trace `019e83a1-5ef9-71d2-a7b5-c2df5ad40ccd`,
  `current plan next step after T106`: active T106 current-plan memory ranked first.
- Trace `019e83a1-4e57-72d1-b677-0a633936ac84`,
  `continue our work to achieve the Engram Brain Harness goal what should happen next`: active T106
  current-plan memory ranked first.

Source inspection found that `asks_for_current_plan_guidance` already covered `continue`,
`current plan`, `next step`, `move forward`, and similar continuation phrases, while
`asks_for_decision_gate` already classified `What should we do next for Engram?` as not gate mode.
The missing phrases were `what should happen next`, `what should we do next`, and
`what do we do next`.

## Implementation

The code change is intentionally lexical and local:

- Added `what should happen next`, `what should we do next`, and `what do we do next` to
  `asks_for_current_plan_guidance`.
- Added a unit test proving those phrases are current-plan guidance and not gate mode, while
  `what happened next in the old synthesis` is not current-plan guidance.
- Added deterministic search fixture
  `test_memory_search_t107_broad_next_step_promotes_current_plan` with old synthesis,
  research-method, handoff-like, M6 gate, and active current-plan distractors.

## Validation

Source validation:

- `cargo test -p engram-index memory_ranker::tests::what_should_happen_next_is_current_plan_guidance`
  passed.
- `cargo test -p engram-index memory_ranker::tests` passed.
- `cargo test -p engram-tests --test search_tests test_memory_search_t107_broad_next_step_promotes_current_plan`
  passed.
- `cargo test -p engram-tests --test search_tests` passed.
- `cargo fmt --all --check` passed.
- `cargo check -p engram-cli` passed.
- `git diff --check` passed.

Installed-runtime validation:

- Installed `/Users/yuval.meiri/.local/bin/engram` hash
  `aa9f557200ee34367a9218d0b5a2d4a6098286698e2189440b96bfe3b15971f2` and restarted the daemon on
  port `8765`, PID `1236`.
- Trace `019e8443-2d4d-7f30-969b-b7b235324ad5`,
  `what should happen next Engram Brain Harness`: active T106 current-plan memory
  `019e839e-b951-7fe2-a80f-4834622b04d3` ranked first.
- Trace `019e8443-7066-7590-8881-4b3604a601dd`,
  `what should we do next for Engram?`: active T106 current-plan memory ranked first.
- Trace `019e8443-3ba3-79e1-889b-131a1eb88b5f`,
  `should we proceed with M6 migration apply?`: migration gate context ranked above current-plan
  guidance.
- Trace `019e8443-5652-71e2-bf07-ffbe6b3719e3`,
  `approved M6 write apply deletion cleanup legacy simplification now`: migration gate context
  ranked first, M6 approval gate ranked second, and current-plan guidance did not lead.

AI Council supported the narrow phrase calibration and warned against broad score-weight tuning.
Claude Bridge critique and native Claude Code parity were attempted but unavailable: Claude Bridge
timed out twice, and native `claude -p` could not connect through the configured API/auth path in
this environment. Treat cross-harness parity for T107 as not validated.

## Completion Matrix Delta

| Area | T107 state | Evidence |
| --- | --- | --- |
| Direct `search` current-plan retrieval | Narrowly improved | Broad next-step phrases now return the active current-plan memory first in deterministic tests and installed Codex MCP runtime. |
| `orient` hot path | Unchanged | No `orient` code or payload change. |
| Gate behavior | Preserved for tested prompts | Explicit M6/apply queries still return migration gate context above current-plan guidance. |
| Cross-harness parity | Not validated for T107 | Claude Bridge/native Claude Code were unavailable due timeout/auth-network failure. |
| M6 migration | Still gated | T69 exact approval remains required before inspecting the two review-export files. |
| Document indexing | Still gated | T70 exact-file indexing approval remains pending. |
| Harness readiness | Still risky/not ready | T47 exact harness-write gate remains pending. |

## Next Gate

This slice does not authorize migration, lifecycle cleanup, document indexing, harness writes,
schema/storage/index behavior changes, public MCP changes, or `orient` expansion.

The next product-moving gate remains exact T69 approval:
`Approve T69: inspect index.md and 0012-skip-plan.md.`
