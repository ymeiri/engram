<!-- engram:harness-adapter:v1 -->
# Engram Memory OS Harness

- Start work by calling `orient` with the current project, cwd, prompt, and harness name.
- Keep the returned memory cursor and call `memory(action=changes_since)` before major
  decisions, before final response, and during long sessions.
- Record source-grounded decisions, preferences, rules, limitations, and non-obvious
  discoveries. Use writer provenance so Claude Code, Codex, and other harnesses can be
  distinguished.
- Use `obligations(action=detect)` at task start and before final response. Resolve or explicitly
  skip document, failed-tool, source/design reading, verification, handoff, and commit-preference
  obligations before claiming the task is done.
- Before context compaction or expected context loss, update `handoff` and record or commit
  compact durable memory for the next session.
- Maintain rolling handoffs for multi-turn work. Handoffs must include next actions.
- Keep migration review-gated. Do not auto-promote orphan, digest, or legacy data.
- Treat lifecycle enforcement as soft: warn about skipped steps, but do not block coding.
