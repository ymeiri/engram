# Brain Harness T53 T52 Claude Parity

Status: Completed read-only cross-harness smoke; pass with expected stale-target noise.
Date: 2026-05-31
Scope: Claude Code parity for post-T52 current-plan retrieval

This smoke did not run M6 inventory, review export, apply, deletion, lifecycle mutation, harness
writes, schema/storage/index changes, public MCP changes, ranking changes, or `orient` payload
changes.

## Research Question

After T52 current-plan capture, does Claude Code see the same current-plan and pending-decision
shape that Codex sees for the current Brain Harness continuation prompt?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Claude Code surfaces T52 current-plan memory `019e7d5d-c450-7171-9fdb-8d1a5e745b0b` first in lean `orient` and direct `search`, while treating stale repo current-plan target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` as pending-decision evidence only. |
| Null | Claude Code misses T52 or ranks the stale repo current-plan target as authoritative current guidance. |
| Simpler alternative | Treat Codex's post-T52 sanity orient as enough evidence and defer Claude parity until after a user-approved gated action. |
| Failure | The parity smoke creates lifecycle, migration, harness, schema, ranking, or `orient` changes, or is mistaken for approval to archive or replace memory. |

## Measurement

The smoke is a pass only if:

- Codex's lean `orient` and direct `search` return T52 first for the continuation prompt class.
- Claude Code can call the live Engram MCP read paths through Claude Bridge.
- Claude Code lean `orient` returns T52 first.
- Claude Code direct `search` returns T52 first.
- Stale repo current-plan target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` may appear, but is
  treated only as stale or pending-decision evidence.
- Any synthetic obligations from the smoke are resolved or skipped with evidence.

## Codex Baseline

Codex lean `orient` trace `019e7d5f-3fb4-7430-ab79-320a0e938156` returned:

- T52 current-plan memory `019e7d5d-c450-7171-9fdb-8d1a5e745b0b` first;
- read-only M6 inventory approval limit second;
- commit preference third;
- harness-write gate fourth;
- stale repo current-plan target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` fifth.

Codex direct `search` trace `019e7d5f-a55f-7e61-9d71-286093777d46` returned:

- T52 current-plan memory first;
- stale repo current-plan target second;
- current-plan retrieval calibration and lifecycle predicate memories below those.

Codex also read the current matrix, research method, architecture checkpoint, `ORIENT_CONTRACT.md`,
T52 resolution request, and T50 Claude parity report before selecting this read-only slice.

## Claude Code Smoke

Claude Bridge ran a read-only personal-harness task with only `mcp__engram__orient`,
`mcp__engram__search`, and `mcp__engram__obligations` allowed. Claude reported:

- lean `orient` trace `019e7d60-64af-76d3-948f-5dd6068aa3d8`;
- direct `search` trace `019e7d60-67e9-71d0-a421-f3364d4a5131`;
- T52 current-plan memory `019e7d5d-c450-7171-9fdb-8d1a5e745b0b` at orient rank 1;
- stale repo current-plan target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` at orient rank 5;
- T52 at direct-search rank 1 with score `0.9494`;
- stale repo current-plan target at direct-search rank 2 with score `0.9269`.

Codex then read both telemetry records. The telemetry confirmed:

- Claude orient trace `019e7d60-64af-76d3-948f-5dd6068aa3d8` returned T52 and stale target among
  returned memory IDs, with T52 first in Claude's reported lean Brain Loop ordering;
- Claude search trace `019e7d60-67e9-71d0-a421-f3364d4a5131` returned T52 first and stale target
  second among memory IDs.

## Obligation Cleanup

The Claude smoke created two prompt-derived obligations:

- `019e7d60-32ff-7e61-9342-d1d76c2c0cfc`, design-context reading;
- `019e7d60-32ff-7e61-9342-d1c7ca1e83fa`, source reading.

Codex resolved the design-context obligation after reading the governing docs and skipped the
source-reading obligation because T53 is a read-only retrieval parity and documentation slice with
no source edits or new source-level behavior claims. `obligations(action="doctor")` then returned
no open obligations and no warnings.

## Verdict

Pass for this narrow post-T52 continuation prompt class.

Claude Code reproduced the post-T52 retrieval shape: T52 was first in lean `orient` and direct
`search`. The stale repository-scoped current-plan memory still appears below T52, including second
in direct search, but it is expected stale/pending-decision evidence under T52 rather than
authoritative current guidance.

## Next Action

The approval gates remain unchanged. T52 still requires the user to choose archive-only,
replacement-then-archive, or scope-correction/merge before any lifecycle write. T45 M6 inventory and
T47 harness repair writes still require separate explicit approval before execution. Further work
should stay read-only or documentation-only unless the user explicitly approves one of those gated
paths.
