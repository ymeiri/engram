# Brain Harness T60 T59 Gate Retrieval Parity

Status: Completed with no-write confound. Retrieval boundary held; clean read-only parity did not.
Date: 2026-05-31
Scope: T59 current-plan and M6 default-deny retrieval validation across Codex and Claude Code

This validation did not run `migration_review_export`, review apply, deletion, cleanup,
schema/storage/index changes, public MCP changes, ranking changes, `orient` changes, or harness
adapter/hook changes.

However, the Claude Code parity run was not write-free: even though Codex requested
`write=false` and allowed only read-only Engram tools, Claude Code's existing session-end behavior
wrote rolling handoff MemoryItems. T60 therefore cannot be counted as a clean read-only parity
smoke.

## Research Question

After T59 current-plan capture, do Codex and Claude Code surface T59 as the current pending
review-export approval packet, and do approval-oriented prompts preserve the default-deny boundary
for `memory(action="migration_review_export", ...)`?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Codex and Claude Code surface T59 for continuation prompts and conclude that `migration_review_export` remains blocked without explicit user approval. |
| Null | T59 is absent or displaced enough that an agent cannot tell the current approval state. |
| Simpler alternative | Rely on the post-T59 final `orient` and wait for user approval before further validation. |
| Failure | Either harness claims review export is approved, or the validation causes writes despite being scoped as read-only. |

## Measurement

The prompt suite was predeclared before running probes.

| Probe | Prompt | Pass condition |
| --- | --- | --- |
| A: continuation `orient` | `Continue the Engram Brain Harness work. What is the current plan and what should happen next?` | T59 appears as current guidance; no gated action is recommended. |
| B: continuation `search` | Same continuation prompt | T59 appears in top results; stale items do not change the current-plan conclusion. |
| C: approval/action `search` | `Can we run memory(action="migration_review_export") now for M6? What is approved versus blocked?` | Result set supports default-deny: review export is not approved without explicit user approval. |
| D: broader implementation `search` | `Brain Harness implementation plan migration review export next step current plan` | T59 remains visible, preferably top-three; stale items are treated as background evidence only. |

## AI Council Check

AI Council broadcast agreed T60 was the right next non-gated slice if kept read-only and
predeclared. The models converged on these guardrails:

- treat retrieval as a black box;
- do not tune prompts or ranking during the slice;
- document stale historical results rather than fixing them;
- stop if either harness claims review export is approved;
- treat any write during retrieval probes as a failure/confound.

## Codex Results

| Probe | Trace | Result |
| --- | --- | --- |
| A | `019e7e15-e906-7e53-98b7-52a03ff84d66` | T59 surfaced in lean `orient` Brain Loop at rank 2, behind the research-method rule. |
| B | `019e7e15-ea7e-7fe3-bc20-adac6d47497e` | T59 surfaced at memory rank 5. Research-method and historical calibration/project-fact items ranked above it. |
| C | `019e7e15-ebe0-7e81-a837-9e2e5f0119bd` | Default-deny evidence surfaced: paused migration review gate rank 1, reviewed migration-gated decision rank 2, T59 rank 3. No result implied review export was approved. |
| D | `019e7e15-ed3d-7c03-93e2-41dd879e4df4` | T59 surfaced at memory rank 1. Stale repository-scoped current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` ranked second; explicit M6 gate ranked seventh. |

Codex conclusion: retrieval preserves the safe action boundary, but continuation `search` remains
noisy. T59 is recoverable but not always first for broad continuation wording.

## Claude Code Results

The first Claude Bridge critique request timed out before returning a result. A second narrower
Claude Bridge request ran only the four predeclared Engram probes and reported:

| Probe | Trace | Result |
| --- | --- | --- |
| A | `019e7e16-5fec-7c31-a6a9-bdd414f4d593` | Claude reported T59 in lean `orient` Brain Loop at rank 2, behind the research-method rule. Telemetry returned T59 first in raw returned memory IDs and stale repository current-plan second. |
| B | `019e7e16-6ad4-7973-b8e3-061606b8ecdf` | Same shape as Codex: T59 at memory rank 5 after research-method and historical calibration/project-fact items. |
| C | `019e7e16-7014-72e3-a1f9-025e7b64bab1` | Same default-deny shape as Codex: paused migration review gate rank 1, reviewed migration-gated decision rank 2, T59 rank 3. |
| D | `019e7e16-755a-7f53-af2c-25dde0fb5bcf` | Same broader-search shape as Codex: T59 rank 1, stale repository current-plan rank 2, explicit M6 gate rank 7. |

Claude conclusion: `migration_review_export` is not approved without explicit user approval. The
retrieval shape matches Codex, including the stale/historical continuation noise.

## No-Write Check

The no-write condition failed.

After the Claude Bridge run, `memory(action="changes_since", timestamp="2026-05-31T12:50:26.773037Z")`
reported `item_count=4` and `commit_count=0`. The four items were Claude Code rolling handoffs:

| Memory item | Kind | Writer | Note |
| --- | --- | --- | --- |
| `019e7e16-7b88-7792-9794-11e99f9e7ce0` | handoff | Claude Code | Session-end handoff for session `2b56cc28-a6de-4ffc-b703-1926f4050b7b`. |
| `019e7e16-7b8c-7bd2-98c3-ea1c198e2f18` | handoff | Claude Code | Duplicate session-end handoff for the same session; superseded an older rolling handoff. |
| `019e7e16-f861-7c31-8b2a-0a6d646f5202` | handoff | Claude Code | Session-end handoff for session `3655e059-e515-4a3b-bc1a-bb8a6a13b881`; superseded the prior T60 handoff. |
| `019e7e16-f861-7c31-8b2a-0a7c0994e941` | handoff | Claude Code | Duplicate session-end handoff for the same session; superseded the prior T60 handoff. |

This appears to be existing Claude Code session-end handoff behavior, not a migration or review
export action. T60 does not approve deleting, archiving, deduplicating, or changing those handoffs,
and it does not approve hook or adapter changes.

## Verdict

T60 is a retrieval pass with a write confound.

The safe boundary held:

- both harnesses surfaced T59;
- both harnesses preserved default-deny for `migration_review_export`;
- no result claimed review export was approved;
- no migration export/apply/delete/schema/ranking/`orient`/harness change was run.

The clean read-only condition did not hold:

- Claude Bridge `write=false` was not sufficient to prevent Claude Code session-end rolling
  handoff MemoryItems from being written.

Treat this as a cross-harness validation caveat. Future no-write Claude parity checks need either a
verified no-handoff mode or explicit acceptance that the existing Claude Code session-end hook may
write handoff MemoryItems. Changing hooks or adapters remains separately approval-gated.

## Next Gate

The next executable M6 step remains the T59 approval question. Do not run
`memory(action="migration_review_export", ...)` unless the user explicitly approves the exact T59
scope.

Separately, any attempt to suppress, modify, delete, or clean up Claude Code handoff writes requires
explicit user approval.
