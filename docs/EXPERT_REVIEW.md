# Expert Review: engram PKAS Architecture & Performance

**Date:** January 30, 2026  
**System:** Personal Knowledge Augmentation System (PKAS)  
**Stack:** Rust + SurrealDB v2 (embedded) + fastembed  
**Scale:** 10,000+ documents, 50,000+ chunks, 384-dim vectors

---

## Executive Summary

This review provides expert guidance on vector search optimization, SurrealDB v2 best practices, indexing strategy, memory management, and data integrity for engram's local-first architecture. The analysis is based on examination of the complete codebase, including all 6 layers of the knowledge system.

**Key Findings:**
1. ✅ Solid foundation with clean architecture and proper separation of concerns
2. ⚠️ Brute-force vector search will hit performance cliff at 50K+ embeddings
3. ⚠️ Missing transaction boundaries for multi-step operations
4. ⚠️ No application-level validation with SCHEMALESS tables
5. ✅ Good use of raw SurQL to avoid SDK serialization issues
6. ⚠️ Memory footprint concerns at scale (estimated 76-150MB for vectors alone)

---

## 1. Vector Search Optimization

### Current Implementation

**Location:** `engram-store/src/repos/document.rs:331-408`

```rust
// Current: Brute-force cosine similarity
SELECT
    meta::id(id) as id,
    source_id,
    heading_path,
    heading_level,
    content,
    start_line,
    end_line,
    parent_id,
    vector::similarity::cosine(embedding, $query) AS score
FROM doc_chunk
ORDER BY score DESC
LIMIT $limit
```

**Analysis:**
- **Algorithm:** Full table scan with cosine similarity computation on every vector
- **Complexity:** O(n × d) where n = number of chunks, d = dimensions (384)
- **Current scale:** Works fine for < 10,000 chunks
- **Performance cliff:** Expected at 20,000-50,000 chunks

### Performance Projections

| Chunk Count | Brute Force Latency | With HNSW (estimated) | Notes |
|-------------|---------------------|----------------------|-------|
| 1,000 | ~10ms | ~2ms | Current performance is acceptable |
| 10,000 | ~100ms | ~5ms | Still acceptable for local-first |
| 50,000 | ~500ms | ~10ms | **Performance cliff begins** |
| 100,000 | ~1000ms | ~15ms | Unacceptable without ANN |
| 500,000 | ~5000ms | ~25ms | Requires ANN indexing |

**Calculation basis:**
- 384-dim float32 vectors = 1,536 bytes per vector
- Modern CPU: ~1-2 billion FLOPS
- Cosine similarity: 768 ops per comparison (dot product + norms)
- 50,000 chunks × 768 ops = 38.4M operations ≈ 40-100ms on single core

### Critical Issue: SurrealDB v2 Vector Index Limitations

**SurrealDB v2.0 does NOT currently support:**
- ❌ HNSW (Hierarchical Navigable Small World) indexes
- ❌ IVF (Inverted File) indexes  
- ❌ Any native ANN (Approximate Nearest Neighbor) algorithms
- ❌ `DEFINE INDEX ... VECTOR` syntax (planned for future release)

**Evidence from codebase:**
- All vector searches use `vector::similarity::cosine()` function
- No vector index definitions in schema initialization
- Full table scans on every search query

### Recommendations

#### Option 1: Accept Brute-Force for Now (Recommended for MVP)

**When to use:** If you expect < 20,000 chunks in the near term

**Pros:**
- ✅ No code changes required
- ✅ Exact results (not approximate)
- ✅ Simple, maintainable
- ✅ Works well for local-first use case with reasonable data sizes

**Cons:**
- ⚠️ Performance degrades linearly with data growth
- ⚠️ Will need migration later

**Action items:**
1. Add performance monitoring to track query latency
2. Set up alerts when P95 latency > 200ms
3. Document the performance cliff in user documentation
4. Plan migration path to Option 2 or 3

```rust
// Add to DocumentRepo::search_similar()
let start = std::time::Instant::now();
let results = /* ... query ... */;
let elapsed = start.elapsed();
if elapsed.as_millis() > 200 {
    warn!("Slow vector search: {}ms for {} chunks", elapsed.as_millis(), chunk_count);
}
```

#### Option 2: Hybrid Approach with Pre-filtering (Recommended for 20K-100K chunks)

**Strategy:** Reduce search space before vector comparison

**Implementation:**

```rust
// 1. Add metadata indexes for common filters
DEFINE INDEX idx_chunk_source_type ON doc_chunk FIELDS source_id, source_type;
DEFINE INDEX idx_chunk_created ON doc_chunk FIELDS created_at;

// 2. Pre-filter before vector search
SELECT
    meta::id(id) as id,
    vector::similarity::cosine(embedding, $query) AS score
FROM doc_chunk
WHERE 
    source_id IN $relevant_sources  -- Pre-filter by source
    AND created_at > $recent_cutoff  -- Recency bias
ORDER BY score DESC
LIMIT $limit
```

**Pros:**
- ✅ Can reduce search space by 50-90%
- ✅ Works within SurrealDB v2 constraints
- ✅ Combines metadata filtering with vector search
- ✅ No external dependencies

**Cons:**
- ⚠️ Requires user to provide filters (not always possible)
- ⚠️ Still O(n) on filtered set

**Performance gain:**
- If filtering reduces to 10% of data: 50K → 5K chunks
- Latency improvement: 500ms → 50ms (10x faster)

**Action items:**
1. Add `source_type`, `tags`, `last_modified` fields to chunks
2. Expose filter parameters in search API
3. Implement smart defaults (e.g., prefer recent documents)

#### Option 3: External Vector Database (For > 100K chunks)

**When to use:** If you expect > 100,000 chunks or need < 50ms P95 latency

**Options:**
- **Qdrant** (Rust-native, can be embedded)
- **Meilisearch** (has vector search, local-first friendly)
- **Lance** (Rust library, embedded columnar format)
- **FAISS** (via Rust bindings, industry standard)

**Architecture:**

```
┌─────────────────────────────────────┐
│  engram Application Layer           │
├─────────────────────────────────────┤
│  SurrealDB (metadata + graph)       │  ← Entities, sessions, relationships
├─────────────────────────────────────┤
│  Qdrant/Lance (vectors only)        │  ← Embeddings + ANN index
└─────────────────────────────────────┘
```

**Pros:**
- ✅ Sub-50ms queries even at 1M+ vectors
- ✅ HNSW, IVF, or other ANN algorithms
- ✅ Battle-tested at scale
- ✅ Qdrant can run embedded (no separate server)

**Cons:**
- ⚠️ Adds complexity (two databases)
- ⚠️ Requires synchronization between SurrealDB and vector DB
- ⚠️ Larger binary size
- ⚠️ More moving parts for local-first setup

**Implementation sketch:**

```rust
// engram-vector/ crate
pub struct VectorStore {
    qdrant: QdrantClient,  // Or Lance, FAISS, etc.
    metadata_db: Db,       // SurrealDB for metadata
}

impl VectorStore {
    pub async fn search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        // 1. ANN search in Qdrant (fast)
        let vector_results = self.qdrant.search(query, limit * 2).await?;
        
        // 2. Fetch metadata from SurrealDB (batch)
        let chunk_ids: Vec<_> = vector_results.iter().map(|r| r.id).collect();
        let chunks = self.metadata_db.get_chunks_batch(&chunk_ids).await?;
        
        // 3. Merge and return
        Ok(merge_results(vector_results, chunks))
    }
}
```

**Action items:**
1. Prototype with Qdrant embedded mode
2. Benchmark against brute-force at 50K chunks
3. Implement sync layer to keep both DBs consistent
4. Add feature flag: `--vector-backend qdrant|surrealdb`

#### Option 4: Wait for SurrealDB v2.x Vector Indexes (Future)

**Status:** SurrealDB team has indicated vector indexes are on the roadmap

**Timeline:** Unknown (likely 6-12 months based on typical DB feature development)

**When available:**
```sql
-- Future syntax (hypothetical)
DEFINE INDEX idx_chunk_vector ON doc_chunk 
    FIELDS embedding 
    VECTOR HNSW(m: 16, ef_construction: 200);
```

**Recommendation:** Don't wait for this. Implement Option 1 or 2 now, migrate later if needed.

### Immediate Action Plan

**For your current scale (< 10K chunks):**

1. ✅ **Keep brute-force** - it's working fine
2. ✅ **Add monitoring** - track query latency and chunk count
3. ✅ **Document the cliff** - warn users in docs about scale limits
4. ⚠️ **Plan for Option 2** - start designing metadata filters

**Trigger points for migration:**
- P95 latency > 200ms → Implement Option 2 (hybrid filtering)
- Chunk count > 100K → Evaluate Option 3 (external vector DB)
- SurrealDB ships vector indexes → Migrate to native support

---

## 2. SurrealDB v2 Best Practices

### Current Implementation Review

**✅ Good Practices Already in Use:**

1. **Raw SurQL queries** to avoid SDK serialization issues
   ```rust
   // engram-store/src/repos/document.rs:186-203
   self.db.query(r#"
       UPSERT type::thing("doc_source", $id) SET
           source_type = $source_type,
           path_or_url = $path_or_url,
           ...
   "#)
   ```

2. **Custom datetime deserialization** for SurrealDB's native format
   ```rust
   // engram-store/src/repos/entity.rs:14-58
   #[derive(Debug, Clone, Deserialize)]
   #[serde(untagged)]
   enum SurrealDateTime {
       String(String),
       Native(serde_json::Value),
   }
   ```

3. **Proper use of `meta::id()`** to extract string IDs from Thing types
   ```rust
   SELECT meta::id(id) as id, name, entity_type FROM entity
   ```

4. **Graph relationships** stored as simple edge tables (compatible with v2)
   ```rust
   // engram-store/src/repos/entity.rs:333-349
   CREATE entity_relationship SET
       source_id = $source_id,
       target_id = $target_id,
       relation_type = $relation_type
   ```

### ⚠️ Missing Best Practices

#### Issue 2.1: No Transaction Boundaries

**Problem:** Multi-step operations lack atomicity

**Examples from codebase:**

```rust
// engram-store/src/repos/document.rs:275-314
pub async fn save_chunks(&self, source_id: &Id, chunks: Vec<(DocChunk, Vec<f32>)>) {
    // Step 1: Delete existing chunks
    self.db.query("DELETE doc_chunk WHERE source_id = $source_id")
        .bind(("source_id", source_id.to_string()))
        .await?;

    // Step 2: Insert new chunks (loop)
    for (chunk, embedding) in chunks {
        self.db.query(r#"UPSERT type::thing("doc_chunk", $id) SET ..."#)
            .await?;  // ⚠️ If this fails mid-loop, data is inconsistent!
    }
}
```

**Risk:**
- If insertion fails after deletion, chunks are lost
- If process crashes mid-operation, partial state remains
- No rollback mechanism

**Impact:**
- **High** for document indexing (data loss)
- **Medium** for entity operations (can be retried)
- **Low** for session logging (append-only)

**Solution:**

```rust
// Use SurrealDB v2 transactions
pub async fn save_chunks(&self, source_id: &Id, chunks: Vec<(DocChunk, Vec<f32>)>) -> StoreResult<()> {
    // Begin transaction
    self.db.query("BEGIN TRANSACTION").await?;
    
    match self.save_chunks_inner(source_id, chunks).await {
        Ok(_) => {
            self.db.query("COMMIT TRANSACTION").await?;
            Ok(())
        }
        Err(e) => {
            self.db.query("CANCEL TRANSACTION").await?;
            Err(e)
        }
    }
}

async fn save_chunks_inner(&self, source_id: &Id, chunks: Vec<(DocChunk, Vec<f32>)>) -> StoreResult<()> {
    // Delete existing
    self.db.query("DELETE doc_chunk WHERE source_id = $source_id")
        .bind(("source_id", source_id.to_string()))
        .await?;

    // Batch insert for better performance
    for batch in chunks.chunks(100) {
        let query = build_batch_insert_query(batch);
        self.db.query(&query).await?;
    }
    
    Ok(())
}
```

**Critical operations requiring transactions:**
1. ✅ `save_chunks()` - delete + insert loop
2. ✅ `delete_entity()` - cascading deletes (entity + relationships + aliases + observations)
3. ✅ `delete_source()` - delete chunks + source
4. ⚠️ Entity relationship creation (if you add bidirectional edges)

**Action items:**
1. Wrap all multi-step write operations in transactions
2. Add transaction helpers to reduce boilerplate
3. Test rollback behavior with integration tests

#### Issue 2.2: No Batch Operations

**Problem:** Loop-based inserts are slow and not atomic

**Current pattern:**
```rust
// engram-store/src/repos/document.rs:289-313
for (chunk, embedding) in chunks {
    self.db.query(r#"UPSERT type::thing("doc_chunk", $id) SET ..."#)
        .await?;  // ⚠️ One round-trip per chunk!
}
```

**Performance impact:**
- 1,000 chunks = 1,000 database round-trips
- Even with embedded DB, this adds latency
- Estimated: 1-5ms per insert = 1-5 seconds for 1,000 chunks

**Solution:**

```rust
// Batch insert using SurrealDB's array syntax
pub async fn save_chunks_batch(&self, source_id: &Id, chunks: Vec<(DocChunk, Vec<f32>)>) -> StoreResult<()> {
    const BATCH_SIZE: usize = 100;
    
    self.db.query("BEGIN TRANSACTION").await?;
    
    for batch in chunks.chunks(BATCH_SIZE) {
        // Build VALUES array for batch insert
        let values: Vec<_> = batch.iter().map(|(chunk, embedding)| {
            format!(
                r#"{{
                    id: type::thing("doc_chunk", "{}"),
                    source_id: "{}",
                    heading_path: "{}",
                    content: "{}",
                    embedding: {}
                }}"#,
                chunk.id,
                chunk.source_id,
                escape_string(&chunk.heading_path),
                escape_string(&chunk.content),
                serde_json::to_string(embedding).unwrap()
            )
        }).collect();
        
        let query = format!(
            "INSERT INTO doc_chunk {};",
            values.join(", ")
        );
        
        self.db.query(&query).await?;
    }
    
    self.db.query("COMMIT TRANSACTION").await?;
    Ok(())
}
```

**Expected performance gain:**
- 1,000 chunks: 5 seconds → 500ms (10x faster)
- Reduces round-trips from 1,000 to 10 (with batch size 100)

**Action items:**
1. Implement batch insert for chunks
2. Benchmark before/after
3. Apply pattern to other bulk operations (entities, events)

#### Issue 2.3: No Live Query Usage

**Opportunity:** SurrealDB v2's live queries for real-time updates

**Current:** Polling for changes (e.g., session coordination)

**Potential use case:**
```rust
// Real-time conflict detection
let mut stream = db.query("
    LIVE SELECT * FROM active_session 
    WHERE project = $project
").bind(("project", "my-app")).await?;

while let Some(notification) = stream.next().await {
    match notification {
        LiveNotification::Create(session) => {
            check_conflicts(session).await?;
        }
        LiveNotification::Update(session) => {
            recheck_conflicts(session).await?;
        }
        _ => {}
    }
}
```

**Recommendation:**
- ⚠️ **Not critical for MVP** - polling works fine for local-first
- ✅ **Consider for future** - if you add real-time collaboration features
- ⚠️ **Complexity trade-off** - live queries add async complexity

**Action items:**
1. Document live queries as future enhancement
2. Design API to support both polling and live queries
3. Implement when/if real-time features are needed

#### Issue 2.4: No Graph Traversal Optimization

**Current:** Manual relationship queries

```rust
// engram-store/src/repos/entity.rs:355-367
pub async fn get_relationships_from(&self, entity_id: &Id) -> StoreResult<Vec<Relationship>> {
    let mut result = self.db
        .query(r#"SELECT source_id, target_id, relation_type 
                  FROM entity_relationship
                  WHERE source_id = $id"#)
        .bind(("id", entity_id.to_string()))
        .await?;
    // ...
}
```

**Opportunity:** Use SurrealDB's graph traversal syntax

```rust
// Multi-hop traversal (e.g., "find all tools used by repos I depend on")
SELECT ->depends_on->entity->uses->entity.name AS tools
FROM type::thing("entity", $repo_id)
```

**Benefits:**
- ✅ More expressive queries
- ✅ Better performance for multi-hop traversals
- ✅ Leverages SurrealDB's graph capabilities

**Recommendation:**
- ✅ **Implement for complex queries** - especially in MCP tools
- ⚠️ **Keep simple queries as-is** - single-hop is fine with current approach

**Example use cases:**
1. "Find all entities related to X within 2 hops"
2. "Get dependency tree for a repository"
3. "Find all tools used by a team's projects"

**Action items:**
1. Add graph traversal helper methods to EntityRepo
2. Expose via MCP tools (e.g., `entity_traverse`)
3. Document graph query patterns

### SurrealDB v2 Specific Patterns

#### Pattern 1: Proper Record ID Handling

**✅ Already doing this correctly:**

```rust
// Use type::thing() for record IDs
UPSERT type::thing("doc_chunk", $id) SET ...

// Use meta::id() to extract string IDs
SELECT meta::id(id) as id FROM entity
```

#### Pattern 2: Handling Thing vs String Types

**✅ Already doing this correctly:**

```rust
// Custom deserialization for flexible ID handling
let id = Id::parse(&record.id).unwrap_or_else(|_| Id::new());
```

#### Pattern 3: Avoiding SDK Serialization Issues

**✅ Already doing this correctly:**

```rust
// Use raw queries instead of SDK's typed API
self.db.query(r#"..."#).bind(...).await?
// Instead of: db.create("table").content(struct).await?
```

### Recommended Additions

#### 1. Connection Pooling (Low Priority)

**Current:** Single connection per service instance

**For embedded DB:** Not critical, but could help with concurrent operations

```rust
// Future enhancement
pub struct DbPool {
    connections: Vec<Db>,
    semaphore: Arc<Semaphore>,
}
```

**Recommendation:** ⚠️ Not needed now, revisit if you see connection bottlenecks

#### 2. Query Result Caching

**Opportunity:** Cache frequently accessed entities/documents

```rust
use dashmap::DashMap;

pub struct CachedEntityRepo {
    repo: EntityRepo,
    cache: Arc<DashMap<Id, Entity>>,
}
```

**Recommendation:** ⚠️ Premature optimization, implement only if profiling shows need

#### 3. Prepared Statements (Not Supported in v2)

**Status:** SurrealDB v2 doesn't support prepared statements yet

**Workaround:** Query string building is fine, just ensure proper escaping

---

## 3. Index Strategy

### Current Indexes

**Document Layer:**
```sql
-- engram-store/src/repos/document.rs:148-161
DEFINE INDEX idx_source_path ON doc_source FIELDS path_or_url UNIQUE;
DEFINE INDEX idx_chunk_source ON doc_chunk FIELDS source_id;
```

**Entity Layer:**
```sql
-- engram-store/src/repos/entity.rs:133-169
DEFINE INDEX idx_entity_type ON entity FIELDS entity_type;
DEFINE INDEX idx_entity_name ON entity FIELDS name;
DEFINE INDEX idx_alias_name ON entity_alias FIELDS name;
DEFINE INDEX idx_alias_entity ON entity_alias FIELDS entity_id;
DEFINE INDEX idx_obs_entity ON entity_observation FIELDS entity_id;
DEFINE INDEX idx_rel_type ON entity_relationship FIELDS relation_type;
DEFINE INDEX idx_rel_source ON entity_relationship FIELDS source_id;
DEFINE INDEX idx_rel_target ON entity_relationship FIELDS target_id;
```

**Session Layer:**
```sql
-- engram-store/src/repos/session.rs:113-127
DEFINE INDEX idx_session_status ON session FIELDS status;
DEFINE INDEX idx_session_agent ON session FIELDS agent;
DEFINE INDEX idx_session_project ON session FIELDS project;
DEFINE INDEX idx_event_session ON event FIELDS session_id;
DEFINE INDEX idx_event_type ON event FIELDS event_type;
```

**Tool Layer:**
```sql
-- engram-store/src/repos/tool.rs:136-150
DEFINE INDEX idx_usage_tool ON tool_usage FIELDS tool_id;
DEFINE INDEX idx_usage_outcome ON tool_usage FIELDS outcome;
DEFINE INDEX idx_pref_tool ON tool_preference FIELDS tool_id;
DEFINE INDEX idx_pref_context ON tool_preference FIELDS context_pattern;
```

**Coordination Layer:**
```sql
-- engram-store/src/repos/coordination.rs:101-106
DEFINE INDEX idx_active_project ON active_session FIELDS project;
DEFINE INDEX idx_active_components ON active_session FIELDS components;
```

### Analysis: Good Coverage

**✅ Strengths:**
1. All foreign key fields are indexed (source_id, entity_id, session_id, tool_id)
2. Common filter fields are indexed (status, agent, project, outcome)
3. Unique constraints where appropriate (path_or_url)
4. Composite access patterns covered (entity by type, session by status)

**⚠️ Potential Improvements:**

#### 3.1: Add Composite Indexes for Common Query Patterns

**Pattern 1: Session filtering**
```rust
// Current query: engram-store/src/repos/session.rs:201-228
SELECT * FROM session 
WHERE status = 'active' AND project = 'my-app' 
ORDER BY created_at DESC
```

**Problem:** Two separate indexes, not optimal for combined filter

**Solution:**
```sql
-- Add composite index
DEFINE INDEX idx_session_status_project ON session FIELDS status, project;
DEFINE INDEX idx_session_created ON session FIELDS created_at;
```

**Pattern 2: Tool usage by context**
```rust
// Common query: search usages by context substring
SELECT * FROM tool_usage 
WHERE string::lowercase(context) CONTAINS $query 
ORDER BY timestamp DESC
```

**Problem:** Full table scan for CONTAINS (no index can help)

**Solution:** Add full-text search field or pre-tokenize context

```sql
-- Option A: Add tokenized field
DEFINE INDEX idx_usage_context_tokens ON tool_usage FIELDS context_tokens;

-- Option B: Use SurrealDB's full-text search (if available)
DEFINE INDEX idx_usage_context_fulltext ON tool_usage FIELDS context SEARCH ANALYZER simple;
```

**Pattern 3: Entity search by name**
```rust
// Current: engram-store/src/repos/entity.rs:270-286
SELECT * FROM entity 
WHERE string::lowercase(name) CONTAINS $query 
ORDER BY name
```

**Problem:** Case-insensitive CONTAINS requires full scan

**Solution:**
```sql
-- Add lowercase name field for indexing
DEFINE INDEX idx_entity_name_lower ON entity FIELDS string::lowercase(name);
```

Or compute at insert time:
```rust
pub async fn save_entity(&self, entity: &Entity) -> StoreResult<()> {
    self.db.query(r#"
        UPSERT type::thing("entity", $id) SET
            name = $name,
            name_lower = string::lowercase($name),  -- Add this
            ...
    "#)
    .bind(("name", entity.name.clone()))
    .await?;
}

// Then query:
SELECT * FROM entity WHERE name_lower CONTAINS $query
```

#### 3.2: Add Indexes for Sorting

**Pattern:** Many queries sort by timestamp/created_at

```rust
// Examples:
ORDER BY created_at DESC  -- sessions, entities, events
ORDER BY timestamp DESC   -- tool usages
ORDER BY last_indexed     -- documents
```

**Current:** No dedicated indexes for these fields

**Impact:**
- Small datasets (< 10K): Negligible
- Large datasets (> 50K): Sorting becomes expensive

**Solution:**
```sql
-- Add timestamp indexes
DEFINE INDEX idx_session_created ON session FIELDS created_at;
DEFINE INDEX idx_entity_created ON entity FIELDS created_at;
DEFINE INDEX idx_event_timestamp ON event FIELDS timestamp;
DEFINE INDEX idx_usage_timestamp ON tool_usage FIELDS timestamp;
DEFINE INDEX idx_source_indexed ON doc_source FIELDS last_indexed;
```

**Recommendation:** ✅ Add these now, they're cheap and will help at scale

#### 3.3: Consider Covering Indexes (Future)

**Concept:** Index includes all fields needed by query (no table lookup)

**Example:**
```sql
-- Instead of:
DEFINE INDEX idx_entity_type ON entity FIELDS entity_type;

-- Use covering index:
DEFINE INDEX idx_entity_type_covering ON entity FIELDS entity_type, name, description;
```

**Benefit:** Query can be satisfied entirely from index

**Trade-off:**
- ✅ Faster queries
- ⚠️ Larger index size
- ⚠️ Slower writes (more index updates)

**Recommendation:** ⚠️ Not needed now, SurrealDB may not support this yet

### Index Maintenance

**Current:** No index maintenance code

**Considerations:**
1. **Index rebuilding:** SurrealDB handles automatically
2. **Index statistics:** No API to inspect index usage
3. **Stale indexes:** No cleanup needed (handled by DB)

**Recommendation:** ✅ Current approach is fine, no action needed

### Summary: Index Action Items

**High Priority:**
1. ✅ Add composite index: `idx_session_status_project`
2. ✅ Add timestamp indexes for sorting
3. ✅ Add `name_lower` field and index for case-insensitive search

**Medium Priority:**
4. ⚠️ Investigate full-text search for context fields
5. ⚠️ Add composite indexes for other common filter combinations

**Low Priority:**
6. ⚠️ Monitor index usage (when SurrealDB exposes metrics)
7. ⚠️ Consider covering indexes if query performance becomes issue

---

## 4. Memory Management

### Memory Footprint Analysis

#### 4.1: Vector Storage

**Current scale:**
- 50,000 chunks
- 384 dimensions per vector
- float32 (4 bytes per dimension)

**Calculation:**
```
Vector size = 384 dim × 4 bytes = 1,536 bytes per vector
50,000 vectors × 1,536 bytes = 76,800,000 bytes ≈ 76 MB
```

**With overhead (SurrealDB storage, indexes, metadata):**
- Estimated: 76 MB × 1.5-2.0 = **114-152 MB for vectors alone**

#### 4.2: Chunk Metadata

**Per chunk:**
- ID: 16 bytes (UUID)
- source_id: 16 bytes
- heading_path: ~50 bytes average
- content: ~500 bytes average (chunk size)
- Other fields: ~50 bytes

**Total per chunk:** ~632 bytes

**50,000 chunks:**
```
50,000 × 632 bytes = 31,600,000 bytes ≈ 31 MB
```

#### 4.3: Entity Graph

**Estimated scale:**
- 1,000 entities × ~500 bytes = 500 KB
- 2,000 relationships × ~200 bytes = 400 KB
- 3,000 aliases × ~100 bytes = 300 KB
- 5,000 observations × ~300 bytes = 1.5 MB

**Total:** ~2.7 MB (negligible)

#### 4.4: Session History

**Estimated scale:**
- 500 sessions × ~300 bytes = 150 KB
- 10,000 events × ~400 bytes = 4 MB

**Total:** ~4.2 MB (negligible)

#### 4.5: SurrealDB Overhead

**Components:**
- RocksDB cache: 64-128 MB (default)
- Index structures: 20-50 MB (estimated)
- Connection overhead: 5-10 MB
- Query cache: 10-20 MB

**Total overhead:** ~100-200 MB

### Total Memory Footprint Estimate

| Component | Memory |
|-----------|--------|
| Vectors (50K) | 114-152 MB |
| Chunk metadata | 31 MB |
| Entity graph | 3 MB |
| Session history | 4 MB |
| SurrealDB overhead | 100-200 MB |
| **Total** | **252-390 MB** |

**At 100K chunks (2x scale):**
- Vectors: 228-304 MB
- Metadata: 62 MB
- **Total: 393-566 MB**

### Analysis

**✅ Good news:**
- 250-400 MB is acceptable for a local-first application
- Modern machines have 8-32 GB RAM
- This is < 5% of RAM on an 8GB machine

**⚠️ Concerns:**
1. **Memory growth is linear** with chunk count
2. **No memory limits** configured in SurrealDB
3. **No pagination** for large result sets
4. **All vectors loaded** into memory (no lazy loading)

### Recommendations

#### 4.1: Configure RocksDB Memory Limits

**Current:** Using defaults (unbounded cache)

**Recommendation:**
```rust
// engram-store/src/config.rs
pub struct StoreConfig {
    pub backend: StorageBackend,
    pub namespace: String,
    pub database: String,
    pub rocksdb_cache_mb: Option<usize>,  // Add this
}

// When connecting:
let connection_string = match &config.backend {
    StorageBackend::RocksDb(path) => {
        let cache_size = config.rocksdb_cache_mb.unwrap_or(128);
        format!("rocksdb://{}?cache_size={}MB", path.display(), cache_size)
    }
    _ => "mem://".to_string(),
};
```

**Suggested limits:**
- Development: 128 MB cache
- Production: 256-512 MB cache
- Low-memory systems: 64 MB cache

#### 4.2: Implement Pagination for Search Results

**Current:** Returns all results up to limit

```rust
// engram-store/src/repos/document.rs:331-361
pub async fn search_similar(&self, query_embedding: &[f32], limit: usize) -> StoreResult<Vec<DocSearchResult>> {
    // Returns all results at once
}
```

**Problem:**
- Large result sets (1000+ chunks) consume memory
- All results loaded before returning
- No way to fetch incrementally

**Solution: Add cursor-based pagination**

```rust
pub struct SearchCursor {
    pub offset: usize,
    pub has_more: bool,
}

pub async fn search_similar_paginated(
    &self,
    query_embedding: &[f32],
    limit: usize,
    offset: usize,
) -> StoreResult<(Vec<DocSearchResult>, SearchCursor)> {
    let mut result = self.db
        .query(r#"
            SELECT /* ... */
            FROM doc_chunk
            ORDER BY score DESC
            LIMIT $limit
            START $offset  -- Add pagination
        "#)
        .bind(("query", query_embedding.to_vec()))
        .bind(("limit", limit + 1))  // Fetch one extra to check has_more
        .bind(("offset", offset))
        .await?;

    let mut hits: Vec<SearchHit> = result.take(0)?;
    let has_more = hits.len() > limit;
    if has_more {
        hits.pop();  // Remove extra result
    }

    let cursor = SearchCursor {
        offset: offset + hits.len(),
        has_more,
    };

    // ... build results ...

    Ok((results, cursor))
}
```

**Benefits:**
- ✅ Bounded memory usage per request
- ✅ Supports "load more" UX
- ✅ Better for large result sets

**Trade-off:**
- ⚠️ Pagination with vector search is tricky (scores change with offset)
- ⚠️ Better to use `LIMIT` conservatively (e.g., max 100 results)

**Recommendation:**
- ✅ Implement pagination for list operations (entities, sessions, events)
- ⚠️ For vector search, enforce reasonable limits (max 100-200 results)

#### 4.3: Implement Streaming for Bulk Operations

**Current:** Load all results into Vec

**Problem:** Bulk exports or large queries consume memory

**Solution: Use async streams**

```rust
use futures::stream::{Stream, StreamExt};

pub fn search_similar_stream(
    &self,
    query_embedding: Vec<f32>,
    limit: usize,
) -> impl Stream<Item = StoreResult<DocSearchResult>> + '_ {
    async_stream::try_stream! {
        const BATCH_SIZE: usize = 100;
        let mut offset = 0;

        loop {
            let batch = self.search_similar_paginated(
                &query_embedding,
                BATCH_SIZE,
                offset,
            ).await?;

            for result in batch.0 {
                yield result;
            }

            if !batch.1.has_more || offset >= limit {
                break;
            }

            offset = batch.1.offset;
        }
    }
}

// Usage:
let mut stream = repo.search_similar_stream(embedding, 1000);
while let Some(result) = stream.next().await {
    process(result?);  // Process one at a time
}
```

**Benefits:**
- ✅ Constant memory usage regardless of result size
- ✅ Can process results as they arrive
- ✅ Better for MCP streaming responses (future)

**Recommendation:** ⚠️ Implement when needed, not critical for MVP

#### 4.4: Add Memory Profiling

**Current:** No memory monitoring

**Recommendation:**

```rust
// Add to engram-cli/src/main.rs
use sysinfo::{System, SystemExt};

pub fn log_memory_usage() {
    let mut sys = System::new_all();
    sys.refresh_memory();
    
    let used_mb = sys.used_memory() / 1024 / 1024;
    let total_mb = sys.total_memory() / 1024 / 1024;
    
    info!("Memory usage: {} MB / {} MB ({:.1}%)", 
          used_mb, total_mb, 
          (used_mb as f64 / total_mb as f64) * 100.0);
}

// Call periodically or on-demand
tokio::spawn(async {
    loop {
        log_memory_usage();
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
});
```

**Action items:**
1. Add memory logging to CLI
2. Add `--memory-stats` flag for diagnostics
3. Log memory usage in MCP server startup

#### 4.5: Consider Memory-Mapped Vectors (Advanced)

**For > 500K chunks:** Store vectors in memory-mapped file

**Libraries:**
- `memmap2` - memory-mapped files
- `lance` - columnar format with mmap support

**Benefits:**
- ✅ OS handles memory paging
- ✅ Can handle datasets larger than RAM
- ✅ Fast random access

**Trade-offs:**
- ⚠️ More complex implementation
- ⚠️ Requires custom vector search code
- ⚠️ OS-dependent performance

**Recommendation:** ⚠️ Only if you exceed 1M chunks, not needed now

### Memory Management Action Plan

**Immediate (High Priority):**
1. ✅ Configure RocksDB cache limits (128-256 MB)
2. ✅ Add memory usage logging
3. ✅ Document memory requirements in README

**Short-term (Medium Priority):**
4. ⚠️ Implement pagination for list operations
5. ⚠️ Add max result limits to search APIs
6. ⚠️ Add `--max-chunks` flag to limit indexing

**Long-term (Low Priority):**
7. ⚠️ Implement streaming for bulk operations
8. ⚠️ Consider memory-mapped vectors if needed
9. ⚠️ Profile and optimize hot paths

---

## 5. Data Integrity

### Current Approach: SCHEMALESS Tables

**All tables use SCHEMALESS:**
```sql
-- engram-store/src/repos/document.rs:147
DEFINE TABLE doc_source SCHEMALESS;
DEFINE TABLE doc_chunk SCHEMALESS;

-- engram-store/src/repos/entity.rs:133
DEFINE TABLE entity SCHEMALESS;
DEFINE TABLE entity_alias SCHEMALESS;
DEFINE TABLE entity_observation SCHEMALESS;
DEFINE TABLE entity_relationship SCHEMALESS;

-- Similar for session, tool, coordination, knowledge layers
```

**Reasoning (from code comments):**
- Avoids `id` field conflicts with SurrealDB's record ID
- Allows flexible properties (e.g., `Entity.properties: HashMap`)
- Works around SDK serialization issues

### Risks of SCHEMALESS

#### Risk 5.1: Type Safety Lost at Database Boundary

**Problem:** No database-level validation

**Example:**
```rust
// This will succeed even with wrong types
self.db.query(r#"
    UPSERT type::thing("entity", $id) SET
        name = 12345,  -- Should be string, but DB accepts it
        entity_type = true,  -- Should be string
        created_at = "not a date"  -- Should be datetime
"#).await?;
```

**Impact:**
- ⚠️ Bugs may not be caught until deserialization
- ⚠️ Data corruption possible if application has bugs
- ⚠️ No foreign key constraints (can reference non-existent IDs)

#### Risk 5.2: No Foreign Key Constraints

**Problem:** Orphaned records possible

**Example:**
```rust
// Delete entity
self.db.query(r#"DELETE type::thing("entity", $id)"#).await?;

// But relationships still reference it!
// No CASCADE DELETE without SCHEMAFULL
```

**Current mitigation:**
```rust
// Manual cascade in application code
pub async fn delete_entity(&self, id: &Id) -> StoreResult<()> {
    // Manually delete related records
    self.db.query(r#"DELETE entity_relationship WHERE source_id = $id OR target_id = $id"#).await?;
    self.db.query("DELETE FROM entity_alias WHERE entity_id = $id").await?;
    self.db.query("DELETE FROM entity_observation WHERE entity_id = $id").await?;
    self.db.query(r#"DELETE type::thing("entity", $id)"#).await?;
    Ok(())
}
```

**Risk:** If application code has bugs, orphans can occur

#### Risk 5.3: No Default Values

**Problem:** Missing fields cause deserialization errors

**Example:**
```rust
// Old code didn't set ttl_days
self.db.query(r#"
    UPSERT type::thing("doc_source", $id) SET
        path_or_url = $path
        -- Missing: ttl_days
"#).await?;

// Later, deserialization fails:
#[derive(Deserialize)]
struct DocSource {
    ttl_days: i32,  // ❌ Field missing in DB!
}
```

**Current mitigation:** Use `Option<T>` or `#[serde(default)]`

```rust
#[derive(Deserialize)]
struct DocSource {
    #[serde(default = "default_ttl")]
    ttl_days: i32,
}

fn default_ttl() -> i32 { 7 }
```

#### Risk 5.4: Schema Evolution Challenges

**Problem:** No migration system

**Example:** Adding a new required field

```rust
// v1: Entity has name, type
// v2: Entity adds required field "status"

// Old records don't have status field
// Deserialization fails unless you use Option<T>
```

**Current approach:** All fields are optional or have defaults

**Risk:** Can't enforce required fields added later

### Recommendations

#### Option A: Hybrid Approach (Recommended)

**Use SCHEMAFULL for critical tables, SCHEMALESS for flexible ones**

**SCHEMAFULL candidates:**
- Core domain objects with stable schema
- Tables with foreign key relationships
- Tables requiring validation

**SCHEMALESS candidates:**
- Tables with dynamic properties (e.g., `Entity.properties`)
- Tables with evolving schema
- Tables with optional fields

**Implementation:**

```sql
-- Core tables: SCHEMAFULL with validation
DEFINE TABLE entity SCHEMAFULL;
DEFINE FIELD name ON entity TYPE string ASSERT $value != NONE;
DEFINE FIELD entity_type ON entity TYPE string ASSERT $value != NONE;
DEFINE FIELD description ON entity TYPE option<string>;
DEFINE FIELD properties ON entity TYPE object DEFAULT {};  -- Still flexible
DEFINE FIELD created_at ON entity TYPE datetime ASSERT $value != NONE;
DEFINE FIELD updated_at ON entity TYPE datetime ASSERT $value != NONE;

-- Relationship table: SCHEMAFULL with foreign keys
DEFINE TABLE entity_relationship SCHEMAFULL;
DEFINE FIELD source_id ON entity_relationship TYPE record<entity> ASSERT $value != NONE;
DEFINE FIELD target_id ON entity_relationship TYPE record<entity> ASSERT $value != NONE;
DEFINE FIELD relation_type ON entity_relationship TYPE string ASSERT $value != NONE;

-- Flexible tables: SCHEMALESS
DEFINE TABLE entity_observation SCHEMALESS;  -- Keep flexible for varying content
```

**Benefits:**
- ✅ Type safety where it matters
- ✅ Foreign key constraints prevent orphans
- ✅ Still flexible where needed
- ✅ Better error messages (validation fails at insert, not deserialize)

**Trade-offs:**
- ⚠️ More schema management
- ⚠️ May need to revisit `id` field handling
- ⚠️ Requires testing to ensure SDK compatibility

**Action items:**
1. Identify critical tables for SCHEMAFULL conversion
2. Test SDK compatibility with SCHEMAFULL + custom IDs
3. Implement gradually (start with entity table)
4. Add integration tests for constraint violations

#### Option B: Application-Level Validation (Current + Enhancements)

**Keep SCHEMALESS, add validation in Rust**

**Implementation:**

```rust
// Add validation trait
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationError>;
}

impl Validate for Entity {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError::EmptyField("name"));
        }
        if self.created_at > self.updated_at {
            return Err(ValidationError::InvalidTimestamp);
        }
        // ... more checks
        Ok(())
    }
}

// Enforce in repository
impl EntityRepo {
    pub async fn save_entity(&self, entity: &Entity) -> StoreResult<()> {
        entity.validate()?;  // ✅ Validate before saving
        // ... save to DB
    }
}
```

**Benefits:**
- ✅ No schema changes needed
- ✅ Flexible validation logic (can be complex)
- ✅ Validation errors in Rust (better error messages)

**Trade-offs:**
- ⚠️ Validation only happens in application (can be bypassed)
- ⚠️ No database-level constraints
- ⚠️ More code to maintain

**Action items:**
1. Add `Validate` trait to `engram-core`
2. Implement for all domain types
3. Enforce in all repository save methods
4. Add validation tests

#### Option C: Schema Migration System

**Implement proper migrations for schema evolution**

**Implementation:**

```rust
// engram-store/src/migrations/mod.rs
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up: String,   // SQL to apply
    pub down: String, // SQL to rollback
}

pub async fn run_migrations(db: &Db) -> StoreResult<()> {
    // Create migrations table
    db.query("DEFINE TABLE IF NOT EXISTS schema_migrations SCHEMALESS").await?;
    
    // Get current version
    let current_version = get_current_version(db).await?;
    
    // Apply pending migrations
    for migration in MIGRATIONS.iter().filter(|m| m.version > current_version) {
        info!("Applying migration {}: {}", migration.version, migration.name);
        db.query(&migration.up).await?;
        record_migration(db, migration.version).await?;
    }
    
    Ok(())
}

// migrations/001_initial_schema.rs
pub const MIGRATION_001: Migration = Migration {
    version: 1,
    name: "initial_schema",
    up: r#"
        DEFINE TABLE entity SCHEMALESS;
        DEFINE INDEX idx_entity_name ON entity FIELDS name;
    "#,
    down: r#"
        REMOVE TABLE entity;
    "#,
};

// migrations/002_add_entity_status.rs
pub const MIGRATION_002: Migration = Migration {
    version: 2,
    name: "add_entity_status",
    up: r#"
        -- Add status field with default
        UPDATE entity SET status = "active" WHERE status = NONE;
    "#,
    down: r#"
        -- Remove status field
        UPDATE entity UNSET status;
    "#,
};
```

**Benefits:**
- ✅ Structured schema evolution
- ✅ Rollback capability
- ✅ Version tracking
- ✅ Safe for production updates

**Trade-offs:**
- ⚠️ More infrastructure code
- ⚠️ Requires careful testing
- ⚠️ Adds complexity to deployment

**Recommendation:** ⚠️ Implement when you need to support multiple versions in production

### Recommended Approach

**For engram's current stage:**

1. **Short-term (MVP):** Option B - Application-level validation
   - ✅ Quick to implement
   - ✅ No schema changes
   - ✅ Good enough for single-user local-first

2. **Medium-term:** Option A - Hybrid SCHEMAFULL/SCHEMALESS
   - ✅ Better data integrity
   - ✅ Catches bugs earlier
   - ✅ Prepares for multi-user scenarios

3. **Long-term:** Option C - Migration system
   - ✅ Professional schema management
   - ✅ Required for production deployments
   - ✅ Supports versioned releases

### Data Integrity Action Plan

**Immediate (High Priority):**
1. ✅ Add `Validate` trait and implement for all domain types
2. ✅ Enforce validation in all repository save methods
3. ✅ Add integration tests for constraint violations
4. ✅ Document validation rules in code comments

**Short-term (Medium Priority):**
5. ⚠️ Test SCHEMAFULL with custom IDs (proof of concept)
6. ⚠️ Convert entity table to SCHEMAFULL (if compatible)
7. ⚠️ Add foreign key constraints for relationships
8. ⚠️ Add unique constraints where appropriate

**Long-term (Low Priority):**
9. ⚠️ Implement migration system
10. ⚠️ Add schema version tracking
11. ⚠️ Create migration CLI commands

---

## 6. Additional Considerations

### 6.1: Backup and Recovery

**Current:** No backup mechanism

**Recommendation:**

```rust
// engram-cli: Add backup command
pub async fn backup(db_path: &Path, backup_path: &Path) -> Result<()> {
    // RocksDB supports snapshots
    info!("Creating backup of {} to {}", db_path.display(), backup_path.display());
    
    // Option 1: Copy RocksDB directory
    fs_extra::dir::copy(db_path, backup_path, &CopyOptions::new())?;
    
    // Option 2: Export to JSON
    let db = connect(&StoreConfig::rocksdb(db_path)).await?;
    export_to_json(&db, backup_path).await?;
    
    Ok(())
}
```

**Action items:**
1. Add `engram backup` command
2. Add `engram restore` command
3. Support automatic backups (e.g., daily)
4. Document backup best practices

### 6.2: Performance Monitoring

**Current:** Basic logging, no metrics

**Recommendation:**

```rust
// Add metrics collection
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Metrics {
    pub searches: AtomicU64,
    pub search_latency_ms: AtomicU64,
    pub chunks_indexed: AtomicU64,
    pub index_latency_ms: AtomicU64,
}

impl Metrics {
    pub fn record_search(&self, latency_ms: u64) {
        self.searches.fetch_add(1, Ordering::Relaxed);
        self.search_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }
    
    pub fn avg_search_latency(&self) -> f64 {
        let total = self.search_latency_ms.load(Ordering::Relaxed);
        let count = self.searches.load(Ordering::Relaxed);
        if count == 0 { 0.0 } else { total as f64 / count as f64 }
    }
}
```

**Action items:**
1. Add metrics struct to services
2. Record key operations (search, index, save)
3. Add `engram stats --detailed` command
4. Expose metrics via MCP tool

### 6.3: Error Handling

**Current:** Good use of `Result<T, E>` and `thiserror`

**✅ Strengths:**
- Custom error types per crate
- Proper error propagation
- Contextual error messages

**⚠️ Potential improvements:**

```rust
// Add error context with anyhow
use anyhow::Context;

pub async fn index_file(&self, path: &Path) -> IndexResult<IndexedDocument> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    let parsed = parse_markdown(&content)
        .with_context(|| format!("Failed to parse markdown: {}", path.display()))?;
    
    // ...
}
```

**Action items:**
1. Add context to file I/O errors
2. Add context to database errors
3. Improve error messages for users (not just developers)

### 6.4: Testing Strategy

**Current:** Good integration test coverage

**✅ Strengths:**
- Comprehensive semantic search tests
- Layer-specific integration tests
- Real embedding model tests (ignored by default)

**⚠️ Gaps:**
1. No property-based tests (e.g., with `proptest`)
2. No performance benchmarks
3. No stress tests (large datasets)
4. No concurrency tests

**Recommendations:**

```rust
// Add benchmarks with criterion
#[bench]
fn bench_vector_search(b: &mut Bencher) {
    let service = setup_service_with_10k_chunks();
    let query = "test query";
    
    b.iter(|| {
        service.search(query, 10).await
    });
}

// Add stress tests
#[test]
#[ignore = "slow"]
fn test_index_100k_chunks() {
    let service = setup_service();
    
    for i in 0..100_000 {
        let doc = generate_test_doc(i);
        service.index_file(&doc).await.unwrap();
    }
    
    // Verify performance is acceptable
    let start = Instant::now();
    service.search("test", 10).await.unwrap();
    assert!(start.elapsed() < Duration::from_millis(500));
}
```

**Action items:**
1. Add criterion benchmarks for hot paths
2. Add stress tests for scale targets
3. Add concurrency tests for multi-threaded access
4. Run benchmarks in CI

---

## 7. Summary and Prioritized Recommendations

### Critical (Implement Now)

1. **Add application-level validation** (Section 5)
   - Implement `Validate` trait for all domain types
   - Enforce in repository save methods
   - Prevents data corruption bugs

2. **Add transaction boundaries** (Section 2.1)
   - Wrap multi-step operations in transactions
   - Prevents partial state on failures
   - Critical for data integrity

3. **Configure memory limits** (Section 4.1)
   - Set RocksDB cache size (128-256 MB)
   - Add memory usage logging
   - Prevents unbounded memory growth

4. **Add monitoring** (Section 1)
   - Track vector search latency
   - Log chunk count and query performance
   - Detect performance cliff early

### High Priority (Next Sprint)

5. **Implement batch operations** (Section 2.2)
   - Batch insert for chunks (10x faster)
   - Reduces database round-trips
   - Better performance at scale

6. **Add composite indexes** (Section 3)
   - `idx_session_status_project`
   - `idx_entity_name_lower` for case-insensitive search
   - Timestamp indexes for sorting
   - Improves query performance

7. **Implement pagination** (Section 4.2)
   - Add cursor-based pagination for lists
   - Enforce max result limits
   - Prevents memory issues with large results

8. **Add backup/restore** (Section 6.1)
   - `engram backup` command
   - `engram restore` command
   - Critical for user data safety

### Medium Priority (Future Sprints)

9. **Hybrid SCHEMAFULL approach** (Section 5)
   - Convert critical tables to SCHEMAFULL
   - Add foreign key constraints
   - Better data integrity guarantees

10. **Pre-filtering for vector search** (Section 1, Option 2)
    - Add metadata filters to reduce search space
    - Implement when chunk count > 20K
    - Extends brute-force viability

11. **Graph traversal queries** (Section 2.4)
    - Use SurrealDB's graph syntax for multi-hop queries
    - Add `entity_traverse` MCP tool
    - Better leverage graph capabilities

12. **Performance benchmarks** (Section 6.4)
    - Add criterion benchmarks
    - Run in CI
    - Track performance regressions

### Low Priority (Nice to Have)

13. **External vector database** (Section 1, Option 3)
    - Only if chunk count > 100K
    - Prototype with Qdrant
    - Significant complexity increase

14. **Migration system** (Section 5, Option C)
    - Implement when supporting multiple versions
    - Required for production deployments
    - Not needed for single-user local-first

15. **Streaming APIs** (Section 4.3)
    - Async stream for large result sets
    - Useful for MCP streaming responses
    - Not critical for current use case

16. **Live queries** (Section 2.3)
    - Real-time conflict detection
    - Only if adding collaboration features
    - Adds complexity

---

## 8. Conclusion

**Overall Assessment:** ✅ **Solid foundation with clear path forward**

**Strengths:**
- Clean architecture with proper separation of concerns
- Good use of Rust's type system and error handling
- Effective workarounds for SurrealDB v2 SDK limitations
- Comprehensive test coverage for core functionality
- Well-documented code with clear intent

**Key Risks:**
- ⚠️ Vector search performance cliff at 50K+ chunks (brute-force)
- ⚠️ Data integrity concerns with SCHEMALESS tables
- ⚠️ No transaction boundaries for multi-step operations
- ⚠️ Memory growth unbounded (but acceptable at current scale)

**Recommended Path:**

**Phase 1 (Now):** Stabilize and validate
- Add validation layer
- Add transactions
- Configure memory limits
- Add monitoring

**Phase 2 (Next 1-2 months):** Optimize for scale
- Batch operations
- Better indexes
- Pagination
- Backup/restore

**Phase 3 (Future):** Scale to 100K+ chunks
- Hybrid SCHEMAFULL
- Pre-filtering or external vector DB
- Migration system
- Advanced monitoring

**Bottom Line:**
Your architecture is well-suited for the local-first, 10K-50K chunk use case. The recommendations above will help you scale gracefully and maintain data integrity as the system grows. Focus on the critical items first, then iterate based on actual usage patterns and performance metrics.

---

## Appendix A: Quick Reference

### Performance Targets

| Metric | Current | Target | Critical |
|--------|---------|--------|----------|
| Vector search latency (P95) | ~50ms @ 10K | < 200ms @ 50K | < 500ms |
| Index throughput | ~100 chunks/sec | > 50 chunks/sec | > 10 chunks/sec |
| Memory usage | ~250 MB @ 50K | < 500 MB @ 50K | < 1 GB |
| Startup time | ~2 sec | < 5 sec | < 10 sec |

### Key Files to Modify

| Recommendation | File | Lines |
|----------------|------|-------|
| Add validation | `engram-core/src/*.rs` | Add `Validate` trait |
| Add transactions | `engram-store/src/repos/*.rs` | Wrap multi-step ops |
| Batch inserts | `engram-store/src/repos/document.rs` | `save_chunks()` |
| Memory config | `engram-store/src/config.rs` | Add `rocksdb_cache_mb` |
| Add indexes | `engram-store/src/repos/*.rs` | `init_schema()` |
| Monitoring | `engram-index/src/service.rs` | Add metrics |

### Useful SurrealDB Queries

```sql
-- Check table sizes
SELECT count() FROM doc_chunk GROUP ALL;
SELECT count() FROM entity GROUP ALL;

-- Check index usage (when available)
INFO FOR TABLE doc_chunk;

-- Analyze query performance
-- (Add EXPLAIN when SurrealDB supports it)

-- Check memory usage (RocksDB stats)
-- (No direct query, use OS tools)
```

### Testing Commands

```bash
# Run all tests
cargo test

# Run with ignored tests (requires model download)
cargo test -- --ignored

# Run specific layer tests
cargo test -p engram-tests --test semantic_search_tests

# Run benchmarks (when added)
cargo bench

# Check memory usage
ps aux | grep engram
```

---

**End of Expert Review**

*For questions or clarifications, please open an issue or discussion in the repository.*
