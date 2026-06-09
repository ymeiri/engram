# Brain Harness T387 Exact Coordination Orphan Cleanup

Date: 2026-06-08
Branch: `yuval.meiri/memory-os-phase1`
Head before slice: `7081ff39c4630553b037f513307af6c2aaf91a9c`

## Question

Can the remaining Engram-project coordination-only active rows be removed without broad lifecycle
cleanup and without touching durable session history?

## Scope

This slice addresses exactly the three active coordination rows for project `engram` that remained
after T386 and whose durable session records were missing:

- `019e683b-1560-7361-b535-53b012e04aa5`
- `d238eb81-870e-4da7-8e5d-381014f151b0`
- `d96b3edb-f8ac-4d45-b644-92be32d0eae4`

No broad `lint apply_safe`, stale-session sweep, source change, M6 mutation, ranking change,
`orient` change, document indexing, native-Claude launch, hook execution, PR ready/merge/tag/publish
action, or hosted-CI fallback acceptance was performed.

## Evidence Review

Pre-cleanup `coord(list, project="engram")` returned exactly three rows:

- `019e683b-1560-7361-b535-53b012e04aa5`: agent `codex`, empty `components`,
  `current_file=null`, started `2026-05-31T08:48:27Z`, last heartbeat
  `2026-06-01T14:22:52Z`, goal was the T47 docs-only harness repair packet.
- `d238eb81-870e-4da7-8e5d-381014f151b0`: agent `codex`, empty `components`,
  `current_file=null`, started `2026-06-01T11:35:00Z`, last heartbeat
  `2026-06-01T11:41:05Z`, goal was the T86 non-gated continuity/evidence-quality slice.
- `d96b3edb-f8ac-4d45-b644-92be32d0eae4`: agent `codex`, empty `components`,
  `current_file=null`, started and last heartbeat `2026-06-01T11:27:51Z`, goal was the T85
  non-gated evidence/implementation slice.

`session(get)` returned `not found` for all three session IDs. Project-scoped lint was already
empty before this cleanup, so these were not stale active-session findings. They were orphaned
coordination rows only.

Source inspection confirmed the boundary:

- `engram-index/src/coordination.rs` documents `unregister` as the operation used when ending a
  coordination session.
- `engram-store/src/repos/coordination.rs` implements `unregister` by deleting only
  `active_session`.
- `engram-tests/tests/coordination_tests.rs` asserts unregister removes a coordination row and
  that unregistering a nonexistent coordination session succeeds.

## Changes

Unregistered exactly the three orphan coordination rows through `coord(action="unregister")`:

- `019e683b-1560-7361-b535-53b012e04aa5`
- `d238eb81-870e-4da7-8e5d-381014f151b0`
- `d96b3edb-f8ac-4d45-b644-92be32d0eae4`

No durable session records or session events were deleted. The row contents are preserved in this
evidence document.

## Validation

Post-cleanup `coord(list, project="engram")` returns no active coordination rows:

```json
{
  "sessions": [],
  "count": 0
}
```

Project-scoped lint remains clean:

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

Scoped obligations remain clean:

```json
{
  "open": [],
  "warnings": []
}
```

`coord_stats` still reports global active coordination rows outside this exact project slice; those
were not inspected or changed.

## Remaining Gates

This slice removes project-local orphan coordination noise. It does not make Engram production/GA
ready.

Known remaining gates are unchanged:

- PR #3 still needs release-owner local-validation fallback acceptance or restored hosted CI, then
  ready/merge/tag/publish mechanics.
- Prompt-bearing native Claude, effective-hook visibility, and live host-label proof remain
  separate production gates.
- Direct legacy deprecation/deletion, M6 write-apply expansion, broad lifecycle cleanup, and broad
  `lint apply_safe` remain separate exact-scope work.
