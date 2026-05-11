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
Use obligations(action=detect) for source/design reading, durable document disposition, failed tool recovery, verification, handoff, and commit preference checks.
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
