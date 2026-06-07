# Brain Harness T287 CI Checkout Node 24 Hardening - 2026-06-06

## Research Question

Can PR #2's GitHub Actions workflow remove the current Node.js 20 checkout-action warning without
changing Rust build behavior or reopening any Brain Harness product gate?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Upgrading `actions/checkout` from `v4` to `v5` removes the Node.js 20 deprecation warning while preserving the existing CI job contracts. |
| Null | The warning is cosmetic and the workflow change does not matter for current PR correctness. |
| Simpler alternative | Leave the workflow unchanged and rely on GitHub's temporary Node.js 20 opt-out behavior. |
| Failure | `actions/checkout@v5` requires an incompatible runner or changes checkout behavior enough to fail CI. |

## Evidence

- PR CI run `27061750059` passed on head `1138bb48c56d95fdc8e05cc8fc3b4f838801a3b2`, but every
  job emitted a GitHub annotation that `actions/checkout@v4` runs on Node.js 20 and that Node.js 20
  will be removed from GitHub Actions runners on September 16, 2026.
- `gh release view v5.0.0 --repo actions/checkout` reports that checkout `v5.0.0` updates checkout
  to use Node 24 and requires runner version `v2.327.1` or newer.
- The repository workflow uses GitHub-hosted `ubuntu-latest` runners and has five
  `actions/checkout@v4` steps: Check, Test, Format, Clippy, and Docs.

## Change

T287 updates only `.github/workflows/ci.yml`, replacing all five `actions/checkout@v4` references
with `actions/checkout@v5`.

## Validation Plan

1. Confirm no `actions/checkout@v4` references remain.
2. Confirm all five workflow jobs use `actions/checkout@v5`.
3. Run `git diff --check`.
4. Push and require a fresh PR CI run on the T287 head.

## Non-Claims

T287 is CI hardening only. It does not change Rust source behavior, complete PR readiness, execute
native Claude, prove effective-hook visibility, prove live Claude host labels, mutate lifecycle or
M6 state, or deprecate direct legacy behavior.
