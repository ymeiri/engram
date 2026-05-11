<!-- engram:harness-adapter:v1 -->
# Engram Memory OS Harness

- Start work by calling `orient` with the current project, cwd, prompt, and harness name.
- Keep the returned memory cursor and call `memory(action=changes_since)` before major
  decisions, before final response, and during long sessions.
- Keep returned `trace_id` values from `orient` or `search` and call
  `telemetry(action=submit_feedback)` with `task_success`, `preference_adhered`,
  `repeated_context_questions`, `bad_memory_used`, `missing_context`, `used_memory_ids`, and
  `rejected_memory_ids`, plus `stale_memory_ids` and `wrong_scope_memory_ids` before final
  response when those outcomes or attribution judgments can be made. Use `used_memory_ids` for
  returned memory that shaped the answer, implementation, safety decision, or plan; leave it empty
  only when no returned memory influenced behavior.
- Record source-grounded decisions, preferences, rules, limitations, and non-obvious
  discoveries. Use writer provenance so Claude Code, Codex, and other harnesses can be
  distinguished.
- Use `memory(action=capture_current_plan)` when the current method, plan, or next action should
  survive resume.
- Use `obligations(action=detect)` at task start and before final response. Resolve or explicitly
  skip document, failed-tool, source/design reading, verification, handoff, and commit-preference
  obligations before claiming the task is done.
- Before context compaction or expected context loss, update `handoff` and record or commit
  compact durable memory for the next session.
- Maintain rolling handoffs for multi-turn work. Handoffs must include next actions.
- Keep migration review-gated. Do not auto-promote orphan, digest, or legacy data.
- Treat lifecycle enforcement as soft: warn about skipped steps, but do not block coding.
