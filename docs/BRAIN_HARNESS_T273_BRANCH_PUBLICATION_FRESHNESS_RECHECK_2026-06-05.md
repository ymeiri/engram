# Brain Harness T273 Branch Publication Freshness Recheck

Date: 2026-06-05
Status: completed read-only branch publication and pull-hint freshness recheck.

## Scope

T273 refreshes the T268/T271 branch evidence after the T272 commit and after another observed
`git pull` reconciliation hint:

```text
fatal: Need to specify how to reconcile divergent branches.
```

This slice does not run `git pull`, merge, rebase, reset, checkout, push, set upstream, set
`pull.rebase`, change global or repo Git config, publish a PR, edit harness files, run native
Claude/Gemini, mutate lifecycle or M6 state, initialize or compile the canonical vault, change
ranking or `orient`, change public MCP/schema/storage/index/document-index behavior, delete data,
or touch user-owned files.

## Research Question

After T272, does the current Brain Harness branch require local reconciliation before further
repo-local work, or is the remaining branch gate still only remote publication/upstream policy?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A fresh remote check will show that `origin/main` is still the merge-base and an ancestor of `HEAD`; the pull hint remains a no-op for repo-local work on this branch. |
| Null | `origin/main` moved or local branch state changed enough that a new local merge/rebase plan is needed. |
| Simpler alternative | Rely on T268/T271 without rechecking. |
| Failure | Treat the pull hint as permission to set pull config, pull, merge, rebase, push, or set upstream. |

## Measurement

Fresh read-only branch evidence after commit `534796d`:

| Command | Result |
| --- | --- |
| `git fetch origin` | Completed with no output. |
| `git status --short --branch` | `## yuval.meiri/memory-os-phase0`; only `?? AGENTS.md`. |
| `git rev-parse HEAD` | `534796d9f5a7e59d364e4075cfb7b45df5811a4c`. |
| `git rev-parse origin/main` | `e6697eee18530bc64f64ae94b6fd6006c24c7423`. |
| `git merge-base HEAD origin/main` | `e6697eee18530bc64f64ae94b6fd6006c24c7423`. |
| `git merge-base --is-ancestor origin/main HEAD` | Exit code 0. |
| `git rev-list --left-right --count HEAD...origin/main` | `387 0`. |
| `git rev-list --left-right --count main...origin/main` | `0 107`. |
| `git branch -r` | `origin/HEAD -> origin/main`, `origin/main`, and `origin/yuval.meiri/dogfood-baf004-rerun-no-memory`; no `origin/yuval.meiri/memory-os-phase0`. |
| `git rev-parse --abbrev-ref --symbolic-full-name @{upstream}` | `fatal: no upstream configured for branch 'yuval.meiri/memory-os-phase0'`. |
| `git show-ref refs/remotes/origin/yuval.meiri/memory-os-phase0` | Exit code 1, no ref found. |
| `git config --get-regexp branch` | `main` tracks `origin/main`; current branch only has `branch.yuval.meiri/memory-os-phase0.vscode-merge-base origin/main`. |
| `git config --get-regexp pull` | Exit code 1, no pull policy configured. |
| `git remote -v` | `origin` fetch/push is `git@github.com:ymeiri/engram.git`. |

One earlier `git for-each-ref --format=%(...)` check failed because zsh parsed the unquoted
format string. It is not branch-state evidence. The successful `git show-ref` and `git branch -r`
checks above provide the remote-branch evidence.

## Decision

The current branch still does not need a local pull, merge, or rebase before repo-local Brain
Harness work:

- `origin/main` is exactly the merge-base with `HEAD`.
- `origin/main` is an ancestor of `HEAD`.
- The branch is ahead of `origin/main` by 387 commits and behind by 0.
- There is no same-named remote branch.
- The branch has no upstream.
- No pull policy is configured.

The repeated pull hint should be treated as evidence that bare `git pull` is the wrong primitive
for this branch, not as evidence that local reconciliation is required. Continue using explicit
`fetch`, `status`, `rev-list`, `merge-base`, upstream, and remote-ref checks.

Do not set global or repo pull policy as part of the Brain Harness goal. That would change future
Git behavior without closing a product requirement.

## Completion Impact

T273 refreshes the branch-publication gate only. It does not publish the branch, set upstream, open
a PR, or change Git config.

The future default operation from T271 remains the smallest remote-publication action if the user
wants publication:

```text
git push --set-upstream origin HEAD:refs/heads/yuval.meiri/memory-os-phase0
```

Before any future publication, rerun the same preflight against the then-current `HEAD`. PR
creation remains a separate exact approval.

The broader Brain Harness goal remains incomplete on M6 dispositions/deferral, lifecycle
cleanup/deferral, prompt-bearing native Claude, effective-hook visibility, live Claude host-label
proof, canonical vault execution, and optional remote publication/upstream.

## Validation

Validation for T273:

- lean `orient` and direct Engram searches before acting;
- actual docs read: architecture, Memory OS implementation plan, research method, orient contract,
  T255, T268, T269, T270, T271, and T272;
- fresh `git fetch origin`;
- read-only branch/status/rev-list/merge-base/upstream/config/remote inspections listed above;
- no pull, merge, rebase, push, upstream config, pull config, branch creation, PR publication,
  lifecycle/M6/vault/native-harness mutation, deletion, or user-owned-file edit.
