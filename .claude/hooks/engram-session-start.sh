#!/usr/bin/env bash
# engram:harness-adapter:v1
set -euo pipefail

INPUT=$(cat)
CWD=$(printf '%s' "$INPUT" | jq -r '.cwd // empty')
SOURCE=$(printf '%s' "$INPUT" | jq -r '.source // empty')
SESSION_ID=$(printf '%s' "$INPUT" | jq -r '.session_id // empty')

if [ -z "$CWD" ] || [ "$CWD" = "null" ]; then
  CWD="${CLAUDE_PROJECT_DIR:-}"
fi

PROJECT_NAME=""
if [ -n "$CWD" ] && [ "$CWD" != "null" ]; then
  PROJECT_NAME=$(basename "$CWD")
fi

CONTEXT="<engram_session_activation source=\"$SOURCE\" project=\"$PROJECT_NAME\" session_id=\"$SESSION_ID\">
Engram is the durable Memory OS for this Claude Code session.
Before making claims or edits, call the Engram MCP orient tool with project, cwd, prompt, and agent=claude_code.
Keep the returned memory cursor and use memory(action=changes_since) before major decisions and before final response.
Keep returned trace_id values from orient/search and submit telemetry(action=submit_feedback) with task_success, preference_adhered, repeated_context_questions, bad_memory_used, missing_context, used_memory_ids, rejected_memory_ids, stale_memory_ids, and wrong_scope_memory_ids before final response when those outcomes or attribution judgments can be made.
Use used_memory_ids for returned memory that shaped the answer, implementation, safety decision, or plan; leave it empty only when no returned memory influenced behavior.
Use obligations(action=detect) for source/design reading, durable document disposition, failed tool recovery, verification, handoff, and commit preference checks.
When the current method, plan, or next action should survive resume, use memory(action=capture_current_plan) with compact content and file/tool/manual-review evidence.
Before context compaction or session end, update handoff and commit compact durable memory when useful.
This is a soft contract: resolve obligations or state explicit skip reasons; do not fabricate missing memory.
</engram_session_activation>"

CONTEXT_JSON=$(printf '%s' "$CONTEXT" | jq -Rs .)

cat <<EOF
{
  "continue": true,
  "systemMessage": $CONTEXT_JSON
}
EOF
