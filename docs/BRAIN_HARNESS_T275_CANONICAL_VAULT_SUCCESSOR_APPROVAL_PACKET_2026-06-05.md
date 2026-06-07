# Brain Harness T275 Canonical Vault Successor Approval Packet

Date: 2026-06-05
Status: docs-only/default-deny successor packet. Not executed.

## Scope

T275 supersedes T267 only as the future execution packet shape for the canonical generated Memory
OS vault at:

```text
/Users/yuval.meiri/.engram/vault
```

T267 remains historical evidence for the T266 fixed-count baseline. T272 showed that T267 is not
executable under current counts, and T275 avoids creating another fixed-count packet that will
drift through normal Memory OS writes.

T275 does not initialize or compile the canonical vault, create directories, write vault files,
delete or clean up files, mutate Memory OS lifecycle state, run `lint apply_safe`, run
M6/migration/quarantine actions, run native Claude or bridge writes, edit harness files, publish
branches, change ranking or `orient`, change public MCP/schema/storage/index/document-index
behavior, roll back, force-kill, use elevated privileges, or touch user-owned files.

This repo slice updates only:

- `docs/BRAIN_HARNESS_ARCHITECTURE.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- this packet

## Research Question

What successor approval packet can safely replace T267's stale fixed-count canonical vault gate
without executing durable vault writes now?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | A two-phase same-run snapshot-and-lock packet is safer than another fixed-count packet because it lets future execution bind to live counts the user has just seen. | Supported. |
| Null | A refreshed fixed-count packet using today's counts is sufficient. | Rejected because counts already drifted from T266 to T272 to T275 through ordinary current-plan/handoff writes. |
| Simpler alternative | Leave T272 as the only successor guidance and do not prepare a packet. | Rejected because T272 names the need for a fresh successor packet before canonical execution. |
| Failure | T275 is mistaken for vault execution approval, permits cleanup/deletion after partial failure, or weakens M6/lifecycle/native/harness/schema/ranking gates. | Avoided. |

## Current Read-Only Evidence

T266 compiled only the temp vault at `/private/tmp/engram-t266-vault-smoke-20260605` and left the
canonical path absent. Its point-in-time counts were:

| Source count | T266/T267 baseline |
| --- | ---: |
| MemoryItems | 1,585 |
| KnowledgeCommits | 536 |
| Repositories | 9 |
| Entities | 32 |
| Projects | 79 |
| Expected generated files | 2,245 |

T272 then recorded live drift to 1,591 MemoryItems, 542 KnowledgeCommits, and 2,257 expected files,
making T267 non-executable by its own hard stop.

Fresh T275 read-only `vault(action="status", vault_path="/Users/yuval.meiri/.engram/vault")`
returned:

| Source count | Fresh T275 status |
| --- | ---: |
| MemoryItems | 1,599 |
| KnowledgeCommits | 546 |
| Repositories | 9 |
| Entities | 32 |
| Projects | 79 |
| Expected generated files | 2,269 |

The expected generated file count is derived from the current source counts:

```text
expected = memory_items + knowledge_commits + repositories + entities + projects + 4
```

Fresh canonical status also returned:

- `exists=false`
- `initialized=false`
- `total_file_count=0`
- `generated_file_count=0`
- `user_file_count=0`
- missing expected directories: `99_System`, `memory`, `memory/items`, `memory/commits`,
  `entities`, `projects`, and `repositories`.

Read-only OS path checks showed:

- `/Users/yuval.meiri/.engram` exists and is a directory owned by `yuval.meiri`.
- `/Users/yuval.meiri/.engram/vault` does not exist.
- `test -L /Users/yuval.meiri/.engram/vault` exited non-zero because the target is absent, not a
  symlink.
- `df -k /Users/yuval.meiri/.engram` reported about 86 GB available and low inode usage.
- A shell `test -w /Users/yuval.meiri/.engram` probe exited non-zero under the current managed
  sandbox; treat this as an execution-environment caveat, not proof that the future MCP/approved
  execution cannot write. Future execution must re-check write ability in its own approved
  execution context and stop before user approval if it cannot write without escalation beyond the
  packet.

Source inspection confirmed:

- `engram-index/src/vault.rs` treats the vault as a generated projection and skips existing files
  that lack the Engram generated marker.
- `write_memory_vault` calls `fs::create_dir_all(root)`, writes generated files, and records
  `files_written` and `files_skipped`.
- `inspect_memory_vault` is read-only and reports live source counts plus
  `expected_generated_file_count`.
- `MemoryService::export_vault` and `vault_status` both read current memory, commit, and
  repository snapshots at call time.
- The MCP `vault(action="init"|"compile"|"status"|"page")` wrapper does not enforce absent,
  empty, non-symlink, parent-resolution, or same-run count-lock checks; those checks must be part
  of the future execution protocol.

## Consultation

AI Council recall found the T266/T267/T272 vault guidance: temp compileability is useful evidence,
the canonical vault is a separate durable-write gate, T267 used fixed source-count parity, and T272
warned not to create another fixed-count packet unless execution is imminent.

AI Council broadcast to `claude-sonnet-4.6`, `gpt-5.4`, and `gemini-3.1-pro` agreed that a
snapshot-and-lock packet is safer than another fixed-count packet. The Council recommended:

- present a live Snapshot A to the user and require exact approval of those counts;
- capture Snapshot B immediately after approval and hard-stop on any drift;
- gate all count dimensions: MemoryItems, KnowledgeCommits, repositories, entities, projects, and
  expected generated files;
- define empty as zero entries, including hidden files;
- verify target and parent symlink/path behavior;
- distinguish source drift, target conflict, and tool/runtime failure;
- do not clean up partial output without separate approval.

Claude Bridge read-only isolated critique agreed with the direction and added two stricter points:

- Snapshot B must be the final read-only action before the first canonical vault write, with no
  intervening tool calls.
- Any partial init/compile state must be reported as quarantined pending explicit cleanup approval;
  do not retry, delete, or repair in the same packet.

Model agreement is blind-spot evidence only. The packet below follows from source inspection,
fresh status, and the T272 count-drift failure.

## Successor Protocol

T275 is a two-phase gate. A future agent may not execute the canonical vault write from this
packet alone.

### Phase A: Read-Only Preflight

The future agent may perform read-only preflight only after the user explicitly asks to execute
T275 or asks to prepare the live T275 snapshot. Phase A must not write files or Memory OS state.

Phase A must collect and present Snapshot A:

| Snapshot A field | Requirement |
| --- | --- |
| Canonical path | Exactly `/Users/yuval.meiri/.engram/vault`. |
| Path state | Absent, or an existing directory with zero entries including hidden files. |
| Symlink state | Target is not a symlink; parent path resolution does not escape the approved user-space path. |
| File type | Target is not a regular file, device, socket, mount surprise, or non-directory object. |
| Write ability | Future execution context can create/write the path without `sudo`, chmod/chown, ownership changes, or broader escalation. |
| Disk/inode context | Sufficient space and inodes for the expected generated projection. |
| Source counts | MemoryItems, KnowledgeCommits, repositories, entities, projects, and expected generated files. |
| Formula | `expected = memory_items + knowledge_commits + repositories + entities + projects + 4`. |
| Git status | Tracked worktree clean; only known user-owned untracked root `AGENTS.md` may be present. |
| Boundary | No M6, lifecycle, native-Claude, bridge-write, harness, schema/storage/index/public-MCP/document-index, ranking/`orient`, branch-publication, deletion, cleanup, rollback, or user-owned-file work is bundled. |

If any Phase A check fails, stop and write a read-only result report only if that report is in the
repo worktree and does not require cleanup.

### Phase B: Exact Snapshot Approval

After presenting Snapshot A, the future agent must stop for exact user approval that includes:

- the canonical path;
- the Snapshot A source counts and expected generated file count;
- authorization to run only the T275 canonical `vault init` and `vault compile` sequence against
  that snapshot if Snapshot B matches;
- acceptance that any partial output will not be cleaned up by this packet.

Generic `go ahead`, `approved`, or broad continuation wording is not enough. Approval must be
given after Snapshot A is shown.

### Phase C: Snapshot Lock And Execution

Immediately after exact Snapshot A approval, the future agent must collect Snapshot B with the same
path and source-count fields.

Hard-stop before any canonical write if Snapshot B differs from Snapshot A in any field, or if any
tool call, Memory OS write, repo write, branch action, or unrelated read/write action occurs
between Snapshot B and the first vault write.

Authorized writes after a matching Snapshot B are limited to:

```text
vault(action="init", vault_path="/Users/yuval.meiri/.engram/vault")
vault(action="compile", vault_path="/Users/yuval.meiri/.engram/vault")
```

After `init`, the future agent should re-check source counts before `compile` if that can be done
without an intervening source write. If counts drift after init but before compile, stop and report
the skeleton-only partial state; do not clean it up.

## Postflight Requirements

Postflight is read-only except for the required repo result report, exact docs indexing,
current-plan capture, and rolling handoff update after the canonical write has completed or failed.

Postflight must report:

- `vault(status)` for the canonical path;
- `files_written` and `files_skipped` from compile;
- generated file count equals Snapshot B expected generated file count;
- user file count is zero;
- source counts still match Snapshot B, or a clear source-drift-after-compile warning;
- vault index, one current-plan MemoryItem page, and `projects/engram/index.md` are present,
  generated, and include frontmatter plus `<!-- engram:generated:file memory-vault-v1 -->`;
- direct marker/frontmatter scan has no generated file missing marker/frontmatter;
- no M6, lifecycle, native-Claude, bridge-write, harness, schema/storage/index/public-MCP/
  document-index, ranking/`orient`, branch-publication, deletion, cleanup, rollback, or
  user-owned-file action occurred.

If postflight fails, do not retry, delete, clean up, repair, or overwrite. Report the canonical
vault as partial/quarantined pending separate explicit cleanup or recovery approval.

## Positive Allowlist

Future T275 execution allows only:

1. Read this packet and related T266/T267/T272 docs.
2. Read-only Engram orient/search/status checks.
3. Read-only OS path, symlink, emptiness, disk/inode, and git-status checks.
4. Present Snapshot A and wait for exact approval.
5. Re-read Snapshot B immediately after approval.
6. Run canonical `vault init` and `vault compile` only if Snapshot A and B match.
7. Run read-only postflight vault status/page/marker/count checks.
8. Write and commit a T275 execution result report plus implementation-plan note.
9. Index exact changed docs, run obligations doctor, capture current-plan memory, update rolling
   handoff, and submit telemetry feedback.

Everything else is forbidden by default.

## Explicitly Forbidden

T275 does not authorize future execution to:

- write to any path other than `/Users/yuval.meiri/.engram/vault`;
- proceed if the target exists with any entry, including hidden files;
- proceed if the target or unexpected parent path component is a symlink or resolves outside the
  approved path;
- delete, move, rename, clean up, or repair any path before or after execution;
- use `sudo`, chmod/chown, ownership changes, or elevated privileges;
- accept drift between Snapshot A and Snapshot B;
- make any tool call between Snapshot B and the first vault write;
- retry after partial failure or postflight mismatch;
- run M6 migration inventory/export/status/prioritize/apply or quarantine actions;
- edit generated M6 review pages or infer candidate dispositions;
- archive, supersede, reject, review, or delete MemoryItems, or run `lint apply_safe`;
- change ranking/`orient`, public MCP contracts, schema/storage/index behavior, or
  document-index behavior;
- run native Claude, Claude Bridge writes, prompt-bearing Claude, `/hooks`, harness install,
  settings edits, hook edits, adapter edits, runtime refresh, old-binary reinstall, or
  `adopt_user_owned`;
- push, set upstream, create a PR, pull, merge, rebase, or perform remote publication;
- edit or stage root `AGENTS.md` or other user-owned files.

## Approval Wording

T275 intentionally does not provide a one-shot write approval phrase because live counts must be
shown first. To start the future read-only preflight, the user may say:

```text
Approve T275 Phase A: run only the read-only canonical vault preflight from docs/BRAIN_HARNESS_T275_CANONICAL_VAULT_SUCCESSOR_APPROVAL_PACKET_2026-06-05.md and present Snapshot A for /Users/yuval.meiri/.engram/vault. Do not initialize or compile the vault, write files, mutate Memory OS state, run M6/lifecycle/native-Claude/harness/schema/ranking/branch actions, delete, clean up, or touch user-owned files.
```

After Snapshot A is presented, the future write approval must name the exact Snapshot A counts and
path. Any shorter approval is non-authorization for Phase C.

## Validation For This Packet

Validation for this docs-only packet:

- lean `orient` trace `019e992d-1295-75f1-8351-933724d10eee`;
- direct vault successor search trace `019e992d-2292-7531-895b-8cc1f78f627d`;
- reads of T266, T267, and T272 vault reports;
- read-only `vault(action="status", vault_path="/Users/yuval.meiri/.engram/vault")`;
- read-only OS checks for canonical path absence, parent directory type, symlink state, disk/inode
  context, and the current sandbox write-probe caveat;
- source reads of `engram-index/src/vault.rs`, `engram-index/src/memory.rs`,
  `engram-mcp/src/tools.rs`, and `engram-cli/src/main.rs`;
- AI Council recall plus three-model broadcast;
- Claude Bridge isolated read-only critique;
- AI Council stored synthesis insight for future recall.
- `git diff --check` before final validation returned clean output;
- obligation detection dry-run identified only expected document-disposition, source-reading, and
  commit-preference obligations for this slice, and `obligations(action="doctor")` returned no open
  obligations or warnings before staging;
- exact `docs(action="index")` for this packet, the architecture doc, and the implementation plan
  created 17, 126, and 437 chunks respectively, with no warnings;
- document search for `T275 canonical vault successor Snapshot A Snapshot B exact approval
  canonical vault path counts` returned this packet as the top result, with the exact Snapshot
  approval chunk first;
- unified search trace `019e9934-730c-7cc1-b2c9-fcd766db27df` returned this packet's document
  chunks; it also showed the T274 handoff/current-plan memory still ahead of the new T275 document
  before the post-commit current-plan and handoff update.

## Completion Criteria For Future Execution

Successful future T275 execution closes only the canonical vault init/compile gate if:

- Snapshot A and Snapshot B match exactly;
- the canonical path preflight passes;
- canonical `vault init` and `vault compile` run without skipped files or errors;
- generated file count equals Snapshot B expected generated files;
- sampled pages and marker/frontmatter checks pass;
- a result report and implementation-plan note are committed;
- current-plan memory and rolling handoff are updated;
- all other gates remain unchanged.

It does not complete M6 migration, lifecycle cleanup, native Claude parity, effective-hook
visibility, live Claude/Gemini host-label adoption, branch publication, or the full Brain Harness
goal.
