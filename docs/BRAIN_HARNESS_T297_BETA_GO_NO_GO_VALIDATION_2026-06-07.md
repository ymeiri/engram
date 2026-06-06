# T297 Brain Harness Beta Go/No-Go Validation

Date: 2026-06-07

## Research Question

Does the current branch have enough fresh local evidence to support the T295 initial beta
scope cut, without reopening production-complete host parity gates?

## Hypotheses

| Hypothesis | Prediction | Decision |
| --- | --- | --- |
| H1: Beta must still wait for full production parity. | Native Claude prompt-bearing proof, effective-hook visibility, live host labels, direct legacy deletion, and broad lifecycle cleanup must close before beta. | Rejected for beta. |
| H2: The beta can proceed on the supported local Brain Loop path if the T295 gates have fresh evidence. | Exact-head CI, canonical vault refresh/readability, lean `orient`, M6 inventory/export/status, supported-path doctors, and honest limitations are enough for initial beta. | Accepted, subject to exact-head CI after this report commit. |

## Evidence

- PR #2 was draft and `CLEAN` on head
  `f03fb4b714d7b20a561d3a2316c7444878af93fe`.
- PR CI run `27073249090` passed Check, Format, Docs, Clippy, and Test on that head.
- A fresh fetch showed the feature branch and upstream were `0 0` apart,
  `origin/main...HEAD` was `0 413`, and `origin/main` was an ancestor of `HEAD`.
  The generic divergent-branch pull hint is therefore not current evidence to pull, merge,
  rebase, or change pull policy.
- Lean `orient` trace `019e9ed6-784b-7513-be7f-ba7bb209e352` returned the T296
  current-plan memory first, included `used_memory_candidate_ids`, and reported no open
  obligations.
- Canonical vault preflight status for `/Users/yuval.meiri/.engram/vault` was initialized
  with `2368` generated files and `expected_generated_file_count=2369`, showing it needed a
  normal generated refresh.
- `vault(action=compile)` refreshed the canonical generated vault. Postflight status reported
  `total_file_count=2369`, `generated_file_count=2369`, `user_file_count=0`,
  `memory_item_count=1666`, `knowledge_commit_count=579`, `repository_count=9`,
  `entity_count=32`, `project_count=79`, and `expected_generated_file_count=2369`.
- The canonical vault index page `99_System/Vault-Index.md` was readable, generated, and
  included frontmatter plus the Engram generated marker. Its summary matched the postflight
  counts.
- Direct scans counted `2369` vault files and `2369` files with frontmatter.
- M6 `memory(action=migration_inventory, project_name=engram, limit=10, ...)` scanned
  `121` sources, found `61` candidates, returned `10`, and warned that the run was dry-run
  only and no Memory OS records were written.
- M6 `memory(action=migration_review_export)` wrote a temporary generated review batch at
  `/private/tmp/engram-t297-m6-review-20260607` with `index.md` plus ten candidate pages.
- M6 `memory(action=migration_review_status)` on that temporary batch scanned ten files,
  reported no skipped files, no conflicts, no files outside the index, no missing indexed
  files, no warnings, and `ready_to_apply=false` because none of the generated candidates had
  reviewer decisions.
- `obligations(action=doctor, project=engram, cwd=/Users/yuval.meiri/projects/engram)`
  returned `open=[]` and `warnings=[]`.
- `harness(action=doctor, harness=codex, root=/Users/yuval.meiri)` returned `ready=true`
  with no missing MCP tools; the only warning was the soft lifecycle-compliance caveat.
- `harness(action=doctor, harness=generic, root=/Users/yuval.meiri)` returned
  `ready=true` with no missing MCP tools; the only warning was the soft
  lifecycle-compliance caveat.
- Telemetry `real_session_eval(project=engram, limit=20)` did not pass the confidence gate:
  `feedback_count=7` and `feedback_coverage=0.35`.
- Telemetry `real_session_eval(project=engram, limit=50)` passed the confidence gate:
  `feedback_count=32`, `feedback_coverage=0.62`, `distinct_intent_count=13`,
  `task_failure_count=0`, and `bad_memory_used_count=0`.

## Result

The supported local beta path is freshly validated: canonical generated vault refresh and
readability work, lean `orient` surfaces current-plan plus used-memory and obligation state,
M6 inventory/export/status paths work without writes when reviewer decisions are absent, and
Codex/generic supported-path doctors are ready.

The exact-head CI gate remains head-specific. If this report is committed, the beta-review head
becomes the report commit and must get its own green PR CI before the beta is treated as ready
for review.

## Non-Claims

T297 does not mark PR #2 ready, merge the PR, tag a release, run native Claude, prove
effective hooks, prove live host labels, delete or deprecate direct legacy data, run broad
`lint apply_safe`, apply M6 decisions, change schema/storage/index behavior, or prove exact-head
CI for commits made after `f03fb4b714d7b20a561d3a2316c7444878af93fe`.
