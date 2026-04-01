# Expert Review Summary: engram PKAS

**Full review:** See [EXPERT_REVIEW.md](./EXPERT_REVIEW.md) for detailed analysis.

---

## TL;DR

✅ **Your architecture is solid for 10K-50K chunks**  
⚠️ **Performance cliff expected at 50K+ chunks** (brute-force vector search)  
⚠️ **Data integrity needs application-level validation** (SCHEMALESS tables)  
✅ **Memory footprint is acceptable** (~250-400 MB at 50K chunks)

---

## Critical Issues & Solutions

### 1. Vector Search Performance Cliff

**Problem:** Brute-force cosine similarity will hit ~500ms latency at 50K chunks.

**Why it matters:** User experience degrades, queries feel slow.

**Solution (choose one):**
- **Now:** Accept brute-force, add monitoring, document limits
- **20K-100K chunks:** Add pre-filtering by metadata (10x speedup)
- **100K+ chunks:** Use external vector DB (Qdrant, Lance, FAISS)

**SurrealDB v2 limitation:** No native HNSW/ANN indexes yet.

**Action:** Add latency monitoring now, plan migration path.

---

### 2. Data Integrity with SCHEMALESS

**Problem:** No database-level validation, type safety, or foreign key constraints.

**Why it matters:** Bugs can corrupt data, orphaned records possible.

**Solution:**
1. **Short-term:** Add `Validate` trait, enforce in repositories
2. **Medium-term:** Convert critical tables to SCHEMAFULL
3. **Long-term:** Implement migration system

**Action:** Implement validation trait this week.

---

### 3. Missing Transaction Boundaries

**Problem:** Multi-step operations (delete + insert loops) lack atomicity.

**Why it matters:** Failures mid-operation leave partial state, data loss risk.

**Solution:**
```rust
self.db.query("BEGIN TRANSACTION").await?;
// ... multi-step operations ...
self.db.query("COMMIT TRANSACTION").await?;
```

**Critical operations:**
- `save_chunks()` - delete + insert loop
- `delete_entity()` - cascading deletes
- `delete_source()` - delete chunks + source

**Action:** Wrap all multi-step writes in transactions.

---

### 4. Memory Growth Unbounded

**Problem:** No memory limits configured, all vectors loaded into memory.

**Why it matters:** Could exhaust memory on low-end systems or large datasets.

**Solution:**
1. Configure RocksDB cache limit (128-256 MB)
2. Add memory usage logging
3. Implement pagination for large result sets
4. Enforce max result limits

**Current footprint:** ~250-400 MB at 50K chunks (acceptable).

**Action:** Set cache limits and add monitoring.

---

## Prioritized Action Plan

### 🔴 Critical (This Week)

1. **Add validation trait** - Prevent data corruption
   - Implement `Validate` for all domain types
   - Enforce in repository save methods
   - File: `engram-core/src/*.rs`

2. **Add transactions** - Ensure atomicity
   - Wrap `save_chunks()`, `delete_entity()`, `delete_source()`
   - Add rollback on errors
   - File: `engram-store/src/repos/*.rs`

3. **Configure memory limits** - Prevent unbounded growth
   - Set RocksDB cache size (128-256 MB)
   - Add memory logging
   - File: `engram-store/src/config.rs`

4. **Add monitoring** - Detect performance issues early
   - Track vector search latency
   - Log chunk count
   - Alert when P95 > 200ms
   - File: `engram-index/src/service.rs`

### 🟡 High Priority (Next Sprint)

5. **Batch operations** - 10x faster indexing
   - Batch insert for chunks (100 per query)
   - Reduces round-trips from 1,000 to 10
   - File: `engram-store/src/repos/document.rs`

6. **Add indexes** - Faster queries
   - `idx_session_status_project` (composite)
   - `idx_entity_name_lower` (case-insensitive search)
   - Timestamp indexes for sorting
   - File: `engram-store/src/repos/*.rs`

7. **Implement pagination** - Bounded memory usage
   - Cursor-based pagination for lists
   - Max result limits (100-200)
   - File: `engram-store/src/repos/*.rs`

8. **Backup/restore** - Data safety
   - `engram backup` command
   - `engram restore` command
   - File: `engram-cli/src/main.rs`

### 🟢 Medium Priority (Future)

9. **Hybrid SCHEMAFULL** - Better data integrity
10. **Pre-filtering** - Extend brute-force viability
11. **Graph traversal** - Multi-hop queries
12. **Benchmarks** - Track performance

---

## Performance Projections

| Chunk Count | Brute Force | With Pre-filtering | With ANN (Qdrant) |
|-------------|-------------|-------------------|-------------------|
| 10,000 | ✅ ~100ms | ✅ ~50ms | ✅ ~5ms |
| 50,000 | ⚠️ ~500ms | ✅ ~100ms | ✅ ~10ms |
| 100,000 | ❌ ~1000ms | ⚠️ ~200ms | ✅ ~15ms |
| 500,000 | ❌ ~5000ms | ❌ ~1000ms | ✅ ~25ms |

**Recommendation:**
- **< 20K chunks:** Keep brute-force
- **20K-100K chunks:** Add pre-filtering
- **> 100K chunks:** Use external vector DB

---

## Memory Footprint Estimate

| Component | 50K chunks | 100K chunks |
|-----------|-----------|-------------|
| Vectors (384-dim) | 114-152 MB | 228-304 MB |
| Chunk metadata | 31 MB | 62 MB |
| Entity graph | 3 MB | 3 MB |
| Session history | 4 MB | 4 MB |
| SurrealDB overhead | 100-200 MB | 100-200 MB |
| **Total** | **252-390 MB** | **393-566 MB** |

**Acceptable for local-first application** (< 5% of 8GB RAM).

---

## SurrealDB v2 Best Practices

### ✅ Already doing well:
- Raw SurQL queries (avoids SDK issues)
- Custom datetime deserialization
- Proper use of `meta::id()`
- Graph relationships as edge tables

### ⚠️ Missing:
- Transaction boundaries
- Batch operations
- Live queries (not critical)
- Graph traversal syntax

---

## Index Strategy

### Current indexes: ✅ Good coverage
- All foreign keys indexed
- Common filters indexed (status, agent, project)
- Unique constraints where appropriate

### Recommended additions:
1. Composite: `idx_session_status_project`
2. Case-insensitive: `idx_entity_name_lower`
3. Sorting: timestamp indexes
4. Full-text: context search (if available)

---

## Data Integrity Strategy

### Current: SCHEMALESS everywhere
- **Pros:** Flexible, avoids SDK issues
- **Cons:** No validation, no foreign keys, no defaults

### Recommended approach:
1. **Short-term:** Application-level validation
2. **Medium-term:** Hybrid SCHEMAFULL/SCHEMALESS
3. **Long-term:** Migration system

---

## Key Takeaways

1. **Your foundation is solid** - clean architecture, good patterns
2. **Scale to 50K chunks is achievable** with current approach
3. **Beyond 50K requires changes** - pre-filtering or external vector DB
4. **Data integrity needs attention** - add validation and transactions
5. **Memory is manageable** - configure limits and monitor

---

## Next Steps

1. Read full review: [EXPERT_REVIEW.md](./EXPERT_REVIEW.md)
2. Implement critical items (validation, transactions, monitoring)
3. Set performance targets and track metrics
4. Plan migration path for scale (pre-filtering → external vector DB)

---

## Questions?

- **Vector search too slow?** → Add monitoring, then pre-filtering
- **Data corruption concerns?** → Add validation trait
- **Memory issues?** → Configure RocksDB cache limits
- **Need to scale beyond 100K?** → Evaluate Qdrant/Lance

See full review for detailed implementation guidance.
