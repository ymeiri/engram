# Brain Harness T193 Stale T106 Handoff Lifecycle Approval Packet

Date: 2026-06-03

Status: pending exact user approval. This packet records a proposed future lifecycle write; it is
not approval and does not archive, delete, reject, review, supersede, or edit memory.

## Scope

Ask for future exact approval to archive exactly one active stale rolling handoff MemoryItem:

- `019e839e-f061-71c2-95d3-f4c44029ac7b`

The archive would preserve the item as historical evidence. It would not delete data.

This packet does not run `lint apply_safe`, archive any other memory, change handoff semantics,
ranking, `orient`, public MCP parameters, schema/storage/index/document-index behavior,
M6/migration/quarantine state, native Claude, Claude Bridge, Claude hooks, process signals, harness
installs/settings/hooks/adapters, deletion, rollback, or user-owned files.

## Research Question

Can Engram safely ask for exact future approval to archive one stale pre-T107 rolling handoff that
still appears as active handoff noise, using read-only evidence only?

## Hypotheses

Preferred hypothesis: a single-target approval packet is the smallest safe follow-up because the
target is active, project-scoped, and still describes T106/T69/T70/T47-era gates, while the latest
active handoff and current-plan memory now carry T192/T191/T187/T186/M6 gates.

Null hypothesis: tolerate the stale active handoff because current-plan retrieval still ranks the
latest current-plan memory first.

Simpler alternative: submit telemetry feedback only and defer lifecycle cleanup until a broader
human-approved stale-handoff batch.

Failure hypothesis: a packet is mistaken for approval, archives the wrong item, sweeps broad old
handoffs, runs lint safe actions, or claims a direct latest-to-target supersession chain that the
graph evidence does not prove.

## Measurement

Fresh startup and direct retrieval evidence:

- Lean `orient` trace `019e8e93-25bf-76c0-9213-6597e2705520` returned current-plan memory
  `019e8e8f-a725-7700-b8b6-55a13049d484` first and reported no open obligations.
- Direct current-plan search trace `019e8e93-33aa-78e1-bc6e-699f3d209ce4` returned the same
  current-plan memory first, then stale active handoff noise including this target.
- Focused memory search trace `019e8e95-35ae-73d1-9119-c6614299c894` returned current-plan memory
  first, then stale active rolling handoffs; target `019e839e-f061-71c2-95d3-f4c44029ac7b`
  remained active search noise.

Fresh read-only target evidence:

- `memory(get)` for `019e839e-f061-71c2-95d3-f4c44029ac7b` shows status `active`, kind `handoff`,
  project scope `engram`, tags `handoff` and `rolling`, and content updated 2026-06-01 around T106.
- The target content says T69, T70, and T47 remain exact gates, which is obsolete relative to the
  current T192 handoff and implementation plan.
- The target supersedes `019e8394-4714-73f3-9cf0-37592841317f`.
- `handoff(get)` returns latest active handoff `019e8e8f-fae1-77f0-8b77-68053c3173e7`, not this
  target.
- Graph `around` depth 1 shows `019e83a4-79fd-7f72-ab8b-4a953d3dd7b9` supersedes this target.
- Graph `path` from latest handoff `019e8e8f-fae1-77f0-8b77-68053c3173e7` to this target only
  traverses project scope. It does not prove a direct latest-to-target supersession chain.

Fresh guardrail evidence:

- `lint(action="run", write=false, limit=20)` reported existing wrong-scope feedback and many
  superseded-active findings, but no safe action was applied.
- `obligations(action="doctor", project="engram")` reported no open obligations or warnings.
- `git status --short --branch` showed branch `yuval.meiri/memory-os-phase0` with only the
  pre-existing untracked root `AGENTS.md`.
- Recent commits remain `5393eed` T192, `bc25df8` T191, `a11d7bd` T190, `3b71d3b` T189,
  `0ed1202` T188, and `88100e1` T187.

## Completion Matrix

| Area | State | Evidence |
| --- | --- | --- |
| Current-plan retrieval | Implemented and currently healthy | `orient` trace `019e8e93-25bf...`; search trace `019e8e93-33aa...` |
| Latest handoff continuity | Implemented | `handoff(get)` latest `019e8e8f-fae1...` |
| Target stale-handoff evidence | Partially validated | Target is active/stale; graph proves local supersession but not direct latest-chain supersession |
| Lifecycle action | Missing and gated | No archive performed; exact future approval required |
| Broad stale-handoff cleanup | Missing/risky | Out of scope; lint lists many possible stale items |
| T192 document indexing | Blocked on exact approval | Separate T192 packet |
| T191/T187 lifecycle archive packets | Blocked on exact approval | Separate packets |
| T186 process cleanup and T172 visibility | Blocked on exact approval/evidence | Separate native-Claude gates |
| M6 migration completion | High-risk blocked on exact approval | Migration write/apply remains review-gated |

## Proposed Archive Payload

Only after exact approval and fresh matching preflight evidence:

```text
memory(
  action="archive",
  id="019e839e-f061-71c2-95d3-f4c44029ac7b",
  archive_reason="Stale active rolling handoff from T106/T69/T70/T47 era. Latest active handoff 019e8e8f-fae1-77f0-8b77-68053c3173e7 carries T192/T191/T187/T186/M6 gates, while this target still appears as active handoff search noise and points at obsolete gates/current-plan memory. Human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)
```

## Required Preflight For Any Future Archive Write

Before the archive write, rerun fresh read-only evidence and stop if any condition fails:

- `handoff(get)` still returns latest active handoff `019e8e8f-fae1-77f0-8b77-68053c3173e7`, or the
  user re-approves after a newer handoff.
- `memory(get)` for `019e839e-f061-71c2-95d3-f4c44029ac7b` still shows active, project-scoped,
  kind `handoff`, tags `handoff` and `rolling`, and stale T106/T69/T70/T47-era content.
- Current-plan `orient` or direct search recovers T192 or newer current-plan memory before any
  archive write.
- Focused search still shows the target as active stale handoff noise, or the user explicitly
  re-approves despite changed search evidence.
- `lint(action="run", write=false)` is read and recorded; do not run `lint apply_safe`.
- Graph around the target is read again; if it shows an unexpected dependency, stop.
- Git status has no tracked diff other than the intentional lifecycle-result docs, and root
  `AGENTS.md` remains untouched.
- `obligations(action="doctor", project="engram")` remains clean or any open obligations are
  resolved/skipped with evidence before the write.

## Approval Wording

Approve T193: after fresh matching read-only handoff/get/current-plan-orient-or-search/focused-search/lint/graph/git/obligations evidence and no intervening writes, archive exactly MemoryItem `019e839e-f061-71c2-95d3-f4c44029ac7b` with the archive payload in `docs/BRAIN_HARNESS_T193_STALE_T106_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md`. Do not run `lint apply_safe`, archive any other memory, change handoff semantics, ranking, orient, public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude Bridge, Claude hooks, process signals, harness installs/settings/hooks/adapters, deletion, rollback, or user-owned files.
