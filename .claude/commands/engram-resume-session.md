<!-- engram:harness-adapter:v1 -->
# Resume Engram Session

1. Call `orient` with the explicit project and current cwd.
2. Read the returned context pack, ambiguities, and memory cursor.
3. If a rolling handoff exists, inspect `handoff(action=get)`.
4. Check `memory(action=changes_since)` during the session before major decisions.
5. Check `obligations(action=detect)` for document, tool-failure, source-reading, and design
   obligations; close or explicitly skip open items before final response.
6. Store only source-grounded decisions, rules, limitations, and non-obvious discoveries.
7. If this is a resume after compaction, first inspect `handoff(action=get)` and recent
   `memory(action=changes_since)` before continuing.
