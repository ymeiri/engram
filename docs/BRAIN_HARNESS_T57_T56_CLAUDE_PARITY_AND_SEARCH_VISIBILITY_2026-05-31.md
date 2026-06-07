# Brain Harness T57 T56 Claude Parity and Search Visibility

Status: Completed read-only cross-harness smoke; pass with broader-search caveat.
Date: 2026-05-31
Scope: Claude Code parity for post-T56 current-plan retrieval and broader implementation-plan search

This smoke did not run M6 inventory, review export, apply, deletion, lifecycle mutation, harness
writes, schema/storage/index changes, public MCP changes, ranking changes, or `orient` payload
changes.

## Research Question

After T56 current-plan capture, does Claude Code see the same current-plan and gated-next-work shape
as Codex, and does the broader implementation-plan query keep T56 visible even if older calibration
history ranks above it?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Claude Code surfaces T56 current-plan memory `019e7d74-51e9-7103-92b7-67104e6e22e9` first in lean `orient` and exact continuation `search`; for the broader implementation-plan query, T56 remains in the top three and any older item above it is background calibration/history rather than current guidance. |
| Null | Claude Code misses T56 or ranks older current-plan, migration, or calibration records as authoritative current guidance. |
| Simpler alternative | Treat Codex's post-T56 final orient and direct search as enough evidence and defer Claude parity until after a user-approved gated action. |
| Failure | The smoke creates lifecycle, migration, harness, schema, ranking, or `orient` changes, or is mistaken for approval to archive, replace, migrate, or repair anything. |

## Measurement

The smoke is a pass only if:

- Codex's lean `orient` returns T56 first for the continuation prompt.
- Codex's exact continuation direct `search` returns T56 first.
- Codex's broader implementation-plan direct `search` keeps T56 in the top three and treats any
  older item above it as non-current context.
- Claude Code can call the live Engram MCP read paths through Claude Bridge.
- Claude Code lean `orient` returns T56 first.
- Claude Code exact continuation direct `search` returns T56 first.
- Claude Code broader implementation-plan direct `search` keeps T56 in the top three and treats any
  older item above it as non-current context.
- Any synthetic obligations from the smoke are resolved or skipped with evidence.

## Codex Baseline

Codex lean `orient` trace `019e7d75-8458-7df1-9581-996188f71d27` returned:

- T56 current-plan memory `019e7d74-51e9-7103-92b7-67104e6e22e9` first;
- telemetry feedback weak-signal rule second;
- non-gated calibration limitation third;
- commit preference fourth;
- stale repository-scoped current-plan target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` fifth.

Codex exact continuation direct `search` trace `019e7d75-aff0-72a1-8dc9-9a473ff4da89`
returned T56 first with score `0.9494`.

Codex broader implementation-plan direct `search` trace
`019e7d75-b03f-7da0-922f-7a4c878ce3b7` returned:

- non-gated continuation search calibration `019e68d8-2a71-73c0-9bc6-8c589cb9e4f7`
  first with score `0.8944`;
- T56 current-plan memory second with score `0.8894`;
- older direct-search and mission-class current-plan calibration records below those.

The rank-1 calibration record is a historical project fact about a prior prompt-class repair, not
current work guidance.

## Claude Code Smoke

Claude Bridge ran a read-only personal-harness task with only `mcp__engram__orient`,
`mcp__engram__search`, and `mcp__engram__obligations` allowed. Claude reported:

- lean `orient` trace `019e7d76-62e1-7b73-901e-5f839bcec551`;
- exact continuation direct `search` trace `019e7d76-67ee-7a72-9bb1-e41a5489d7fe`;
- broader implementation-plan direct `search` trace `019e7d76-6d71-7d50-ab3d-1fcd32453cbd`;
- T56 current-plan memory at orient rank 1;
- T56 current-plan memory at exact-search rank 1 with score `0.9494`;
- non-gated continuation search calibration at broader-search rank 1 with score `0.8944`;
- T56 current-plan memory at broader-search rank 2 with score `0.8894`.

Codex then read all three telemetry records. The telemetry confirmed:

- Claude orient trace `019e7d76-62e1-7b73-901e-5f839bcec551` returned T56 in returned memory IDs
  and Claude reported it first in the lean Brain Loop ordering;
- Claude exact continuation search trace `019e7d76-67ee-7a72-9bb1-e41a5489d7fe` returned T56
  first among memory IDs;
- Claude broader implementation-plan search trace `019e7d76-6d71-7d50-ab3d-1fcd32453cbd`
  returned the same calibration-first, T56-second shape observed in Codex.

## Obligation Cleanup

The Claude smoke created two prompt-derived obligations:

- `019e7d76-3805-7402-a7f5-921f289b3e30`, design-context reading;
- `019e7d76-3805-7402-a7f5-920eedc9d48d`, source reading.

Codex resolved the design-context obligation after reading the governing docs, `ORIENT_CONTRACT.md`,
and the latest T56 evidence. Codex skipped the source-reading obligation because T57 is a read-only
retrieval parity and documentation slice with no source edits or new source-level behavior claims.

## Verdict

Pass for the post-T56 continuation prompt class, with a documented broader-search caveat.

Claude Code reproduced the current-plan shape: T56 was first in lean `orient` and exact continuation
direct `search`. The broader implementation-plan query kept T56 visible at rank 2 in both Codex and
Claude Code, behind a historical non-gated calibration fact. That caveat does not justify broad
ranking churn by itself because the current plan remains top-three and the older rank-1 item is not
actionable current guidance.

## Next Action

The approval gates remain unchanged. M6 inventory/review-export, lifecycle
archive/replacement/scope-correction, and harness adapter/settings/hook writes still require
separate explicit user approval before execution. Further work should stay read-only or
documentation-only unless the user explicitly approves one of those gated paths.
