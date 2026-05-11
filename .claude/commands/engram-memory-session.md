<!-- engram:harness-adapter:v1 -->
# Engram Memory Session

Use this command when a Claude Code session needs persistent project memory.

Lifecycle contract:
- At task/session start, call `orient` with the project, cwd, prompt, and harness.
- Before major decisions, call `memory(action=changes_since)` with the orientation cursor.
- After non-obvious discoveries, record source-grounded memory or a session event.
- Before final response, call `changes_since`; if relevant updates appeared, account for them.
- Before final response, call `obligations(action=detect)` and `obligations(action=doctor)`;
  resolve open obligations or report explicit skip reasons.
- Before context compaction, context transition, or any expected loss of conversation state,
  update `handoff` and record/commit compact durable memory for future sessions.
- At session end, compile a handoff and create a knowledge commit candidate.
- In commit workflows, consult memory for relevant preferences, rules, and limitations first.

This is a soft contract. Missing lifecycle steps should be reported as warnings, not blockers.
