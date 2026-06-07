# T298 Brain Harness Exact Lifecycle Archive Batch

Date: 2026-06-07

## Research Question

Can the next sampled superseded-active lifecycle findings be reduced with another exact
target-reviewed archive batch, without running broad `lint apply_safe` or touching native
Claude, hooks, host labels, M6, direct legacy data, ranking, schema, storage, or public MCP
behavior?

## Hypotheses

| Hypothesis | Prediction | Decision |
| --- | --- | --- |
| H1: Broad lifecycle cleanup is required. | The next cleanup step must run `lint apply_safe` or archive all sampled findings. | Rejected. |
| H2: A small exact batch can safely reduce lifecycle debt. | Five concrete safe-action targets can be verified with `memory(get)` and `graph(around)`, archived, and removed from the sampled lint queue. | Accepted. |

## Preflight

- `git status --short --branch` showed the branch on
  `yuval.meiri/memory-os-phase0...origin/yuval.meiri/memory-os-phase0` with only untracked
  root `AGENTS.md`.
- `git rev-list --left-right --count HEAD...@{u}` returned `0 0`; the generic divergent
  pull hint was not current evidence to pull, merge, rebase, or change pull policy.
- `lint(action=run, limit=25)` reported the next sampled superseded-active safe-action
  targets as:
  - `019dfc97-4f9b-7301-b401-38179a03aeec`
  - `019dfca2-cd3c-7241-a206-522556d5158b`
  - `019dfce1-c566-7031-b024-86ae45ac9132`
  - `019dfd36-487d-7552-97cb-c81cf53d1be5`
  - `019dfd36-d0e5-7d12-81ad-d5b84db1d514`

## Target Review

All five targets were active rolling handoffs before archive.

| Target | Scope | Direct successor evidence |
| --- | --- | --- |
| `019dfc97-4f9b-7301-b401-38179a03aeec` | `project:dd-source-pr428950` | `019dfca2-cd3c-7241-a206-522556d5158b` supersedes it. |
| `019dfca2-cd3c-7241-a206-522556d5158b` | `project:dd-source-pr428950` | `019dfce1-c566-7031-b024-86ae45ac9132` supersedes it. |
| `019dfce1-c566-7031-b024-86ae45ac9132` | `project:dd-source-pr428950` | `019dfe85-07ea-7600-987e-97aa1842c9e7` supersedes it. |
| `019dfd36-487d-7552-97cb-c81cf53d1be5` | `project:tmp` | `019dfd36-d0e5-7d12-81ad-d5b84db1d514` supersedes it. |
| `019dfd36-d0e5-7d12-81ad-d5b84db1d514` | `project:tmp` | `019dfd38-fc3d-7352-83a6-c9bbd16349ea` supersedes it. |

## Result

The five reviewed targets were archived with successor-specific archive reasons. Post-archive
`memory(get)` confirmed all five targets now have `status=archived`. Post-archive
`lint(action=run, limit=25)` no longer returned any of the five targets and advanced the
sampled queue to `019dfd38-fc3d-7352-83a6-c9bbd16349ea`.

## Non-Claims

T298 does not prove exhaustive lifecycle cleanup, run broad `lint apply_safe`, delete or
deprecate direct legacy data, run native Claude, prove effective hooks, prove live host labels,
apply M6 decisions, mark PR #2 ready, merge the PR, tag a release, or change schema, storage,
ranking, `orient`, document-index, public MCP, harness settings, hooks, or adapters.
