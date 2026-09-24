# Native strict successor Stage C2B — acquisition research note

Date: 2026-09-06 (Asia/Jerusalem)

## Status and authority boundary

**Research only. Not frozen. Not implementation authority.**

This note preserves provider-free source research performed while C2A acceptance remained blocked
only by the mandatory filesystem-reserve gate. It does not modify or supersede the frozen C2A
design, authorize a source boundary, choose a dependency change, or satisfy any C2B claim.

The rejected broad Stage C design remains evidence only. No code may be implemented from it. The
only current semantic authority is the frozen C2A design and its deferred C2B gate.

## Inputs revalidated during research

- frozen C2A design SHA-256:
  `5a3599038198ba95e1c6ad7421b4ba2ee703b2b96021c04a34dca196952e27bc`
- implemented C2A semantic source SHA-256:
  `4da0ce228b5127587a72f65d19efd58ddbc7d6e7d00dc98a41117ff18dbce4b5`
- `engram-eval/src/lib.rs` SHA-256:
  `2eea0ad9fcafdd9cc7f8a9e813a41e594e293a853a453b482c2ec41cd1064a74`
- workspace `Cargo.lock` SHA-256:
  `8741698619a41dcce8f58aae3609e6ee48929d6c2e91b955caf0ed586920572e`
- SurrealDB version: exact workspace lock at 2.6.0
- `getrandom` version already present transitively: 0.3.4

The research made no source, manifest, lockfile, live-store, daemon, provider, authentication,
network, or adapter change.

## Unanimous findings

Three independent reviews converged on the following facts.

### Arbitrary Engram database handles cannot prove locality

`engram_store::Db` is `Surreal<Any>`, and the existing connection path accepts Memory, RocksDB, or
Remote configuration. C2B therefore must not accept a caller-supplied Engram `Db`, `StoreConfig`,
endpoint, datastore URL/path, namespace/database string, backend enum, or locality boolean. The
target selector's repository URL and checkout path remain semantic input, not datastore authority.
A typed `Surreal<local::Db>` also does not by itself retain whether its origin was Memory or
RocksDB.

Any valid acquisition owner must construct and retain the exact evaluator-owned origin itself. It
must expose only exact internal operations, never a handle or reference that a caller can clone and
retain. An SDK-erased handle may exist only if the sealed owner constructed it and retained its
non-caller origin witness; the owner may not wrap or reopen a caller-provided store. Memory cannot
be reopened onto the same datastore, and RocksDB normally rejects a second open because it owns an
exclusive lock.

### One fixed statement can provide one logical transaction

The candidate query shape is one compile-time-fixed bare `RETURN` object containing exactly nine
fixed `SELECT *` subqueries, each with the semantic table cap plus one:

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

The final design must freeze and structurally allowlist the parsed AST, not merely trust
`Response::num_statements() == 1`. A minimal allowlist can reject added statements, options,
transactions, bindings, filters, projections, functions, scripts, timeouts, fetches, ordering, and
writeable subqueries; excluding ordering is a design choice, not a read-only requirement. If rows
remain unordered, every fallible pre-C2A row check must aggregate deterministically rather than
select a first error by engine order. The collector must also require exactly one response before
extraction.

Pinned SurrealDB 2.6.0 evidence shows that the executor opens one transaction for one bare output
statement; output/object/subquery/select writeability recursively resolves to non-writeable for
the literal shape above; and all nested object subqueries share the same transaction context. The
classifier is crate-private, so a future external implementation can prove this only through an
exact AST allowlist plus pinned upstream source evidence, or a separately reviewed classifier
hook. RocksDB attaches a transaction snapshot to its read options. The final acceptance suite
still needs its own black-box concurrent generation-toggle test for both supported backends;
upstream tests alone are not product proof. This proves a transaction boundary, not yet a raw
persisted-row boundary.

SurrealDB's `SELECT *` pluck path enables futures and recursively computes the document. A stored
`Future`, block, function, subquery, expression, or other executable native value can therefore run
before C2B sees the response. It may substitute another value, perform a data-dependent read beyond
the nine literal tables, or consume ambient capabilities. The outer AST allowlist cannot inspect a
program stored in a row.

The fixed query is usable only if the evaluator owns every ingress and has already rejected all
executable stored variants, or if a pinned query-engine patch preserves that exact bare
`RETURN { SELECT ... }` shape while returning values without computing them. A raw-KV or cursor
path would contradict the current frozen C2A statement requirement; choosing it requires an
explicitly superseding C2A freeze and rerunning C2A acceptance. Process containment can bound
effects but cannot by itself turn a computed response back into the raw persisted representation.

### Conversion must be path-aware before C2A

`SELECT *` is necessary to expose unknown fields. The collector must consume Surreal values
directly rather than use generic JSON conversion, because generic conversion collapses `NONE` and
`NULL` and stringifies record IDs. It must meter and secret-scan the source value before any
transformation, reject a stored `record_id` collision, extract the native record-object `id`, and
inject the canonical `record_id`.

The final freeze needs an exhaustive path-aware native-value matrix. A record Thing is permitted
only at the outer record ID, its table must byte-equal the fixed queried table, and its inner ID is
accepted only as the exact canonical lowercase UUID-v7 string form used by current writers. Native
UUID, number, array, object, generated, and every other ID form reject unless a later freeze proves
and authorizes one explicitly. Native datetime and `NONE` are permitted only at explicitly frozen
top-level fields; recursively JSON-native embedded and unknown values can be preserved; every
unsupported Thing, datetime, `NONE`, bytes, duration, geometry, non-JSON number, or other native
variant must fail categorically after source metering rather than be collapsed or stringified.

After exact response/object/key/array shape succeeds, all nine row-count overflow bits must be
computed before row normalization or any row-derived error. Complete unknown keys and values must
participate in raw byte/node/depth accounting and secret scanning.

### The OS-CSPRNG mint can be narrow

A future separately authorized dependency boundary can add exact direct `getrandom = "=0.3.4"`
use.
The sole production constructor should allocate the private 32-byte key buffer, invoke one
`getrandom::fill` call on that exact buffer, and move the key into exactly one C2A validation. It
must have no caller bytes, seed, configuration, retry, clone, accessor, serialization, persistence,
or log surface. Test entropy must remain sealed and test-only.

“One call” means one public library fill invocation. The operating-system backend may internally
perform more than one syscall. Zeroization can cover the currently owned key buffer, not registers,
compiler copies, allocator residue, swap, crash dumps, debuggers, the kernel, or hostile same-UID
processes.

The source call and version pin do not alone prove OS entropy. `getrandom` supports opt-in custom,
hardware-only, and unsupported backends through compiler configuration. A future freeze must pin
the exact target triple, toolchain, dependency source, effective compiler configuration and final
artifact hash; forbid custom, `rdrand`, `rndr`, `unsupported`, and every other non-OS backend
override; reject ambient build-flag injection; and prove that no custom backend symbol can satisfy
the build.

## Hard limits in stock SurrealDB 2.6.0

### Store opening is not a read-only observer operation

High-level local startup calls datastore version checking and mutating bootstrap/node maintenance,
starts background work, and opens RocksDB read/write. RocksDB also consumes ambient `SURREAL_*`
configuration and can create, flush, compact, and log its path. Direct core datastore construction
avoids the high-level bootstrap call but does not eliminate the read/write engine, environment,
thread, cache, compaction, or path-log properties.

Therefore a stock in-process collector cannot honestly claim that opening an existing RocksDB
store is filesystem-read-only or environment-independent. A narrower defensible statement is that
the fixed snapshot query is classified non-writeable after an evaluator-owned store has already
been opened under a separately proven lifecycle.

### Public queries materialize before evaluator metering

The public query path fully materializes the complete Surreal `CoreValue` response before caller
code can inspect it. Per-table cap-plus-one limits row count but does not bound one corrupt or
hostile unknown key/value, nesting depth, or intermediate allocation. A post-response walk can
bound data before constructing `C2RawSnapshot`; it cannot prove bounded allocation during
acquisition. A separate preflight query would lose the one-snapshot guarantee.

Consequently, a normal in-process call over Memory or RocksDB cannot satisfy the strong frozen
acquisition-bound claim unless the evaluator exclusively owns the complete store lifecycle and
proves that every write ingress enforces equivalent storage-representation bounds before insertion.
An arbitrary existing RocksDB store cannot meet that premise.

### `SELECT *` computes stored values before collection

Pinned SurrealDB's document pluck path explicitly enables futures. Field `*` computes the entire
document, and native `Future`, block, function, subquery, expression, and related variants execute
during that computation. A stored value can therefore change representation or trigger additional
reads before C2B can meter, scan, reject, or preserve it. Query-AST inspection proves nothing about
that data-driven execution.

An arbitrary existing store consequently requires a narrowly patched non-computing query collector
that retains the exact frozen bare `RETURN { SELECT ... }` contract. A raw low-level cursor is an
alternative only after an explicitly superseding C2A freeze and repeated C2A acceptance. A newly
evaluator-owned store may instead prove at every write ingress that executable native variants can
never be stored. Network denial and process containment are still useful defense in depth, but
neither alone proves raw persisted-state fidelity.

### Complete raw heap zeroization is not available

Surreal responses and `serde_json::Value` do not have zeroizing drops. Current C2A also moves or
clones some decoded strings/values into semantic state and retains complete correction-binding
rows. A recursive custom drop can best-effort overwrite buffers still owned at that point, but it
cannot prove erasure of dependency copies, old reallocations, allocator residue, database caches,
swap, or crash dumps.

The final design must state the exact narrower property it can prove: no intentional raw
serialization, persistence, logging, or outward return; categorical dependency errors; best-effort
zeroization of currently owned buffers; and, if process isolation is chosen, disposal of the
collector address space at process exit. It must not call that complete heap erasure.

## Smallest source boundary supported by the research

The least invasive Rust boundary is a private child module:

```text
engram-eval/src/native_successor_semantic/acquisition.rs
```

with one private `mod acquisition;` declaration in the semantic parent. A child can consume C2A's
private raw snapshot, key, binding, and validator types without widening them to `pub(super)` or
`pub(crate)`. `lib.rs`, peer modules, public facades, CLI, MCP, adapters, and historical runners can
remain unaware.

This boundary is only a recommendation for a future freeze. No file or declaration has been
created by this research.

## Recommended staged decision

Do not disguise a Memory-only proof as persistent-store closure.

1. **C2B1 — private Memory semantic-acquisition prototype.** A fully evaluator-owned fresh Memory
   store can exercise the fixed AST, response validation, path-aware conversion, cap-plus-one mask,
   semantic handoff, one-shot key mint, and snapshot atomicity without accepting caller storage
   authority. By itself this is not frozen bounded-acquisition closure because the public response
   materializes before metering. It becomes a candidate proof only if the final freeze owns the
   entire Memory lifecycle, prohibits handle escape, bounds every write ingress before insertion,
   and closes ambient Surreal configuration/background behavior. It does not prove the real
   persisted RocksDB journey.
2. **C2B2 — persistent acquisition containment.** Before any flagship or live-store claim, prove a
   persistent path with an evaluator-owned RocksDB lifecycle, bounded ingress, and no executable
   stored variants, or add a pinned non-computing query patch that preserves the exact frozen bare
   statement. A raw/cursor read instead requires a superseding C2A freeze and repeated acceptance.
   If a separately frozen collector process is used, it also needs hard memory, time,
   output-length, environment, network, and path capabilities. The child must return only a bounded
   sanitized result/receipt, never raw rows. A hostile oversized value must become a categorical
   acquisition failure without exposing source text. Containment alone does not fix computed
   `SELECT *` semantics.
3. **C2B3 — correction journey authority.** Separately freeze the move-only operator-action seal
   and process receipt that bind one successful pre-collection, the exact action, and one
   post-collection. Both validations use fresh keys. Unchanged state is compared through moved
   private canonical state or a private scoped equality witness, never MAC equality.

This split keeps the fast pure acquisition work moving while retaining an explicit hard gate for
the persistent journey required by the flagship goal.

## Alternatives requiring a future explicit freeze

The persistent bound conflict has three technically honest families of solution:

1. pin and review a narrow SurrealDB query patch that retains the exact bare statement, prevents
   stored-value computation, and meters values while materializing them inside the same read
   transaction;
2. explicitly supersede and re-freeze C2A to permit a bounded low-level transaction cursor, then
   rerun all C2A acceptance; or
3. combine the exact compatible query patch or a closed-ingress guarantee with a hard
   resource-capped collector process, length-delimited IPC, and a fully evaluator-owned source/copy
   lifecycle.

The current recommendation is to prototype the third option only after C2B1 exercises the semantic
acquisition contract, but only if a closed-ingress guarantee or compatible non-computing query is
available. It can avoid exposing the unbounded dependency heap to the parent evaluator, although a
query-engine patch may still be necessary. It is not implementation authority: the freeze must
define OOM/process-death categorization, exact resource controls, no-network and environment
clearing, executable/hash authority, path-retention and anti-swap controls, source-copy atomicity,
IPC framing, output canary scanning, cleanup, and platform limits.

No live user store may be copied under this note. A future copy design must separately prove source
ownership and authorization, exclusive-lock acquisition or a validated engine checkpoint,
writer-restart exclusion, retained source/destination identity, symlink and path-swap resistance,
owner-only destination permissions, positive disk reserve, bounded lifetime, and cleanup behavior.

## Required future acceptance evidence

- compile/source gates reject caller-supplied or publicly accepted `Surreal<Any>`, Remote,
  configuration, endpoints, arbitrary datastore paths, caller booleans, arbitrary handles, and
  peer/public callers; any owner-internal erased handle must remain bound to an internally minted
  origin witness and never escape;
- a recursive closed-world source firewall over the proposed child-module directory and every
  descendant, with an explicit capability/import allowlist and no unreviewed peer dependency;
- exact provenance, bounds, and capability controls for the `C2RawTarget` supplied alongside the
  nine table arrays;
- exact literal/AST golden plus a mutation corpus rejecting every extra or changed construct;
- stored `Future`, block, function, subquery, expression, and other executable-value cases proving
  they are impossible at bounded ingress or returned uncomputed by the selected raw path;
- query success, exactly-one-response, error, outer-object, exact-key, array, and row-shape cases;
- exact cap and cap-plus-one for every table, complete nine-bit overflow reporting, and
  unknown-field retention;
- exact and plus-one global bytes, nodes, depth, key length, string length, and hostile unknown
  key/value cases, with proof C2A is not called after acquisition failure;
- native `NONE`, `NULL`, record ID/collision, datetime, and exhaustive supported/unsupported nested
  value conversion matrices;
- concurrent A/B generation toggles on Memory and persistent evaluator-owned RocksDB, with every
  complete nine-table generation written in one atomic transaction, proving every observed
  nine-table snapshot is wholly A or wholly B;
- exactly one 32-byte entropy fill, partial/error/no-retry behavior, deterministic sealed-fake
  distinct-output tests, OS-provenance/source-shape evidence, pinned target/toolchain/effective
  compiler flags, rejection of custom and non-OS backend overrides, final artifact provenance, and
  compile-time use-after-move rejection; any live random-collision check is diagnostic only;
- currently owned custom-buffer/key drop probes on every exit, categorical errors, and canary scans
  of every formatted, serialized, logged, and IPC-visible output; raw values deliberately moved
  into a C2A prior binding leave the acquisition buffer owner's immediate Rust ownership but remain
  C2B process-lifecycle responsibility and require separately frozen destruction/drop handling;
- pre/action/post typestate tests proving exactly one consumed binding, fresh independent keys,
  one action, one post-state, private equality evidence, and one bound receipt; and
- final provider-free focused, full library, integration, strict Clippy, format, whitespace,
  external-target, repository-`target/debug`, dependency/hash, and independent exact-source audits.

## Explicit non-claims

This research does not prove or authorize a live Engram store, daemon, network, provider, relay,
adapter, correction execution, deletion propagation beyond the frozen tables, filesystem byte
immutability, path-swap resistance, complete memory erasure, protection from a malicious same-UID
process/kernel/debugger/hardware, atomicity across separate calls or multi-transaction writers, or
flagship completion.
