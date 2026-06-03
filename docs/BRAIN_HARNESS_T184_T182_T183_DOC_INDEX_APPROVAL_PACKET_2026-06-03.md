# T184 T182/T183 Document Index Approval Packet

Date: 2026-06-03
Status: docs-only/default-deny approval packet. No document indexing has been run by this packet.

## Scope

This packet prepares a future exact-file document-index visibility repair for the newest T182/T183
Brain Harness gate documents:

- `docs/BRAIN_HARNESS_T182_T181_DIRECT_SEARCH_CURRENT_PLAN_APPROVAL_PACKET_2026-06-03.md`
- `docs/BRAIN_HARNESS_T183_POST_T182_COMPLETION_GATE_AUDIT_2026-06-03.md`

It intentionally does not include `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`, because that file is
already part of the separate pending T181 exact-file indexing gate. It also does not index T179 or
T180.

This packet does not run document indexing, change document-index behavior, implement T182 ranking,
send input to the live native Claude PTY, signal or kill PID `49349`, launch native Claude, run
Claude Bridge, edit hooks/settings/adapters, run harness install, mutate lifecycle state, run
M6/migration/quarantine actions, make candidate decisions, change ranking/`orient`, change public
MCP/schema/storage/index behavior, delete, roll back, reinstall binaries, or touch user-owned files.

## Current Evidence

- T182 committed the direct-search current-plan approval packet as `9bb51e6`.
- T183 committed the post-T182 completion/gate audit as `c75b509`.
- T183 captured active current-plan memory `019e8e5c-70be-7b70-b9f5-3a5a4115cd86`, which keeps
  exact T182 approval as the recommended next product-moving gate.
- Fresh document-only search for `T182 T181 Direct Search Current-Plan Approval Packet` returned
  older T174/T162/T176 documents instead of the T182 packet.
- Fresh document-only search for `T183 Post-T182 Completion And Gate Audit` returned older T161 and
  Live Feedback Batch documents instead of the T183 audit.
- Fresh document-only search for `c75b509 Record T183 completion gate audit` returned T176/T175 and
  older evidence documents instead of the T183 audit.
- Current files exist and have these sizes and SHA-256 hashes:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `docs/BRAIN_HARNESS_T182_T181_DIRECT_SEARCH_CURRENT_PLAN_APPROVAL_PACKET_2026-06-03.md` | 8576 | `dc320981135c0d672e2920773feeddee3b7e4ff369e9964981fd6ac7f7a94bad` |
| `docs/BRAIN_HARNESS_T183_POST_T182_COMPLETION_GATE_AUDIT_2026-06-03.md` | 9289 | `0300d8b8ec5d9de7f017be4fed047ddc6d467cc2b09007853718e98d7f27f449` |
- Git status is clean except pre-existing untracked root `AGENTS.md`.
- PID `49349` remains live as `/Users/yuval.meiri/.local/bin/claude`; this packet does not resolve
  that process.

## Research Question

Can Engram safely make the latest T182/T183 gate documents visible through document search without
changing retrieval code, expanding T181, or crossing any ranking/native-Claude/M6/lifecycle gate?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Indexing exactly T182 and T183 makes the latest direct-search ranking packet and completion/gate audit recoverable through document search while preserving all underlying gates. |
| Null | The files index successfully, but semantic search remains noisy; repo files and current-plan memory remain authoritative. |
| Simpler alternative | Defer indexing and require repo-file inspection for T182/T183 exact approval wording and matrix notes. |
| Failure | The operation expands into directory indexing, cleanup/reindex, T181 execution, T182 ranking, MemoryItem creation for packet docs, lifecycle mutation, native Claude input or cleanup, M6 work, schema/storage changes, or implied approval for any gated action. |

## Proposed Approved Scope

If the user approves this packet, Codex may perform only the following:

1. Read-only file-existence preflight for these exact paths:
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T182_T181_DIRECT_SEARCH_CURRENT_PLAN_APPROVAL_PACKET_2026-06-03.md`
   - `/Users/yuval.meiri/projects/engram/docs/BRAIN_HARNESS_T183_POST_T182_COMPLETION_GATE_AUDIT_2026-06-03.md`
2. Index exactly those two files through MCP `docs(action="index", path=...)`.
3. Run read-only validation searches:
   - `T182 T181 Direct Search Current-Plan Approval Packet`
   - `Approve T182: implement the narrow direct-search current-plan ranking fixture`
   - `T183 Post-T182 Completion And Gate Audit`
   - `c75b509 Record T183 completion gate audit`
   - `T183 audit committed exact T182 approval remains recommended next gate`
4. Record a Markdown result report and implementation-plan note if documentation changes are made.
5. Commit only intended documentation files.
6. Capture current-plan memory and submit telemetry feedback for assessable retrieval traces.

## Success Criteria

- T182 appears in the top five document results for its exact title or exact approval phrase.
- T183 appears in the top five document results for its exact title, commit probe, or current-plan
  wording probe.
- Every underlying approval gate remains unchanged:
  - T182 still requires exact approval before ranking/source/test changes.
  - T181 still requires exact approval before indexing T179, T180, and
    `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`.
  - T180 still requires exact approval before sending any input to the live native Claude PTY.
  - T174 still requires exact approval before read-only M6 candidate-decision/dry-run scoping.
  - Candidate decisions, migration dry-run/apply, deletion, lifecycle cleanup, native Claude
    prompt-bearing validation, harness writes, ranking/`orient`, public MCP, schema/storage/index,
    and document-index behavior changes remain separately gated.
- No MemoryItem is created for these document artifacts.
- No native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude, harness write,
  lifecycle archive, `lint apply_safe`, M6/migration/quarantine action, candidate decision,
  deletion, cleanup, schema/storage/index behavior change, document-index behavior change, public
  MCP change, ranking change, or `orient` change occurs.

## Stop Conditions

Stop and report without continuing if any of these occur:

- Approval is missing, conditional, abbreviated, or ambiguous.
- Either exact file is missing or is not a regular file.
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
Approve T184: index exact files T182 and T183.
```
