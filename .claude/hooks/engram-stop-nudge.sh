#!/usr/bin/env bash
# engram:harness-adapter:v1
set -euo pipefail

INPUT=$(cat)
STOP_HOOK_ACTIVE=$(printf '%s' "$INPUT" | jq -r '.stop_hook_active // false')

if [ "$STOP_HOOK_ACTIVE" = "true" ]; then
  cat <<'EOF'
{
  "continue": true,
  "systemMessage": "Engram final-response check already ran for this Stop turn."
}
EOF
  exit 0
fi

cat <<'EOF'
{
  "continue": true,
  "systemMessage": "Engram final-response check: call memory(action=changes_since), obligations(action=detect), and obligations(action=doctor); resolve or explicitly skip open obligations, update handoff if context would be lost, then answer."
}
EOF
