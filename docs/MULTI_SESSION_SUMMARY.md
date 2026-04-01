# Multi-Session Support - Quick Summary

## Problem
RocksDB's exclusive file lock prevents multiple Claude Code sessions from using engram simultaneously.

## Recommended Solution: SurrealDB Server Mode

**Architecture:**
- Run SurrealDB as a local server process (single instance)
- Multiple engram processes connect via WebSocket
- Each Claude Code session spawns its own engram process (no change)
- All engram processes share the same database server

## Implementation Overview

### Phase 1: Add Server Mode Support (Minimal Changes)

**1. Update `engram-store/src/config.rs`:**
```rust
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
            StorageBackend::Server { url } => url.clone(), // "ws://localhost:8000"
        }
    }
}
```

**2. Update `Cargo.toml`:**
```toml
surrealdb = { version = "2", features = ["kv-rocksdb", "kv-mem", "protocol-ws"] }
```

**3. Add CLI flag:**
```rust
Commands::Serve {
    memory: bool,
    server_url: Option<String>, // NEW: --server-url ws://localhost:8000
} => {
    let config = if memory {
        StoreConfig::memory()
    } else if let Some(url) = server_url {
        StoreConfig::server(url)
    } else {
        StoreConfig::rocksdb(StoreConfig::default_data_dir())
    };
    // ... rest unchanged
}
```

### Phase 2: Server Management

**Add `engram start-server` command:**
```bash
engram start-server --port 8000
# Starts: surreal start --bind 0.0.0.0:8000 file:~/.engram/data/
```

**Auto-detection (optional):**
- Check if server is running on `ws://localhost:8000`
- If yes, use server mode; if no, use embedded mode

### Phase 3: Project-Specific Mode (Exception Case)

**Add `--project` flag for isolation:**
```bash
engram serve --project my-project
# Uses: ~/.engram/data-project-my-project/
# Isolated database, no sharing
```

## User Workflow

### Shared Mode (Default)
```bash
# Start SurrealDB server (one time, or auto-start)
engram start-server

# Each Claude Code session uses:
engram serve
# Automatically connects to server at ws://localhost:8000
```

### Project-Specific Mode (Isolated)
```bash
engram serve --project my-project
# Uses separate database, no sharing with other projects
```

## Benefits

✅ **Minimal code changes** - Only connection string logic  
✅ **No MCP changes** - Still uses stdio transport  
✅ **Backward compatible** - Embedded mode still works  
✅ **True sharing** - All sessions see same knowledge  
✅ **Production-ready** - SurrealDB server mode is stable  

## Migration Path

1. Existing users: Continue using embedded mode (no change)
2. New multi-session users: Start server, use server mode
3. Optional: Auto-detect and migrate automatically

## Testing

```bash
# Terminal 1: Start server
surreal start --bind 0.0.0.0:8000 file:~/.engram/data/

# Terminal 2: Test connection
engram serve --server-url ws://localhost:8000

# Terminal 3: Test second session (should work!)
engram serve --server-url ws://localhost:8000
```

## See Also

- [Full Architecture Analysis](./MULTI_SESSION_ARCHITECTURE.md) - Detailed evaluation of all options
- [SurrealDB Documentation](https://surrealdb.com/docs) - Server mode details
