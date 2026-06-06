# T300 Brain Harness Current-Plan Evidence Refresh

Date: 2026-06-07

## Research Question

After T299 exact-head CI passed, does Engram's Brain Loop still orient agents from stale
T297 release posture, and what is the smallest safe repair?

## Evidence

- `git fetch --prune origin` completed without changing the branch.
- `origin/main...HEAD` was `0 416`, and `origin/main` was an ancestor of `HEAD`.
- PR #2 remained draft and clean at
  `37ca96f060293e4b584c4c9490a8205e010d3b6a`.
- PR CI run `27076011668` passed on that exact T299 head: Check `1m32s`,
  Format `17s`, Docs `1m11s`, Clippy `1m44s`, and Test `40m10s`.
- Startup search trace `019e9f45-0e4c-7fa1-9a40-5a50a661ee84` returned stale
  current-plan MemoryItem `019e9eff-f670-7031-ac60-f9f68aa99255`, which still named
  T297 head `5a1905398bfb5255b3314b1a78339cd655ccb964` and CI run `27074430051`.
- `memory(get)` showed rolling handoff `019e9f00-12ed-7e01-ba33-4bb2ab816f38`
  had the same stale T297 PR posture.

## Decision

Treat stale current-plan retrieval as a Brain OS reliability issue. The repair should be an exact
current-plan/handoff refresh plus docs alignment, not a broad memory cleanup, ranking rewrite, or
production-parity claim.

## Changes

- Captured current-plan MemoryItem `019e9f46-9a46-7fe1-a061-711e5a221863`.
  It records T299 head `37ca96f060293e4b584c4c9490a8205e010d3b6a`, exact-head CI
  run `27076011668`, draft PR #2 posture, and the remaining production gates.
- The capture automatically superseded stale current-plan MemoryItem
  `019e9eff-f670-7031-ac60-f9f68aa99255`.
- Wrote rolling handoff `019e9f46-c1d4-7220-98de-baefc5bd043e`, superseding stale
  handoff `019e9f00-12ed-7e01-ba33-4bb2ab816f38`.
- Updated the architecture, implementation plan, and T299 report so repo docs record the
  completed T299 exact-head CI run.

## Validation

- Lean `orient` trace `019e9f46-d254-7003-8fc2-6b1dbd18cdcf` returned
  `019e9f46-9a46-7fe1-a061-711e5a221863` as the first Brain Loop item.

## Non-Claims

T300 does not mark PR #2 ready, merge the PR, tag a release, run native Claude, prove effective
hooks, prove live host labels, deprecate or delete direct legacy data, run broad `lint apply_safe`,
or close the production-quality Brain OS goal.
