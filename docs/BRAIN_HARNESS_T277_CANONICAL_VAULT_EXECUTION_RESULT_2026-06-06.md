# Brain Harness T277 Canonical Vault Execution Result

Date: 2026-06-06
Status: canonical vault initialized, compiled, and postflight-validated.

## Scope

T277 executes the T275 canonical generated Memory OS vault path at:

```text
/Users/yuval.meiri/.engram/vault
```

The execution was authorized by the 2026-06-06 pasted `/goal` standing authorization, which
explicitly covers canonical vault Phase A/B/init/compile and says T275's previous approval gap is
now covered. The same prompt also directed the successor flow to record Snapshot A internally,
immediately collect Snapshot B, and continue through init/compile if Snapshot A/B match and path
checks pass.

T277 does not run M6 migration, mutate MemoryItem lifecycle state, run `lint apply_safe`, run
native Claude or bridge writes, edit harness files, publish branches, change ranking or `orient`,
change public MCP/schema/storage/index/document-index behavior, delete data, roll back, or touch
user-owned files.

## Research Question

Can Engram safely close the canonical generated vault init/compile gate by executing T275's
same-run Snapshot A/B protocol against the live canonical path?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | The current canonical path is absent/non-symlink, source counts are stable across Snapshot A/B, and `vault init` plus `vault compile` can generate the full projection with no skipped or user-owned files. | Supported. |
| Null | Source counts or path state drift between Snapshot A and Snapshot B, so the write must stop. | Rejected. |
| Simpler alternative | Keep only the T266 temp compileability result and defer durable canonical output. | Rejected because the current request explicitly authorizes T275 execution. |
| Failure | Partial output, skipped files, marker/frontmatter gaps, or bundled unrelated gate work makes the result unsafe to claim. | Avoided. |

## Snapshot A

Phase A used read-only source, path, and git checks.

| Field | Snapshot A |
| --- | --- |
| Canonical path | `/Users/yuval.meiri/.engram/vault` |
| Path state | Absent |
| Symlink state | Target is not a symlink; `/Users`, `/Users/yuval.meiri`, and `/Users/yuval.meiri/.engram` were not symlinks in the checked path. |
| Parent resolution | `/Users/yuval.meiri/.engram` resolved to `/Users/yuval.meiri/.engram`. |
| Parent owner/mode | `yuval.meiri:staff`, `drwxr-xr-x`, directory |
| Write ability | Parent was writable by the current user. |
| Disk/inode context | About 70,808,960 KiB available and 2% inode use on `/System/Volumes/Data`. |
| MemoryItems | 1,605 |
| KnowledgeCommits | 549 |
| Repositories | 9 |
| Entities | 32 |
| Projects | 79 |
| Expected generated files | 2,278 |
| Formula | `1605 + 549 + 9 + 32 + 79 + 4 = 2278` |
| Git status | Tracked worktree clean; only untracked root `AGENTS.md`. |
| Branch freshness | After `git fetch origin`, `origin/main` was still an ancestor of `HEAD`; `HEAD...origin/main` was `391 0`; no upstream was configured. |

Source inspection before execution confirmed that `write_memory_vault` treats the vault as a
generated projection, writes only generated files, and skips existing files without the Engram
generated marker. `vault(action="init")` and `vault(action="compile")` are direct wrappers around
`init_vault` and `export_vault`.

## Snapshot B

Snapshot B was collected immediately after Snapshot A and before the first canonical write.

Snapshot B matched Snapshot A exactly for path state, symlink state, parent resolution, git status,
and all source counts:

```text
MemoryItems=1605
KnowledgeCommits=549
Repositories=9
Entities=32
Projects=79
ExpectedGeneratedFiles=2278
```

No tool call occurred between Snapshot B and `vault(action="init")`.

## Execution

`vault(action="init", vault_path="/Users/yuval.meiri/.engram/vault")` created only the expected
seven directories:

```text
99_System
memory
memory/items
memory/commits
entities
projects
repositories
```

A post-init status check showed the source counts still matched Snapshot B, with zero files and no
user files. `vault(action="compile", vault_path="/Users/yuval.meiri/.engram/vault")` then wrote the
generated projection.

Compile output reported:

| Field | Value |
| --- | ---: |
| MemoryItems | 1,605 |
| KnowledgeCommits | 549 |
| Repositories | 9 |
| Entities | 32 |
| Projects | 79 |
| Files skipped | 0 |

## Postflight

Postflight `vault(action="status")` returned:

| Field | Value |
| --- | ---: |
| Exists | true |
| Initialized | true |
| Total files | 2,278 |
| Generated files | 2,278 |
| User files | 0 |
| MemoryItems | 1,605 |
| KnowledgeCommits | 549 |
| Repositories | 9 |
| Entities | 32 |
| Projects | 79 |
| Expected generated files | 2,278 |

Read-only scans found:

- `missing_marker_count=0`
- `missing_frontmatter_count=0`
- `99_System/Vault-Index.md` exists, has frontmatter, and has the generated marker
- the current T276 current-plan MemoryItem page exists, has frontmatter, and has the generated marker
- `projects/engram/index.md` exists, has frontmatter, and has the generated marker

`lint(action="run", vault_path="/Users/yuval.meiri/.engram/vault", write=false, limit=20)` returned
known global superseded-active lifecycle pressure and applied zero safe actions. It did not surface
a vault metadata finding in the returned set. `obligations(action="doctor")` returned no open
obligations and no warnings.

## Decision

The canonical generated Memory OS vault init/compile gate is closed for the current source snapshot.
The durable vault exists at `/Users/yuval.meiri/.engram/vault`, is initialized, contains exactly
2,278 generated Markdown files, has no user-owned vault files, and matches the Snapshot B source
counts.

This does not complete M6 migration, lifecycle cleanup, prompt-bearing native Claude validation,
effective-hook visibility, live Claude/Gemini host-label proof, branch publication, or the full
Brain Harness goal. Future vault update policy remains a separate operational concern.

## Validation

- Engram `orient(project="engram", intent="plan_work", response_shape="lean")`
- Direct Engram searches for T276 plan, architecture, implementation plan, T275/T276, M6,
  lifecycle, harness/host-label gates, and design preferences
- AI Council recall for T275 canonical vault execution
- Source reads of `engram-index/src/vault.rs`, `engram-index/src/memory.rs`,
  `engram-mcp/src/tools.rs`, and `engram-cli/src/main.rs`
- `git fetch origin`
- Snapshot A/B path, source-count, and git checks
- Canonical `vault init`
- Canonical `vault compile`
- Postflight `vault status`
- Marker/frontmatter scans
- `lint(action="run", vault_path="/Users/yuval.meiri/.engram/vault", write=false, limit=20)`
- `obligations(action="doctor")`
