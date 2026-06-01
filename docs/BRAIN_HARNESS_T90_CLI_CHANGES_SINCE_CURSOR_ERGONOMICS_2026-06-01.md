# Brain Harness T90 CLI Changes Since Cursor Ergonomics

Status: Implemented and locally validated
Date: 2026-06-01
Scope: Existing `engram memory changes-since` CLI cursor guidance

T89 fixed the MCP `changes_since` commit-id-only error, but source inspection showed the CLI path
still described `--timestamp` only as an RFC3339 value. That is technically correct but weaker than
the Brain Harness contract agents now follow: use `memory_cursor.timestamp` from `orient` or
`engram memory cursor`, with `memory_cursor.commit_id` as optional context.

T90 does not change cursor semantics, CLI option names, public MCP parameters, ranking,
`orient` payload shape, document indexing, migration state, lifecycle state,
schema/storage/index behavior, or harness hooks/adapters.

## Research Question

Can the CLI `changes-since` path carry the same cursor guidance as MCP without changing behavior or
expanding the hot path?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Update CLI help and invalid timestamp errors to point users at `memory_cursor.timestamp`, while keeping `--timestamp` required and `--commit-id` optional. |
| Null | CLI users already understand that `--timestamp` means the cursor timestamp. |
| Simpler alternative | Only document the CLI usage in `ORIENT_CONTRACT.md`. |
| Failure | The CLI wording implies `commit_id` can replace the timestamp or introduces new CLI/API semantics. |

## Measurement

Before implementation:

- `engram-cli/src/main.rs` described `changes-since --timestamp` as only
  `Cursor timestamp in RFC3339 format`.
- The CLI parsed the timestamp before constructing `MemoryCursor`.
- Invalid timestamp errors said only `Invalid RFC3339 timestamp: ...`.

After implementation:

- CLI help points `--timestamp` at `orient` `memory_cursor.timestamp` or
  `engram memory cursor`.
- CLI help says `--commit-id` comes from `memory_cursor.commit_id` and does not replace the
  required timestamp.
- Invalid timestamp errors now name `memory_cursor.timestamp`.
- Focused unit test:
  `cargo test -p engram-cli invalid_rfc3339_timestamp_error_names_cursor_timestamp`.

## Completion Matrix

| Area | Status | Evidence | Remaining risk |
| --- | --- | --- | --- |
| Cursor semantics | Preserved | CLI still constructs `MemoryCursor { commit_id, timestamp }` after parsing a required timestamp | None for this slice |
| CLI request shape | Unchanged | Existing `--timestamp` and `--commit-id` flags remain | CLI still requires the caller to preserve the timestamp |
| Runtime guidance | Improved | Invalid timestamp error names `memory_cursor.timestamp` | Missing `--timestamp` remains a Clap-required-argument error |
| Orient contract | Updated | `docs/ORIENT_CONTRACT.md` now includes the CLI usage shape | Generated harness text may still be less explicit |
| Gated surfaces | Untouched | No archive, migration, document indexing, ranking, `orient`, schema/storage, public MCP, or harness write | T69/T70/T88 remain exact-approval gated |

## Result

The CLI now matches the MCP guidance from T89: `timestamp` is the required freshness clock, and
`commit_id` is optional context. This reduces a continuity papercut for agents and humans using the
CLI path while preserving the existing interface.
