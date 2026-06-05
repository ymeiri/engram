# Brain Harness T271 Branch Publication Approval Packet

Date: 2026-06-05
Status: docs-only/default-deny approval packet. Not executed.

## Scope

This packet prepares a future exact approval for publishing the current local Brain Harness branch
to `origin` and optionally setting upstream. It exists because T268 showed the recurring `git pull`
reconciliation hint is not evidence of local branch divergence, but the current branch still has no
same-named remote branch and no upstream.

T271 does not run `git push`, set upstream, open a PR, run `git pull`, merge, rebase, reset,
checkout, set `pull.rebase`, change global or repo Git pull policy, edit harness files, run native
Claude/Gemini, mutate lifecycle or M6 state, initialize or compile the canonical vault, change
ranking or `orient`, change public MCP/schema/storage/index/document-index behavior, delete data,
rollback, or touch user-owned files.

## Research Question

Can Engram make the remaining branch publication policy executable as a future exact, low-ambiguity
gate without changing remote state or Git configuration now?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A docs-only/default-deny packet can define the future remote publication operation and preflight checks while preserving the current no-push/no-upstream boundary. |
| Null | T268 is enough; publication policy can remain an informal decision until the user asks to publish. |
| Simpler alternative | Ignore the recurring pull hint because `origin/main` is an ancestor of `HEAD`. |
| Failure | The packet is mistaken for permission to push, set upstream, open a PR, or set a pull policy without exact approval. |

## Fresh Branch Evidence

Fresh evidence after commit `657b07a` (`Prepare host label gate`):

| Command | Result |
| --- | --- |
| `git fetch origin` | Completed with no output. |
| `git status --short --branch` | `## yuval.meiri/memory-os-phase0`; only `?? AGENTS.md`. |
| `git rev-list --left-right --count HEAD...origin/main` | `385 0`. |
| `git merge-base --is-ancestor origin/main HEAD` | Exit code 0. |
| `git rev-parse --abbrev-ref --symbolic-full-name @{upstream}` | `fatal: no upstream configured for branch 'yuval.meiri/memory-os-phase0'`. |
| `git branch -r` | `origin/HEAD -> origin/main`, `origin/main`, and `origin/yuval.meiri/dogfood-baf004-rerun-no-memory`; no `origin/yuval.meiri/memory-os-phase0`. |
| `git config --get-regexp '^(branch|pull)\.'` | `main` tracks `origin/main`; current branch only has `branch.yuval.meiri/memory-os-phase0.vscode-merge-base origin/main`; no pull policy. |
| `git remote -v` | `origin` fetch/push is `git@github.com:ymeiri/engram.git`. |
| `git rev-parse HEAD` | `657b07ad7cd43474d4c1ed1609331d927fe2bbee`. |
| `git rev-parse origin/main` | `e6697eee18530bc64f64ae94b6fd6006c24c7423`. |
| `git merge-base HEAD origin/main` | `e6697eee18530bc64f64ae94b6fd6006c24c7423`. |

Decision from the evidence:

- No local merge, rebase, or pull is needed before repo-local work.
- The branch is ahead of `origin/main` by 385 commits and behind by 0.
- The remaining branch gate is only remote publication/upstream/PR policy.

## Recommended Future Operation

The smallest future publication operation is branch backup/upstream only:

```text
git push --set-upstream origin HEAD:refs/heads/yuval.meiri/memory-os-phase0
```

That creates `origin/yuval.meiri/memory-os-phase0` at the current local `HEAD` and records the
upstream for the local branch. It does not open a PR.

Opening a PR is a separate external-publication decision and should stay out of the default T271A
operation unless the exact approval names PR behavior.

## Proposed Approval Wording

Use this exact approval if the next slice should publish the branch and set upstream, without PR
creation:

```text
Approve T271A: execute the branch publication/upstream operation from docs/BRAIN_HARNESS_T271_BRANCH_PUBLICATION_APPROVAL_PACKET_2026-06-05.md. After fresh preflight confirms `origin/main` is an ancestor of HEAD, `HEAD...origin/main` is behind 0, the worktree has no tracked changes and only known untracked root `AGENTS.md`, no same-named remote branch exists, and remote `origin` is `git@github.com:ymeiri/engram.git`, run exactly `git push --set-upstream origin HEAD:refs/heads/yuval.meiri/memory-os-phase0`. Do not open a PR, run git pull, merge, rebase, reset, checkout, set pull.rebase, change global/repo Git config beyond the upstream created by this push, edit harness files, run native Claude/Gemini, mutate lifecycle/M6/vault state, change ranking/orient/public MCP/schema/storage/index/document-index behavior, delete, rollback, or touch user-owned files.
```

Any PR publication must use a separate exact approval that names draft/ready state, title/body
source, target branch, validation required immediately before PR creation, and whether GitHub
metadata can be written.

Shorter approvals, generic continuation, or approvals naming only T255/T267/T269/T270/M6/lifecycle
work are not authorization to execute T271.

## Completion Impact

T271 does not publish the branch. It turns the remote-publication gate from an informal remaining
policy question into a future exact operation with a bounded preflight and a no-PR default.

The broader goal still remains incomplete on M6 dispositions/deferral, lifecycle cleanup/deferral,
prompt-bearing native Claude, effective-hook visibility, live Claude host-label proof, canonical
vault execution, and the future branch publication operation itself if the user wants remote
publication.
