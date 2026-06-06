# Brain Harness T279 Lifecycle Archive Execution Result - 2026-06-06

## Scope

T279 executed the exact lifecycle cleanup path for the three previously packeted
T234/T247/T248 MemoryItems under the 2026-06-06 standing authorization. The slice archived only
these MemoryItems:

- `019dd3fe-ec94-7122-af04-1f35b839387f` -
  `Memory OS migration completion run finished`
- `019e8291-40aa-71a0-b16b-9ba7b6446cc6` -
  `Post-T76 rolling telemetry gate remains false`
- `019e01f2-0a87-7f73-9b0b-7f2443eac7bb` -
  `Resume continuity probe uses active MemoryItems before ranking changes`

T279 did not run `lint apply_safe`, delete memory, change ranking or `orient`, mutate schema,
storage, public MCP, document-index, harness, native-Claude, branch, or legacy-deprecation state.

## Research Question

Can the exact T234/T247/T248 lifecycle targets be safely archived after T278, given the newer
standing authorization and fresh evidence, without using stale packet payloads or broad lint
cleanup?

Preferred hypothesis: the 2026-06-06 standing authorization removes the old approval blocker, but
each archive reason must be revalidated against post-T278 facts and executed as an exact
`memory.archive` write.

Null hypothesis: the old T252 boundary still blocks lifecycle archive writes, or T278 makes one of
the targets useful active guidance again.

Failure hypothesis: T279 archives a still-current item, relies on stale T234 pre-T278
`ready_to_apply=false` wording, or uses broad `lint apply_safe`.

## Preflight Evidence

- Lean `orient` trace `019e9bdd-9abe-7960-942e-dda14f9e538f` returned the T278 current-plan
  memory first and no open obligations.
- AI Council recall recovered the older T252 boundary. A focused 3-model broadcast then agreed
  that the new explicit standing authorization covers the action class, while the factual archive
  reasons still need fresh validation. The useful correction was to not reuse T234's pre-T278
  payload mechanically.
- T278 scope reads confirmed the current M6 review batch is applied, but direct legacy
  deprecation/deletion, broad legacy simplification, and lifecycle cleanup remained separate.
- Fresh `memory(get)` confirmed all three target IDs were active immediately before archive,
  project-scoped to `engram`, with unchanged titles, tags, and expected timestamps.
- Fresh `graph(around, depth=1)` showed project/evidence/commit/session edges only; no direct
  dependent MemoryItem or replacement edge appeared for any target.
- Direct search trace `019e9bdf-bcb3-77d2-bca0-d1d64561ede2` returned current T278 plan first and
  still returned the three stale targets as active memory results.
- Fresh `telemetry(action="real_session_eval", project="engram", limit=50)` generated at
  `2026-06-06T07:39:56.545235Z` passed with `feedback_coverage=0.5400000214576721`,
  `distinct_intent_count=5`, `task_failure_count=0`, `bad_memory_used_count=0`,
  `wrong_scope_memory_count=0`, and `missing_context_count=0`, contradicting T247's active
  "gate remains false" wording.
- Read-only `lint(action="run", write=false, limit=160, vault_path=...)` was dominated by
  unrelated global superseded-active/open-obligation findings. It was recorded as pressure only,
  not as an archive path.
- `obligations(action="doctor", project="engram")` returned `open=[]` and `warnings=[]`.
- Git status showed only the known user-owned untracked root `AGENTS.md`.

## Archive Writes

All three archives were direct exact-target `memory(action="archive")` calls with T279-specific
reasons:

- T234 target `019dd3fe...` was archived because it is historical and overbroad current guidance
  after T277/T278. T277 created the durable canonical vault, T278 applied the current M6 batch and
  refreshed that vault, and T278 explicitly did not close broad legacy deprecation/deletion.
- T247 target `019e8291...` was archived because it accurately recorded a 2026-06-01 telemetry
  failure, but fresh T279 telemetry now passes the confidence gate and the active title/content was
  stale current guidance.
- T248 target `019e01f2...` was archived because it was valid 2026-05-07 probe guidance, but later
  current-plan/retrieval work and the T278 current plan supersede it as next-action guidance.

Memory OS KnowledgeCommit `019e9be1-67ff-7e92-a87e-f92667fa3582` records the archive batch as
`Archive T279 exact lifecycle targets`.

## Validation

- `memory(changes_since)` from cursor `019e9bdc-29f8-76a0-8cd7-80729f20d8bf` and timestamp
  `2026-06-06T07:39:33.420996Z` returned exactly the three archived MemoryItems.
- Post-archive direct search trace `019e9be0-ab65-7c02-8acc-d2925ec977cd` returned the T278
  current plan first and did not return the three archived targets in the active memory results.
- Canonical vault compile completed with `files_skipped=[]`.
- Final vault status is synchronized: `total_file_count=2291`, `generated_file_count=2291`,
  `user_file_count=0`, `memory_item_count=1614`, `knowledge_commit_count=553`, `repository_count=9`,
  `entity_count=32`, `project_count=79`, and `expected_generated_file_count=2291`.
- Corrected marker scan using `<!-- engram:generated:file memory-vault-v1 -->` and frontmatter
  scan returned no missing files.
- The three target vault pages now have `status: "archived"`.

## Completion Matrix Delta

| Area | State After T279 | Evidence | Remaining Gate |
| --- | --- | --- | --- |
| T234/T247/T248 exact lifecycle targets | Closed | Three exact archives, changes_since, post-archive search, vault status | No further action for those target IDs |
| Broad lifecycle cleanup | Partially closed, not exhaustive | Global lint still reports unrelated lifecycle/open-obligation pressure | Future exact-target review or explicit deferral; no broad `lint apply_safe` |
| M6 current review batch | Still closed by T278 | T279 did not rerun M6 | Direct legacy deprecation/deletion remains separate |
| Canonical vault | Refreshed after lifecycle archive and KnowledgeCommit | Final status 2,291 generated files, zero user files | Future update policy remains separate |

## Out Of Scope

T279 does not prove broad lifecycle cleanup is complete. It closes only the three already-packeted
T234/T247/T248 targets. Native Claude prompt-bearing execution, effective-hook visibility, live
Claude host-label proof, branch publication/upstream/PR, and direct legacy deprecation/deletion
remain separate gates.
