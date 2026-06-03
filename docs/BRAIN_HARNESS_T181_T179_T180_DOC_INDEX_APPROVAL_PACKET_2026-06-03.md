# T181 T179/T180 Document Index Approval Packet

Date: 2026-06-03
Status: docs-only/default-deny approval packet. No document indexing has been run by this packet.

## Scope

This packet prepares a future exact-file document-index visibility repair for the newest native
Claude gate documents:

- `docs/BRAIN_HARNESS_T179_T172_NATIVE_CLAUDE_EFFECTIVE_HOOK_RESULT_2026-06-03.md`
- `docs/BRAIN_HARNESS_T180_T179_NATIVE_CLAUDE_LIVE_PROCESS_RECOVERY_APPROVAL_PACKET_2026-06-03.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`

It does not run document indexing, change document-index behavior, send input to the live native
Claude PTY, signal or kill PID `49349`, launch native Claude, run Claude Bridge, edit
hooks/settings/adapters, run harness install, mutate lifecycle state, run M6/migration/quarantine
actions, make candidate decisions, change ranking/`orient`, change public MCP/schema/storage/index
behavior, delete, roll back, reinstall binaries, or touch user-owned files.

## Current Evidence

- T178 last indexed `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` before T179 and T180 existed.
- T179 committed the T172 hard-stop result as `602f1a1`.
- T180 committed the live-process recovery approval packet as `9e6f78f`.
- T179 and T180 both updated `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` after the last
  implementation-plan indexing.
- Direct unified search still recovers current-plan memory
  `019e8e51-145d-76d2-85ff-8319f77a79f4` first for T180 continuation prompts, so this is not a
  current-plan retrieval failure.
- Fresh document-only search trace `019e8e52-51b7-75a0-9856-477152592b34` queried
  `T180 T179 Native Claude Live-Process Recovery Approval Packet` and returned older T160/T173/T159
  chunks instead of the T180 packet.
- Fresh document-only search trace `019e8e52-6830-70c1-8308-36cf07a0f658` queried the T180 filename
  stem and returned T176/T160/T159 chunks instead of the T180 packet.
- Fresh document-only search trace `019e8e52-d112-7e03-9513-e56a8d5d1295` queried
  `T179 T172 Native Claude Effective-Hook Result hard-stop PID 49349` and returned only older T172
  approval-packet chunks.
- Current files exist and have these sizes and SHA-256 hashes:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `docs/BRAIN_HARNESS_T179_T172_NATIVE_CLAUDE_EFFECTIVE_HOOK_RESULT_2026-06-03.md` | 8963 | `798bc87faac381daa1fd0ac4cd38a46310f939b1bd20e71468efc24abc7627ea` |
| `docs/BRAIN_HARNESS_T180_T179_NATIVE_CLAUDE_LIVE_PROCESS_RECOVERY_APPROVAL_PACKET_2026-06-03.md` | 11176 | `a38c505e2d414f73a35297b429c4571cbadeb18af495153be45259f814494817` |
| `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` | 305225 | `c6a72f7682ce0ab753a2be1285d39e933f6749e277a28adcffc4c96836dcda5c` |
- Git status is clean except pre-existing untracked root `AGENTS.md`.
- PID `49349` remains live as `/Users/yuval.meiri/.local/bin/claude`; this packet does not resolve
  that process.

## Research Question

Can Engram safely make the latest T179/T180 result and approval-gate documents visible through
document search without changing retrieval code, creating MemoryItems for packet docs, or crossing
the T180 native-Claude recovery gate?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Indexing exactly T179, T180, and the updated implementation plan makes the latest native-Claude hard-stop and recovery-gate evidence recoverable through document search while preserving all underlying gates. |
| Null | The files index successfully, but semantic search remains noisy; repo files and current-plan memory remain authoritative. |
| Simpler alternative | Defer indexing and require repo-file inspection for T179/T180 exact approval wording and matrix notes. |
| Failure | The operation expands into directory indexing, cleanup/reindex, code changes, MemoryItem creation, lifecycle mutation, native Claude input or cleanup, M6 work, ranking changes, or implied approval for T180 recovery. |

## Proposed Approved Scope

If the user approves this packet, Codex may perform only the following:

1. Read-only file-existence preflight for these exact paths:
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T179_T172_NATIVE_CLAUDE_EFFECTIVE_HOOK_RESULT_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T180_T179_NATIVE_CLAUDE_LIVE_PROCESS_RECOVERY_APPROVAL_PACKET_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
2. Index exactly those three files through MCP `docs(action="index", path=...)`.
3. Run read-only validation searches:
   - `T179 T172 Native Claude Effective-Hook Result`
   - `T179 hard-stop result PID 49349`
   - `T180 T179 Native Claude Live-Process Recovery Approval Packet`
   - `Approve T180: execute the T179 native Claude live-process recovery packet`
   - `T180 matrix note one additional Ctrl-C same live PTY`
4. Record a Markdown result report and implementation-plan note if documentation changes are made.
5. Commit only intended documentation files.
6. Capture current-plan memory and submit telemetry feedback for assessable retrieval traces.

## Success Criteria

- T179 appears in the top five document results for its exact title or hard-stop/PID probe.
- T180 appears in the top five document results for its exact title, filename-stem, exact approval
  phrase, or matrix-note probe.
- The updated `MEMORY_OS_IMPLEMENTATION_PLAN.md` T180 matrix note appears in the top five for the
  matrix-note probe or another approved validation probe.
- Every underlying approval gate remains unchanged:
  - T180 still requires exact approval before sending any input to the live native Claude PTY.
  - T174 still requires exact approval before read-only M6 candidate-decision/dry-run scoping.
  - Effective-hook visibility remains open unless later evidence closes it under an approved
    native-Claude packet.
  - Candidate decisions, dry-run apply, write apply, deletion, native Claude prompt-bearing
    validation, lifecycle cleanup, ranking/`orient`, public MCP, schema/storage/index, and
    document-index behavior changes remain separately gated.
- No MemoryItem is created for these document artifacts.
- No native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude, harness write,
  lifecycle archive, `lint apply_safe`, M6/migration/quarantine action, candidate decision,
  deletion, cleanup, schema/storage/index behavior change, document-index behavior change, public
  MCP change, ranking change, or `orient` change occurs.

## Stop Conditions

Stop and report without continuing if any of these occur:

- Approval is missing, conditional, abbreviated, or ambiguous.
- Any of the three exact files is missing or is not a regular file.
- The available index surface cannot target exact files.
- The operation would index a directory, recurse into broader docs, run cleanup/reindex, or require
  a schema/storage migration.
- The operation proposes code changes, public MCP changes, ranking changes, or document-index
  behavior changes.
- The operation creates or requires active MemoryItems for packet docs.
- The result appears to run or require native Claude, Claude Bridge, Claude `/hooks`,
  prompt-bearing Claude, harness writes, lifecycle mutation, `lint apply_safe`, M6 migration or
  quarantine work, candidate decisions, deletion, cleanup, process signals, force-kill, rollback,
  old-binary reinstall, or user-owned-file edits.

## Approval Question

Reply exactly:

```text
Approve T181: index exact files T179, T180, and MEMORY_OS_IMPLEMENTATION_PLAN.
```
