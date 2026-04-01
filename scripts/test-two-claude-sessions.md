# Testing Two Claude Code Sessions with Shared Engram

This document describes how to test multi-session support with two actual Claude Code sessions.

## Prerequisites

1. Ensure engram daemon is running:
   ```bash
   cd /Users/yuval.meiri/projects/engram
   ./target/debug/engram daemon status
   # Should show: Daemon status: 🟢 running

   # If not running:
   ./target/debug/engram daemon start
   ```

2. Configure Claude Code to use engram MCP server.

## Claude Code MCP Configuration

Add this to your `~/.claude.json` or project's `.mcp.json`:

```json
{
  "mcpServers": {
    "engram": {
      "command": "/Users/yuval.meiri/projects/engram/target/debug/engram",
      "args": ["serve"]
    }
  }
}
```

**Note**: The `engram serve` command (without `--http`) automatically:
1. Starts a daemon if not already running
2. Runs as a stdio-to-HTTP proxy
3. Connects to the shared daemon

## Test Procedure

### Terminal 1 (First Claude Session)

1. Open Claude Code in a terminal
2. Ask Claude to create an entity:
   ```
   Create an engram entity called "multi-session-test-repo" with type "repo"
   ```
3. Verify creation by listing entities:
   ```
   List all engram entities
   ```

### Terminal 2 (Second Claude Session)

1. Open Claude Code in another terminal
2. Ask Claude to list entities:
   ```
   List all engram entities
   ```
3. You should see "multi-session-test-repo" created by the first session

### Verify Coordination (Optional)

In Terminal 1:
```
Register this session for coordination with project "test-project", components ["api", "auth"]
```

In Terminal 2:
```
Register this session for coordination with project "test-project", components ["api", "database"]
Check for coordination conflicts
```

Terminal 2 should detect a conflict on the "api" component.

## Expected Results

1. **State Sharing**: Entity created in session 1 should be visible in session 2
2. **Independent Sessions**: Each session has its own MCP session ID
3. **Conflict Detection**: Coordination system should detect overlapping work areas

## Troubleshooting

### Check daemon health
```bash
curl http://127.0.0.1:8765/health
# Should return: {"status":"ok","service":"engram","version":"0.1.0"}
```

### View daemon logs
```bash
./target/debug/engram daemon logs
```

### Restart daemon
```bash
./target/debug/engram daemon stop
./target/debug/engram daemon start
```
