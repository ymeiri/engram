# Brain Harness T267 Canonical Vault Approval Packet

Date: 2026-06-05
Status: docs-only/default-deny approval packet. Not executed.

## Scope

This packet defines the future gate for initializing and compiling the canonical generated Memory OS
Markdown vault at:

```text
/Users/yuval.meiri/.engram/vault
```

This packet does not execute that gate. It does not create the canonical vault, compile into the
canonical vault, clean up the temp vault, mutate Memory OS data, run M6 migration or quarantine
actions, edit generated review pages, mutate lifecycle state, archive memory, delete data, change
ranking or `orient`, change public MCP/schema/storage/index/document-index behavior, run native
Claude, edit harness files, push branches, or touch user-owned files.

## Current Evidence

- T266 initialized and compiled only:
  `/private/tmp/engram-t266-vault-smoke-20260605`.
- T266 temp status after compile was:
  - `total_file_count=2245`
  - `generated_file_count=2245`
  - `user_file_count=0`
  - `memory_item_count=1585`
  - `knowledge_commit_count=536`
  - `repository_count=9`
  - `entity_count=32`
  - `project_count=79`
  - `expected_generated_file_count=2245`
- T266 sampled the vault index, the T265 current-plan MemoryItem page, and the `engram` project
  page. All were generated and contained frontmatter plus the Engram generated marker.
- T266 direct scans found no temp vault file missing the generated marker or frontmatter.
- Canonical `/Users/yuval.meiri/.engram/vault` status before and after T266 remained:
  - `exists=false`
  - `initialized=false`
  - `total_file_count=0`
  - `generated_file_count=0`
  - `user_file_count=0`
  - `expected_generated_file_count=2245`

## Research Question

What exact future approval is required before Engram may convert T266 temp compileability evidence
into a durable canonical vault initialization and compile?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A docs-only/default-deny packet can safely define a one-time canonical vault init+compile gate with exact path, exact source-count parity, hard stops, and narrow completion wording. |
| Null | A canonical vault gate is still too ambiguous because durable vault path/update policy is not specified. |
| Simpler alternative | Keep the canonical vault gate unresolved and ask the user informally for approval. |
| Failure | The packet is mistaken for execution approval, permits broad durable writes, overwrites user files, treats count drift as acceptable, or implies M6/lifecycle/native-Claude/remote completion. |

## Consultation

AI Council recall found the T266 synthesis: temp-path vault compile is useful compileability
evidence only, while the canonical user-facing vault remains a separate durable path/update-policy
decision. A fresh three-model broadcast for T267 agreed that this packet is the right next
non-destructive slice and emphasized:

- exact canonical path approval;
- path absent-or-empty and no-symlink checks;
- source-count parity with T266 before writing;
- output count parity after compile;
- no adjacent M6, lifecycle, harness, schema, public API, ranking, deletion, or remote work;
- no overclaiming beyond canonical vault initialization/compile if later executed.

Claude Bridge read-only critique timed out after 120 seconds and is recorded as a consultation
confound, not supporting evidence.

## Recommended Future Approval

The future execution requires explicit approval with the canonical path, the durable-write nature,
and the T266 baseline. Short approvals such as `go ahead`, `approved`, generic continuation, or
approval that omits the canonical path are not enough.

```text
Approve T267: execute the canonical Engram vault initialization and compile gate from docs/BRAIN_HARNESS_T267_CANONICAL_VAULT_APPROVAL_PACKET_2026-06-05.md. I authorize creating /Users/yuval.meiri/.engram/vault and durably writing the generated vault projection there only if every documented preflight check passes: the target path is absent or an empty non-symlink directory, source counts match the T266 baseline of 1585 MemoryItems, 536 KnowledgeCommits, 9 repositories, 32 entities, and 79 projects, expected generated output remains 2245 files, no elevated privileges are required, and no unexpected tracked worktree changes exist. Run only vault init and compile for that canonical path, one postflight status/page/marker validation, and write a T267 result report plus implementation-plan note. Do not run M6/migration/quarantine actions, lifecycle archive/apply_safe, deletion, cleanup, schema/storage/index/document-index/public MCP/ranking/orient changes, native Claude, Claude Bridge writes, harness install/settings/hooks/adapters, remote publication, rollback, or user-owned-file edits.
```

If the user wants refreshed source counts instead of exact T266 baseline counts, that is a separate
approval shape and must explicitly say that count drift is allowed and why.

## If Approved: Authorized Operations

The future execution may:

1. Re-read this packet and the T266 report.
2. Verify tracked git status is clean except the known untracked root `AGENTS.md`.
3. Run read-only status for `/Users/yuval.meiri/.engram/vault`.
4. Verify the canonical path is either absent or an empty, non-symlink directory.
5. Verify the live source counts match the T266 baseline:
   - 1585 MemoryItems
   - 536 KnowledgeCommits
   - 9 repositories
   - 32 entities
   - 79 projects
   - 2245 expected generated files
6. Run `vault(action="init", vault_path="/Users/yuval.meiri/.engram/vault")`.
7. Run `vault(action="compile", vault_path="/Users/yuval.meiri/.engram/vault")`.
8. Run one postflight read-only vault status check for the canonical path.
9. Sample the vault index, one recent current-plan MemoryItem page, and the `engram` project page.
10. Run direct marker/frontmatter scans over the canonical vault.
11. Write a T267 execution result report and update `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`.
12. Index exact changed docs, run obligations doctor, commit only intended repo docs, capture
    current-plan memory, and submit telemetry feedback.

## Explicitly Forbidden

T267 does not authorize the future execution to:

- write to any path other than `/Users/yuval.meiri/.engram/vault`;
- initialize or compile if the canonical path is a symlink, resolves unexpectedly, or contains
  existing files/directories;
- use `sudo`, elevated privileges, `rm`, cleanup, rollback, force-kill, or deletion;
- accept source-count drift from the T266 baseline unless a separate approval explicitly allows it;
- run M6 migration inventory/export/status/prioritize/apply or quarantine actions;
- edit generated M6 review pages or infer candidate dispositions;
- run lifecycle archive, lifecycle cleanup, or `lint(action="apply_safe")`;
- mutate active MemoryItems except for the normal post-execution current-plan capture after repo
  documentation is committed;
- change ranking/`orient`, public MCP contracts, schema/storage/index behavior, or document-index
  behavior;
- run native Claude, Claude Bridge writes, harness install, settings edits, hook edits, adapter
  edits, runtime refresh, old-binary reinstall, or `adopt_user_owned`;
- push, set upstream, create a PR, or perform remote publication;
- edit or stage root `AGENTS.md` or other user-owned files.

## Hard Stops

Stop before any canonical write if:

- exact T267 approval is missing, shortened, ambiguous, or bundled with another gate;
- git status has unexpected tracked changes;
- `/Users/yuval.meiri/.engram/vault` exists and is not an empty directory;
- `/Users/yuval.meiri/.engram/vault` is a symlink or resolves outside the approved canonical path;
- any preflight source count differs from the T266 baseline;
- expected generated file count differs from 2245;
- the operation requires elevated privileges, deletion, cleanup, schema migration, source-data
  mutation, or network/remote work;
- any tool proposes or attempts M6, lifecycle, harness, native-Claude, public MCP, ranking, or
  schema/storage/index/document-index behavior changes.

Stop after the canonical write and report without cleanup if:

- postflight `generated_file_count` differs from `expected_generated_file_count`;
- any generated file is missing the Engram generated marker or frontmatter;
- sampled pages are missing or not marked generated;
- canonical status reports user-owned files;
- any unexpected Memory OS write, lifecycle mutation, M6 state change, git change, or user-owned
  file change appears.

## Measurements For Future Execution

| Measurement | Required Output |
| --- | --- |
| Preflight canonical path state | `exists`, `initialized`, file counts, symlink/path-resolution result. |
| Source-count parity | Live counts versus the T266 baseline for MemoryItems, KnowledgeCommits, repositories, entities, projects, and expected files. |
| Files written | `files_written` and `files_skipped` from the canonical compile. |
| Postflight canonical status | `total_file_count`, `generated_file_count`, `user_file_count`, expected count, initialized state. |
| Page sampling | Vault index, current-plan MemoryItem page, and `engram` project page found/generated with frontmatter and marker. |
| Marker/frontmatter scan | Counts or empty output proving no generated files are missing marker/frontmatter. |
| Scope preservation | Explicit statement that M6, lifecycle, native-Claude, remote publication, schema/public API/ranking/index behavior, deletion, and user-owned files were untouched. |

## Completion Criteria For Future Execution

The future T267 execution can be marked complete only if it creates the canonical vault, compiles
exactly the approved generated projection, validates postflight counts and sampled pages, writes and
commits a result report plus implementation-plan note, captures current-plan memory, and leaves all
other Brain Harness gates unchanged.

Successful future execution would close only the canonical vault init/compile gate. It would not
complete M6 migration, lifecycle cleanup, native Claude parity, live Claude/Gemini host-label
adoption, remote publication, or the full Brain Harness goal.
