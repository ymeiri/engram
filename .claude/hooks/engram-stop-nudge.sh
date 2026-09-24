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
  "systemMessage": "Engram final-response check is advisory: if this hook surfaced open obligations, resolve them or record an explicit skip reason. Store only compact, evidenced durable memory that a future session genuinely needs, then answer."
}
EOF
