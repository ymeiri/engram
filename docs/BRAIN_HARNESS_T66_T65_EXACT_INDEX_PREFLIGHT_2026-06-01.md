# Brain Harness T66 T65 Exact Index Preflight

Status: Completed source-only preflight. No document indexing has been run.
Date: 2026-06-01
Scope: Verify whether the pending T65 exact-file indexing request is executable without broadening
the approval packet.

## Research Question

Can the existing document-index surfaces target exactly the three files named in T65 without
indexing the full `docs/` directory or changing document-index behavior?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Existing MCP `docs(action="index", path=...)` and CLI `engram index <PATH>` surfaces can index one file at a time, so T65 can be executed as three exact file-path calls if approved. |
| Null | The public surfaces only index directories or otherwise require broader document-index changes, so T65 needs a revised approval packet before execution. |
| Simpler alternative | Continue manual repo-file inspection for T59 and defer document-index repair. |
| Failure | A preflight accidentally runs indexing, M6 review export/apply, lifecycle writes, schema/storage changes, ranking changes, `orient` changes, or harness writes. |

## Measurement

This slice used source and help-text inspection only:

- Confirm whether the MCP document handler has a file branch separate from a directory branch.
- Confirm whether the CLI exposes a file-path index command and a read-only plan mode.
- Confirm whether directory indexing has broader recursive behavior that should remain excluded
  from the T65 approval.
- Confirm whether re-indexing an existing source preserves source identity and replaces chunks
  rather than creating a second source.

No `docs(action="index")`, `docs(action="plan")`, `engram index <target>`, M6 migration command,
or lifecycle command was run.

## Source Findings

- MCP `docs(action="index")` requires a `path` and branches on `path.is_dir()` vs `path.is_file()`.
  File paths call `service.index_file(path)` and return `documents_indexed=1`; directory paths call
  `service.index_directory(path)`.
  Evidence: `engram-mcp/src/tools.rs:10377`.
- MCP `docs(action="plan")` is read-only and calls `service.plan_path(...)`, but T65 execution does
  not need a separate plan call if approval is limited to exact file indexing plus validation.
  Evidence: `engram-mcp/src/tools.rs:10406`.
- CLI `engram index <PATH>` accepts one path and supports `--plan`, but the command handler ignores
  the declared `recursive` flag and uses `PipelineConfig::default()` for plan mode. Directory paths
  therefore stay broader than T65 should allow.
  Evidence: `engram-cli/src/main.rs:137`, `engram-cli/src/main.rs:5710`.
- The default document pipeline indexes only `md` and `markdown` extensions and has
  `recursive=true`, which is another reason T65 should avoid directory paths.
  Evidence: `engram-index/src/pipeline.rs:90`, `engram-index/src/pipeline.rs:230`.
- `DocumentService::index_file` looks up an existing source by the same displayed path and reuses
  that source identity when present.
  Evidence: `engram-index/src/service.rs:77`.
- Saving chunks replaces existing chunks for the source document before inserting the new chunks.
  Evidence: `engram-store/src/repos/document.rs:286`.

## Completion Matrix

| Item | Status | Evidence | Risk or gate |
| --- | --- | --- | --- |
| Exact file targetability | Implemented | MCP and CLI have file branches that call `index_file` for file paths. | T65 approval is still required before running index writes. |
| Directory exclusion | Validated as necessary | Directory branches call `index_directory`; default pipeline recursion is true. | Do not pass `docs/` or any directory path for T65. |
| Dry-run/plan visibility | Implemented | MCP `plan` and CLI `--plan` exist. | Not needed before approval; do not use target-file preflight as a substitute for approval. |
| Re-index behavior | Partially validated from source | Existing source identity is reused and chunks are replaced. | This is source evidence only; post-approval validation should still check retrieval results. |
| M6 safety boundary | Preserved | No M6 command or document-index write was run. | Review export/apply/deletion remain separately gated. |

## Decision

The T65 approval packet is executable as written if the user approves it, with one operational
clarification: use three exact file-path MCP `docs(action="index", path=...)` calls against the
running daemon, not a directory path and not the CLI write path. The CLI write path can target a
file, but it opens the default RocksDB store directly and is more likely to conflict with the live
daemon lock in this workspace.

This preflight does not approve or run T65. It only reduces the ambiguity in the pending approval
packet.

## Next Gate

Ask the user to approve or reject T65 exact-file document indexing. If approved, execute only the
three file paths named in T65, then run read-only retrieval validation and record the result.
