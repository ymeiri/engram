# Brain Harness T55 T54 Claude Parity

Status: Completed read-only cross-harness smoke; pass after personal-harness rerun.
Date: 2026-05-31
Scope: Claude Code parity for post-T54 current-plan retrieval

This smoke did not run M6 inventory, review export, apply, deletion, lifecycle mutation, harness
writes, schema/storage/index changes, public MCP changes, ranking changes, or `orient` payload
changes.

## Research Question

After T54 current-plan capture, does Claude Code see the same current-plan and gated-next-work
shape that Codex sees for the current Brain Harness continuation prompt?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Claude Code surfaces T54 current-plan memory `019e7d68-a1b5-74c1-beb6-3a27d8495b93` first in lean `orient` and direct `search`, while treating stale repository-scoped current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` and old migration records as non-authoritative history. |
| Null | Claude Code misses T54 or ranks older current-plan or migration records as authoritative current guidance. |
| Simpler alternative | Treat Codex's post-T54 sanity orient and direct search as enough evidence and defer Claude parity until after a user-approved gated action. |
| Failure | The parity smoke creates lifecycle, migration, harness, schema, ranking, or `orient` changes, or is mistaken for approval to archive, replace, migrate, or repair anything. |

## Measurement

The smoke is a pass only if:

- Codex's lean `orient` and direct `search` return T54 first for the continuation prompt class.
- Claude Code can call the live Engram MCP read paths through Claude Bridge.
- Claude Code lean `orient` returns T54 first.
- Claude Code direct `search` returns T54 first.
- Stale or wrong-scope current-plan history may appear, but is treated only as historical evidence.
- Any synthetic obligations from the smoke are resolved or skipped with evidence.

## Codex Baseline

Codex lean `orient` trace `019e7d6b-ad40-7002-a777-4f5c6fdc0923` returned:

- T54 current-plan memory `019e7d68-a1b5-74c1-beb6-3a27d8495b93` first;
- non-gated calibration limitation second;
- harness adapter and hook write approval gate third;
- commit preference fourth;
- stale repository-scoped current-plan target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` fifth.

Codex direct `search` trace `019e7d6b-f000-7323-99c0-5010649f2dc1` returned:

- T54 current-plan memory first with score `0.9494`;
- older AI Council next-step synthesis second;
- direct-search current-plan smoke history third;
- stale repository-scoped current-plan target fourth;
- non-gated calibration and older migration records below those.

## Claude Code Smoke

The first Claude Bridge attempt used the `project` harness and was unmeasured: Claude Code reported
only file tools (`Glob`, `Grep`, and `Read`) and could not see the Engram MCP tools. This records a
bridge/harness exposure caveat, not a retrieval failure.

The rerun used the same read-only prompt through the `personal` harness with only
`mcp__engram__orient`, `mcp__engram__search`, and `mcp__engram__obligations` allowed. Claude
reported:

- lean `orient` trace `019e7d6d-460f-7ae1-bae0-1a662ace3e5d`;
- direct `search` trace `019e7d6d-4648-71e2-9cd7-c702a5b9cd48`;
- T54 current-plan memory at orient rank 1;
- non-gated calibration limitation at orient rank 2;
- harness adapter and hook write approval gate at orient rank 3;
- commit preference at orient rank 4;
- stale repository-scoped current-plan target at orient rank 5;
- T54 at direct-search rank 1 with score `0.9494`;
- stale repository-scoped current-plan target at direct-search rank 4.

Codex then read both telemetry records. The telemetry confirmed:

- Claude orient trace `019e7d6d-460f-7ae1-bae0-1a662ace3e5d` returned T54 in the returned memory
  IDs and Claude reported it first in the lean Brain Loop ordering;
- Claude search trace `019e7d6d-4648-71e2-9cd7-c702a5b9cd48` returned T54 first among memory IDs.

## Obligation Cleanup

The Claude smoke created two prompt-derived obligations:

- `019e7d6c-e09a-7fd1-9af3-3ea4733a6551`, design-context reading;
- `019e7d6c-e09a-7fd1-9af3-3e9579396dd3`, source reading.

Codex resolved the design-context obligation after reading the governing docs and skipped the
source-reading obligation because T55 is a read-only retrieval parity and documentation slice with
no source edits or new source-level behavior claims.

## Verdict

Pass for this narrow post-T54 continuation prompt class.

Claude Code reproduced the post-T54 retrieval shape: T54 was first in lean `orient` and direct
`search`. Older current-plan and migration-related memories still appear below T54, but they did
not outrank the current plan and were interpreted as historical evidence only.

The failed `project` harness attempt is a harness-exposure caveat: Claude Bridge parity checks that
need Engram MCP should continue using the `personal` harness unless the project harness is repaired
or separately validated.

## Next Action

The approval gates remain unchanged. T45 M6 inventory, T47 harness repair writes, and T52 lifecycle
resolution still require separate explicit user approval before execution. Further work should stay
read-only or documentation-only unless the user explicitly approves one of those gated paths.
