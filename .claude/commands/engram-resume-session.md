<!-- engram:harness-adapter:v1 -->
# Resume Engram Session

1. Call `orient` with the explicit project and current cwd.
2. Read the returned context pack, ambiguities, and memory cursor.
3. Keep returned `trace_id` values from `orient` or `search`; submit telemetry feedback with
   outcome, gap, and attribution fields before final response when memory quality can be judged.
   Include `used_memory_ids` for returned memory that shaped behavior and `rejected_memory_ids` for
   returned memory considered but not used. Include `stale_memory_ids` and
   `wrong_scope_memory_ids` for rejected memory specifically judged stale or out of scope.
4. If a rolling handoff exists, inspect `handoff(action=get)`.
5. Check `memory(action=changes_since)` during the session before major decisions.
6. Check `obligations(action=detect)` for document, tool-failure, source-reading, and design
   obligations; close or explicitly skip open items before final response.
7. Store only source-grounded decisions, rules, limitations, and non-obvious discoveries.
   Use `memory(action=capture_current_plan)` for compact current method, plan, or next-action
   guidance that should surface on the next resume.
8. If this is a resume after compaction, first inspect `handoff(action=get)` and recent
   `memory(action=changes_since)` before continuing.
