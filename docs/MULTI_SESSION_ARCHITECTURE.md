# Multi-Session Architecture Analysis

## Executive Summary

**Problem**: RocksDB's exclusive file lock prevents multiple Claude Code sessions from using engram simultaneously. Each session spawns a new engram process, but only one can hold the database lock.

**Recommended Solution**: **SurrealDB Server Mode** (Option 3) - Run SurrealDB as a local server process, allowing multiple engram clients to connect concurrently. This provides the best balance of simplicity, performance, and compatibility with existing code.

**Alternative**: **HTTP/SSE Transport with Daemon** (Option 1) - Run engram as a daemon with HTTP/SSE transport. More complex but provides better isolation and control.

---

## Current Architecture

```
┌─────────────┐         ┌─────────────┐
│ Claude Code │         │ Claude Code │
│  Session A  │         │  Session B  │
└──────┬──────┘         └──────┬──────┘
       │                      │
       │ stdio                │ stdio
       │                      │
       ▼                      ▼
┌─────────────┐         ┌─────────────┐
│  engram     │         │  engram     │
│  Process A  │         │  Process B  │
└──────┬──────┘         └──────┬──────┘
       │                      │
       └──────────┬───────────┘
                  │
                  ▼
         ┌─────────────────┐
         │ SurrealDB        │
         │ (Embedded)       │
         │ RocksDB Backend  │
         │ ~/.engram/data/  │
         └─────────────────┘
              ❌ EXCLUSIVE LOCK
```

**Current Flow:**
1. Each Claude Code session spawns `engram serve` via stdio
2. Each engram process connects to embedded SurrealDB with RocksDB backend
3. RocksDB enforces exclusive file lock - only one process can access
4. Second session fails to start: "database lock held by another process"

---

## Solution Evaluation

### Option 1: HTTP/SSE Transport with Daemon ⭐⭐⭐⭐

**Architecture:**
```
┌─────────────┐         ┌─────────────┐
│ Claude Code │         │ Claude Code │
│  Session A  │         │  Session B  │
└──────┬──────┘         └──────┬──────┘
       │                      │
       │ HTTP/SSE             │ HTTP/SSE
       │ (localhost:8000)    │ (localhost:8000)
       │                      │
       └──────────┬───────────┘
                  │
                  ▼
         ┌─────────────────┐
         │  engram daemon   │
         │  (single proc)   │
         └─────────┬───────┘
                   │
                   ▼
         ┌─────────────────┐
         │ SurrealDB        │
         │ (Embedded)       │
         │ RocksDB Backend  │
         └─────────────────┘
```

**Implementation:**
- Run `engram daemon` as a background service
- Clients connect via HTTP/SSE using rmcp's `transport-sse` feature
- Single engram process holds the database lock
- Multiple clients share the same server instance

**Pros:**
- ✅ Single database connection (efficient)
- ✅ Shared state across all sessions (true knowledge sharing)
- ✅ rmcp supports SSE transport (`transport-sse-server` feature)
- ✅ Better resource management (one process vs many)
- ✅ Can add authentication/rate limiting later
- ✅ Easier to monitor and debug (single process)

**Cons:**
- ⚠️ Requires daemon management (start/stop/restart)
- ⚠️ More complex deployment (background service)
- ⚠️ Need to handle daemon lifecycle (crash recovery, auto-start)
- ⚠️ MCP clients need to support HTTP/SSE transport (Claude Code compatibility?)
- ⚠️ Port management (conflicts, firewall)

**rmcp Support:**
- ✅ `transport-sse` and `transport-sse-server` features available
- ✅ Can use `rmcp::transport::sse::Server` for HTTP/SSE server
- ⚠️ Need to verify Claude Code MCP client supports SSE transport

**Code Changes:**
```rust
// New daemon command
engram daemon --port 8000

// Server implementation
use rmcp::transport::sse::Server;
let server = Server::bind("127.0.0.1:8000").await?;
let service = engram_server.serve(server).await?;
```

**Client Configuration:**
```json
{
  "mcpServers": {
    "engram": {
      "url": "http://localhost:8000/sse"
    }
  }
}
```

**Compatibility Risk:** ⚠️ **MEDIUM** - Depends on Claude Code MCP client supporting HTTP/SSE transport. May need to check MCP spec compliance.

---

### Option 2: Different Database Backend ⭐⭐⭐

**Architecture:**
- Switch from RocksDB to SQLite with WAL mode (multiple readers, single writer)
- Or use SurrealDB with different backend (if available)

**SQLite WAL Mode:**
- ✅ Multiple readers simultaneously
- ✅ Single writer (writes queue, readers continue)
- ✅ Well-tested, reliable
- ⚠️ SurrealDB doesn't support SQLite backend natively
- ⚠️ Would require significant refactoring or switching away from SurrealDB

**SurrealDB Backends:**
- Current: `kv-rocksdb` (embedded, exclusive lock)
- Available: `kv-mem` (in-memory, not persistent)
- Server mode: HTTP/WebSocket (see Option 3)

**Verdict:** ❌ **Not viable** - SurrealDB doesn't support SQLite backend. Would require abandoning SurrealDB entirely, losing graph/vector capabilities.

---

### Option 3: SurrealDB Server Mode ⭐⭐⭐⭐⭐ **RECOMMENDED**

**Architecture:**
```
┌─────────────┐         ┌─────────────┐
│ Claude Code │         │ Claude Code │
│  Session A  │         │  Session B  │
└──────┬──────┘         └──────┬──────┘
       │                      │
       │ stdio                │ stdio
       │                      │
       ▼                      ▼
┌─────────────┐         ┌─────────────┐
│  engram     │         │  engram     │
│  Process A  │         │  Process B  │
└──────┬──────┘         └──────┬──────┘
       │                      │
       │ ws://localhost:8000  │ ws://localhost:8000
       │                      │
       └──────────┬───────────┘
                  │
                  ▼
         ┌─────────────────┐
         │ SurrealDB Server │
         │ (single proc)    │
         │ RocksDB Backend  │
         │ ~/.engram/data/  │
         └─────────────────┘
```

**Implementation:**
- Run SurrealDB as a local server: `surreal start --bind 0.0.0.0:8000 --log info file:~/.engram/data/`
- Each engram process connects via WebSocket: `ws://localhost:8000`
- Multiple engram processes can connect simultaneously
- Single SurrealDB process holds the database lock

**Pros:**
- ✅ **Minimal code changes** - Only connection string changes
- ✅ **Proven architecture** - SurrealDB designed for this
- ✅ **Multiple concurrent clients** - Native support
- ✅ **Keep existing stdio transport** - No MCP changes needed
- ✅ **Same database features** - Graph, vector, relational all work
- ✅ **Production-ready** - SurrealDB server mode is stable
- ✅ **Easy to manage** - Standard server lifecycle

**Cons:**
- ⚠️ Need to manage SurrealDB server process (start/stop)
- ⚠️ Additional process running (but lightweight)
- ⚠️ Network overhead (localhost WebSocket, minimal)

**Code Changes:**
```rust
// engram-store/src/config.rs
pub enum StorageBackend {
    Memory,
    RocksDb(PathBuf),
    Server { url: String }, // NEW
}

impl StoreConfig {
    pub fn server(url: impl Into<String>) -> Self {
        Self {
            backend: StorageBackend::Server { url: url.into() },
            ..Default::default()
        }
    }
    
    pub fn connection_string(&self) -> String {
        match &self.backend {
            StorageBackend::Memory => "mem://".to_string(),
            StorageBackend::RocksDb(path) => format!("rocksdb://{}", path.display()),
            StorageBackend::Server { url } => url.clone(), // e.g., "ws://localhost:8000"
        }
    }
}
```

**CLI Changes:**
```rust
// engram-cli/src/main.rs
Commands::Serve { 
    memory: bool,
    server_url: Option<String>, // NEW: --server-url ws://localhost:8000
} => {
    let config = if memory {
        StoreConfig::memory()
    } else if let Some(url) = server_url {
        StoreConfig::server(url)
    } else {
        // Check if SurrealDB server is running, use it
        // Otherwise, fall back to embedded mode
        StoreConfig::rocksdb(StoreConfig::default_data_dir())
    };
    // ... rest of serve logic
}

// New command to start SurrealDB server
Commands::StartServer {
    port: Option<u16>,
} => {
    // Start SurrealDB server process
    // surrealdb start --bind 0.0.0.0:{port} file:~/.engram/data/
}
```

**User Workflow:**
```bash
# Option A: Manual server start
engram start-server  # Starts SurrealDB server in background
engram serve         # Connects to server automatically

# Option B: Auto-detect and use server
engram serve         # Checks for server, uses if available, else embedded

# Option C: Explicit server URL
engram serve --server-url ws://localhost:8000
```

**Compatibility Risk:** ✅ **LOW** - No MCP transport changes, only database connection string.

**SurrealDB Feature Verification:**
- ✅ SurrealDB v2 supports WebSocket connections (`protocol-ws` feature)
- ✅ Connection strings: `ws://localhost:8000` or `http://localhost:8000`
- ✅ Current Cargo.toml already includes `kv-rocksdb` feature
- ⚠️ Need to add `protocol-ws` feature to surrealdb dependency for client connections

**Required Cargo.toml Changes:**
```toml
# Cargo.toml
surrealdb = { version = "2", features = ["kv-rocksdb", "kv-mem", "protocol-ws"] }
```

---

### Option 4: Named Pipes/Unix Sockets ⭐⭐

**Architecture:**
- Single engram process listens on Unix socket
- Clients connect via socket
- Requires custom IPC protocol (not MCP standard)

**Pros:**
- ✅ Efficient (no network overhead)
- ✅ Single database connection

**Cons:**
- ❌ **Not MCP-compliant** - Would need custom protocol
- ❌ rmcp doesn't support Unix sockets natively
- ❌ Platform-specific (Windows vs Unix)
- ❌ More complex implementation

**Verdict:** ❌ **Not recommended** - Breaks MCP standard, too complex.

---

### Option 5: Separate DBs Per Project with Optional Sync ⭐⭐

**Architecture:**
- Each project gets its own database: `~/.engram/data/project-a/`, `~/.engram/data/project-b/`
- Optional sync mechanism to share knowledge between projects
- Use `--project` flag to specify database

**Pros:**
- ✅ Solves the immediate problem (no lock conflicts)
- ✅ Project isolation (good for exception case)
- ✅ Simple implementation

**Cons:**
- ❌ **Doesn't solve shared knowledge** - Each project isolated
- ❌ Sync complexity (when/how/what to sync)
- ❌ Data duplication
- ❌ Not what user wants (they want SHARED knowledge)

**Verdict:** ❌ **Doesn't meet requirements** - User wants shared knowledge, not isolation.

**However:** This is perfect for the **exception case** (project-specific, non-shared mode).

---

## Recommended Architecture

### Primary Solution: SurrealDB Server Mode (Option 3)

**Why:**
1. **Minimal changes** - Only connection string, no MCP transport changes
2. **Proven** - SurrealDB designed for multi-client access
3. **Compatible** - Works with existing stdio transport
4. **Flexible** - Can still use embedded mode for single-session use

### Implementation Plan

#### Phase 1: Add Server Mode Support
1. Extend `StorageBackend` enum to support server URLs
2. Update `StoreConfig` to handle `ws://` and `http://` connections
3. Add `--server-url` flag to `engram serve`
4. Auto-detect running SurrealDB server (check `ws://localhost:8000`)

#### Phase 2: Server Management
1. Add `engram start-server` command
2. Manage SurrealDB server process (start/stop/status)
3. Optional: Auto-start server if not running

#### Phase 3: Project-Specific Mode (Exception Case)
1. Add `--project <name>` flag to `engram serve`
2. Use separate database per project: `~/.engram/data/project-<name>/`
3. Document that project mode = isolated, no sharing

#### Phase 4: Migration Path
1. Detect existing embedded database
2. Offer migration to server mode
3. Maintain backward compatibility (embedded still works)

---

## Exception Case: Project-Specific Mode

**Requirement:** "Need a flag/config to define engram for a specific project (persists but NOT shared)"

**Solution:**
```bash
# Project-specific mode (isolated database)
engram serve --project my-project

# Uses: ~/.engram/data/project-my-project/
# No sharing with other projects
# Still uses SurrealDB server mode (for consistency)
```

**Implementation:**
```rust
Commands::Serve {
    memory: bool,
    server_url: Option<String>,
    project: Option<String>, // NEW
} => {
    let db_path = if let Some(project) = project {
        // Project-specific database
        StoreConfig::default_data_dir()
            .parent()
            .unwrap()
            .join(format!("data-project-{}", project))
    } else {
        // Shared database
        StoreConfig::default_data_dir()
    };
    
    let config = if memory {
        StoreConfig::memory()
    } else if let Some(url) = server_url {
        StoreConfig::server(url)
    } else {
        StoreConfig::rocksdb(db_path)
    };
    // ...
}
```

**Configuration File Option:**
```toml
# ~/.engram/config.toml
[default]
server_url = "ws://localhost:8000"

[project.my-project]
database_path = "~/.engram/data-project-my-project/"
shared = false
```

---

## Comparison Matrix

| Solution | Code Changes | Complexity | Performance | Compatibility | Shared State |
|----------|-------------|------------|-------------|---------------|--------------|
| **1. HTTP/SSE Daemon** | Medium | Medium | High | ⚠️ Medium | ✅ Yes |
| **2. Different DB** | High | High | Medium | ✅ High | ✅ Yes |
| **3. SurrealDB Server** ⭐ | **Low** | **Low** | **High** | ✅ **High** | ✅ **Yes** |
| **4. Unix Sockets** | High | High | High | ❌ Low | ✅ Yes |
| **5. Separate DBs** | Low | Low | Medium | ✅ High | ❌ No |

---

## Questions Answered

### 1. What is the BEST architecture for multi-session MCP servers with shared state?

**Answer:** **SurrealDB Server Mode** - Run the database as a server process, connect multiple MCP server instances via WebSocket. This is the standard pattern for multi-client database access.

### 2. Does the rmcp crate support HTTP/SSE transport?

**Answer:** ✅ **Yes** - rmcp supports `transport-sse` and `transport-sse-server` features. However, this requires MCP clients (Claude Code) to support SSE transport, which may not be guaranteed.

### 3. What are the trade-offs between the potential solutions?

**Answer:** See comparison matrix above. **SurrealDB Server Mode** offers the best balance: minimal code changes, high compatibility, proven architecture.

### 4. How should we handle the exception case (project-specific non-shared mode)?

**Answer:** Add `--project <name>` flag that uses a separate database directory (`~/.engram/data-project-<name>/`). This provides isolation while maintaining the same architecture.

### 5. What's the recommended approach for production MCP servers needing concurrency?

**Answer:** **Database server mode** - This is the standard pattern. The database handles concurrency, MCP servers remain stateless clients. This scales better and is easier to manage.

---

## Implementation Checklist

### Phase 1: Core Server Mode Support
- [ ] Add `StorageBackend::Server { url: String }` variant
- [ ] Update `StoreConfig::connection_string()` to handle server URLs
- [ ] Add `--server-url` flag to `engram serve`
- [ ] Test connection to SurrealDB server
- [ ] Update documentation

### Phase 2: Server Management
- [ ] Add `engram start-server` command
- [ ] Implement server process management (start/stop/status)
- [ ] Add auto-detection of running server
- [ ] Handle server lifecycle (crash recovery)

### Phase 3: Project-Specific Mode
- [ ] Add `--project <name>` flag
- [ ] Implement project-specific database paths
- [ ] Update configuration system
- [ ] Document project isolation behavior

### Phase 4: Migration & Compatibility
- [ ] Maintain backward compatibility (embedded mode)
- [ ] Add migration path documentation
- [ ] Update CLI help text
- [ ] Add examples for both modes

---

## Next Steps

1. ✅ **SurrealDB server mode validated** - SurrealDB v2 supports WebSocket (`protocol-ws` feature)
2. **Add protocol-ws feature** - Update `Cargo.toml` to include `protocol-ws` feature
3. **Prototype server mode** - Implement Phase 1 changes and test with multiple concurrent sessions
4. **Test SurrealDB server startup** - Verify `surreal start` command works with RocksDB backend
5. **User testing** - Get feedback on server management UX

## Quick Validation Test

To verify SurrealDB server mode works:

```bash
# Terminal 1: Start SurrealDB server
surreal start --bind 0.0.0.0:8000 --log info file:~/.engram/data/

# Terminal 2: Test connection (using SurrealDB CLI)
surreal sql --conn ws://localhost:8000 --user root --pass root

# Or test from Rust code:
let db = Surreal::init();
db.connect("ws://localhost:8000").await?;
db.use_ns("engram").use_db("main").await?;
```

---

## References

- [SurrealDB Rust SDK](https://surrealdb.com/docs/surrealdb/integration/sdk/rust)
- [rmcp crate documentation](https://docs.rs/rmcp)
- [MCP Specification](https://modelcontextprotocol.io/specification)
- [SurrealDB Server Mode](https://surrealdb.com/docs/surrealdb/installation/running/binary)
