<!-- engram:harness-adapter:v1 -->
# End Engram Session

Before ending:
- Call `memory(action=changes_since)` from the latest cursor.
- Call `obligations(action=detect)` and `obligations(action=doctor)`.
- Resolve open obligations or state explicit skip reasons in the handoff.
- Update or compile `handoff` with completed work, open decisions, next actions, and risks.
- If durable memory changed, prepare a `memory(action=commit)` candidate.
- Use this same flow before context compaction or any context transition.
- Leave migration and digest promotions review-gated; do not auto-promote orphan data.
