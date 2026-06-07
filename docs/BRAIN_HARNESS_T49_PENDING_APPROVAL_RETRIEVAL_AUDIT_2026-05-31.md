# Brain Harness T49 Pending Approval Retrieval Audit

Status: Completed read-only audit; partial result.
Date: 2026-05-31
Scope: Retrieval evidence for pending approval gates only

This audit did not run M6 inventory, review export, apply, deletion, lifecycle mutation, harness
writes, schema/storage/index changes, public MCP changes, ranking changes, or `orient` payload
changes.

## Research Question

When an agent asks what approval gates are pending before continuing Brain Harness work, does Engram
recover the currently pending T45, T47, and T48 gates without relying on guesswork?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Direct pending-approval retrieval returns the active M6, harness-write, and stale-current-plan lifecycle gates prominently enough for an agent to stop before gated writes. |
| Null | Pending approval prompts miss one or more active gates, so the agent must rely on prior transcript context or manual doc search. |
| Simpler alternative | Treat the latest current-plan memory as the only required approval queue until stronger evidence justifies a retrieval or payload change. |
| Failure | The audit is mistaken for approval to run M6, archive memory, write harness adapters/settings/hooks, or change the `orient` hot path. |

## Measurement

The audit is a pass only if both direct unified `search` and lean `orient` make the three currently
pending approval gates visible in the returned top context:

- T45: one bounded inventory-only M6 scoping proposal.
- T47: five exact harness `write=true` repair calls, each gated by a fresh matching dry-run.
- T48: one exact stale current-plan archive proposal.

A result is partial if the gates are recoverable only indirectly, for example through the current
plan's full content rather than as individually surfaced gate memories.

## Evidence

Direct `search` trace `019e7d43-c140-72f1-86c4-6b32096c6095` for
`what approval gates are pending before continuing Brain Harness` returned the active M6 gate
MemoryItem `019e7ce5-155d-7a10-85f5-00b9dcc69cd0`, the harness-write gate MemoryItem
`019e7cde-b517-77d0-aaac-c8638811d4e8`, and the T48 lifecycle approval packet memory
`019e7d41-e88b-7301-b250-d1354e027eb9` as the top three memory results.

Direct `search` trace `019e7d43-c096-70b3-897c-5a5d205817a7` for
`pending approvals T45 T47 T48 M6 harness lifecycle approval packet` returned the T48 current-plan
memory first, the harness-write gate second, and the active M6 gate fourth. This is sufficient to
recover the gates, but the T45/T47/T48 packet identities are partly carried by the T48 current-plan
content rather than by all three packet documents appearing as top-ranked documents.

Lean `orient` trace `019e7d43-c1d2-78b0-b815-a398f924a765` for
`What approvals are pending before continuing Engram Brain Harness work?` returned the T48
current-plan memory first, but did not individually surface the M6 gate or harness-write gate in
the top items. A scoped current-plan list returned exactly one active project current-plan item,
`019e7d41-e88b-7301-b250-d1354e027eb9`, whose full content says the next step requires explicit
user approval before running T47 harness writes, the T48 lifecycle archive, or T45 M6 inventory.

Claude Bridge reviewed the evidence in read-only mode and recommended a partial verdict: direct
search exposes the gates, but lean `orient` leaves the M6 and harness gates latent inside current
plan content instead of presenting them as independent top items.

## Verdict

Partial.

Direct unified `search` is good enough for an explicit pending-approval query: it surfaces the
active M6, harness-write, and T48 lifecycle gates prominently. Lean `orient` is weaker for the same
question because it exposes the approval queue indirectly through the latest current-plan memory
instead of returning the active M6 and harness-write gates as individual top items.

This is not evidence for broad ranking churn or `orient` payload expansion. Prior design guidance
still applies: `orient` is the compact task-boundary hot path, and graph traversal, lint, migration,
and approval-audit behavior should stay out of normal orientation unless a future approved slice
shows a narrowly scoped need.

## Next Action

No code change is justified by T49 alone. Continue treating T45, T47, and T48 as explicit pending
approval gates. If the next desired step is to execute one of them, ask for the exact approval in
that packet. If the next desired step is to make lean `orient` surface pending approval gates more
directly, pre-register a separate prompt-class slice and get explicit approval before changing
ranking or expanding the hot-path payload.
