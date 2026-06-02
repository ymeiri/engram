# T136 Stale Active Handoff Noise Audit

Date: 2026-06-02
Status: completed read-only audit
Scope: direct-search and lifecycle evidence for active rolling handoff noise

No lifecycle archive, `lint(apply_safe)`, memory status update, ranking change, `orient` change,
schema/storage/index change, public MCP change, document-index change, M6 action, harness install,
hook edit, or settings edit was run for T136.

## Research Question

Why do older rolling handoff MemoryItems still appear as active direct-search noise after newer
handoff updates, and does that require action before the T135 harness repair gate?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | `handoff(update)` creates a supersedes edge to the previous handoff but intentionally leaves the previous item `active`; direct search then treats those older handoffs as normal active memory. This should be documented as lifecycle evidence, not fixed without approval. |
| Null | The observed handoff noise is only a query artifact, with no active stale handoff chain in live memory. |
| Simpler alternative | Pause solely for T135 approval and do no lifecycle audit. |
| Failure | The audit becomes proxy authorization for archive/apply/ranking/`orient` changes or hides that the active evidence base is noisy. |

## Measurement

The audit used read-only calls only:

- lean `orient` startup trace `019e8842-7a4e-7621-b87e-94e174a079e4`;
- direct current-plan search trace `019e8842-c5a3-7843-95a5-28cfd1a10d87`;
- direct risk search trace `019e8842-e2ee-72c2-8f65-91070727ed72`;
- `handoff(action="get", project="engram")`;
- `memory(action="list", project_name="engram", tags=["handoff","rolling"], status_filter="active")`;
- `memory(action="list", project_name="engram", tags=["current-plan"], status_filter="active")`;
- `lint(action="run", limit=20)`;
- representative direct-search probes:
  - `019e8844-f830-75c1-b0a9-d0bad593cec4`;
  - `019e8844-f873-7a41-aaf1-e7c23306ab28`;
  - `019e8844-f8b2-7252-9102-5116c3aa87dc`;
- source reads in `engram-index/src/handoff.rs`, `engram-index/src/memory.rs`,
  `engram-index/src/lint.rs`, and `engram-index/src/memory_ranker.rs`.

AI Council recall found prior lifecycle-gate guidance. A fresh broadcast agreed that this audit is
acceptable only as docs-only/read-only evidence and must stop before any archive, apply, ranking,
`orient`, schema/storage/index, document-index, M6, or harness/settings mutation.

Claude Bridge was not used because the installed Claude `SessionEnd` hook remains a known stale
durable handoff-write path until the T135 repair gate is explicitly approved and executed.

## Findings

### 1. The Current Plan Is Not Ambiguous

Lean startup `orient` returned the active T135 current-plan memory
`019e8840-eb0e-78a1-a0a0-f25e7bdc3c62`, plus the harness-write gate, M6 gate, design preference,
and research-method rule. A scoped current-plan memory list returned exactly one active
project-scoped current-plan item: the same T135 decision.

`handoff(get)` also returned the latest rolling handoff,
`019e8841-0e50-7a32-9994-c1bd02501741`, whose next actions point to exact T135 approval and
preserve the same exclusions.

### 2. Active Rolling Handoff Noise Is Real

The active project-scoped rolling-handoff list returned 50 items at the requested limit, so the
live corpus has at least 50 active project rolling handoffs. The first items form a visible chain:

| Position | Memory ID | Summary |
| --- | --- | --- |
| 1 | `019e8841-0e50-7a32-9994-c1bd02501741` | Latest T135 handoff; supersedes T133A handoff. |
| 2 | `019e8838-1d2c-7ef0-8755-62729131bb74` | T133A handoff; still `active`; supersedes T134 handoff. |
| 3 | `019e8805-3431-7a73-96af-4eb40cd2107c` | T134 handoff; still `active`; supersedes T133 handoff. |
| 4 | `019e8800-6bf3-7f83-b265-795d6263e000` | T133 handoff; still `active`; supersedes T130 handoff. |
| 5 | `019e87b1-e641-7f00-a042-a6d101f9b36f` | T130 handoff; still `active`; supersedes T132 handoff. |

The same list also includes lower-information Claude session-end handoffs such as
`019e8468-8ce9-73f2-a2ec-fda4d27caccc`,
`019e8468-8ce9-73f2-a2ec-fd99f464642d`,
`019e845c-7283-7ce2-8b40-46b4719ae771`, and
`019e845c-7283-7ce2-8b40-46abfd5b0985`.

### 3. Direct Search Can Be Usable And Noisy At Once

For the direct current-plan query, trace `019e8844-f8b2-7252-9102-5116c3aa87dc` ranked T135
current-plan memory first, then returned active rolling handoffs including the latest T135 handoff,
T134, T133, and T132.

For the risk-oriented query, trace `019e8844-f873-7a41-aaf1-e7c23306ab28` returned older active
rolling handoffs at the top. That is not by itself a wrong answer because the query asked for
failures, caveats, and risks, but it shows that the active handoff chain is search-visible and can
crowd out the freshest gate unless the agent cross-checks `orient`, `handoff(get)`, current-plan
memory, and repo docs.

### 4. Source Semantics Explain The Noise

`HandoffService::update` finds the previous active handoff and adds it to the new item with
`with_superseded_item(previous.id)`, but it only saves the new item; it does not mark the previous
handoff as `superseded`.

Evidence:

- `engram-index/src/handoff.rs:88` gets the previous handoff.
- `engram-index/src/handoff.rs:107` to `engram-index/src/handoff.rs:108` adds the previous ID to
  the new handoff's `supersedes`.
- `engram-index/src/handoff.rs:111` to `engram-index/src/handoff.rs:112` saves the new item only.
- `engram-index/src/handoff.rs:160` to `engram-index/src/handoff.rs:166` selects from active
  handoffs for `handoff(get)`.

That differs from `capture_current_plan`, which explicitly marks prior current-plan guidance as
`MemoryStatus::Superseded` after adding supersedes edges.

Evidence:

- `engram-index/src/memory.rs:747` to `engram-index/src/memory.rs:754` records superseded IDs.
- `engram-index/src/memory.rs:768` to `engram-index/src/memory.rs:781` writes each previous
  current-plan item back with `MemoryStatus::Superseded`.

The ranker then gives active items full status score while superseded and archived items get zero:
`engram-index/src/memory_ranker.rs:552` to `engram-index/src/memory_ranker.rs:557`.

### 5. Lint Already Detects The Class, But Applying It Is Gated

Read-only `lint(action="run", limit=20)` returned `applied_safe_actions=0`. It showed many
`superseded_item_still_active` findings with `safe_action="archive_memory_item"`, plus
telemetry-backed stale/wrong-scope findings with `safe_action="none"`.

Source confirms the boundary:

- `engram-index/src/lint.rs:332` to `engram-index/src/lint.rs:352` detects active items referenced
  by another active item's `supersedes` list and marks them archive candidates.
- `engram-index/src/lint.rs:117` to `engram-index/src/lint.rs:139` archives those items only via
  `apply_safe`.

T136 did not run `lint(action="apply_safe")`; archive/apply remains a lifecycle mutation requiring
exact approval.

### 6. CLI Direct-Store Listing Is Not The Right Live Inspection Path

Attempts to use `engram memory list --status active --json` failed because the global daemon already
held the RocksDB lock at `/Users/yuval.meiri/.engram/data/LOCK`. This matches the known daemon
ownership model. The audit therefore treats MCP/daemon responses as authoritative live evidence.

## Completion Matrix Delta

| Area | T136 status | Evidence |
| --- | --- | --- |
| Current-plan continuity | Validated for this startup | Lean `orient` and current-plan list recover T135 as the sole active project current plan. |
| Latest handoff retrieval | Validated but narrow | `handoff(get)` returns the T135 handoff. |
| Direct-search evidence quality | Partially validated, noisy | Current-plan search is usable; risk-oriented search can rank old active handoffs first. |
| Handoff lifecycle hygiene | Missing, gated | At least 50 active rolling handoffs remain; many are superseded by newer handoffs. |
| Lifecycle cleanup | Risky, gated | Lint can identify archive candidates, but `apply_safe`/archive needs exact approval. |
| Harness readiness | Still blocked on T135 approval | No harness/settings writes were made. |
| M6 migration completion | Still gated | No M6 action was run. |

## Decision

T136 does not justify a code change, lifecycle cleanup, ranking change, or `orient` expansion by
itself. The active evidence base is noisy, but the current-plan and latest-handoff surfaces still
recover the correct T135 gate when used as intended.

The next product-moving gate remains exact T135 approval. A later lifecycle slice could be prepared
only with explicit approval and should name exact targets or an exact dry-run/apply-safe policy,
because the active handoff chain contains both useful historical handoffs and low-information
Claude session-end stubs.

## Stop Conditions For Follow-Up

Stop and ask before any follow-up that would:

- archive, supersede, reject, delete, or otherwise change MemoryItem lifecycle state;
- run `lint(action="apply_safe", write=true)`;
- change `handoff(update)` semantics;
- change search ranking or `orient` selection;
- change public MCP, schema, storage, index, document-index, M6, or harness/settings behavior;
- treat T136 as approval for T135 or any lifecycle cleanup.
