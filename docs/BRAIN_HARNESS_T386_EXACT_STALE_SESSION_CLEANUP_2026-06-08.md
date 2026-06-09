# Brain Harness T386 Exact Stale Session Cleanup

Date: 2026-06-08
Branch: `yuval.meiri/memory-os-phase1`
Head before slice: `ca4d41553000cc41fe0ffc2f7b6749612a0f5965`

## Question

Can the next project-scoped stale active-session findings be closed without broad lifecycle
cleanup, without deleting durable session history, and without mutating unrelated active
coordination state?

## Scope

This slice addresses exactly the five project-scoped `stale_active_session` lint findings that
were current after T385:

- `019dd063-ee4f-7943-8409-0450bac3a724`
- `019e7d38-68d7-7652-b62f-3e8e635253ae`
- `019e7e6e-73e2-7e72-9351-da62e69686af`
- `019e8470-5d37-7db0-ac21-1725184849e7`
- `019e990f-e4fb-7a02-a840-77a38dceab3e`

No broad `lint apply_safe`, source change, M6 mutation, ranking change, `orient` change,
document indexing, native-Claude launch, hook execution, PR ready/merge/tag/publish action, or
hosted-CI fallback acceptance was performed.

## Evidence Review

Pre-cleanup `session(get)` and `coord(list)` showed these were stale historical sessions, not
current work:

- `019dd063-ee4f-7943-8409-0450bac3a724`: active for 1009 hours; only April 27 completed
  migration/quarantine review milestones and an installed-runtime refresh event; matching
  coordination row had empty components/current_file and last heartbeat
  `2026-04-27T19:21:48Z`.
- `019e7d38-68d7-7652-b62f-3e8e635253ae`: active for 203 hours; no events; no matching active
  coordination row.
- `019e7e6e-73e2-7e72-9351-da62e69686af`: active for 198 hours; one old May 31 plan event;
  matching coordination row had empty components/current_file and last heartbeat
  `2026-05-31T15:05:27Z`.
- `019e8470-5d37-7db0-ac21-1725184849e7`: active for 170 hours; one old June 1 resume
  observation; matching coordination row had empty components/current_file and last heartbeat
  `2026-06-01T18:27:03Z`.
- `019e990f-e4fb-7a02-a840-77a38dceab3e`: active for 74 hours; no events; matching
  coordination row had empty components/current_file and last heartbeat
  `2026-06-05T18:33:42Z`.

The cleanup preserves session records and events. It only changes the session lifecycle status to
`completed` and removes the matching ephemeral active coordination rows for sessions that had
them.

## Changes

Ended the five stale sessions through `session(action="end")`, each with an evidence summary that
records the lint age, event state, coordination state, and history-preservation boundary.

Unregistered exactly four matching active coordination rows through `coord(action="unregister")`:

- `019dd063-ee4f-7943-8409-0450bac3a724`
- `019e7e6e-73e2-7e72-9351-da62e69686af`
- `019e8470-5d37-7db0-ac21-1725184849e7`
- `019e990f-e4fb-7a02-a840-77a38dceab3e`

`019e7d38-68d7-7652-b62f-3e8e635253ae` had no matching active coordination row to unregister.

## Validation

Post-cleanup `session(get)` shows the five targeted sessions have `status="completed"` and
non-null `ended_at` timestamps. Their summaries record the exact cleanup rationale.

Post-cleanup project-scoped lint is empty:

```bash
./target/debug/engram lint run --scope-project engram --limit 20 --json
```

Output:

```json
{
  "findings": [],
  "applied_safe_actions": 0
}
```

Post-cleanup `coord(list, project="engram")` no longer includes the four targeted coordination
rows.

## Remaining Gates

This slice removes stale active-session lint debt. It does not make the system production/GA
ready.

Known remaining gates are unchanged:

- Three old coordination-only rows still exist for session IDs that no longer resolve through
  `session(get)`. They were not part of the stale active-session lint findings and should be
  handled only by a separate exact coordination-orphan cleanup slice.
- PR #3 still needs release-owner local-validation fallback acceptance or restored hosted CI, then
  ready/merge/tag/publish mechanics.
- Prompt-bearing native Claude, effective-hook visibility, and live host-label proof remain
  separate production gates.
- Direct legacy deprecation/deletion, M6 write-apply expansion, broad lifecycle cleanup, and broad
  `lint apply_safe` remain separate exact-scope work.
