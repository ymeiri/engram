# Orient Contract

`orient` is the frictionless Brain Loop v1 entrypoint for agents at task boundaries. It should make
the next agent decision better without forcing the agent to choose among specialist memory tools.

## Hot Path Guarantees

- Return a scoped orientation packet for the supplied `project` and `cwd`.
- Return a memory cursor so long-running sessions can later ask what changed.
- Return a `trace_id` when telemetry is initialized so agents can submit retrieval/outcome
  feedback for the exact orientation they received.
- Rank active `MemoryItem` guidance by scope, prompt relevance, recency, confidence, review state,
  and freshness.
- When recent context is requested, include a bounded current-branch Git commit summary in
  repository context so resume prompts can see fresh project-plan commits without widening Brain
  Loop beyond selected `MemoryItem` guidance.
- Keep unreviewed or inferred claims out of active guidance; return them in `review_needed` with a
  recommended review action.
- Return the Brain Loop v1 projection from the same selected memory as the raw orientation arrays.
- For `intent=follow_user_preference`, repeat prompt-matching reviewed preferences in a top
  `Hot Context` section before lower-priority decision sections so the agent-visible preview
  includes the preference that should shape behavior, and surface their stable IDs in
  top-level `hot_context_ids` / `hot_context_items` before the larger `context_pack` so agents can
  reliably populate `used_memory_ids`.
- Surface already-open, currently applicable obligations as a compact summary and bounded list.
- For `intent=prepare_handoff`, present one latest applicable current-plan item across matching
  scopes, pin that current plan in Brain Loop, and keep stale current-plan guidance out of the
  lean handoff candidate IDs without mutating lifecycle state.
- When a `prepare_handoff` prompt explicitly asks for `approval gate` context, prefer active
  MemoryItems that use the same approval-gate wording over generic gate or calibration chatter.
- Keep graph traversal, obligation detection, lint, migration, and raw entity observation lookup out
  of the normal `orient` hot path.
- Keep the full orientation packet as the default response. Callers that only need read-only
  verification context can request `response_shape="lean"` to receive trace/cursor/scope,
  candidate IDs, Brain Loop guidance without repeated trust metadata, recommended actions,
  ambiguities, and the obligation summary/list.

## Bounds

- The request `limit` applies to each memory bucket returned by the orientation packet.
- Open obligations are capped at 5 entries and set `obligation_summary.has_more` when more current
  obligations exist.
- Git-status document obligations are shown only when the target is still current for the checkout.
- Untracked root instruction files such as `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` are suppressed
  from the open-obligation summary.

## Size Diagnostic

The 2026-05-25 Claude Code source-reuse smoke showed that full `orient` can be too large for
frictionless read-only verification tasks: Claude reported about 178,747 characters / 3,944 lines
and had to spill the MCP result to a file. A follow-up live daemon measurement for
`intent=verify_decision` confirmed the issue:

| Shape | Bytes | Lines | Trace |
| --- | ---: | ---: | --- |
| Default full response | 185,087 | 4,081 | `019e5f86-9d39-7aa0-bb19-48572874dba7` |
| `limit=6` full response | 86,919 | 1,898 | `019e5f86-9db5-75b0-87c4-b34f552afd43` |
| `limit=6`, no recent commits | 75,890 | 1,625 | `019e5f86-9e06-7061-9c19-93f6fb3c88b4` |

The largest default sections were `active_decisions` (68,219 bytes), `context_pack` (30,844
bytes), `recent_knowledge_commits` (26,050 bytes), `memory_metadata` (24,103 bytes), `limitations`
(11,294 bytes), and `brain_loop` (8,029 bytes). This is representation duplication, not a single
bad memory item: the same selected memory is returned as raw buckets, Markdown context, Brain Loop
items, and standalone trust metadata.

For the next design step, keep the full response available and add an explicit lean/read-only
shape instead of changing retrieval or ranking. Measured candidate envelopes from the same trace:

- Context-pack envelope with trace/cursor/obligation metadata: about 31,742 bytes.
- Structured response without raw memory buckets or context pack: about 39,249 bytes.
- Brain Loop envelope without repeated trust payloads: about 3,984 bytes.

Implementation follow-up: MCP `orient` now accepts `response_shape="lean"` for the Brain Loop
envelope. The lean response intentionally omits `context_pack`, raw memory buckets,
`memory_metadata`, and `recent_knowledge_commits`; it is a presentation option only and must not
change retrieval, ranking, trace creation, memory attribution IDs, or obligation surfacing.

Live daemon smoke after commit `168b06e` installed the new binary
(`c2c8f3370f1b87a305c223646d9c4e3e54467c89f48591140768055ce53cc76d`) and restarted the daemon on
port `8765`, PID `58736`. The same `verify_decision` request with `include_recent_commits=false`
and `limit=6` measured:

| Shape | Bytes | Lines | Trace |
| --- | ---: | ---: | --- |
| Full response | 78,025 | 1,676 | `019e632c-4221-7820-9932-8c39b407825e` |
| Lean response | 4,367 | 89 | `019e632c-4272-7552-bcbf-dcf3ba5acd75` |

The lean response preserved `trace_id`, `memory_cursor`, five candidate memory IDs, and the
obligation summary. It omitted `context_pack`, all raw memory buckets, `memory_metadata`,
`recent_knowledge_commits`, and repeated `trust` objects on Brain Loop items.

Native Codex Desktop smoke after restart confirmed the refreshed MCP schema exposes
`response_shape`. Calling `orient(response_shape="lean")` through the native Engram tool path
returned trace `019e6358-44bc-7662-88f2-bc67d08101bb`, preserved the memory cursor and five
`used_memory_candidate_ids`, returned Brain Loop guidance without trust payloads, and reported
`open_obligations=[]` / no warnings.

Native Claude Code smoke after restart confirmed the refreshed MCP schema also exposes
`response_shape` there. Calling `orient(response_shape="lean")` returned trace
`019e636a-730e-77f3-86a6-96b7beb2e3fd`, preserved the memory cursor
`019e6359-cbee-7e91-8dff-15e4a7238062`, five `used_memory_candidate_ids`, Brain Loop guidance,
`obligation_summary`, and `open_obligations`, and omitted `context_pack`, raw memory buckets,
`memory_metadata`, `recent_knowledge_commits`, and repeated `trust` payloads. The three open
obligations were prompt-derived and were explicitly skipped because this was a read-only schema
smoke with no source edits, no commit composition, and no failed tool recovery.

A later native Claude Code CLI smoke after installing binary hash
`4f3bda71eb441d492ece4b1bb5983993be9cf47802fd10cdb3484f31f7e23f9c`
confirmed the same lean shape remained usable for the current continuation prompt. Trace
`019e68fe-6150-7ab3-9df7-8339e3766c76` returned a compact inline packet whose top five Brain Loop
items included the latest current-plan memory `019e68f9-31b1-7270-9095-4f0be5ffa94b` at position
2, behind the non-gated calibration limitation. The paired direct search trace
`019e68fe-6417-7590-8331-85ddf3dd4a86` returned that current-plan memory first. Claude Bridge did
not reproduce this smoke because its project harness exposed only file-read tools, not the Engram
MCP tools; treat that as a bridge tool-exposure limitation, not as a native Claude Code MCP failure.

Real read-only verification smoke then used lean `orient` as the entrypoint for a normal status
check instead of another schema check. Trace `019e6452-d272-75b3-bdce-52abb30018db` returned scope,
cursor, Brain Loop guidance, candidate memory IDs, and `open_obligations=[]`. The agent could use
that compact packet to verify with `memory(action=changes_since)`, `obligations(action=doctor)`,
`git status --short`, and `git log -1 --oneline` that there were no newer memory writes since the
cursor, no open obligations, the latest commit was `f88b683 Record Claude Code lean orient smoke`,
and the only working-tree item was the user-owned untracked root `AGENTS.md`. The run also exposed
a ranking caveat: the latest current-plan memory was not in the returned top five for this
next-step verification prompt; older BAF target memories outranked it. That did not block this
read-only check, but it showed that lean `orient` still needed current-plan calibration before it
could be the only planning input for broader product steps.
The diagnostic fixture
`orient_mission_prompt_diagnostic_distinguishes_intent_from_ranking` now preserves this distinction:
explicit current-plan prompts and `resume_session` intent return the latest current plan first.
Mission-class `plan_work` prompts now promote the latest current plan within active decisions and
include it in used memory candidates, but they do not use the `resume_session` Brain Loop pin.
The stale BAF sealed-target memories that previously outranked current-plan context have also been
superseded by accepted outcome memories, so they should not appear as active implementation targets.
Direct unified `search` now has separate continuation-prompt fixtures for scoped current-plan
ranking, including a `non-gated` continuation wording fixture that preserves migration-gate
queries; that calibration does not expand or change this `orient` contract.
Current-plan lifecycle management now uses the same operational meaning as search ranking: only
active `decision` or `rule` MemoryItems with the `current-plan` tag are treated as current-plan
guidance for capture supersession and orientation post-prioritization. Other active memory kinds
with that tag remain normal evidence and are not auto-superseded by
`memory(action=capture_current_plan)`.

Installed-runtime T39 validation then hardened the `prepare_handoff` approval-gate wording path.
After installing binary hash
`d9db0ee830ef261c582e31f0c327f8198d4b6d1f556f11820bcec27fc64dfe42`, Codex trace
`019e7ce5-4d19-7060-aa12-ab0f6d9b5695` and native Claude Code `2.1.158` trace
`019e7ce5-b4e4-7830-94a4-48f87ebf56b2` both surfaced harness-write gate
`019e7cde-b517-77d0-aaac-c8638811d4e8` and M6 gate
`019e7ce5-155d-7a10-85f5-00b9dcc69cd0`, and omitted stale current-plan memory
`019e5e0a-86b4-73e3-aa9b-ca350e83e915`. After current-plan capture, Codex trace
`019e7ceb-fda5-79b1-a997-725a9914840e` returned the new T39 current-plan memory
`019e7ceb-d8bb-73f0-960c-85b667b872de` first with both gates still present. This is still
prompt-class validation only; it does not make `orient` a generated handoff or approval-audit tool.

## Feedback Expectations

- Agents should preserve the `trace_id` returned by `orient` so feedback can link to the exact
  orientation packet.
- Agents should pass `memory_cursor.timestamp` to `memory(action="changes_since")`; if
  `memory_cursor.commit_id` is present, pass it as additional context, not as a replacement for the
  timestamp. Memory item freshness is timestamp-based.
- When the task result is assessable, submit `telemetry(action=submit_feedback)` before final
  response.
- Include key behavioral fields when known: `task_success`, `preference_adhered`,
  `repeated_context_questions`, `bad_memory_used`, and `missing_context`.
- Include `used_memory_ids` for every returned memory item that materially shaped the answer,
  implementation, safety decision, or plan. This includes current-plan, preference, and rule items
  used indirectly; leave it empty only when no returned memory influenced behavior.
- Include `rejected_memory_ids` for returned memory items that were considered but not used because
  they were stale, noisy, wrong-scope, or irrelevant.
- Treat agent feedback as a weak signal, not ground truth; correlate it with the transcript, tests,
  user judgment, or later memory edits before using it for ranking or migration decisions.

## Non-Contract For Now

- `orient` does not enforce a byte or token budget yet; tests should not pretend that it does.
- `orient` does not promote entity observations or migrated data into active memory.
- `orient(intent=prepare_handoff)` is a compact orientation packet, not a generated handoff or
  approval audit. It does not run lint, migration, harness doctor, graph traversal, obligation
  detection, or gate synthesis; explicit gates must already exist as active MemoryItems to appear.
- Migration completion remains review-gated and should wait until this contract stays green.

## Contract Tests

- `engram-tests/tests/memory_tests.rs` covers MCP orientation, review gating, prompt-specific
  ranking, and promoted reviewed memory surfacing.
- `engram-tests/tests/obligation_tests.rs` covers obligation surfacing, bounds, `has_more`, and
  stale-obligation suppression.
- `engram-tests/tests/telemetry_tests.rs` covers orientation trace emission, feedback linkage, and
  scenario/arm tagging.
