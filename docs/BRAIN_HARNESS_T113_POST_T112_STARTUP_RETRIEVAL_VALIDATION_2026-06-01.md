# Brain Harness T113 Post-T112 Startup Retrieval Validation

Status: Completed read-only startup retrieval validation; no behavior change
Date: 2026-06-01
Scope: Validate the actual post-T112 startup surface before choosing any gated behavior change.

This slice did not add a recommendation string, generate calibration traces, run M6 inspection or
apply, inspect migration export files, mutate lifecycle state, run `lint(action="apply_safe")`,
index documents, change ranking, expand `orient`, change public MCP/schema/storage/index behavior,
change document-index behavior, or write harness adapters/hooks.

## Research Question

After T112, does the real startup path recover the active T112 current plan and exact T111 gate
context well enough to continue non-gated work, without treating generic approval or stale retrieval
noise as authorization?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Fresh lean `orient`, direct search, handoff, repo docs, obligations, lint, and git checks recover T112 as current guidance, while keeping T111, M6, lifecycle, document-index, and harness gates closed. |
| Null | T112 is recoverable only from the handoff or repo docs, so startup retrieval still leaves the agent likely to act from stale T111/T110 context. |
| Simpler alternative | Stop after T112 and wait for the user to provide an exact T111 Option A or B phrase. |
| Failure | The validation reframes generic `i approve` as scoped authorization, hides stale current-plan risk, or implies ranking/lifecycle work. |

## Measurement

Fresh startup evidence:

- Lean `orient` trace `019e847d-bf8e-7c23-95a9-d3dc9a5528b9` returned
  `019e847b-87a8-7a12-b1cf-4dc94e87ba79` first: "Current plan after T112 recommendation surface
  audit." It also returned older stale repository-scoped current-plan memory
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` as lower-ranked active context.
- Direct search trace `019e847d-f4dd-75b0-a0f7-1d2d7bd7bd58` for current-plan, next-step, T112,
  T111, and gated-approval terms returned the T112 current-plan memory first.
- Direct search trace `019e847d-f672-78b0-a8d9-b03e53d37252` for Brain Harness architecture and
  retrieval/lifecycle/migration gates returned the T112 current-plan memory first.
- Direct search trace `019e847d-f7f6-7e70-9bc6-02038633daab` for the Memory OS implementation
  plan and M6/T69/T70/T108 returned the active T112 handoff first, not a migration authorization.
- Direct search trace `019e847d-f990-7912-9a31-2074f26ed938` for the user's software design
  philosophy returned the reviewed software-design preference
  `019e6924-256b-7093-b1c5-286ec4d02461` in the top memory results; the matching lean preference
  `orient` trace `019e847f-0a94-7641-9420-0cb55cb40fca` placed that preference first in hot
  context.
- Direct search trace `019e847d-fb33-72b2-8fb5-23fbadc139d9` for recent failures and open risks
  returned the T112 current-plan memory first and the T111 paused handoff below it.
- Direct search trace `019e847f-083e-7522-9da0-c69d8fe9b1e1` for "what should happen next Engram
  Brain Harness after T112 recommendation surface audit" returned the T112 current-plan memory
  first.
- Direct search trace `019e847f-09ce-7db1-83d8-f1f16a499c83` for the exact T111 Option A approval
  context returned the T112 current-plan memory first and surfaced the T111 paused handoff below it.
- `handoff(action="get", project="engram")` returned active handoff
  `019e847b-b157-7270-bb02-ffb6243ca9dd`, which names commit `3588f95`, says T112 is complete, and
  preserves exact gates.
- `obligations(action="doctor", project="engram")` returned no open obligations or warnings.
- `lint(action="run", limit=10)` still reported stale current-plan target
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` first, with `safe_action=none`.
- Git status was clean except untracked user-owned root `AGENTS.md`; latest commit was `3588f95`
  (`Record T112 recommendation surface audit`) on branch `yuval.meiri/memory-os-phase0`.
- Repo docs read for this slice included `docs/BRAIN_HARNESS_ARCHITECTURE.md`,
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`, `docs/BRAIN_HARNESS_RESEARCH_METHOD.md`,
  `docs/ORIENT_CONTRACT.md`, T112, T110, T109, T108, and T105 report docs.

## Completion Matrix

| Area | T113 state | Evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| Memory OS substrate | Implemented | Governing docs and existing MCP surfaces remain in place | Legacy layers remain substrate until evals justify simplification. |
| `orient` hot path | Validated for this startup shape | Lean T113 startup and preference-oriented traces returned compact context and no obligations | Do not expand payload or add specialist engines without explicit approval. |
| Current-plan / next-step retrieval | Validated for post-T112 continuation probes | T112 current-plan memory ranked first in fresh lean `orient` and direct continuation/current-plan searches | Older active handoffs and stale current-plan memory still appear below current guidance. |
| T111 eval design | Blocked on exact choice, not implementation | T112 and T113 recover the exact Option A/B gate context | Generic `i approve` is not enough; report-content behavior remains public observable output. |
| Evidence and feedback loop | Partially validated | T110 regression and T113 traces preserve the default-window caveat | Agent feedback remains weak unless tied to transcript, tests, user review, or controlled artifacts. |
| Memory lifecycle | Risky/gated | Lint still reports stale current-plan target with `safe_action=none` | No archive, scope correction, `apply_safe`, or broad cleanup is authorized. |
| Document visibility | Partially validated | T67 exact-file indexing already improved T59 visibility; repo docs were authoritative for T113 | T70 remains exact-file indexing only; it does not replace T69 inspection. |
| M6 migration | Stopped/gated | T58 inventory and T68 stopped export remain documented; T113 ran no M6 action | T69 exact inspection approval is still required before reading the two export files; apply/deletion/simplification require later explicit approval. |
| Cross-harness readiness | Partially validated/risky | Prior Codex and Claude Code smokes exist; T106 still reports supported harness readiness false | T47 exact harness-write gate remains pending. |
| Git/worktree hygiene | Validated | Only untracked root `AGENTS.md`; latest commit `3588f95` | Leave `AGENTS.md` untouched and unstaged. |

## Interpretation

The post-T112 startup path is healthy enough to continue non-gated evidence work: current-plan
retrieval is no longer the immediate blocker for continuation prompts in Codex. The main product
blockers are still explicit gates, not missing startup context:

- T111 needs an exact Option A or Option B choice before any `real_session_eval` recommendation
  text changes.
- T69 needs exact inspection approval before reading the two T68 export files.
- T70 remains a separate exact-file indexing gate, not migration approval.
- The stale current-plan target remains visible but has `safe_action=none`.
- Harness writes remain gated by the T47 packet.

## Next Gate

If the project owner wants the T111 behavior change, require:

`Approve T111 Option A: add the contextual default-window recommendation string.`

If the project owner wants to keep the report unchanged, require:

`Approve T111 Option B: keep T111 docs-only and do not change real_session_eval recommendations.`

Neither phrase authorizes M6 apply/deletion/simplification, lifecycle writes, document indexing,
harness writes, broad ranking changes, `orient` expansion, public MCP request-parameter changes,
schema/storage/index behavior changes, or document-index behavior changes.
