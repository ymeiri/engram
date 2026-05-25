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
- Keep graph traversal, obligation detection, lint, migration, and raw entity observation lookup out
  of the normal `orient` hot path.

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

## Feedback Expectations

- Agents should preserve the `trace_id` returned by `orient` so feedback can link to the exact
  orientation packet.
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
- Migration completion remains review-gated and should wait until this contract stays green.

## Contract Tests

- `engram-tests/tests/memory_tests.rs` covers MCP orientation, review gating, prompt-specific
  ranking, and promoted reviewed memory surfacing.
- `engram-tests/tests/obligation_tests.rs` covers obligation surfacing, bounds, `has_more`, and
  stale-obligation suppression.
- `engram-tests/tests/telemetry_tests.rs` covers orientation trace emission, feedback linkage, and
  scenario/arm tagging.
