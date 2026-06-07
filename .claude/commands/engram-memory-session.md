<!-- engram:harness-adapter:v1 -->
# Engram Memory Session

Use this command when a Claude Code session needs persistent project memory.

Lifecycle contract:
- At task/session start, call `orient` with the project, cwd, prompt, and harness.
- Keep the returned `trace_id` from `orient` or `search`; before final response, call
  `telemetry(action=submit_feedback)` with `task_success`, `preference_adhered`,
  `repeated_context_questions`, `bad_memory_used`, `missing_context`, `used_memory_ids`, and
  `rejected_memory_ids`, plus `stale_memory_ids` and `wrong_scope_memory_ids` when those
  outcomes or attribution judgments can be made. Use `used_memory_ids` for returned memory that
  shaped the answer, implementation, safety decision, or plan; leave it empty only when no returned
  memory influenced behavior.
- Before major decisions, call `memory(action=changes_since)` with the orientation cursor.
- After non-obvious discoveries, record source-grounded memory or a session event.
- When the current method, plan, or next action should survive resume, use
  `memory(action=capture_current_plan)` with compact content and file/tool/manual-review evidence.
- Before final response, call `changes_since`; if relevant updates appeared, account for them.
- Before final response, call `obligations(action=detect)` and `obligations(action=doctor)`;
  resolve open obligations or report explicit skip reasons.
- Before context compaction, context transition, or any expected loss of conversation state,
  update `handoff` and record/commit compact durable memory for future sessions.
- At session end, compile a handoff and create a knowledge commit candidate.
- In commit workflows, consult memory for relevant preferences, rules, and limitations first.

This is a soft contract. Missing lifecycle steps should be reported as warnings, not blockers.
