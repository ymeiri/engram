# Brain Harness T407 Post-Release Native Claude Preflight

Date: 2026-06-09
Status: completed post-release preflight hardening.

## Scope

T407 updates `scripts/native-claude-gate-preflight.sh` for the post-`v0.2.0-beta.1`
production-gate track. The script now defaults its expected branch to `main` and supports
`--expected-branch <branch>` for explicit development-branch or historical checks.

This slice also refreshed the canonical generated vault after release memory writes. It did not
launch native Claude, run `/hooks`, signal processes, mutate settings/adapters, accept hosted-CI
fallback, mark a PR ready, merge, tag, publish, mutate M6, or run lifecycle cleanup.

## Research Question

Can the native Claude production-gate preflight be run after beta publication without relying on a
stale pre-merge branch default, while still failing closed before any native Claude execution?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | The preflight should default to `main` after the beta merge and expose an explicit branch override for non-main checks. | Supported. |
| Null | The `EXPECTED_BRANCH` environment variable is enough. | Rejected because release evidence should be self-documenting at the command line. |
| Simpler alternative | Remove branch checking from the preflight. | Rejected because branch/upstream sync is part of the production-gate evidence. |
| Failure | The preflight hardening launches Claude or cleans up running processes. | Avoided. It remains read-only and reports blockers. |

## Evidence

Before the vault refresh, the post-release preflight on `main` reported:

```json
{
  "gate_state": "blocked",
  "blockers": [
    "canonical vault is not generated-count aligned",
    "native Claude CLI processes are already running"
  ],
  "vault": {
    "generated_file_count": 2823,
    "expected_generated_file_count": 2833
  }
}
```

After `engram vault compile`, vault status reported `generated_file_count=2833` and
`expected_generated_file_count=2833`. The native-Claude preflight then reported a single remaining
blocker:

```json
{
  "gate_state": "blocked",
  "blockers": [
    "native Claude CLI processes are already running"
  ]
}
```

## Validation

Validation performed for this slice:

- `bash -n scripts/native-claude-gate-preflight.sh`
- `git diff --check`
- `./scripts/native-claude-gate-preflight.sh --expected-branch
  yuval.meiri/post-beta-native-preflight --allow-worktree-changes --json`
- `./scripts/native-claude-gate-preflight.sh --allow-worktree-changes --json`

The branch-override run removed the branch-mismatch blocker and retained fail-closed blockers for
the unpushed feature branch and already-running native Claude processes. The default run on the
feature branch reported `branch mismatch: expected main`, confirming that the post-release default
now targets `main`.

## Gate Impact

T407 removes a stale post-release branch default and clears the canonical vault-count blocker.
Native Claude prompt-bearing proof, effective-hook visibility, and live host-label proof remain
blocked until the pre-existing native Claude process ambiguity is resolved under explicit approval
or by external state change.
