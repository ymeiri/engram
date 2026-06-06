# T296 Brain Harness Beta CI Evidence Alignment

Date: 2026-06-06

## Research Question

Does the beta-scope documentation distinguish exact-head CI evidence from future PR-head
readiness clearly enough to avoid overstating beta readiness after later commits?

## Hypotheses

| Hypothesis | Prediction | Decision |
| --- | --- | --- |
| H1: The T295 wording is sufficient. | Readers will understand that run `27071097151` proves only the T294 head even though the T295 doc changed the branch afterward. | Rejected. |
| H2: The docs should explicitly separate historical exact-head CI from current PR-head readiness. | The beta bar remains green CI on the head being reviewed, while historical T294/T295 runs stay useful bounded evidence. | Accepted. |

## Evidence

- PR CI run `27071097151` passed on head
  `688f8fe75f03a62e1712185258010edb22ae4574`.
- PR CI run `27072115918` passed on T295 documentation head
  `ac75ec7f487a939ace9f7db7b6251e809de917aa`.
- T286 had already recorded a related wording gotcha: avoid phrasing that makes a historical
  run sound like proof for an unspecified "current PR head."
- PR #2 remained draft at the start of T296.

## Result

T296 updates the beta-scope document, architecture notes, and implementation matrix so they state:

1. T294 and T295 CI runs are exact-head historical evidence.
2. The beta readiness bar is fresh green CI on the head intended for beta review.
3. Later documentation or code commits require their own PR checks before being treated as ready.

## Non-Claims

T296 does not mark PR #2 ready, merge the PR, tag a release, run native Claude, prove effective
hooks, prove live host labels, delete legacy data, archive lifecycle targets, run broad
`lint apply_safe`, or change schema/storage/index behavior.
