# Native strict successor Stage C2B1 — frozen sealed-Memory acquisition

## Status and exact claim boundary

This design is frozen for implementation only after Stage C2A has passed its deferred integration
gate and has an accepted evidence record. It authorizes one provider-free prototype: a fresh
evaluator-owned SurrealDB 2.6.0 Memory datastore whose complete lifetime and every write ingress
remain sealed inside a private child of the C2A semantic module.

If accepted, C2B1 may prove only this statement:

> A continuously owned fresh Memory store, populated exclusively through the frozen bounded
> data-only ingress, can execute the exact C2A bare snapshot statement and move one atomic,
> normalized, bounded snapshot into C2A without exposing raw rows or its MAC key.

C2B1 is not persistent-store closure. It neither reads nor models an existing, copied, reopened or
live Engram database. It does not authorize a correction action, daemon, provider, authentication
operation, adapter, relay, CLI run surface or flagship claim.

No implementation is authorized while the C2A acceptance gate is incomplete. Adding the C2B1
child and dependencies will change the semantic parent and lockfile identities, so all C2A
goldens and source firewalls must run again during C2B1 acceptance.

## Frozen inputs and provenance

The authoritative inputs at this freeze are:

```text
C2A design
  evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2A_SEMANTIC_VALIDATOR_FROZEN_2026-09-06.md
  5a3599038198ba95e1c6ad7421b4ba2ee703b2b96021c04a34dca196952e27bc

C2B acquisition research
  evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B_ACQUISITION_RESEARCH_2026-09-06.md
  3d5c6e2203ce95568ef38e208cc093e889ec5c23fbbcc27d16bf12b203de4129

C2A implementation baseline
  engram-eval/src/native_successor_semantic.rs
  4da0ce228b5127587a72f65d19efd58ddbc7d6e7d00dc98a41117ff18dbce4b5

Private module declaration baseline, which C2B1 must not change
  engram-eval/src/lib.rs
  2eea0ad9fcafdd9cc7f8a9e813a41e594e293a853a453b482c2ec41cd1064a74

Evaluator manifest baseline
  engram-eval/Cargo.toml
  5238c63cb31eb420ad27ae50c44e92cfbb39bdab1c895dade1cf066770c480c6

Workspace manifest baseline, which C2B1 must not change
  Cargo.toml
  ec61531f8ffd401211f649c7b91e28a4ee5f0fa1a0fe2f199c63a841c9724f5a

Dependency lock baseline
  Cargo.lock
  8741698619a41dcce8f58aae3609e6ee48929d6c2e91b955caf0ed586920572e
```

The accepted target is `aarch64-apple-darwin`. The accepted compiler is
`rustc 1.93.0 (254b59607 2026-01-19)` and the accepted Cargo is 1.93.0 commit
`083ac5135f967fd9dc906ab057a2315861c7a80d`.

The two new direct dependency edges are exact:

```toml
getrandom = "=0.3.4"
surrealdb-core = { version = "=2.6.0", default-features = false, features = ["kv-mem"] }
```

Their pinned registry checksums are:

```text
getrandom 0.3.4
  899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd
surrealdb-core 2.6.0
  c48e42c81713be2f9b3dae64328999eafe8b8060dd584059445a908748b39787
```

No new package version is expected because both packages already occur in the baseline lockfile.
The lockfile will still change to record direct evaluator dependency edges and its accepted hash
must be recorded after implementation.

Acceptance must record the exact resolved feature graph and prove that C2B1 imports
`surrealdb_core`, not the high-level `surrealdb::Surreal` SDK. Workspace feature unification may
compile storage or network-capable code for other Engram crates; that is not C2B1 authority. The
recursive C2B1 source firewall and literal Memory constructor are the reachable-code boundary.

## Exact source boundary

Only these implementation changes are authorized:

```text
engram-eval/src/native_successor_semantic/acquisition.rs       new private child
engram-eval/src/native_successor_semantic.rs                   one private child declaration
engram-eval/Cargo.toml                                         two exact direct dependencies
Cargo.lock                                                     derived dependency-edge update
```

The parent adds exactly one production declaration:

```rust
mod acquisition;
```

`engram-eval/src/lib.rs` remains byte-for-byte unchanged. No public or crate-visible export, peer
module reference, binary, CLI, MCP method, adapter, runner, daemon, relay, correction, isolation or
VM integration is authorized. C2A types remain private; they are not widened to `pub(super)` or
`pub(crate)`. The descendant may access its ancestor's private types under normal Rust privacy.

The child and all of its types remain non-public. At most one `pub(super)` acquisition entry may
exist solely so the semantic parent can consume it in a later separately frozen slice. C2B1 itself
has no non-test production caller. There is no runnable surface.

The accepted implementation record must hash the new child, modified semantic parent, unchanged
`lib.rs`, evaluator manifest, unchanged workspace manifest, lockfile, focused test executable and
all final reports. Any source or dependency change outside this boundary invalidates C2B1.

## Sealed owner and lifecycle

The child defines a move-only owner with states equivalent to:

```text
C2MemoryOwner<Empty>
  -> C2MemoryOwner<Ready>
  -> consumed success or terminal categorical failure
```

The owner and its state markers implement no `Clone`, `Copy`, `Debug`, `Display`, `Serialize` or
`Deserialize`. The owner is not generic over a backend and accepts no endpoint, path, configuration,
backend enum, connection, `Db`, `Surreal<Any>`, locality boolean, session, query, AST, closure or
handle.

Only `C2MemoryOwner<Empty>` may construct the datastore. It uses exactly:

- direct `surrealdb_core::kvs::Datastore::new("memory")` with that literal;
- `surrealdb_core::dbs::Capabilities::none()`;
- one owner session with exact namespace `engram` and database `main`;
- no notifications, slow-log configuration, temporary directory or caller variables.

Neither a datastore reference nor a session reference escapes. The owner contains the datastore,
the target, and only non-content cumulative accounting needed to prove its closed ingress. It does
not retain a second raw-row mirror after a successful initial write.

The owner serializes its own access and permits exactly one production initialization and exactly
one production collection. Any validation, conversion, engine or outcome-certainty failure consumes
or permanently terminalizes it. There is no retry, reconnect, restart, reset or reopen path.

Authority exists only when the one owner-consuming acquisition future is driven to completion and
returns success. Dropping or aborting that future returns no value, can never mint authority and
leaves no owner with which to retry. A canceled future may leave a SurrealDB commit-pool job alive
until the dependency completes it or the test process exits. C2B1 does not claim cancellation-safe
worker teardown. A later production caller must separately prove a non-cancelable execution
boundary before consuming this prototype.

The source must not reference `engram-store`, `engram-index`, `engram-mcp`, filesystem, environment,
network, process, provider, logging or time-now APIs. It may use async only for the exact direct
datastore calls. It creates no runtime, task, process or explicit thread.

SurrealDB Memory commits use a process-global `OnceLock` commit pool. C2B1 therefore does not claim
immediate dependency-worker teardown, absence of dependency threads, timing isolation or complete
destruction of dependency-owned buffers when the owner is dropped. Successful commit completion is
awaited before acquisition may proceed.

## Closed typed ingress

The sole production ingress is one move-only, private complete generation. It contains the target
and exact domain values for these eight valid-state families:

```text
MemoryItem
CorrectionProposal
Project
Task
GitRepository
LocalCheckout
MonorepoComponent
ProjectRepositoryLink
```

There is no production forget-receipt input because a nonempty `memory_forget_receipt` table is
terminal-invalid in C2A. The generation accepts no `T: Serialize`, arbitrary map,
`serde_json::Value`, Surreal value, query text, AST, binding map, raw bytes or callback.

Each statically typed family has its own non-generic conversion. It must:

1. exhaustively visit borrowed domain fields before JSON serialization;
2. reject over-limit strings, vectors, table counts and checked aggregate payload estimates before
   allocating a second content representation;
3. serialize only that exact domain type to JSON;
4. construct all exact persisted projections from the frozen C2A formulas;
5. reject duplicate domain IDs and record/projection collisions;
6. construct the exact prospective C2A row, including `record_id`;
7. charge and secret-scan the complete prospective C2A virtual input with empty prior bindings;
8. only after every row and the complete generation pass, convert rows to restricted native values;
9. write the entire complete generation in one transaction; and
10. drop all input and intermediate row content after commit.

Before the first JSON allocation, the borrowed visitor computes a checked conservative upper bound
for the exact JSON shape that the later domain serializer will emit. It mirrors every field,
container, enum tag, optional-field omission and persisted projection of those concrete types. Its
charges are frozen as follows:

- object and array delimiters, commas and colons are charged at their exact byte counts;
- a string or object key of `n` UTF-8 bytes is charged as `2 + 6 * n` bytes;
- `null`, `true` and `false` are charged as four, four and five bytes respectively; and
- every finite number is charged as 32 bytes.

Every multiplication and addition is checked. The visitor first enforces the exact scalar, vector,
table and projection-collision limits, then rejects arithmetic overflow or a complete-generation
upper bound above `16 * MAX_RAW_BYTES` (33,554,432 bytes) as
`LimitExceeded { table_overflow_mask: None }`. The factor is a duplicate-allocation ceiling, not an
acceptance relaxation: the exact prospective C2A virtual-input meter still requires at most
`MAX_RAW_BYTES` before mutation. The conservative string and number charges must be proven no
smaller than every byte sequence emitted by the pinned `serde_json` serializer, including control
escapes. Thus every C2A-valid generation can reach exact accounting while child-created duplicate
content remains bounded even before serialization.

The preflight reuses C2A's private `validate_limit_and_secret_boundary` and exact accounting rules.
It occurs before datastore construction or mutation. It enforces all table counts, per-row bytes,
global bytes, nodes, depth, object-key counts, vector counts, scalar limits and checked arithmetic
against the complete prospective generation. Per-row checks alone are insufficient.

The borrowed visitor prevents the child from duplicating an obviously oversized typed generation,
but the caller necessarily allocated that generation before entry. C2B1 claims bounded accepted
store content and bounded child-created duplicate representations, not bounded allocation by a
future caller before the private entry is invoked.

Because production permits only one complete initialization, it has no replacement-delta API. A
future mutation slice must separately freeze complete post-state accounting with checked removal
and addition. Existing Engram `UPSERT ... SET` writers are forbidden because schemaless updates can
retain unknown fields and their separate calls do not form one nine-table generation transaction.

A `#[cfg(test)]`-only, bounded JSON fixture seam may add unknown JSON-native fields, a synthetic
forget receipt or cap-plus-one rows to test preservation and rejection. It must remain private,
must pass explicit test bounds before write, and must never accept a native Surreal value in its
production-shaped path. Any separate native-variant injector is also test-only and may exist only
to prove that the collector rejects variants which the engine can represent.

## Restricted JSON-to-native conversion

The writer never calls a generic Surreal serializer or a public `.bind(T: Serialize)` path. Generic
Surreal serialization is forbidden because it has fast paths for executable `Future`, `Block`,
`Function`, `Subquery` and `Expression` values.

After complete JSON preflight, a private path-aware converter admits only:

- JSON null to native `NULL`;
- boolean;
- bounded signed `i64`;
- finite bounded `f64`;
- bounded UTF-8 string;
- bounded array; and
- bounded object with bounded keys.

Unsigned integers outside `i64` and non-finite or non-representable numbers reject. No generic
`Value::from` or serde bridge may choose a native variant for an object.

For each top-level row, the converter extracts `record_id`, requires canonical UUID-v7 text,
constructs one native `Thing` with the exact fixed table and string UUID record ID, and removes
`record_id` from stored content. Caller content named `id` or a second `record_id` rejects. The
converter creates native `Datetime` only at the exact top-level timestamp paths frozen by C2A and
only after the text is proven canonical UTC RFC3339. Embedded payload timestamps remain strings.
Required nullable fields persist as explicit native `NULL`; optional fields remain omitted. The
production writer never creates native `NONE`.

All other native variants are forbidden, including native UUID, Decimal, Bytes, Duration,
Geometry, Range, Param, Idiom, Table, Edges, Refs, Model, Closure, Query, Block, Future, Function,
Subquery, Expression and any future unknown variant. The outer record `Thing` and authorized
top-level `Datetime` values are the only non-JSON-native exceptions.

This closed ingress makes `SELECT *` computation observationally data-only. The claim fails if a
raw handle, generic serializer, arbitrary query, imported state, second writer or unverified reopen
is introduced.

## Atomic complete-generation write

Initialization uses this exact 572-byte literal with no trailing newline and SHA-256
`9f6c41b6c7d0006db0a03f29fd951bb3c81df19946748faaac768af013c98a05`:

```sql
BEGIN TRANSACTION;
INSERT INTO memory_item $memory_item RETURN NONE;
INSERT INTO correction_proposal $correction_proposal RETURN NONE;
INSERT INTO memory_forget_receipt $memory_forget_receipt RETURN NONE;
INSERT INTO work_project $work_project RETURN NONE;
INSERT INTO work_task $work_task RETURN NONE;
INSERT INTO git_repository $git_repository RETURN NONE;
INSERT INTO local_checkout $local_checkout RETURN NONE;
INSERT INTO monorepo_component $monorepo_component RETURN NONE;
INSERT INTO project_repository_link $project_repository_link RETURN NONE;
COMMIT TRANSACTION;
```

The parsed AST has exactly 11 statements: `Begin`, the nine `Insert` statements in the displayed
order, and `Commit`. Every insert has one fixed literal destination table, one
`Data::SingleExpression` containing only its same-named parameter, `Output::None`, and false/absent
values for ignore, update, timeout, parallel, relation and version.

The internal `Variables` map has exactly the nine displayed parameter names. Each value is one
restricted native array. Every row is one native object whose only `Thing` is its `id`; its other
members are the preflighted persisted fields after removing `record_id`. The
`memory_forget_receipt` array is exactly empty in production. Every other array may be empty, and an
empty array must make its insert succeed with no mutation.

The complete parsed write AST is structurally checked before use. It inserts all nine arrays inside
one explicit transaction and uses `RETURN NONE` for every data statement. Neither parameter names
nor values come through a generic bind or caller map.

The transaction begins only after the complete generation and every native variable have passed
preflight. `Datastore::process` must return exactly nine responses because `Begin` and `Commit` do
not produce response entries. Every response must be successful native `Value::Array` with zero
elements. Under the pinned engine, `RETURN NONE` maps every row output to `Error::Ignore`, so the
statement iterator collects no rows and converts that empty vector to the empty native array. The
response vector is returned only after a successful commit; a commit failure rewrites the
transaction responses as failures. Only that exact vector of nine empty arrays may mint
`C2MemoryOwner<Ready>`. Native `NONE`, native `NULL`, a nonempty array or any other value is an
outcome-contract failure. Any process error, response-length mismatch, outcome-contract failure or
uncertain commit state maps to `engine_outcome_uncertain`, consumes the empty owner and yields no
usable datastore.

Production never deletes, replaces or initializes twice. The `#[cfg(test)]`-only A/B replacement
uses this exact 902-byte literal with no trailing newline and SHA-256
`0a30e2e0a890b16abedf2cd91282d49cd55c82885b85cbc28b6e90e2d161211d`:

```sql
BEGIN TRANSACTION;
DELETE memory_item RETURN NONE;
DELETE correction_proposal RETURN NONE;
DELETE memory_forget_receipt RETURN NONE;
DELETE work_project RETURN NONE;
DELETE work_task RETURN NONE;
DELETE git_repository RETURN NONE;
DELETE local_checkout RETURN NONE;
DELETE monorepo_component RETURN NONE;
DELETE project_repository_link RETURN NONE;
INSERT INTO memory_item $memory_item RETURN NONE;
INSERT INTO correction_proposal $correction_proposal RETURN NONE;
INSERT INTO memory_forget_receipt $memory_forget_receipt RETURN NONE;
INSERT INTO work_project $work_project RETURN NONE;
INSERT INTO work_task $work_task RETURN NONE;
INSERT INTO git_repository $git_repository RETURN NONE;
INSERT INTO local_checkout $local_checkout RETURN NONE;
INSERT INTO monorepo_component $monorepo_component RETURN NONE;
INSERT INTO project_repository_link $project_repository_link RETURN NONE;
COMMIT TRANSACTION;
```

Its parsed AST is exactly `Begin`, nine same-order table `Delete` statements, nine same-order
`Insert` statements identical to initialization, and `Commit`. Each delete has its fixed literal
table as the sole `what` value, `only=false`, `Output::None`, no `with`, condition, timeout or
explain value, `parallel=false`, and no other target. The test process requires exactly 18
successful empty native arrays under the same `RETURN NONE` rule. The normal replacement seam
accepts only A or B as already preflighted bounded complete fixtures. One separate `#[cfg(test)]`
rollback seam may replace only the already-preflighted B `$work_task` top-level array with native
`NULL`. The first nine deletes and first four inserts therefore execute inside the replacement
transaction before the fifth insert deterministically fails its required array/object shape. The
seam exposes no other variable, value or statement choice and exists only to prove rollback by
subsequently observing the complete prior A generation. No single-row or generic create, update,
upsert, delete or query surface exists.

## Exact one-statement acquisition

The only production read is this exact 570-byte literal with no trailing newline and SHA-256
`cf1034a335af240c0726e5385207acc4e4e49273dc016891138c863e8ba096f5`:

```sql
RETURN {
  memory_item: (SELECT * FROM memory_item LIMIT 65),
  correction_proposal: (SELECT * FROM correction_proposal LIMIT 33),
  memory_forget_receipt: (SELECT * FROM memory_forget_receipt LIMIT 33),
  work_project: (SELECT * FROM work_project LIMIT 9),
  work_task: (SELECT * FROM work_task LIMIT 65),
  git_repository: (SELECT * FROM git_repository LIMIT 17),
  local_checkout: (SELECT * FROM local_checkout LIMIT 33),
  monorepo_component: (SELECT * FROM monorepo_component LIMIT 65),
  project_repository_link: (SELECT * FROM project_repository_link LIMIT 65)
};
```

The child parses that private constant once for the consumed collection. Before execution it
structurally requires:

- exactly one `Output` statement;
- one object with exactly the nine keys above;
- each value exactly one `Subquery::Select`;
- `SELECT *` from exactly one fixed literal table;
- the exact integer cap-plus-one limit for that table; and
- every other statement, select, object and output field absent or its exact inert default.

The AST rejects any binding, parameter, function, expression, filter, split, group, order, limit
change, start, fetch, version, timeout, parallel option, explain option, temporary file, script,
transaction wrapper, additional field or additional statement. The already inspected owned AST,
not reparsed mutable text, is passed once to `Datastore::process` with no variables. The result must
contain exactly one response and one successful output.

The exact read is non-writeable in pinned SurrealDB 2.6.0 and runs in one engine transaction. C2B1
claims only that this snapshot statement is non-writeable after opening and populating its own
fresh store. Datastore construction and the explicit initialization transaction are mutations.

## Native response gate and C2A handoff

The engine fully materializes its native response before returning. Safety before that point comes
from the fresh store's closed and cumulatively bounded ingress, not from post-query metering.

Immediately after return, before JSON normalization, the collector:

1. requires one outer native object and exactly the nine table keys;
2. requires each table value to be one native array;
3. computes the complete nine-bit table-overflow mask before any row-derived error;
4. dry-runs the exact prospective normalized JSON framing against the C2A bounds;
5. scans every native string and object key for secret material; and
6. rejects unsupported native variants categorically.

Path-aware normalization then:

- accepts the engine-created outer `id` only as a `Thing` for the exact current table;
- requires that record ID to be the exact canonical UUID-v7 string form;
- rejects any stored `record_id` collision;
- removes native `id` and injects its string payload as JSON `record_id`;
- converts native top-level `Datetime` to canonical UTC RFC3339 text;
- maps native `NONE` to JSON null or omission only at the exact C2A path table;
- maps native `NULL` to JSON null;
- preserves every JSON-native unknown key and value recursively; and
- never stringifies, drops, defaults or repairs any other value.

The dry run is the authoritative pre-allocation meter. It traverses the owner-held target, an empty
prior-binding array and the nine native table arrays as the exact 11-key C2A virtual input. It calls
the same private C2A node, byte, depth, key, vector, scalar and `RowBytes` charging primitives that
`account_virtual_input` uses, in the same table and row order.

For prospective normalization it charges the engine `id: Thing` as the replacement
`record_id: <canonical UUID text>` member, charges a native `Datetime` as its canonical UTC RFC3339
JSON string, charges a permitted native `NONE` as either the exact JSON-null member or zero bytes
and nodes for an omitted member, and charges native `NULL` as JSON null. Arrays, objects, numbers,
strings and keys use the exact C2A JSON framing. A stored `record_id` collision rejects before the
replacement charge. Unsupported variants consume one native node for traversal safety but cannot
complete prospective framing and are marked unsupported without inspecting or formatting their
contents.

The dry run allocates no row/object/array JSON representation. A bounded canonical timestamp string
is its only normalization-sized temporary. After the dry run succeeds, the second pass constructs
the JSON rows. C2A's `account_virtual_input` is then run on that value and its complete
`RawAccounting` must equal the dry-run accounting exactly; disagreement is `invalid_response`.

Although production ingress never creates `NONE`, production normalization must handle it exactly
because the frozen C2A collector contract distinguishes `NONE`, `NULL` and omission. Test-only
native fixtures exercise every permitted and forbidden path.

The collector constructs one private `C2RawSnapshot` directly. It never serializes or returns raw
rows. It performs C2A's limit and secret gate again, then mints a fresh key and immediately moves
the raw snapshot, an empty prior-binding vector and the key into `validate_semantic_snapshot`.
C2B1 does not authorize an applied-correction journey, so nonempty prior bindings are impossible.

Every acquisition error is a fixed categorical variant; the overflow variant's optional nine-bit
mask is the only payload. Raw Surreal errors, rows, targets, keys and canary text are never
formatted, serialized, logged or returned. A pre-C2A failure must prove that C2A was not invoked.

## Exact errors and precedence

The private error enum has exactly these variants:

```text
LimitExceeded { table_overflow_mask: Option<u16> }
SecretMaterial
InvalidGeneration
QueryContractInvalid
EngineFailure
EngineOutcomeUncertain
InvalidResponse
UnsupportedNativeValue
EntropyUnavailable
InvalidRecord
InconsistentProjection
IncompleteDeletion
AmbiguousIdentity
ScopeMismatch
AppliedProvenanceUnproven
```

It implements no serialization. Its `Display` values are the exact snake-case variant names above;
`LimitExceeded` displays only `limit_exceeded`, never its mask. `Debug`, if derived for private
tests, contains only the variant and bounded mask. No variant contains source data or a dependency
error.

The engine boundary and error partition are exact. `QueryContractInvalid` covers literal, hash,
parse or AST mismatch before datastore use. `EngineFailure` covers datastore construction failure
and any collection-read process failure, because collection dispatches no write.
`EngineOutcomeUncertain` covers every failure after an initialization or test-replacement write is
dispatched: a process error, wrong response count, unsuccessful response, response value other than
an empty native array, commit uncertainty or a controlled cancellation observation. It permanently
consumes the relevant owner and permits no retry. Typed conversion and complete preflight finish
before write dispatch and retain their earlier categorical errors. After a successful read,
malformed response shape is `InvalidResponse`; after a write, the corresponding shape mismatch is
always `EngineOutcomeUncertain` because mutation may have occurred.

The deterministic precedence is:

1. static read/write literal, hash and AST contract;
2. borrowed typed-input structural limits;
3. complete prewrite C2A table/count/byte/node/depth limits, with the complete table mask;
4. complete prewrite secret scan;
5. typed projection/native-conversion validity;
6. datastore construction and write outcome;
7. response count, outer object, exact table-key set and table-array shape;
8. complete cap-plus-one table mask;
9. prospective native-to-JSON dry-run limit result;
10. supported native siblings' complete secret result;
11. unsupported native-variant result;
12. normalized/dry-run accounting equality;
13. second C2A limit and secret boundary;
14. MAC-key entropy mint; and
15. C2A semantic errors in C2A's own frozen precedence.

An unsupported value is never traversed internally or formatted. Supported siblings are still
scanned, so a supported secret takes precedence over an unsupported sibling after all limits.
Engine failure precedes response inspection because no response authority exists.

## Sole production OS-CSPRNG mint

The child contains the only production constructor of `C2OneShotMacKey`. It constructs one
zero-initialized key value, calls `getrandom::fill` exactly once on its 32-byte field, performs no
module-level retry, and immediately moves the successful key into exactly one C2A validation. On
error, normal drop zeroizes the partially written key and the collector returns only
`entropy_unavailable`.

There is no seed, byte-array parameter, accessor, clone, reuse, persistence, output, fallback or
caller-selected entropy source. The existing deterministic constructor remains `#[cfg(test)]` in
the parent. A separate child-local test seam may accept one `FnOnce` filler only under
`#[cfg(test)]`; it is absent from production compilation.

Acceptance pins `getrandom` 0.3.4, the target, toolchain, effective Cargo configuration, effective
compiler flags and final test/binary hashes. It must prove that no `getrandom_backend` override,
custom backend symbol, `rdrand`, `rndr`, `unsupported`, `RUSTFLAGS` or
`CARGO_ENCODED_RUSTFLAGS` altered the accepted build. On the accepted macOS target the selected
backend is `getentropy`.

The exact claim is one child-source crate-level fill call per MAC-key mint, not one kernel syscall
and not one entropy call in the process. `Datastore::new("memory")` may independently obtain OS
randomness for its internal UUID and uses a system clock. C2B1 does not claim erasure of registers,
allocator copies, dependency buffers, swap, crash dumps or the entire process heap.

## Required provider-free tests

The accepted implementation must include at least these tests:

1. exact read and both write-literal bytes, lengths and hashes, plus mutation of every AST field;
2. rejection of extra statements, object keys, tables, fields, options, functions and variables;
3. zero, one, extra, failed and malformed response/output shapes, including exact proof that every
   successful write response is an empty native array and rejection of `NONE`, `NULL`, nonempty
   arrays and every other value;
4. exact cap and cap-plus-one behavior for all nine tables and every combined overflow mask bit;
5. exact and plus-one row bytes, global bytes, nodes, depth, keys, vectors and scalar sizes;
6. checked arithmetic and complete-generation cumulative accounting;
7. every `NONE`, `NULL`, omission, datetime, record-ID and collision path;
8. every allowed native value and every forbidden executable or extended native variant;
9. proof that a test-injected stored future or inert block is evaluated under `SELECT *`, and that
   a stored function at least enters evaluation and is denied by `Capabilities::none()`, while no
   production ingress can construct or persist any of them;
10. unknown JSON-native fields preserved unchanged through collection and then rejected by C2A;
11. secret canaries in every table, target, unknown key and nested value, with no outward leakage;
12. database, conversion, entropy and C2A failures mapped only to categorical errors;
13. proof that pre-C2A failure never calls C2A and collection consumes its owner;
14. exactly one child MAC-mint filler invocation, partial-write error zeroization, no retry,
    distinct successful test fills and type/source proof that a moved key cannot be reused;
15. two complete valid generations A and B with one common valid target identity, distinguishable
    non-target IDs, projections, content and cross-table relationships;
16. one confirmed exact observation of A and one of B, followed by at least 512 successful
    overlapping replacement/read rounds with zero unexplained acquisition or C2A failures;
17. every overlapping result matching every normalized row and table cardinality of complete A or
    complete B, never merely matching a final audit or a subset;
18. the exact test-only `$work_task = NULL` failure inside one replacement transaction after its
    first nine deletes and four inserts executed, with the complete prior generation then observed
    and no hybrid residue;
19. cancellation at every controlled await seam returning no result, minting no authority,
    permitting no retry and leaking no handle or raw content;
20. tied timestamps and arbitrary insertion order producing the same normalized semantic result;
21. serial and parallel harness execution without shared-store or key cross-talk; and
22. output/canary scans covering errors, test captures and every serializable success audit; and
23. syntax-aware source proof that the owner and both state markers implement none of `Clone`,
    `Copy`, `Debug`, `Display`, `Serialize` or `Deserialize`, and that exactly one production
    initialization path and exactly one production collection path exist.

Test-only concurrency may place one direct Memory datastore in an `Arc` solely inside the child
test module. Production contains no `Arc<Datastore>`, clone or second handle. Test-only overflow,
unknown-field and forbidden-native injectors must be individually source-gated and bounded.

## Source and acceptance firewalls

Source checks are scoped rather than one contradictory token scan:

- **Child production:** private ownership; no returned handle, session, key, raw row, native value
  or query; no high-level `Surreal`, `Any`, Remote, endpoint, path or backend selection; no generic
  type-parametric `Serialize`, bind, dynamic SQL, caller AST or generic native-value ingress; no
  filesystem, environment, network, process, logging, provider or time-now reference; no runtime,
  task or explicit thread construction; no production `Arc<Datastore>`; exactly the allowed
  direct-core constructor, capabilities, session, write ASTs and read AST; and exactly one
  production MAC-key fill call with no fixed/caller-byte constructor. A syntax-aware gate
  enumerates derives and trait impls for the owner and state markers, rejects `Clone`, `Copy`,
  `Debug`, `Display`, `Serialize` and `Deserialize`, and counts exactly one production
  initialization entry and one production collection entry with no alias or second constructor.
- **Child tests:** only the enumerated bounded JSON/native injectors, `FnOnce` entropy seam,
  controlled cancellation seam, exact `$work_task = NULL` rollback injector, A/B
  `Arc<Datastore>` and fixed replacement AST exceptions. Each is under `#[cfg(test)]` and absent
  from the child production prefix.
- **Semantic parent:** the exact accepted C2A source plus only `mod acquisition;`; its original C2A
  capability, serialization and move-only firewalls still pass unchanged.
- **Peers and public surface:** `lib.rs` is unchanged, and no peer, binary or public facade names
  the acquisition child or any C2B1 symbol.

Acceptance requires, in this order:

1. accepted C2A evidence on its pre-child source identity;
2. positive mandatory disk reserve and absent repository `target/debug`;
3. implementation only in the exact source boundary;
4. repeated focused C2B1 and C2A suites under serial and parallel test harnesses;
5. full provider-free library and integration suites;
6. strict Clippy, format, whitespace and source-firewall checks;
7. all build output under the approved external target directory;
8. final dependency tree, features, target, toolchain, config, flag and artifact provenance;
9. final hashes for every authorized file and report; and
10. three fresh independent exact-source audits with P0=0 and P1=0.

The accepted build and test processes start under an outer attested environment with no variable
whose name begins `SURREAL_`, no `RUSTFLAGS`, no `CARGO_ENCODED_RUSTFLAGS` and no Cargo target
config that adds a `getrandom_backend` cfg. The child itself never reads or mutates the environment.
A separate poisoned-environment process test must either produce the same bounded semantic result
or a categorical non-authorizing failure; it may never widen the query or ingress surface.

No provider, daemon, listener, network, filesystem-backed datastore, authentication cache, live
setting or user database may be accessed during C2B1 implementation or acceptance. Normal source,
build and external-target filesystem I/O is not a datastore claim.

## Deferred C2B2 and C2B3 gates

C2B2 remains mandatory. It must prove an evaluator-owned persistent RocksDB lifecycle, exact clean
Rocks configuration, bounded resource behavior, no handle escape and close/reopen persistence.
Stock SurrealDB reads many process-global `SURREAL_*` settings and may create background RocksDB
work. On this macOS host `RLIMIT_RSS` is advisory and the attempted address-space shell limit was
not an enforceable hard cap. Therefore a strong C2B2 claim remains gated on the independently
attested mount-free VM, a namespace/cgroup-equivalent boundary, or a separately reviewed narrow
SurrealDB patch with an adequate resource proof.

C2B3 remains separately mandatory. It must move one prior pending-correction binding from one
successful pre-state, seal exactly one operator action, perform one post-state acquisition with a
fresh independent key, compare unchanged state through moved private canonical evidence rather
than MAC equality, and bind both states plus the action to one live process receipt.

Neither later gate may treat C2B1's Memory result as persistent or live-store evidence.

## Explicit nonclaims

C2B1 does not prove or authorize RocksDB, persistence, close/reopen, import, copy, checkpoint,
crash recovery, a current user store, existing Engram writers, a real correction workflow, deletion
propagation, filesystem immutability, daemon or MCP transport, network denial in compiled
dependencies, provider execution, authentication, adapter/relay integration, process chronology,
safe recovery or retry after async cancellation, immediate dependency-worker teardown, bounded
allocation before a future caller constructs its typed input, exact dependency heap allocation,
complete memory erasure, swap/crash-dump resistance, protection from a malicious same-UID process,
kernel, debugger or hardware, or flagship completion.
