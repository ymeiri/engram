# T330 Local Codex Beta Smoke

Date: 2026-06-07
Status: completed local/Codex beta smoke evidence packet

## Question

Can the current PR #3 head still support the scoped local/Codex Brain Loop beta path after T329,
and can release-facing evidence name the actual head and hosted-CI blocker without widening into
production-parity work?

## Scope

T330 is a release-proof evidence slice for the supported local/Codex beta path. It records current
state and smoke evidence only. It does not run broad lifecycle cleanup, execute M6 write apply,
repair adapters, run native Claude, change hooks/settings, mark PR #3 ready, merge, tag, publish,
or release.

Current PR #3 head:

```text
fe46d0a73d39e3309b149703dda4c108da91fc02
```

Hosted GitHub Actions run:

```text
27096981016
```

The hosted run failed before workflow steps ran. All five jobs are completed with failure
conclusions because GitHub Actions did not start them under the account billing/spending-limit
condition. This remains an external account gate, not source-failure evidence.

## Smoke Evidence

Lean `orient` smoke:

- trace: `019ea2c1-439c-7e30-b5f9-b9cb6e641b48`
- scope: `engram`
- selected project: `engram`
- memory cursor commit: `019ea2bd-0a15-77b1-9417-640d5af4ed7f`
- used-memory candidates included the lean-orient contract, T329 current plan, hosted-CI blocker,
  commit preference, and harness write-approval gate:
  - `019e6931-bd2d-7281-b9f6-952eaa2a20e4`
  - `019ea2bd-09fd-7ca1-ba73-1b9fea6bf909`
  - `019ea1f2-2f2e-76b3-97ac-89125bf173b2`
  - `019e03be-a9a5-7db2-848d-eb26ef78bcb5`
  - `019e7cde-b517-77d0-aaac-c8638811d4e8`
- obligation summary was available and empty.
- open obligations were empty.

Obligations doctor:

```text
open: []
warnings: []
```

Canonical vault status before compile:

```text
root=/Users/yuval.meiri/.engram/vault
initialized=true
generated_file_count=2519
user_file_count=0
expected_generated_file_count=2519
memory_item_count=1760
knowledge_commit_count=635
repository_count=9
entity_count=32
project_count=79
```

Canonical vault compile after the smoke wrote the generated projection with zero skipped files:

```text
files_skipped=[]
memory_item_count=1760
knowledge_commit_count=635
repository_count=9
entity_count=32
project_count=79
```

Memory OS lint sample still shows residual lifecycle debt beginning at:

```text
019e7cf7-560c-70e2-bbeb-3448f4637055
```

That residual lifecycle queue is not a T330 blocker under the beta scope. It remains future exact
target cleanup and must not be handled with broad `lint apply_safe`.

## M6 Smoke

After a read-only gate audit, T330 ran only bounded M6 read/dry-run operations against a fresh temp
path:

```text
/private/tmp/engram-t330-m6-review-20260607
```

Inventory parameters:

```text
action=migration_inventory
project_name=engram
limit=5
include_entity_observations=true
include_session_history=true
include_work_observations=true
```

Inventory result:

```text
sources_scanned=129
total_candidates=69
returned_candidates=5
truncated=true
skipped_already_migrated=60
```

The inventory warnings were the expected dry-run/review-gate warnings:

```text
Dry run only: no Memory OS records were written.
Only explicitly accepted review candidates are eligible for migration writes.
Skipped 60 candidates whose source was already migrated.
```

Temp review export wrote six generated files with no skipped files:

```text
candidates/0001-review-audits-beta-docs-scope-limitations-2026-06-07.md
candidates/0002-review-audits-beta-release-risk-challenge-2026-06-07.md
candidates/0003-review-audits-beta-release-manager-scope-2026-06-07.md
candidates/0004-review-decisions-beta-scope-t299-final.md
candidates/0005-review-audits-beta-search-contract-2026-06-07.md
index.md
```

Review status against the temp batch scanned five candidate files. All five had no decision,
`ready_to_apply=false`, and warnings were empty.

Dry-run apply used writer provenance with `dry_run=true` and `create_commit=false`. It scanned the
same five files, planned zero items, wrote zero items, created no commit, and returned no warnings.

T330 does not edit review pages, infer candidate decisions, touch the canonical T58/T68 review
workspace, or run write apply. Future M6 review-page edits, canonical review workspace reruns, and
`migration_review_apply` with `dry_run=false` remain explicit-approval gates.

## Interpretation

The scoped local/Codex beta path remains supported at the current PR #3 head, including
review-gated M6 inventory/export/status/dry-run behavior. The remaining beta release gate is
procedural: fix the hosted GitHub Actions billing/spending-limit blocker and rerun exact-head
checks, or receive explicit release-owner approval to accept local validation as the beta fallback.
PR #3 must remain draft until explicit approval for ready/merge/tag/publish/release.
