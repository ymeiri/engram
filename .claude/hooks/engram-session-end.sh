#!/usr/bin/env bash
# engram:harness-adapter:v1
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  printf '%s\n' '{"continue":true,"systemMessage":"Engram SessionEnd hook skipped: jq is unavailable."}'
  exit 0
fi

fallback() {
  local message="$1"
  jq -n --arg message "$message" '{continue: true, systemMessage: $message}'
}

if ! command -v curl >/dev/null 2>&1; then
  fallback "Engram SessionEnd hook skipped: curl is unavailable."
  exit 0
fi

INPUT=$(cat)
CWD=$(printf '%s' "$INPUT" | jq -r '.cwd // empty')
SESSION_ID=$(printf '%s' "$INPUT" | jq -r '.session_id // empty')
TRANSCRIPT_PATH=$(printf '%s' "$INPUT" | jq -r '.transcript_path // empty')
REASON=$(printf '%s' "$INPUT" | jq -r '.reason // empty')
WRITE_POLICY=$(printf '%s' "$INPUT" | jq -r '.write_policy // "durable"')

if [ -z "$CWD" ] || [ "$CWD" = "null" ]; then
  CWD="${CLAUDE_PROJECT_DIR:-}"
fi

PORT_FILE="${ENGRAM_DAEMON_PORT_FILE:-$HOME/.engram/daemon.port}"
if [ ! -r "$PORT_FILE" ]; then
  fallback "Engram SessionEnd handoff skipped: daemon port file was not found."
  exit 0
fi

PORT=$(tr -d '[:space:]' < "$PORT_FILE")
if [ -z "$PORT" ]; then
  fallback "Engram SessionEnd handoff skipped: daemon port file was empty."
  exit 0
fi

MCP_URL="http://127.0.0.1:${PORT}/mcp"
HEADERS=$(mktemp)
trap 'rm -f "$HEADERS"' EXIT

INIT_PAYLOAD=$(jq -nc '{jsonrpc:"2.0",id:1,method:"initialize",params:{protocolVersion:"2024-11-05",capabilities:{},clientInfo:{name:"engram-claude-session-end-hook",version:"1.0"}}}')
if ! curl -sS --max-time 5 -D "$HEADERS" -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' -X POST "$MCP_URL" -d "$INIT_PAYLOAD" >/dev/null; then
  fallback "Engram SessionEnd handoff skipped: could not initialize MCP session with daemon."
  exit 0
fi

MCP_SESSION_ID=$(awk 'tolower($1)=="mcp-session-id:" {print $2}' "$HEADERS" | tr -d '\r')
if [ -z "$MCP_SESSION_ID" ]; then
  fallback "Engram SessionEnd handoff skipped: daemon did not return an MCP session id."
  exit 0
fi

curl -sS --max-time 5 -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' -H "mcp-session-id: $MCP_SESSION_ID" -X POST "$MCP_URL" -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' >/dev/null || true

CALL_PAYLOAD=$(jq -nc \
  --arg session_id "$SESSION_ID" \
  --arg cwd "$CWD" \
  --arg transcript_path "$TRANSCRIPT_PATH" \
  --arg reason "$REASON" \
  --arg write_policy "$WRITE_POLICY" \
  '{jsonrpc:"2.0",id:2,method:"tools/call",params:{name:"harness",arguments:{action:"hook_event",harness:"claude_code",hook_event_name:"SessionEnd",session_id:$session_id,cwd:$cwd,transcript_path:$transcript_path,reason:$reason,write_policy:$write_policy,model_provider:"anthropic",model:"claude-code",surface:"claude-code",actor:"agent"}}}')

if ! CALL_RESPONSE=$(curl -sS --max-time 10 -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' -H "mcp-session-id: $MCP_SESSION_ID" -X POST "$MCP_URL" -d "$CALL_PAYLOAD"); then
  fallback "Engram SessionEnd handoff skipped: harness hook_event call failed."
  exit 0
fi

HOOK_JSON=$(printf '%s' "$CALL_RESPONSE" | sed -n 's/^data: //p' | jq -rs -r 'map(select(type=="object" and (.result? != null)))[0].result.content[0].text // ""' 2>/dev/null || true)
if [ -z "$HOOK_JSON" ] || ! printf '%s' "$HOOK_JSON" | jq -e . >/dev/null 2>&1; then
  fallback "Engram SessionEnd handoff attempted, but daemon returned an unreadable hook response."
  exit 0
fi

printf '%s\n' "$HOOK_JSON"
