# Brain Harness Cross-Harness Run Log

Date: 2026-05-11
Status: Claude Phase 1A in progress; Hot Context fixed commit-hygiene salience behavior, with residual protocol and telemetry-ID gaps

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

### Rerun Setup Result

Rerun worktrees were created from the planned base commit:

| Arm | Branch | Worktree | HEAD |
|---|---|---|---|
| `claude_no_memory_rerun1` | `yuval.meiri/calib-claude-current-plan-rerun1-no-memory` | `/Users/yuval.meiri/projects/engram-calib-claude-current-plan-rerun1-no-memory` | `32123670131e5effffbc4cdf72c502a73ccf0c3a` |
| `claude_memoryitem_orient_rerun1` | `yuval.meiri/calib-claude-current-plan-rerun1-orient` | `/Users/yuval.meiri/projects/engram-calib-claude-current-plan-rerun1-orient` | `32123670131e5effffbc4cdf72c502a73ccf0c3a` |

Both worktrees were clean immediately after creation.

Fresh treatment smoke:

- Trace: `019e1835-b4fe-7612-ba40-a792f2feb707`
- Scenario: `claude_rescue_current_plan_001_rerun1`
- Arm: `prearm_smoke`
- Expected target MemoryItem:
  `019e1826-840f-7453-98e8-bb3e77a5f8e5`
  `Claude rescue current-plan attempt exposed memory freshness gate`
- Visibility result: target appeared as the top active decision.

Verdict: rerun treatment launch gate is open. The actual treatment arm must use
its own fresh orient trace with `arm=claude_memoryitem_orient_rerun1`.

### Rerun Arm Results

#### `claude_no_memory_rerun1`

- Worktree:
  `/Users/yuval.meiri/projects/engram-calib-claude-current-plan-rerun1-no-memory`
- Bridge harness: `isolated`
- Engram retrieval: none
- Telemetry trace: `019e1837-ae0b-79d2-be99-9a14d612a2d8`
- Evaluator feedback: `019e1837-c011-7730-ad41-b9c422084303`
- Worktree state after arm: clean

Output summary:

- Claude said the prompt did not contain enough project-specific evidence to
  determine the correct next measurement step.
- It requested the current calibration plan, most recent measurement output, or
  permission to read relevant files/memory.
- It did not claim repo, git, or memory context.

Evaluator verdict: clean expected baseline failure. The arm did not fabricate
context and did not answer the project-specific continuity question.

#### `claude_memoryitem_orient_rerun1`

- Worktree:
  `/Users/yuval.meiri/projects/engram-calib-claude-current-plan-rerun1-orient`
- Bridge harness: `personal`
- Required orient trace: `019e1836-c6b2-76c3-95d4-0541cf100217`
- Claude-submitted feedback: `019e1837-59de-7b81-85ba-7c7bbbaef01a`
- Evaluator feedback: `019e1837-ebc5-70c3-bcd2-b9a0c1f2d6a7`
- Worktree state after arm: clean

Output summary:

- Claude said the next measurement step is to complete
  `claude_rescue_current_plan_001_rerun1` cleanly.
- It said not to proceed to scenarios `002+`, Phase 1B, cross-arm aggregation,
  or the original superseded `001` attempt until this trace lands.

Evaluator verdict: treatment success relative to no-memory. The answer used the
corrected current-plan guidance and showed behavior-linked improvement over the
baseline.

Caveats:

- The treatment answer did not explicitly mention every blocked category in the
  current-plan memory, such as M6 apply/deletion, ranking changes, and broad
  cross-harness benchmarking.
- Claude's own telemetry feedback listed `019e1811-30ff-7382-b9c2-57cdf7b05c40`
  as used, but that memory is superseded and was not the corrected target. The
  evaluator feedback treats `019e1826-840f-7453-98e8-bb3e77a5f8e5` as the
  behavior-shaping memory and marks `019e1811-30ff-7382-b9c2-57cdf7b05c40` as
  stale attribution noise.

### Rerun Finding

`claude_rescue_current_plan_001_rerun1` provides the first clean behavior-linked
Claude rescue signal for the current-plan continuity scenario:

- no-memory could not answer without project context,
- `memoryitem_orient` answered the immediate next step after the freshness
  correction,
- no harmful memory use was observed in evaluator scoring.

This is still one scenario, not enough for cross-harness claims. The next
highest-confidence evidence step is to continue Phase 1A with another Claude
rescue scenario before starting Codex Phase 1B or any broad cross-harness
comparison.

## Commit-Hygiene Launch Gate: `claude_rescue_commit_hygiene_001`

Decision: continue Claude Phase 1A with `claude_rescue_commit_hygiene_001`.

Rationale:

- `claude_rescue_current_plan_001_rerun1` produced the first clean
  behavior-linked Claude rescue signal,
- one scenario is not enough to claim cross-harness or general Brain Harness
  effectiveness,
- commit hygiene is backed by a reviewed user preference MemoryItem and is
  directly relevant to current Engram development practice.

Existing worktrees from the original setup remain clean and at the original
Phase 1A base commit:

| Arm | Branch | Worktree | HEAD |
|---|---|---|---|
| `claude_no_memory` | `yuval.meiri/calib-claude-commit-hygiene-no-memory` | `/Users/yuval.meiri/projects/engram-calib-claude-commit-hygiene-no-memory` | `32123670131e5effffbc4cdf72c502a73ccf0c3a` |
| `claude_memoryitem_orient` | `yuval.meiri/calib-claude-commit-hygiene-orient` | `/Users/yuval.meiri/projects/engram-calib-claude-commit-hygiene-orient` | `32123670131e5effffbc4cdf72c502a73ccf0c3a` |

Fresh treatment smoke:

- Trace: `019e184c-87a6-7441-9cc0-0bb4f66dd0ec`
- Scenario: `claude_rescue_commit_hygiene_001`
- Arm: `prearm_smoke`
- Prompt:
  `Prepare a small Engram doc-only calibration update plan. Include how you will handle unrelated files and when you will commit. Do not implement yet.`
- Expected target MemoryItem:
  `019e03be-a9a5-7db2-848d-eb26ef78bcb5`
  `Commit every meaningful Engram step`
- Visibility result: target appeared as the top preference in the orientation
  packet.

Verdict: treatment launch gate is open. The actual treatment arm must use its
own fresh orient trace with `arm=claude_memoryitem_orient`, and both arms must
avoid reading repository files because the calibration documents exist inside
the original base worktrees.

### Arm Results

#### `claude_no_memory`

- Worktree:
  `/Users/yuval.meiri/projects/engram-calib-claude-commit-hygiene-no-memory`
- Bridge harness: `isolated`
- Engram retrieval: none
- Evaluator trace: `019e184e-c68e-7470-a8af-807dc491cacc`
- Evaluator feedback: `019e184e-d79d-7ea3-854c-1bd8b2eb5857`
- Worktree state after arm: clean

Output summary:

- Claude proposed a generic doc-only plan.
- It said unrelated files should be left untouched and separate issues should
  be noted rather than fixed inline.
- It said it would make one commit after the small doc-only batch is complete.
- It did not mention the reviewed project preference to commit every meaningful
  Engram step, and it did not mention the known unrelated `AGENTS.md` file.

Evaluator verdict: expected baseline limitation. The answer was reasonable
generic commit hygiene, but it lacked the project-specific preference and known
unrelated-file disposition that memory should provide.

#### `claude_memoryitem_orient`

- Worktree:
  `/Users/yuval.meiri/projects/engram-calib-claude-commit-hygiene-orient`
- Bridge harness: `personal`
- Required orient trace: `019e184d-ad1a-7ac3-a7a8-4e747f08c936`
- Claude-submitted feedback: `019e184e-050c-7910-a4c6-e8190c759855`
- Evaluator feedback: `019e184e-f30e-7ab3-82c0-4c8c1d12bd58`
- Worktree state after arm: clean

Output summary:

- Claude produced a stronger generic hygiene plan than the baseline: avoid
  `git add .`, use path-scoped staging, preserve unrelated dirty files, and
  avoid entangling unrelated changes in files it needs to edit.
- Claude did not cite or use the target preference
  `019e03be-a9a5-7db2-848d-eb26ef78bcb5`.
- Claude did not mention the known unrelated `AGENTS.md` file.
- Claude reported that no commit-hygiene memory was visible and suggested
  adding one, even though the orient trace returned the existing reviewed
  preference.

Evaluator verdict: partial/fail, not a clean treatment rescue. The trace
contains the target memory, but the model did not use it. This points to an
orientation presentation problem: the intent-relevant preference can be present
in `returned_memory_ids` while still not being visible or salient enough in the
agent-consumed context.

### Commit-Hygiene Finding

`claude_rescue_commit_hygiene_001` should not be counted as a clean Claude
rescue success:

- no-memory lacked project-specific preference context, as expected,
- treatment retrieved the target memory at the telemetry level,
- treatment did not use the target memory behaviorally,
- the failure mode is likely context-pack noise/salience/truncation, not a
  missing-memory or stale-memory problem.

Next benchmark step should be a narrowly labeled rerun or implementation slice
that improves `follow_user_preference` orientation salience. The immediate
candidate is to ensure reviewed preferences matching the requested intent are
placed in the compact/hot part of `orient`, then rerun this scenario before
moving to Codex Phase 1B or broad cross-harness comparison.

## Manual Terminal Validation: `claude_rescue_commit_hygiene_001`

Question: was the commit-hygiene treatment failure caused by Claude Bridge, or
does it reproduce in real interactive Claude Code?

Manual validation used a fresh worktree at the same Phase 1A base commit:

| Arm | Branch | Worktree | HEAD |
|---|---|---|---|
| `claude_manual_terminal` | `yuval.meiri/calib-claude-commit-hygiene-manual-terminal` | `/Users/yuval.meiri/projects/engram-calib-claude-commit-hygiene-manual-terminal` | `32123670131e5effffbc4cdf72c502a73ccf0c3a` |

Transcript export:

- `/Users/yuval.meiri/projects/engram-calib-claude-commit-hygiene-manual-terminal/2026-05-11-215756-controlled-engram-calibration-arm-clauderescue.txt`

Telemetry:

- Required orient trace: `019e1866-62e0-7c61-82c2-6671ebc0f555`
- Claude-submitted feedback: `019e1866-abad-7463-9e07-17863ddf2c43`
- Evaluator feedback: `019e1868-0537-7ee3-8c29-d8ebcddd1261`

Manual terminal observations:

- Real Claude Code startup hooks fired and injected the Engram session
  activation contract twice.
- The prompt allowed only Engram orient and telemetry. The transcript shows no
  file-read, Bash, edit, or repo-inspection permission use.
- Claude called orient and telemetry, then returned a plan.
- The plan had good generic git hygiene: explicit path-scoped staging, avoid
  `git add .` / `git add -A`, preserve unrelated dirty files, inspect
  `git status` and `git diff --staged` before committing, split commits if
  scope grows.
- Claude did not cite or use the target reviewed preference
  `019e03be-a9a5-7db2-848d-eb26ef78bcb5`.
- Claude did not mention the known unrelated `AGENTS.md` file.
- Claude reported that the visible orient slice did not include a
  commit-hygiene preference and suggested adding one.

Evaluator result: manual terminal validation reproduces the bridge failure.
The manual trace returned both the current-plan memory
`019e184f-a861-7dd1-bb6f-7e8b6dcd8d19` and the target reviewed preference
`019e03be-a9a5-7db2-848d-eb26ef78bcb5`, but Claude did not use the target
preference behaviorally. Therefore the main failure is not Claude Bridge. It is
an Engram orientation salience/presentation issue that affects real Claude Code
as well.

Bridge adequacy conclusion:

- Claude Bridge remains adequate for controlled A/B dogfooding where tool
  access, leakage, and telemetry labels must be constrained.
- Manual terminal validation is still required for failures involving UX,
  startup/stop hooks, MCP result presentation, or tool-output salience.
- For this scenario, the bridge result is representative enough: the same
  failure reproduced in interactive Claude Code.

Do not add a duplicate commit-hygiene preference. The reviewed source of truth
already exists as `019e03be-a9a5-7db2-848d-eb26ef78bcb5`. The next Engram
implementation step should improve `follow_user_preference` orientation so
matching reviewed preferences are visible in the compact/hot agent-consumed
context before lower-priority decisions.

## Hot Context Manual Rerun: `claude_rescue_commit_hygiene_001`

Question: after installing commit `deef133` and restarting the daemon, does the
new `follow_user_preference` Hot Context presentation fix the real Claude Code
commit-hygiene salience failure?

Manual validation used a fresh worktree at the same Phase 1A base commit:

| Arm | Branch | Worktree | HEAD |
|---|---|---|---|
| `claude_manual_terminal_hot_context` | `yuval.meiri/calib-claude-commit-hygiene-hot-context-manual` | `/Users/yuval.meiri/projects/engram-calib-claude-commit-hygiene-hot-context-manual` | `32123670131e5effffbc4cdf72c502a73ccf0c3a` |

Transcript export:

- `/Users/yuval.meiri/projects/engram-calib-claude-commit-hygiene-hot-context-manual/2026-05-11-224936-controlled-engram-calibration-arm-clauderescue.txt`

Telemetry:

- Required orient trace: `019e1894-de44-7a30-8b23-cad71e491599`
- Claude-submitted feedback:
  - malformed first attempt: `019e1895-4d24-7db1-8748-d059429e8d68`
  - clean retry: `019e1895-7ba9-7ba0-8ca3-46b4bef2b305`

Manual terminal observations:

- Real Claude Code startup hooks fired and injected the Engram session
  activation contract twice.
- Claude called `orient` with `intent=follow_user_preference` and
  `arm=claude_manual_terminal_hot_context`.
- The trace returned the reviewed commit-hygiene preference
  `019e03be-a9a5-7db2-848d-eb26ef78bcb5`.
- Claude explicitly cited Hot Context and the reviewed user preference:
  `Commit every meaningful Engram step`.
- Claude mentioned `AGENTS.md` as an unrelated user-owned file that should stay
  out of commits unless explicitly requested.
- Claude's plan included path-scoped staging, no `git add -A` / `git add .`,
  one focused commit per meaningful documentation step, and no WIP commits.
- Claude did not suggest adding a duplicate commit-hygiene preference.

Residual issues:

- The transcript shows a disallowed `Read` tool attempt, followed by
  `PostToolUseFailure:Read` hooks. The failed read did not appear to influence
  the final answer, but it means this was not a protocol-clean controlled run.
- Claude reported that the visible orient preview did not expose stable memory
  IDs, so both submitted feedback records left `used_memory_ids` empty even
  though Claude behaviorally used the reviewed preference.
- The first telemetry feedback attempt had malformed parameters after XML-like
  wrappers leaked into the note. Claude retried successfully, but the retry
  still embedded a `missing_context` note inside the free-form note field rather
  than the structured `missing_context` field.

Evaluator result: Hot Context fixed the salience failure behaviorally, but this
is a partial/qualified pass rather than a fully clean controlled run. Compared
with the previous manual terminal run, Claude now used the reviewed preference,
mentioned `AGENTS.md`, avoided inventing a missing preference, and produced the
expected commit-hygiene plan. The remaining evidence points to two next
implementation needs: expose `hot_context_ids` or equivalent compact IDs near
the top of `orient`, and improve hook/eval handling so disallowed tool attempts
and malformed feedback are captured as first-class scoring signals.
