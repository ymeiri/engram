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

## First Arm Attempt: `claude_rescue_current_plan_001`

This first attempt was run through the local Claude bridge after the smoke-check
report was committed.

### `claude_no_memory`

- Worktree:
  `/Users/yuval.meiri/projects/engram-calib-claude-current-plan-no-memory`
- Bridge harness: `isolated`
- Engram retrieval: none
- Telemetry trace: `019e1825-4476-7761-bc10-d8b058082fc8`
- Telemetry feedback: `019e1825-66a1-7512-92b9-ef61674372d1`
- Worktree state after arm: clean

Evaluator verdict: failed/partial. Claude correctly avoided Engram retrieval
and rejected immediate broad cross-harness calibration, but it recommended a
single-harness no-memory baseline rather than the pre-registered staged Claude
rescue plus Codex redemption path.

Caveat: despite explicit instructions not to use tools or inspect repo files,
the final answer claimed to rely on git-log context. The bridge run did not
grant Bash or Engram tools, but this still makes the arm less clean than the
intended no-tool baseline.

### `claude_memoryitem_orient`

- Worktree:
  `/Users/yuval.meiri/projects/engram-calib-claude-current-plan-orient`
- Bridge harness: `personal`
- Required orient trace: `019e1824-4f0b-7d33-8b34-b1648e14b166`
- Claude-submitted feedback: `019e1824-e6d1-7193-9d14-33358bf3e92f`
- Evaluator-submitted feedback: `019e1825-bdd9-7850-abd8-19d710fa14b5`
- Worktree state after arm: clean

Evaluator verdict: failed due stale current-plan memory. Claude used Engram,
but the active current-plan memory still said the next step was to run smoke
checks. The evaluator had already completed those smoke checks and committed
this run report before launching the arm, but that new state had not been
captured as a current-plan MemoryItem. The treatment answer therefore
recommended the just-completed setup step instead of the actual next action.

Claude's own feedback also noted that the orient payload was very large and
that it attempted to read persisted orient output despite the run's tool
allowlist. The attempted read was blocked by the bridge path policy.

### Immediate Finding

This is a useful failure, not a reason to tune ranking first. The treatment arm
shows that Brain Loop v1 can return the active current-plan memory, but the
memory was stale relative to evaluator progress. The next corrective action is
to capture a new current-plan MemoryItem after each evaluator checkpoint before
launching any treatment arm that depends on "what is next?"

Do not count `claude_rescue_current_plan_001` as a clean treatment success.
If rerun, pre-register it as a rerun after memory freshness correction rather
than silently replacing this attempt.

## Freshness Correction

After recording the failed/partial current-plan attempt, the evaluator captured
a new current-plan MemoryItem:

- MemoryItem:
  `019e1826-840f-7453-98e8-bb3e77a5f8e5`
  `Claude rescue current-plan attempt exposed memory freshness gate`
- Knowledge commit:
  `019e1826-8439-77e2-b203-e508565b0942`
- Superseded stale current-plan memory:
  `019e1811-30ff-7382-b9c2-57cdf7b05c40`

Verification orient trace `019e1826-c2eb-75b0-a32d-99af1c816d92`
returned the new MemoryItem as the top active decision. That confirms the
freshness correction is visible to future treatment arms.

Next benchmark action should be explicitly labeled as either:

- a rerun of `claude_rescue_current_plan_001` after freshness correction, using
  new isolated worktrees and new telemetry labels, or
- a move to another Phase 1A scenario whose target does not depend on the
  just-corrected "what is next?" state.

## Rerun Registration: `claude_rescue_current_plan_001_rerun1`

Decision: rerun `claude_rescue_current_plan_001` after freshness correction.

Rationale:

- the first treatment arm failed because Engram's active current-plan memory was
  stale relative to evaluator progress,
- the stale memory has now been superseded by
  `019e1826-840f-7453-98e8-bb3e77a5f8e5`,
- verification orient trace `019e1826-c2eb-75b0-a32d-99af1c816d92` showed the
  corrected MemoryItem as the top active decision,
- the rerun directly tests whether Claude can now use the corrected current
  plan.

Rerun labels:

- scenario: `claude_rescue_current_plan_001_rerun1`
- no-memory arm: `claude_no_memory_rerun1`
- treatment arm: `claude_memoryitem_orient_rerun1`

Rerun base commit:

```text
32123670131e5effffbc4cdf72c502a73ccf0c3a Pre-register cross-harness calibration
```

This intentionally uses the original scenario base, not the current evaluator
commit. The freshness correction is in Engram memory, not in the arm worktree.
Keeping the rerun worktrees at the original base reduces leakage from the
evaluator run log into the no-memory arm.

Planned worktrees:

- `/Users/yuval.meiri/projects/engram-calib-claude-current-plan-rerun1-no-memory`
- `/Users/yuval.meiri/projects/engram-calib-claude-current-plan-rerun1-orient`

Before launch, run a fresh `prearm_smoke` check for the rerun treatment label
and record whether `019e1826-840f-7453-98e8-bb3e77a5f8e5` appears.
