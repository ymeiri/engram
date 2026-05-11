# Brain Harness Cross-Harness Run Log

Date: 2026-05-11
Status: Claude Phase 1A setup complete; no arms launched yet

## Scope

This run log records the first executable checkpoint after
`docs/BRAIN_HARNESS_CROSS_HARNESS_CALIBRATION_2026-05-11.md`.

The checkpoint covers only:

- exact Phase 1 base commit selection,
- isolated Claude Code rescue worktree creation,
- evaluator-side target-visibility smoke checks for treatment arms.

It does not score Claude Code, Codex Desktop, or cross-harness behavior.

## Base Commit

All Claude Phase 1A worktrees were created from:

```text
32123670131e5effffbc4cdf72c502a73ccf0c3a Pre-register cross-harness calibration
```

The main checkout still has an unrelated untracked `AGENTS.md`. Per the
pre-registration, that file is user-owned state and must not be staged, deleted,
or treated as part of an arm.

## Worktrees

| Scenario | Arm | Branch | Worktree |
|---|---|---|---|
| `claude_rescue_current_plan_001` | `claude_no_memory` | `yuval.meiri/calib-claude-current-plan-no-memory` | `/Users/yuval.meiri/projects/engram-calib-claude-current-plan-no-memory` |
| `claude_rescue_current_plan_001` | `claude_memoryitem_orient` | `yuval.meiri/calib-claude-current-plan-orient` | `/Users/yuval.meiri/projects/engram-calib-claude-current-plan-orient` |
| `claude_rescue_commit_hygiene_001` | `claude_no_memory` | `yuval.meiri/calib-claude-commit-hygiene-no-memory` | `/Users/yuval.meiri/projects/engram-calib-claude-commit-hygiene-no-memory` |
| `claude_rescue_commit_hygiene_001` | `claude_memoryitem_orient` | `yuval.meiri/calib-claude-commit-hygiene-orient` | `/Users/yuval.meiri/projects/engram-calib-claude-commit-hygiene-orient` |
| `claude_rescue_bad_memory_guard_001` | `claude_no_memory` | `yuval.meiri/calib-claude-bad-memory-guard-no-memory` | `/Users/yuval.meiri/projects/engram-calib-claude-bad-memory-guard-no-memory` |
| `claude_rescue_bad_memory_guard_001` | `claude_memoryitem_orient` | `yuval.meiri/calib-claude-bad-memory-guard-orient` | `/Users/yuval.meiri/projects/engram-calib-claude-bad-memory-guard-orient` |

All six worktrees were clean immediately after creation.

## Smoke Check Method

Smoke checks used `orient` through the live Engram MCP path, not direct CLI
store commands, because direct CLI store access can conflict with the running
global daemon.

Each treatment smoke used:

- `project=engram`,
- `agent=claude_code`,
- `arm=prearm_smoke`,
- the exact scenario id,
- the scenario intent from the pre-registration,
- the scenario prompt packet from the pre-registration,
- the treatment worktree path as `cwd`.

All three smoke responses resolved the explicit project with confidence 1.0.
They also reported `repository_context=null` because the new worktrees are not
registered Memory OS checkouts. This is a setup note, not a failed target check,
because the smoke target is project-scoped memory visibility.

## Target-Visibility Results

### `claude_rescue_current_plan_001`

- Smoke trace: `019e1820-7eb5-7b62-a544-c3d52ecc7d56`
- Expected pre-registered current-plan memory:
  `019e17e9-b6d4-76b2-9463-dbeeaf376398`
- Visibility result: superseded/displaced
- Active successor visible:
  `019e1811-30ff-7382-b9c2-57cdf7b05c40`
  `Cross-harness calibration pre-registration committed; smoke checks next`
- Required rule visible:
  `019e01f1-f262-7d63-bd33-a2ca28228c03`
  `Brain Harness work follows research method`

Verdict: pass with recorded supersession. The active successor preserves the
target facts needed by the scenario: do not run broad benchmarking yet, create
isolated worktrees, run target-visibility smoke checks, then launch Claude
rescue before Codex redemption.

### `claude_rescue_commit_hygiene_001`

- Smoke trace: `019e1820-a058-7fa2-86a7-3f001f49625a`
- Expected preference memory visible:
  `019e03be-a9a5-7db2-848d-eb26ef78bcb5`
  `Commit every meaningful Engram step`
- Expected pre-registered current-plan memory:
  `019e17e9-b6d4-76b2-9463-dbeeaf376398`
- Visibility result: preference visible; current-plan target superseded/displaced
- Active successor visible:
  `019e1811-30ff-7382-b9c2-57cdf7b05c40`
  `Cross-harness calibration pre-registration committed; smoke checks next`

Verdict: pass with recorded supersession. The preference needed for the scenario
was directly visible, and the active successor preserves the arm/worktree and
`AGENTS.md` disposition.

### `claude_rescue_bad_memory_guard_001`

- Smoke trace: `019e1820-b359-79f3-90a4-a51df30e9a80`
- Expected pre-registered current-plan memory:
  `019e17e9-b6d4-76b2-9463-dbeeaf376398`
- Visibility result: superseded/displaced
- Active successor visible:
  `019e1811-30ff-7382-b9c2-57cdf7b05c40`
  `Cross-harness calibration pre-registration committed; smoke checks next`
- Required rule visible:
  `019e01f1-f262-7d63-bd33-a2ca28228c03`
  `Brain Harness work follows research method`

The smoke also returned older active guidance
`019e01f2-0a87-7f73-9b0b-7f2443eac7bb`, which says to defer ranking changes,
graph traversal, and obligation hot-path work. That guidance is directionally
consistent with the scenario's bad-memory guard.

Verdict: pass with recorded supersession. The returned context supports
benchmark calibration over M6 write apply, broad ranking changes, and hot-path
expansion.

## Launch Gate

Claude Phase 1A may now start, beginning with
`claude_rescue_current_plan_001`.

Rules for the next step:

- launch Claude Code in the scenario-specific worktree only,
- do not let one arm inspect another arm's worktree, output, transcript, or
  commit,
- for `claude_rescue_current_plan_001`, instruct both arms not to inspect repo
  files or use non-required tools, because the pre-registration document now
  exists in every worktree and contains the target facts,
- no-memory arms must not use Engram retrieval before completing the task,
- treatment arms must call `orient` once in their own fresh Claude session with
  `arm=claude_memoryitem_orient`,
- the `claude_rescue_current_plan_001` treatment arm may use only its required
  `orient` and `telemetry` calls before answering,
- each arm must submit telemetry feedback to its own trace,
- each implementation-bearing arm must run `git diff --check` before commit,
- commits must include only intended arm-local outputs.
