# T295 Brain Harness Beta Scope Cut

Date: 2026-06-06

## Research Question

Can the current Memory OS / Brain Harness branch move toward an initial beta without first
closing production-complete harness parity gates?

## Hypotheses

| Hypothesis | Prediction | Decision |
| --- | --- | --- |
| H1: Full parity gates are beta blockers. | Native Claude prompt-bearing proof, effective-hook visibility, live Claude host labels, direct legacy deletion, and exhaustive lifecycle cleanup must close before PR readiness. | Rejected for beta. |
| H2: The beta can ship on the validated local Brain Loop path with explicit limitations. | Green PR CI, validated `orient`/vault/current-plan/obligation/M6 paths, supported-path doctors, and clear limitations are enough for an honest initial beta. | Accepted. |

## Evidence

- The beta-scope decision was cut after PR #2 head
  `688f8fe75f03a62e1712185258010edb22ae4574` passed CI run `27071097151`:
  Check, Format, Docs, Clippy, and Test all passed; Test completed in `40m17s`.
- The T295 documentation head `ac75ec7f487a939ace9f7db7b6251e809de917aa` later
  passed CI run `27072115918`: Check, Format, Docs, Clippy, and Test all passed;
  Test completed in `43m24s`.
- Historical CI runs prove only the exact heads they ran on. Any later commit intended for
  beta review still needs fresh PR checks on that exact head.
- T294 recorded the latest exact lifecycle archive batch and refreshed the canonical vault.
- AI Council beta-scope review reached consensus that the beta should not wait for production
  parity gates.
- Four read-only side agents corroborated the release path:
  - T294 CI was blocked only on the long serialized Test job.
  - Native Claude execution remains attribution-blocked by ambient native Claude processes.
  - The next lifecycle targets are maintenance, not beta blockers.
  - The current safe CI runtime should not be weakened for speed.

## Beta Blockers

The initial beta requires:

1. Green PR CI on the exact head intended for beta review.
2. Validated Codex/local Brain Loop path.
3. Canonical vault init/compile and readable generated Markdown.
4. Lean `orient` with current-plan, used-memory IDs, and obligation summary.
5. M6 inventory/export/inspection/status paths.
6. Supported-path obligations and harness doctor checks.
7. Clear beta limitations in repo docs and PR text.
8. Preserved approval boundaries for harness writes, native Claude execution, lifecycle writes,
   direct legacy deletion, and broad cleanup.

## Deferred From Beta

The following are known limitations and fast-follow work, not initial beta blockers:

- Native Claude prompt-bearing execution with clean attribution.
- Effective-hook visibility or result proof.
- Live Claude host-label proof.
- Direct legacy deprecation or deletion.
- Exhaustive residual lifecycle cleanup after T294.
- Broad `lint apply_safe`.
- Full multi-host harness parity.

## Result

T295 records the beta release cut: ship the validated core loop first, document the missing
production-hardening gates honestly, and avoid resetting the beta path with more lifecycle
maintenance unless explicitly requested.

PR #2 remains draft until the user explicitly asks to mark it ready.

## Non-Claims

T295 does not mark PR #2 ready, merge the PR, tag a release, run native Claude, prove effective
hooks, prove live Claude host labels, delete legacy data, archive additional lifecycle targets,
run `lint apply_safe`, change schema/storage/index behavior, or prove CI for commits made after
the T295 documentation head.
