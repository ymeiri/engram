# T206 Document Source Metadata Search

Date: 2026-06-04
Status: source change validated
Scope: Repair exact document title and filename-stem retrieval for indexed document sources

## Research Question

Should Engram document search merge source title/path metadata matches with semantic chunk search so
known-item queries can retrieve an indexed document by exact title or filename stem?

## Hypotheses

- Preferred: a narrow metadata-only source match should repair exact title and filename-stem
  visibility without changing public MCP shape, `orient`, memory ranking, schema/storage
  definitions, or document indexing.
- Null: embedding-only chunk search is acceptable and the T205 caveat should remain documented.
- Simpler alternative: leave code unchanged and add only a completion-matrix caveat.
- Failure: lexical title/path matches could create broad ranking noise and swamp semantic results.

## Measurement

Before implementation, T205 showed:

- `docs(action="index")` successfully indexed T202 with one chunk and no warnings.
- `docs(search)` for `test_mcp_handoff_update_supersedes_previous_handoff` returned T202 first.
- `docs(search)` for exact title `T202 Handoff Supersession MCP Boundary Validation` did not
  return T202 in the top five.
- `docs(search)` for filename stem
  `BRAIN_HARNESS_T202_HANDOFF_SUPERSESSION_MCP_BOUNDARY_VALIDATION_2026-06-04` did not return T202
  in the top ten.

## Consultation

AI Council recall found prior Engram guidance favoring narrow retrieval-only calibration when live
evidence shows a bounded retrieval gap, while preserving default-deny gates and avoiding broad
ranking churn.

AI Council broadcast returned 3/3 support for implementing this slice with strict boundaries:
metadata-only matching over `DocSource.title` and `DocSource.path_or_url`, exact title/basename/stem
promotion, bounded strong substring matching with negative controls, deduplication, no schema or
public API change, and no `orient`/memory ranking change.

Claude Bridge isolated read-only critique timed out after 120 seconds, so it is recorded as a
consultation caveat rather than evidence.

## Implementation

The code now:

- adds `DocumentRepo::search_source_metadata(query, limit)`, which normalizes known-item queries and
  matches only source title, path, basename, and filename stem;
- assigns score `1.0` for exact normalized metadata matches and `0.84` for specific strong
  substring matches;
- rejects short generic substring queries below 12 normalized characters;
- returns one representative first chunk per matching source;
- merges lexical source hits with existing vector chunk hits for both direct `DocumentService` and
  unified `SearchService` document results;
- promotes an existing semantic hit for the same source instead of returning duplicate lexical and
  semantic rows;
- preserves the existing result limit.

No MCP request or response parameters changed.

## Validation

Passed:

- `cargo test -p engram-tests --test document_tests source_metadata_search`
- `cargo test -p engram-index document_search`
- `cargo fmt --all --check`
- `cargo test -p engram-tests --test document_tests`
- `cargo test -p engram-tests --test search_tests`
- `cargo check -p engram-cli`
- `git diff --check`

Focused tests prove:

- exact title `T202 Handoff Supersession MCP Boundary Validation` returns the target source first;
- exact filename stem
  `BRAIN_HARNESS_T202_HANDOFF_SUPERSESSION_MCP_BOUNDARY_VALIDATION_2026-06-04` returns the target
  source first;
- specific substring matching works for long title fragments;
- generic `Validation` does not trigger metadata promotion;
- lexical and semantic hits for the same source dedupe by promoting the existing semantic result;
- merged results respect the requested limit.

## Completion Matrix Delta

| Area | State After T206 | Remaining Risk |
| --- | --- | --- |
| T202 document title/stem visibility | Source behavior repaired and covered by deterministic tests | Installed runtime still needs refresh/live validation |
| Direct document search | Merges semantic chunks with source metadata matches | Live daemon still runs the pre-T206 binary until refreshed |
| Unified search document layer | Uses the same merge helper when document embedder is configured | Non-embedder unified-search mode still skips documents as before |
| Orient/memory ranking | Unchanged | None introduced |
| M6/migration | Unchanged | Candidate decisions, dry-run/apply evidence, rollback plan, and explicit migration completion/defer decision remain incomplete |

## Non-Actions

T206 did not:

- change public MCP request or response shape;
- change `orient`, Brain Loop payloads, memory ranking, or memory lifecycle state;
- change schema/storage definitions, document indexing behavior, embeddings, chunking, or cleanup;
- run `lint apply_safe`, migration review status/prioritize/apply, deletion, rollback, or M6 work;
- edit hooks, settings, adapters, runtime configuration, or user-owned files;
- run native Claude, Claude Bridge write actions, harness install, force-kill, or old-binary
  reinstall.
