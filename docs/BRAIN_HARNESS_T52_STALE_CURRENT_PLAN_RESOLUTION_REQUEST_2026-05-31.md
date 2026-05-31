# Brain Harness T52 Stale Current-Plan Resolution Request

Date: 2026-05-31

Status: Pending user decision. No memory lifecycle write is authorized by this document.

## Summary

T52 refreshes the stale current-plan evidence after T51 and turns the former archive-only
proposal into a decision request. The stale repository-scoped current-plan MemoryItem
`019e5e0a-86b4-73e3-aa9b-ca350e83e915` remains active, but it is also the only active
repository-scoped current-plan item for `/Users/yuval.meiri/projects/engram`. Archiving it
without a replacement would leave that repository scope with no active current-plan memory.

Because that scope gap is a real lifecycle consequence, this document does not include an
executable `memory(action="archive")` payload. The next write, if any, requires the user to
choose one resolution path explicitly.

## Research Question

Can Engram ask for explicit approval to resolve stale repository-scoped current-plan guidance
after T51 using fresh read-only evidence, without implying approval for a lifecycle write?

## Hypotheses

Preferred: a T52 resolution request is the right non-gated slice if it presents archive,
replacement-then-archive, and scope-correction paths as separate choices, preserves the current
project-scoped plan, and stops before any memory write.

Null: T51 is sufficient documentation; keep rejecting the stale memory through feedback and do
not ask for a decision yet.

Simpler alternative: write a refreshed archive-only packet using the latest T51 target and stale
feedback count.

Failure: an archive-only packet hides the scope gap, implies user approval for a lifecycle write,
or treats `safe_action=none` as an archive recommendation.

## Measurement

The slice succeeds if this document:

- names the exact stale target and current successor evidence;
- states that no lifecycle write is authorized;
- presents resolution paths without preselecting one;
- records source-inspected archive behavior and its active-retrieval consequence;
- updates the governing docs so future agents do not execute T48 or infer archive approval from
  T52.

## Fresh Read-Only Evidence

- Lean `orient` trace `019e7d58-cdba-7f71-9451-af294a19a866` returned T51 current-plan
  MemoryItem `019e7d55-b103-70b3-a023-6398e96d6430` first, then the harness-write gate, M6 gate,
  commit preference, and stale repository current-plan target.
- Direct `search` trace `019e7d58-d790-7b71-b6f4-ae5d58b4228a` returned T51 first and stale
  repository current-plan target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` second for the T52
  current-plan query.
- `memory(action="get")` for T51 confirmed it is the active project-scoped current-plan and
  supersedes T50 `019e7d4b-f526-7141-809d-035a7003a2ed`.
- `memory(action="get")` for target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` confirmed it is still
  active, repository-scoped to `/Users/yuval.meiri/projects/engram`, tagged `current-plan`, and
  contains the older Codex document-lifecycle follow-through plan.
- `memory(action="list", scope_type="project", project_name="engram", tags=["current-plan"],
  status_filter="active")` returned exactly T51.
- `memory(action="list", scope_type="repository",
  local_path="/Users/yuval.meiri/projects/engram", tags=["current-plan"],
  status_filter="active")` returned exactly target `019e5e0a-86b4-73e3-aa9b-ca350e83e915`.
- `lint(action="run", write=false, limit=30)` reported
  `feedback_stale_current_plan:019e5e0a-86b4-73e3-aa9b-ca350e83e915` with 142 recent
  stale-feedback records and `safe_action=none`.
- `obligations(action="doctor")` returned no open obligations or warnings before this doc edit.
- Git status before this slice had only untracked root `AGENTS.md`, which remains user-owned and
  out of scope.

## Source-Inspected Archive Behavior

Source inspection found:

- `engram-mcp/src/tools.rs` requires `id` and `archive_reason` for `memory(action="archive")` and
  passes optional `archived_by` through to the service.
- `engram-index/src/memory.rs` loads the exact item by ID, calls `with_archive`, saves it, and
  returns the saved item.
- `engram-core/src/memory.rs` sets status to `archived`, stores archive metadata, and updates the
  item timestamp.
- The focused unit test `archive_memory_retires_item_from_active_retrieval` verifies an archived
  item no longer appears in active memory retrieval.

This confirms the write would be a lifecycle mutation and would hide the target from normal active
retrieval. It is not deletion, but it is still approval-gated.

## External Critique

AI Council broadcast, 2026-05-31:

- Two models said a single-target archive approval packet is safe only if it is default-deny,
  revalidation-gated, and prominently names archive-versus-replacement risk.
- One model objected that an archive-only packet is premature because the target is the sole
  repository-scoped current-plan item and `safe_action=none` means the remediation type is not
  settled.

Claude Bridge critique, 2026-05-31:

- Agreed a Markdown document is safe now because it does not execute a lifecycle write.
- Recommended a resolution request rather than an archive-only packet.
- Flagged the repository-scope gap as material: archiving the only repo-scoped current-plan item
  leaves that scope without active current-plan memory.
- Recommended no executable memory-write command in T52 and no follow-up approval packet until the
  user selects a path.

Synthesis: T52 should surface the decision, not bias the user into an archive-only approval.

## Resolution Options

### Option A: Archive Only

Meaning: approve a future archive of `019e5e0a-86b4-73e3-aa9b-ca350e83e915` with no replacement.

Consequence: `/Users/yuval.meiri/projects/engram` would have zero active repository-scoped
current-plan MemoryItems after the archive. The project-scoped T51 current plan would remain
active, but it is not a repository-scoped replacement.

Use this only if the repository-scoped current-plan layer is not needed for Engram now and the
project-scoped current plan is sufficient for the hot path.

### Option B: Create Replacement, Then Archive

Meaning: approve a new repository-scoped current-plan MemoryItem that points to the current T51
state or its then-current successor, then archive `019e5e0a`.

Consequence: repository-scoped retrieval keeps an active current-plan anchor while stale guidance
is retired.

Use this if repository-scoped current-plan continuity is useful or expected, but the old
document-lifecycle plan should not remain active.

### Option C: Scope-Correct Or Merge First

Meaning: do not archive yet. First decide whether repository-scoped current-plan memory should
exist for this repo, whether the target should be superseded by a different scope, or whether the
important content should be merged into project-scoped memory.

Consequence: the stale target remains active until a narrower follow-up packet specifies the
correct lifecycle action.

Use this if the repository/project scope relationship is still unclear.

## Approval Boundary

This document asks the user to choose one option. It does not approve any of them.

No future action is authorized unless the user explicitly selects a path and approves the exact
write scope in a follow-up turn. If the selected path requires a memory write, a follow-up packet
must specify the exact operation, exact target IDs, exact reason text where applicable, fresh
pre-write checks, and stop conditions.

## Pre-Write Checks For Any Future Lifecycle Write

Before any future lifecycle write on this target, re-run read-only checks and stop unless all
relevant facts still match the selected path:

- target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` still exists and has the expected lifecycle
  status;
- project-scoped active current-plan list still contains exactly the current plan or its explicit
  successor;
- repository-scoped active current-plan list for `/Users/yuval.meiri/projects/engram` is understood
  for the selected option;
- lint still reports the target as stale or the selected option explains why lint changed;
- source behavior for the write still matches the packet assumptions;
- user approval is exact, current, and unambiguous.

## Stop Conditions

Stop before any follow-up write if:

- another active repository-scoped current-plan MemoryItem appears;
- T51 has been superseded and the new successor has not been inspected;
- the target is already archived, superseded, rejected, or otherwise changed;
- lint no longer reports the same stale-current-plan class and the change is unexplained;
- the user approval is broad, ambiguous, or omits the selected path;
- executing the selected path would require M6, harness writes, schema/storage/index changes,
  public MCP changes, ranking changes, or `orient` payload changes.

## Out Of Scope

- running `memory(action="archive")` now;
- archiving, superseding, rejecting, deleting, or reviewing any other MemoryItem;
- creating a replacement current-plan MemoryItem without a separate explicit approval;
- running `lint(action="apply_safe", write=true)`;
- running M6 inventory, review export, write apply, cleanup, deletion, or legacy simplification;
- changing harness adapters, settings, hooks, schema, storage, index state, public MCP behavior,
  ranking, or `orient`.

## User Decision Needed

Choose one path before any lifecycle write proceeds:

- Option A: archive only;
- Option B: create a repository-scoped replacement, then archive the stale target;
- Option C: scope-correct or merge first.

Default if no explicit choice is made: do nothing; keep the stale target active and continue to
treat it as noisy evidence.
