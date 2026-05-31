# Brain Harness T50 Claude Pending Approval Parity

Status: Completed read-only cross-harness smoke; pass.
Date: 2026-05-31
Scope: Claude Code parity for post-T49 pending-approval retrieval

This smoke did not run M6 inventory, review export, apply, deletion, lifecycle mutation, harness
writes, schema/storage/index changes, public MCP changes, ranking changes, or `orient` payload
changes.

## Research Question

After T49 current-plan capture, does Claude Code see the same pending-approval retrieval shape that
Codex sees for the current Brain Harness continuation prompt?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Claude Code surfaces the latest T49 current plan, the active harness-write gate, and the active M6 gate through the same read-only `orient`/`search` shapes as Codex. |
| Null | Claude Code misses one or both active approval gates, so the pending-approval retrieval behavior remains Codex-only. |
| Simpler alternative | Treat T49 as Codex-only evidence and defer Claude parity until after a user-approved gated action. |
| Failure | The parity smoke creates writes, starts migration/harness/lifecycle work, or is mistaken for approval to change ranking or expand the `orient` hot path. |

## Measurement

The smoke is a pass only if:

- Claude Code can call the live Engram MCP `orient`, `search`, and `obligations` read paths.
- Lean `orient` returns the T49 current-plan memory and individually surfaces both active approval
  gate memories:
  - harness-write gate `019e7cde-b517-77d0-aaac-c8638811d4e8`;
  - M6 gate `019e7ce5-155d-7a10-85f5-00b9dcc69cd0`.
- Direct `search` for pending approval gates returns the same active gate memories prominently.
- Any synthetic obligations created by the smoke are resolved or skipped with evidence.

## Evidence

Claude Bridge ran a read-only personal-harness task with only `mcp__engram__orient`,
`mcp__engram__search`, and `mcp__engram__obligations` allowed. Claude reported:

- lean `orient` trace `019e7d48-6e97-7513-96af-f49d5a61bfc5`;
- direct `search` trace `019e7d48-905b-75c2-9d5b-e9cb657024c9`;
- T49 current-plan memory `019e7d46-cd85-7012-a994-fcf23bba44a1` at orient position 1;
- harness-write gate `019e7cde-b517-77d0-aaac-c8638811d4e8` at orient position 2;
- M6 gate `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` at orient position 3;
- stale repository-scoped current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` at orient
  position 5;
- direct search returned M6 first and harness-write second, both with score `0.9964`;
- direct search did not return the stale current-plan memory in the top memory hits.

Codex then read the telemetry records for the two Claude traces. `telemetry(action="get_trace")`
confirmed:

- orient trace `019e7d48-6e97-7513-96af-f49d5a61bfc5` returned the T49 current plan, stale
  repository-scoped current plan, harness-write gate, and M6 gate among its returned memory IDs;
- search trace `019e7d48-905b-75c2-9d5b-e9cb657024c9` returned M6 first, harness-write second,
  and the T49 current plan third among memory IDs.

The Claude smoke created two prompt-derived startup obligations. Codex resolved
`019e7d48-49ac-7f33-8693-535fd8d20b03` with design-context evidence and skipped
`019e7d48-49ac-7f33-8693-5343cf0c51dd` because T50 did not change source code or assert
source-level behavior. A final `obligations(action="doctor")` returned no open obligations and no
warnings.

## Verdict

Pass for this narrow cross-harness prompt class.

Claude Code reproduced the post-T49 pending-approval retrieval shape: the latest T49 current plan,
the active harness-write gate, and the active M6 gate were all visible without code changes or
gated writes. The stale repository-scoped current-plan memory still appears in lean `orient`, which
matches the unresolved T48 lifecycle gate rather than a new T50 failure.

## Next Action

The approval gates remain unchanged. T45 M6 inventory, T47 harness repair writes, and T48 lifecycle
archive still require explicit user approval before execution. Further improvement to approval-gate
surfacing in lean `orient` should remain a separately approved prompt-class slice, not an implicit
payload expansion.
