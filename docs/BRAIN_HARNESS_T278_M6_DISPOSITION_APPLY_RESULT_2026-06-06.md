# Brain Harness T278 M6 Disposition Apply Result - 2026-06-06

## Scope

T278 executed the current-data M6 generated review batch at
`/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export` under the
2026-06-06 standing authorization. The slice records evidence-based candidate dispositions, runs
status and dry-run apply validation, applies accepted project-scoped candidates into Memory OS, and
refreshes the canonical generated vault.

This does not deprecate or delete legacy data paths, run lifecycle cleanup, run native Claude, edit
harness files, push the branch, or change ranking/orient/public MCP/schema/storage behavior.

## Research Question

Can the T68/T209 M6 review batch move from undecided to applied without asking for another approval,
given the 2026-06-06 standing authorization and the older T210/T250 "human-provided dispositions"
language?

Preferred hypothesis: the standing authorization supersedes the old approval blocker, while each
candidate still needs evidence-based conservative disposition and write safety validation.

Null hypothesis: T210/T250 still block any candidate checkbox edits because the authorization is not
candidate-specific.

Failure hypothesis: Codex writes stale, out-of-scope, or contradicted memory into active Memory OS.

Measurement before write:

- generated review root has only indexed generated markdown files and no symlinks;
- `migration_review_status` reports exactly one decision per indexed file before apply;
- dry-run apply reports the intended write set, zero duplicates, and zero warnings;
- actual apply writes only the dry-run write set and creates a KnowledgeCommit;
- post-apply status is idempotent, with accepted sources duplicate-skipped;
- the new MemoryItems are retrievable by content;
- canonical vault status matches expected generated-file counts after compile.

## Preflight Evidence

- Pre-disposition status scanned 12 generated files, all undecided, `ready_to_apply=false`,
  no conflicts, no skipped files, no not-in-index files, and no missing indexed files.
- Filesystem preflight found only `index.md` plus 12 candidate markdown files under the review root,
  and no symlinks under the review root.
- Source inspection confirmed `migration_review_status` is read-only, `migration_review_apply`
  writes only accepted or accepted-with-edits candidates, and accepted-with-edits uses the visible
  `#` title, `## Content`, and `Kind` bullet from the reviewed page.
- AI Council recall preserved the older warning that M6 operation classes must stay separate. A
  focused broadcast then agreed that the 2026-06-06 standing authorization supersedes the old
  approval blocker, but only evidence-supported candidates should become active memory. Estimated
  broadcast cost was about `$0.0804`.

## Dispositions Recorded

| Candidate | Disposition | Write effect | Rationale |
| --- | --- | --- | --- |
| `0001-review-dogfood-baf008-accepted-live-2026-05-24.md` | Accept | Active project fact | Durable BAF008 live validation evidence. |
| `0002-review-dogfood-baf008-prearm-setup-2026-05-24.md` | Accept with edits | Active project decision | Original one-time "ready" phrasing was stale; migrated the reusable pre-arm pattern and evidence. |
| `0003-review-dogfood-claude-code-scoped-obligation-smoke-2026-05-24.md` | Accept with edits | Active project decision | Preserves scoped-obligation validation and caveat; does not overclaim memory utility attribution. |
| `0004-review-dogfood-claude-code-obligation-list-scope-fix-2026-05-24.md` | Accept | Active project fact | Durable project-scoped fix for obligation-list leakage. |
| `0005-review-dogfood-claude-code-2026-05-24-review.md` | Reject | No write | Contains stale workaround guidance superseded by candidate 0004. |
| `0006-review-decisions-claude-hook-reenable-prompt-2026-05-24.md` | Quarantine | No write | May be superseded by later harness work; needs fresh native-Claude evidence before active memory. |
| `0007-review-maintenance-disk-cleanup-2026-05-24.md` | Reject | No write | One-time cleanup history, not durable retrieval guidance. |
| `0008-review-decisions-orient-recent-git-context.md` | Accept | Active project decision | Durable orient design decision with validation and remaining limitation. |
| `0009-review-testing-dogfood-pilot-2026-05-07.md` | Reject | No write | Contains stale "do not proceed to M6" guidance superseded by the standing authorization and T278 evidence. |
| `0010-quarantine-telemetry-recall-432971.md` | Quarantine | No write | Entity-scoped review-all/dd-source telemetry is outside the high-confidence Engram project write set. |
| `0011-quarantine-gotchas-shared-worktree-branch-loss.md` | Quarantine | No write | Possibly useful, but entity-scoped to `review-all-system`, not project:engram for this slice. |
| `0012-skip-plan.md` | Reject | No write | Low-confidence stale plan whose approval-gated premise is superseded. |

## Apply Result

After recording dispositions:

- `migration_review_status` scanned 12 files with no missing decisions, conflicts, skipped files,
  not-in-index files, or missing indexed files.
- Status counts before apply: `accepted_count=3`, `accepted_with_edits_count=2`,
  `quarantined_count=3`, `rejected_count=4`, `planned_count=5`, `ready_to_apply=true`,
  `warnings=[]`.
- Dry-run apply matched the same counts, planned five project-scoped writes, and reported
  `duplicate_count=0`, `warnings=[]`.
- Pre-apply Memory OS cursor was commit `019e9bd1-5551-7743-b291-7e73e720f0ca`.
- Actual apply created KnowledgeCommit `019e9bd6-7e8e-7611-8326-1811b3b799a2` with message
  `Apply reviewed migration batch (5 items)`.

Written MemoryItems:

- `019e9bd6-7e8d-7ae2-b57c-f92d131566e9` -
  `Dogfood Baf008 Accepted Live 2026 05 24`
- `019e9bd6-7e8d-7ae2-b57c-f93d294d9acc` -
  `Dogfood Baf008 Prearm Setup 2026 05 24`
- `019e9bd6-7e8d-7ae2-b57c-f94966531ed9` -
  `Dogfood Claude Code Scoped Obligation Smoke 2026 05 24`
- `019e9bd6-7e8d-7ae2-b57c-f952676bf2d6` -
  `Dogfood Claude Code Obligation List Scope Fix 2026 05 24`
- `019e9bd6-7e8d-7ae2-b57c-f962f09936d5` -
  `Decisions Orient Recent Git Context`

Post-apply status remained structurally ready and idempotent: `ready_to_apply=true`,
`planned_count=0`, `duplicate_count=5`, and the only warnings were the expected
already-migrated-source duplicate skips for the five accepted candidates.

## Retrieval And Vault Validation

- Unified memory search trace `019e9bd6-c2c6-7ff1-bd37-2f5a57f20ca1` retrieved the edited BAF008
  pre-arm setup memory first with reviewed active `project:engram` metadata.
- Unified memory search trace `019e9bd6-cceb-7ba1-8b56-2e35ab0abd92` retrieved the scoped-obligation
  fix and scoped-obligation smoke memories at the top with reviewed active `project:engram`
  metadata.
- Before post-apply vault refresh, canonical vault status showed `total_file_count=2278`,
  `generated_file_count=2278`, `user_file_count=0`, `memory_item_count=1612`,
  `knowledge_commit_count=551`, and `expected_generated_file_count=2287`.
- `vault(action="compile", vault_path="/Users/yuval.meiri/.engram/vault")` completed with
  `files_skipped=[]`.
- Post-compile vault status is synchronized: `total_file_count=2287`,
  `generated_file_count=2287`, `user_file_count=0`, `memory_item_count=1612`,
  `knowledge_commit_count=551`, `repository_count=9`, `entity_count=32`, `project_count=79`,
  and `expected_generated_file_count=2287`.
- Corrected marker/frontmatter scans using `rg --files-without-match` returned no files missing the
  generated vault marker or frontmatter.
- Sample generated page
  `/Users/yuval.meiri/.engram/vault/memory/items/019e9bd6-7e8d-7ae2-b57c-f93d294d9acc-dogfood-baf008-prearm-setup-2026-05-24.md`
  contains the edited content, reviewed metadata, backlinks, and manual-review evidence.
- `lint(action="run", vault_path="/Users/yuval.meiri/.engram/vault", write=false, limit=20)`
  returned pre-existing superseded-active lifecycle pressure and `applied_safe_actions=0`; no
  vault metadata finding appeared in the returned set.
- `obligations(action="doctor", project="engram", cwd="/Users/yuval.meiri/projects/engram")`
  returned `open=[]` and `warnings=[]`.

## Remaining Gates

T278 closes the current T68/T209/T210/T250 M6 review-batch disposition/apply gate. It does not close
direct legacy deprecation/deletion, broad lifecycle cleanup, prompt-bearing native Claude execution,
effective-hook visibility, live Claude host-label proof, branch publication/upstream/PR, or broad
Brain Harness completion.
