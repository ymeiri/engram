# Brain Harness T187 Stale Handoff Lifecycle Approval Packet

Date: 2026-06-03
Status: Pending user approval. No memory lifecycle write is authorized by this document.
Scope: Ask for future exact approval to archive exactly three superseded rolling handoff
MemoryItems:

- `019e8e6b-bb32-7832-9389-22dd04cbfcda`
- `019e8e6a-dd68-79d3-8bcb-704bc9c52fca`
- `019e8bc0-59a2-7051-b667-e88a1a4861c0`

Archive means preserving the MemoryItem with archived lifecycle metadata. It is not deletion.

This packet is a request for approval, not approval itself. It does not archive, supersede, reject,
review, delete, or edit any MemoryItem. It does not run `lint apply_safe`, change handoff
semantics, change search ranking or `orient`, run native Claude or Claude Bridge, write harness
files, run M6/migration/quarantine actions, change public MCP/schema/storage/index/document-index
behavior, signal PID `49349`, or touch user-owned files.

## Research Question

Can Engram safely ask for exact future approval to archive the three stale rolling handoff records
superseded during the T186 handoff-refresh maintenance, using read-only evidence and without
performing lifecycle writes now?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A three-target, docs-only lifecycle packet is the smallest safe follow-up because the latest active handoff supersedes these records in a direct chain, direct search still surfaces them as active handoff noise, and current-plan retrieval already returns the new T186-gate guidance first. |
| Null | The stale handoff records are tolerable because `handoff(get)` returns the latest handoff and current-plan search still returns the current-plan item first. |
| Simpler alternative | Submit telemetry for stale handoff noise and defer lifecycle cleanup entirely. |
| Failure | The packet is mistaken for approval, archives the wrong handoff, applies broad lint cleanup, sweeps older T140-T145 handoffs without target-local evidence, changes handoff semantics, ranking, `orient`, M6, harness behavior, Claude process state, schema/storage/index, document-index behavior, or user-owned files. |

## Measurement

This packet used read-only evidence only:

- Lean startup `orient` trace `019e8e6d-7aca-7321-8e44-e062977b87cb` returned active
  current-plan memory `019e8e6b-fac4-72b2-b702-d7df6356908c` first and no open obligations.
- Direct current-plan/lifecycle search trace `019e8e6d-91e0-7d33-951f-f93e859a367c` returned
  current-plan memory `019e8e6b-fac4-72b2-b702-d7df6356908c` first, latest handoff
  `019e8e6c-361a-73a0-933e-fcb12c599247` second, and then stale handoffs
  `019e8e6b-bb32-7832-9389-22dd04cbfcda`,
  `019e8e6a-dd68-79d3-8bcb-704bc9c52fca`, and older handoff noise.
- Direct architecture/lifecycle searches `019e8e6d-9dd4-7fb1-b7cd-8bd278faa0e7` and
  `019e8e6d-aadd-7ab2-8016-75121ab0949b` confirmed active rolling handoffs remain prominent
  search noise, while not authorizing a broad sweep.
- `handoff(get)` after the T186 continuity maintenance returns active handoff
  `019e8e6c-361a-73a0-933e-fcb12c599247`, not the stale Claude SessionEnd stub.
- `memory(get)` for `019e8e6c-361a-73a0-933e-fcb12c599247` confirmed it is active, project-scoped
  to `engram`, tagged `handoff` and `rolling`, and directly supersedes
  `019e8e6b-bb32-7832-9389-22dd04cbfcda`.
- `memory(get)` for `019e8e6b-bb32-7832-9389-22dd04cbfcda` confirmed it is active,
  project-scoped, tagged `handoff` and `rolling`, and directly supersedes
  `019e8e6a-dd68-79d3-8bcb-704bc9c52fca`. Its content names superseded current-plan memory
  `019e8e6b-25a0-7d32-8743-3d743d6776c9`, so it is stale relative to current-plan memory
  `019e8e6b-fac4-72b2-b702-d7df6356908c`.
- `memory(get)` for `019e8e6a-dd68-79d3-8bcb-704bc9c52fca` confirmed it is active,
  project-scoped, tagged `handoff` and `rolling`, and directly supersedes
  `019e8bc0-59a2-7051-b667-e88a1a4861c0`. Its content names superseded T186 current-plan memory
  `019e8e67-dce0-7783-a384-02e217a8cd8c`, so it is stale relative to the current plan.
- `memory(get)` for `019e8bc0-59a2-7051-b667-e88a1a4861c0` confirmed it is active,
  project-scoped, tagged `handoff` and `rolling`, and is a Claude Code SessionEnd stub with only
  a generic resume instruction. It lacks the T179-T186 gate matrix.
- `graph(action="around", node="019e8e6c-361a-73a0-933e-fcb12c599247", depth=3)` showed the
  supersession chain:
  `019e8e6c-361a...` -> `019e8e6b-bb32...` ->
  `019e8e6a-dd68...` -> `019e8bc0-59a2...`.
- `lint(action="run", limit=20, write=false)` was read-only and applied zero safe actions. It
  reported broader lifecycle findings, which are intentionally out of scope for this packet.
- AI Council recall found prior T136 guidance: read-only/docs-only audits for active rolling
  handoff search noise are acceptable, but archive/apply, handoff semantics repair, ranking,
  `orient`, schema/storage/index, document-index, M6, and harness/settings writes remain
  exact-gated.
- Git status before this packet showed only the known user-owned untracked root `AGENTS.md`.
- Fresh process evidence showed PID `49349` remains live as
  `/Users/yuval.meiri/.local/bin/claude`; no signal or input was sent.

## Completion Matrix Delta

| Area | State After T187 Packet | Evidence | Remaining Risk Or Gate |
| --- | --- | --- | --- |
| Active current plan | Recoverable and first | `orient` trace `019e8e6d-7aca...` and search trace `019e8e6d-91e0...` | T186 live-process cleanup remains exact-gated |
| Active rolling handoff | Latest handoff is available | `handoff(get)` returns `019e8e6c-361a...` | Handoff search noise remains until lifecycle cleanup is approved |
| Stale handoff chain | Exact archive targets identified | `memory(get)` and graph supersession chain | Requires exact T187 approval before archive |
| Older T140-T145 handoffs | Still noisy but out of scope | Direct searches return older active handoffs | Needs separate target-local evidence or broader explicit approval |
| Lifecycle cleanup | Still gated | No `memory(action="archive")`; no `lint(action="apply_safe")` | Archive/write approval required |
| Hot path and ranking | Unchanged | No source/runtime behavior changed | Do not change ranking or `orient` from this packet |
| Native Claude process | Still unresolved | PID `49349` remains live | T186 exact approval remains required |
| M6/migration | Still gated | No M6/migration/quarantine action ran | Separate reviewed-candidate and dry-run/apply approval |

## Proposed Approved Archives

If and only if the user approves with the exact phrase below, Codex may run these three Memory OS
archive writes and no others:

```text
memory(
  action="archive",
  id="019e8e6b-bb32-7832-9389-22dd04cbfcda",
  archive_reason="Superseded intermediate rolling handoff from T186 handoff-refresh maintenance. Latest active handoff 019e8e6c-361a-73a0-933e-fcb12c599247 supersedes it, it still appears as active direct-search handoff noise, and its content points at superseded current-plan memory 019e8e6b-25a0-7d32-8743-3d743d6776c9 rather than active current-plan memory 019e8e6b-fac4-72b2-b702-d7df6356908c. Human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)

memory(
  action="archive",
  id="019e8e6a-dd68-79d3-8bcb-704bc9c52fca",
  archive_reason="Superseded intermediate rolling handoff from T186 handoff-refresh maintenance. Handoff 019e8e6b-bb32-7832-9389-22dd04cbfcda supersedes it, it still appears as active direct-search handoff noise, and its content points at superseded current-plan memory 019e8e67-dce0-7783-a384-02e217a8cd8c rather than active current-plan memory 019e8e6b-fac4-72b2-b702-d7df6356908c. Human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)

memory(
  action="archive",
  id="019e8bc0-59a2-7051-b667-e88a1a4861c0",
  archive_reason="Stale Claude Code SessionEnd rolling handoff stub superseded by T186 handoff-refresh maintenance. Handoff 019e8e6a-dd68-79d3-8bcb-704bc9c52fca supersedes it, and its content only says to call orient and inspect the handoff, without the T179-T186 gate matrix now present in latest handoff 019e8e6c-361a-73a0-933e-fcb12c599247. Human-approved manual archive, not lint apply_safe.",
  archived_by="codex"
)
```

## Required Fresh Pre-Write Evidence

Immediately before any future archive calls, in the same execution session, collect fresh read-only
evidence with no intervening writes between the final read-only check and the three archive calls:

| Check | Required result |
| --- | --- |
| `handoff(action="get", project="engram")` | Latest active handoff remains `019e8e6c-361a-73a0-933e-fcb12c599247`, or the user re-approves after seeing the newer handoff. |
| `memory(action="get", id="019e8e6c-361a-73a0-933e-fcb12c599247")` | Latest handoff remains active, project-scoped to `engram`, tagged `handoff` and `rolling`, and supersedes `019e8e6b-bb32-7832-9389-22dd04cbfcda`. |
| `memory(action="get", id="019e8e6b-bb32-7832-9389-22dd04cbfcda")` | Target exists, is `active`, project-scoped to `engram`, tagged `handoff` and `rolling`, and supersedes `019e8e6a-dd68-79d3-8bcb-704bc9c52fca`. |
| `memory(action="get", id="019e8e6a-dd68-79d3-8bcb-704bc9c52fca")` | Target exists, is `active`, project-scoped to `engram`, tagged `handoff` and `rolling`, and supersedes `019e8bc0-59a2-7051-b667-e88a1a4861c0`. |
| `memory(action="get", id="019e8bc0-59a2-7051-b667-e88a1a4861c0")` | Target exists, is `active`, project-scoped to `engram`, tagged `handoff` and `rolling`, and still contains only the Claude SessionEnd stub. |
| Current-plan orient or direct search | Current Engram project guidance remains recoverable before the stale handoffs are archived. |
| Handoff search | At least one target still appears as active stale handoff noise, or the graph/supersession chain still proves staleness. |
| `lint(action="run", write=false)` | The result is read and recorded. This packet does not depend on lint safe actions. |
| `graph(action="around", node="019e8e6c-361a-73a0-933e-fcb12c599247", depth=3)` | The direct supersession chain still includes all three targets and no unexpected direct dependency changes require user review. |
| `git status --short` | Only the known user-owned untracked `AGENTS.md` may be present unless the user approves a different worktree state. |
| `obligations(action="doctor", project="engram")` | Open obligations are absent or explicitly resolved/skipped with evidence before final response. |

## Out Of Scope

T187 does not authorize:

- archiving, superseding, rejecting, reviewing, editing, or deleting any other MemoryItem;
- running `lint(action="apply_safe", write=true)` or any broad lifecycle cleanup;
- changing `handoff(update)` semantics;
- changing search ranking, `orient`, public MCP, schema/storage/index, graph, lint rules,
  telemetry formulas, or document-index behavior;
- sending any native Claude input, including EOF, Ctrl-C bytes, `/hooks`, another slash command, or
  prompt-bearing input;
- sending any process signal, including `SIGINT`, `SIGTERM`, `SIGKILL`, or force-kill fallback;
- launching native Claude or Claude Bridge;
- harness installs, adapter/settings/hook edits, `adopt_user_owned=true`, rollback, force-kill, or
  old-binary reinstall;
- M6 migration inventory, review export, status, prioritize, apply, cleanup, deletion, quarantine
  inspection, candidate decisions, or legacy simplification;
- editing root `AGENTS.md` or other user-owned files.

## Stop Conditions

Stop without archiving if any of these occur:

- approval is missing, conditional, ambiguous, or does not include the exact T187 wording and all
  three target IDs;
- any target UUID, kind, scope, status, tags, or archive payload differs from this packet;
- any target is already archived, rejected, deleted, missing, or no longer a rolling handoff;
- latest active handoff for Engram cannot be identified before the archive;
- active current-plan guidance for Engram cannot be identified before the archive;
- fresh graph no longer shows the expected supersession chain;
- any write occurs after the final fresh pre-write read and before the archive calls;
- any step appears to require creating a replacement memory, applying automatic lint cleanup,
  mutating other memories, changing ranking, changing `orient`, running M6, inspecting quarantine
  candidates, executing Claude, sending process signals, or executing harness writes.

## Approval Wording

To authorize only the three archive actions above, reply exactly:

```text
Approve T187: after fresh matching read-only handoff/get/orient-or-search/handoff-search/lint/graph/git/obligations evidence and no intervening writes, archive exactly MemoryItems 019e8e6b-bb32-7832-9389-22dd04cbfcda, 019e8e6a-dd68-79d3-8bcb-704bc9c52fca, and 019e8bc0-59a2-7051-b667-e88a1a4861c0 with the archive payloads in docs/BRAIN_HARNESS_T187_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md. Do not run lint apply_safe, archive any other memory, change handoff semantics, ranking, orient, public MCP, schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude Bridge, Claude hooks, process signals, harness installs/settings/hooks/adapters, or user-owned files.
```

Any other reply should be treated as non-authorization for T187.
