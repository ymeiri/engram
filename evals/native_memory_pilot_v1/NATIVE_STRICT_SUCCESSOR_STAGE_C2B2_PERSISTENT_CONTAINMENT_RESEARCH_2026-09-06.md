# Native strict successor Stage C2B2 — persistent containment research

Date: 2026-09-06 (Asia/Jerusalem)

## Status and authority boundary

This is a research and sequencing record, not an implementation freeze. It authorizes no source,
manifest, lockfile, VM, container, datastore, provider, adapter, daemon, live-store, copy, deletion
or authentication change.

No C2B2 implementation contract exists yet. The mandatory order remains:

1. pass C2A's deferred integration gate and create its accepted evidence record;
2. implement and accept the already frozen C2B1 Memory slice, rerunning C2A acceptance;
3. freeze, review, implement and accept C2B2; and
4. separately freeze C2B3 correction-action authority.

This note removes a future design ambiguity without bypassing that order. It does not narrow the
flagship requirement to volatile storage or replace persistent closure with a Memory result.

## Exact research inputs

```text
C2B acquisition research
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B_ACQUISITION_RESEARCH_2026-09-06.md
  3d5c6e2203ce95568ef38e208cc093e889ec5c23fbbcc27d16bf12b203de4129

C2B1 frozen Memory design
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B1_MEMORY_ACQUISITION_FROZEN_2026-09-06.md
  04f1be24c758f4a7f52baafab2d9ff27b9fd59241eecc48545deb3e5365600cb

C2B1 review record
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B1_MEMORY_ACQUISITION_REVIEW_2026-09-06.md
  56ee4d0b7dbd99d5742d6a5a80e1a0665a379d0dc44a54c9e092ecce497d73e4

Existing VM foundation, candidate evidence only
  NATIVE_VM_COLLECTOR_FOUNDATION_2026-09-05.md
  5ad40650a92cde03a35db6017aaa2cf72c1d7deb6dea14b4189e531dc6649981

Existing VM implementation, not C2B2 authority
  engram-eval/src/native_vm.rs
  7236583a702a85998e1be5a4c32950df239d32a384d4bf82e2b9dc961c90b69a

Dependency baseline
  Cargo.lock
  8741698619a41dcce8f58aae3609e6ee48929d6c2e91b955caf0ed586920572e
```

The resolved baseline contains SurrealDB 2.6.0, SurrealDB Core 2.6.0,
`surrealdb-rocksdb` 0.24.0-surreal.1 and `surrealdb-librocksdb-sys` 0.17.3+10.6.2.

## Decision direction

The smallest direction capable of reaching the full persistent claim is:

> Run stock pinned SurrealDB Core through a writer, bounded lock contender, bounded release probe
> and fresh semantic reader inside one fresh, dedicated, evaluator-owned VZ guest. Give the guest no
> host shares and a fixed-size private data disk. Put every collector process inside an attested
> Linux cgroup and private network namespace. The writer creates and writes the exact bounded store,
> proves graceful handle release while it remains alive, then exits. The reader independently
> reopens the same path and performs the exact C2A semantic observation.

This is the primary direction for a future freeze. A narrow SurrealDB patch remains a fallback,
not the default. A native macOS subprocess, the current shared Colima instance, a live Engram
store, a copied user store, or a same-container tmpfs result cannot substitute for this claim.

C2B2 should be staged without redefining completion:

- **C2B2a — containment mechanism:** prove an inert collector in the exact guest process boundary,
  including hard memory, CPU, process, descriptor, time, output, disk, path and network limits.
- **C2B2b — persistent journey:** place the distinct sealed Rocks lifecycle in that accepted
  boundary and prove a complete independent close/reopen observation.

The C2B2a result is infrastructure evidence only. C2B2 is not accepted until C2B2b is also green.

The future freeze must also select an explicit collector source boundary. The preferred shape is a
minimal dedicated collector artifact with no Engram store, high-level Surreal SDK, WebSocket or TLS
dependency. If sharing the existing evaluator package is measurably smaller, a recursive
reachable-source and final-binary firewall must prove that only direct Core, the fixed protocol,
the exact ASTs and the accepted semantic code are reachable. Fixed argv and network denial do not
replace that proof.

## Why the tempting alternatives are insufficient

### Native macOS

On this host, the address-space and data limits reject attempts to lower them, `RLIMIT_RSS` is
advisory, and `taskpolicy` is not a documented hard byte ceiling. CPU time, individual file size,
open-file count and watchdogs are individually useful but do not form a hard memory or aggregate
multi-file RocksDB boundary. Deprecated `sandbox-exec` can deny new network or filesystem
acquisitions but cannot close the memory gap.

### Current shared Colima instance

The current Colima 0.10.1 instance runs through Apple Virtualization and exposes a Linux Docker
29.2.1 engine with cgroup v2. It proves that the host has a usable mechanism, not that this running
instance is an acceptable boundary. It has a writable host-home mount, user-mode networking,
broad forwarding and unrelated workloads. C2B2 must create a fresh dedicated profile or guest.

Observed host executable identities are:

```text
limactl 2.0.3
  6455e484927c8d873d4eacca4a5a1610090d62e6ea65d86ce06ee6397780d57e
colima 0.10.1
  37e632654ed2c7e2901927bf752a307140348ba9c22cce9a02a86a89b5565a3b
docker client 29.3.0
  d5f08d666045b1e77a3f97cde421c6a287c45f635b0ec18e9471b82f2eb61003
```

These are observations, not future wildcard authority. The future freeze must re-attest every
selected executable, guest image, tool and effective configuration.

### Container tmpfs

A no-network, no-host-mount, cgroup-limited container with a size-bounded tmpfs can prove that a
RocksDB directory survives handle drop and a fresh process reopen while that same mount remains
alive. It cannot prove disk-backed persistence, container or VM restart persistence, filesystem
sync, crash recovery or power-loss durability. It is useful as a C2B2a negative/control fixture,
not as C2B2b or flagship evidence.

No currently installed image contains the Engram collector or a proven Rust build environment.
The observed local arm64 Debian image is unrelated and carries inherited application metadata.
An exact Linux-arm64 collector artifact and a reviewed deterministic guest payload therefore remain
explicit prerequisites; they must not be silently obtained or self-attested during acceptance.

### Current Engram store

The current `engram_store::Db` is a cloneable `Surreal<Any>` reached through broad configuration
and repository writers. It does not retain evaluator-owned origin, excludes neither Remote nor
other write surfaces, and cannot prove that persisted native values are bounded and data-only.
No C2B2 proof may accept that handle, its configuration, a caller path or a live/copy/reopened user
store.

## Stock SurrealDB boundary facts

The future collector must import exact `surrealdb-core = "=2.6.0"` directly. It must not use the
high-level SDK connection path, which performs version/bootstrap work, creates routing state and
starts maintenance tasks.

Direct Core still has material ambient and lifecycle behavior:

- Rocks configuration uses 35 process-global `LazyLock` reads: `SURREAL_SYNC_DATA` and the complete
  `SURREAL_ROCKSDB_*` family in `kvs/rocksdb/cnf.rs`.
- Core has additional process-global `SURREAL_*` and feature-flag settings. Several defaults depend
  on visible CPUs and memory. Malformed values silently fall back to defaults.
- the affinity pool uses `max(8, num_cpus)` workers, while native Rocks owns background pools and a
  periodic timer;
- enabling `SURREAL_ROCKSDB_BACKGROUND_FLUSH` creates an immortal Rust thread holding the database;
- the SST-space setting is not an aggregate hard disk quota;
- Rocks creates and mutates its directory, WAL, manifests, options, logs and table files; and
- public query results materialize before caller metering, while `SELECT *` computes executable
  stored variants.

Closed typed ingress remains mandatory even inside containment. It proves that the fresh store
cannot contain executable or unbounded values. The outer boundary independently contains native
allocation, threads, filesystem growth, output and unexpected dependency behavior.

C2B2 must freeze this explicit inherited-versus-superseded matrix:

- C2B1's private move-only, non-cloneable, non-serializable and one-shot ownership is inherited
  after bounded wire decoding; uncertainty consumes state and forbids retry.
- Direct Core with `Capabilities::none()` is inherited unchanged in writer, probes and reader.
- The internally constructed namespace `engram` and database `main` session are inherited. The
  writer alone supplies the nine internally constructed fixed variables; the reader supplies none.
- Restricted native conversion, exact write/read ASTs and response gates are inherited unchanged.
- C2A handoff, fresh key mint and private canonical comparison are inherited unchanged.
- The Memory-only constructor is superseded only by one internally minted sealed Rocks origin.
- The in-process typed-only production entry is superseded only by the bounded canonical wire stage
  below.
- The no-filesystem, environment, process or path boundary is superseded only by the frozen C2B2
  containment boundary.

Caller sessions, variables, configuration, endpoints, paths, queries, ASTs, notifications, startup
and bootstrap remain forbidden. No datastore, transaction, session or path handle can escape.

C2B1 supplies semantic preflight, restricted conversion, exact write/read contracts, C2A handoff,
key minting and canonical comparison mechanics. It supplies no wire authority, Rocks origin,
configuration, path, process containment, causal execution witness, close/reopen or persistence
authority.

Before any C2B1 preflight, C2B2 adds a private wire gate. A future freeze must select and hash one
versioned canonical encoding and exact schema. It must read and validate a fixed-size length prefix
and total byte cap before content allocation, then perform bounded streaming or direct typed decode
with per-string, vector, table, node and depth limits. It never decodes through
`serde_json::Value`, an arbitrary map, native Surreal value or generic caller type. Unknown,
duplicate, omitted-required and trailing fields, noncanonical encodings, extra or partial frames
and multiple frames all reject before datastore construction.

The host-local transcript binds the exact plan, target, generation provenance and input-frame
digest. Inside the guest, the frozen supervisor validates that frame, derives one bounded private
canonical expected-state witness, and forwards the same already bounded canonical frame exactly
once over a fixed guest-local pipe. The writer independently decodes it directly into the typed
generation and runs C2B1 preflight. After the writer acknowledges consumption, the guest supervisor
drops the raw frame and retains only the expected-state witness, never raw rows or a mutable store
handle. One separately bounded guest-local canonical-witness frame moves the witness exactly once
into the semantic reader's final comparison and has the same prefix, schema, duplicate, trailing,
depth and allocation protections.

The child starts with an empty environment and a future frozen allowlist. At minimum the effective
configuration must force synchronous data writes, disable Surreal's background-flush thread, set
every stock-exposed Rocks CPU/memory/cache/buffer/job/file/mmap/blob control explicitly, and clear
every `SURREAL_*`, Rocks diagnostic, home and build-flag input not on the allowlist.
Non-configurable native defaults, including periodic statistics, log-flush and trigger-compaction
work, must be bound to exact dependency source and binary hashes and reported as contained
background behavior. The affinity-pool count is bound indirectly through the exact visible guest
CPU count. Exact values
require measurement and review; this research note does not invent them or claim stock disables
every native background task.

## Exact future lifecycle candidate

The future freeze should refine two nested consuming state machines rather than collapsing guest
and host authority or generalizing C2B1's Memory owner.

```text
GuestPrepared
  -> GuestLiveExecutionWitness
  -> WriterProcessOpened
  -> AtomicGenerationCommitted
  -> LiveLockContenderRejected
  -> WriterHandlesGracefullyDropped
  -> ReleaseProbeOpenedAndClosedWhileWriterLives
  -> WriterProcessExited
  -> ReaderProcessIndependentlyOpened
  -> CompleteC2ASnapshotValidatedWithFreshKey
  -> CanonicalStateMatched
  -> HostChallengeChecked
  -> GuestWitnessConsumedIntoStructuralReceipt
  -> GuestAwaitingShutdown

HostPreparedExecutionWitness
  -> GuestBootImagePayloadAndPrivateDiskBound
  -> InputAndControlTranscriptBound
  -> FreshPreReceiptLiveRecheckPassed
  -> GuestStructuralReceiptBound
  -> HostCleanupInProgress
  -> GuestAndDiskDisposed
  -> StableAbsenceVerified
  -> C2B2AcceptedOutcome
```

The trusted guest supervisor internally creates a fresh opaque absolute database path beneath the
dedicated data-disk mount. The leaf is absent before creation. From the collector mount namespace,
the supervisor attests mount ID, device, inode, type, UID, GID, mode, link count and absence of bind
or overlay aliases before and after every open. Those checks are supporting evidence, not a TOCTOU
proof. Rocks opens by pathname and follows filesystem resolution, so the actual safety property is
capability exclusion: no concurrent process outside the frozen supervisor and current collector
can access or replace the private mount or become a writer.

The trusted-capable principals are explicitly the macOS/VZ host, guest kernel/root and frozen guest
supervisor. The unprivileged collector is the sole workload identity with mount access. Protection
from a malicious trusted principal or same-UID host process remains outside the claim.

The writer process:

1. receives one complete bounded typed generation through a fixed length-delimited input frame;
2. rechecks the exact C2B1 preflight and restricted JSON-to-native conversion;
3. opens direct Core Rocks only at the internally bound path;
4. commits the complete nine-table initialization transaction with synchronous writes;
5. accepts only the exact successful write-response contract;
6. while its datastore remains live, permits one distinct bounded contender process to attempt the
   same exact path; that process must fail on the Rocks lock and exit;
7. drops every response, transaction, session and datastore handle but keeps the writer process
   alive on one fixed supervisor control pipe;
8. permits one fresh bounded release-probe process to open and close the store successfully while
   the handle-free writer remains alive; and
9. exits only after the supervisor acknowledges that probe's clean exit.

The successful release probe distinguishes graceful in-process handle release from the kernel
merely closing leaked handles at writer exit. Syntax-aware source gates must additionally prove
that the production owner cannot clone, return, serialize, spawn with or otherwise leak a datastore
or transaction handle.

The semantic reader is a final fresh process, not `Datastore::restart()` and not a retained or
cloned handle. It starts from the same exact environment and executable identity, proves the writer
and both probes are gone, independently opens the bound path, executes the exact 570-byte C2A query,
applies the complete native gate, mints a fresh independent MAC key and moves the result into C2A.
It consumes the guest-local expected-state witness in the exact comparison and never compares MACs.

Application-data directionality is exact. One bounded typed-generation frame and one bounded fresh
challenge may cross host to guest. Only the frozen sequence of bounded sanitized status/control
frames and one terminal structural receipt may cross guest to host. Raw rows necessarily enter the
Rocks data and WAL files, which remain solely on the guest-private disposable disk. The opaque
evaluator-owned database path may occur only in those files and Rocks' guest-private diagnostic
log. Raw rows, keys, source text, dependency errors and secrets are forbidden from standard output,
standard error, outward IPC and diagnostic logs. In-guest scans reduce those surfaces to booleans
before any outward frame. Host framing rejects excess bytes, duplicate frames, trailing data, late
output and unexpected streams. The future freeze enumerates every management and application frame.

The guest-local witness is private, move-only, non-cloneable and non-serializable. It alone owns or
binds the guest dirfds, pidfds, cgroup and network-namespace handles, collector states and in-guest
transcript. The exact guest supervisor consumes it to perform final in-guest checks and answer the
fresh host challenge with the bounded structural receipt. The guest witness never crosses the VM
boundary.

The distinct host-local witness has the same type restrictions. It owns the VZ task/process,
private-disk and run-root handles and sole supervisor channel, and binds the exact contract, guest
boot, image, payload, input/output transcript and guest receipt. After fresh live host checks and
receipt binding, its consuming cleanup transition shuts down the guest, disposes the disk, proves
stable absence and only then returns the in-process C2B2 accepted outcome. It never claims before
cleanup that disposal has occurred.

The persisted guest receipt and any persisted report are structural evidence only and can be
forged or replayed. They cannot recreate either live witness or the accepted outcome.

## Shutdown and persistence claim

`Datastore::restart()` retains the same storage handle and is not a reopen. `Datastore::shutdown()`
is also not authority: it does not consume the handle, first mutates cluster-node metadata, may fail
before the Rocks flush on an unbootstrapped direct-Core store, and the Rocks shutdown implementation
logs and discards WAL and memtable flush errors before returning success.

The writer therefore never relies on either method. A successful release-probe open while the
handle-free writer remains alive proves observable graceful lock release; source gates supply the
no-handle-escape evidence. The later reader's successful open proves independent process-boundary
reopen. A full exact C2A observation then proves normal observable close/reopen persistence for this
bounded generation.

That result does not prove crash recovery, power-loss durability, physical-media synchronization or
filesystem byte immutability. Stock Core and Rocks discard some flush/destructor errors, so the
stock path can classify only an observable lock, reopen or data failure. It cannot claim detection
of every close fault. Arbitrary close-fault propagation or stronger durability requires the narrow
patch or a separately frozen fault/power boundary.

## C2B2a containment obligations

A future C2B2a freeze must specify and prove, with exact numeric values and precedence:

- independently anchored VZ/guest-image and payload provenance;
- no host directory or authentication mount, a fixed-size private data disk, and exact trusted host,
  guest-kernel/root, supervisor and unprivileged-workload identities;
- fixed guest vCPU and physical memory;
- collector `memory.max`, `memory.swap.max=0`, guest swap disabled, process/thread and descriptor
  limits, and an explicit nonclaim about host compression or swap of the VZ process unless
  separately proven;
- a CPU-rate limit plus per-process cumulative CPU limit, hard wall clock and whole-cgroup kill;
- fixed-disk logical and allocated size, host preallocation/reserve, exact data/log/tmp/core
  placement, aggregate exhaustion precedence, core-dump denial and bounded diagnostic logs;
- one fixed guest-management and IPC design: no directory-sharing devices; either no VZ network and
  socket devices or an exact sole supervisor channel; disabled guest/SSH agents, forwarding and
  proxies; and collector access only to fixed standard-input/output/control descriptors;
- a private network namespace with no usable interface, route or pre-opened network descriptor,
  plus reviewed denial of socket creation for IP, packet, netlink, VSOCK and every unneeded address
  family;
- non-root collector identity, no privilege escalation, minimal capabilities and syscall policy;
- mount-ID/device/inode attestation from the collector namespace, no bind/overlay alias, and
  capability exclusion of every concurrent writer; metadata and link checks alone are not race
  proof;
- cleared environment and exact effective Surreal/Rocks configuration receipt;
- one fixed executable/argv/cwd/input program with no arbitrary shell or caller command;
- hard wall-clock supervision, output-length framing and deterministic kill/reap behavior;
- an exact VZ entropy device or guest entropy source, pinned Linux `getrandom` backend and the
  inherited one-fill/no-retry MAC-key contract;
- process/OOM/signal/timeout/disk-full categories that never authorize retry; and
- guest, process, mount, path, inherited-descriptor, socket and output canary inspection before the
  guest supervisor consumes its local witness or the host supervisor advances its host witness.

The current VM foundation is useful code and test input, but it is not C2B2 acceptance. Its shared
network mode must be supplemented by an independently proven collector-private network namespace,
and its management/VSOCK channels must be closed or explicitly reduced to the sole frozen supervisor
transport. Every outstanding foundation gate must close before reuse. Disposing the disk is cleanup,
not secure erasure or proof about APFS snapshots.

## C2B2b persistent-journey obligations

After C2B2a acceptance, C2B2b must prove at least:

- caller path/configuration/handle rejection and one internally minted fresh store origin;
- exact writer, contender, release-probe and reader artifacts, dependency features, toolchain and
  effective environment;
- the explicit inherited-versus-superseded C2B1 matrix, exact write/read ASTs and categorical gates;
- exact canonical wire bytes, prefix/total/per-field/depth bounds, streaming typed decode,
  provenance and digest; every unknown, duplicate, noncanonical, trailing, partial and
  multiple-frame rejection;
- bounded guest-local expected-state witness retention and exactly-once consuming reader comparison;
- contender lock rejection while the writer owns the store, then release-probe success after the
  writer drops every handle but before that writer process exits;
- complete generation A and B close/reopen cases, plus repeated reads with exact canonical equality;
- a distinct test-only same-process, one-sealed-handle Rocks A/B replacement/read stress case with
  every observation wholly A or wholly B, never hybrid; the sequential multi-process persistence
  journey never attempts an unsupported concurrent second writer open;
- exact and plus-one table, byte, node, depth, key, vector, scalar, IPC and output bounds;
- executable/extended native variants impossible at ingress and never observed as acceptable;
- parent-environment poisoning and prior parent Surreal initialization unable to change child
  effective configuration or result;
- disk-full, commit, process-death, timeout, lock, reopen and observable data-failure cases yielding
  only non-authorizing, non-replayed categorical outcomes; arbitrary hidden close errors are not a
  stock-path claim;
- canary scans across standard streams, IPC, receipts and guest-private Rocks diagnostic logs
  without exporting those logs or their opaque path;
- an explicit minimal collector source boundary or recursive reachable-source/final-binary firewall;
- nested guest/host witness typestate tests proving cross-boundary separation, ordered challenge,
  receipt binding, cleanup and stable absence, while persisted evidence cannot recreate authority;
- full cleanup and stable absence of the processes, guest and disposable disk;
- final focused/full tests, lint, format, external-target, dependency and artifact evidence; and
- three independent exact-source audits with P0=0 and P1=0.

## Patch fallback

A pinned SurrealDB fork is justified only if the contained stock path cannot emit an adequate exact
effective-configuration or close/reopen proof. The narrow patch would add an explicit
`RocksDbConfig`, an exact constructor that bypasses every Rocks `LazyLock`, a fallible affinity-pool
initializer, forced background-flush disablement, propagated WAL/memtable flush errors and a
consuming storage-only close that does not delete node metadata.

Even that patch cannot provide a hard C++ heap cap, aggregate filesystem quota, pathname race
protection or network denial. It therefore does not remove the guest/cgroup boundary and is not a
faster substitute by itself. A low-level cursor alternative would supersede C2A and require its
complete freeze and acceptance to be repeated.

## Current gate

C2A's frozen design, implementation and dependency hashes remain unchanged, and repository
`target/debug` remains absent. Available disk is still far below the mandatory 19,427,004 KiB
reserve. The verified 20,781,808 KiB cache at
`/private/tmp/engram-claude-host-action-target-20260904-01` remains untouched pending exact
path-specific deletion confirmation.

This note does not itself authorize C2B2 implementation. C2B2 freeze work may begin only after
accepted C2A and C2B1 evidence is recorded, using this note as research input rather than frozen
authority. The immediate executable next action is still the already frozen C2A integration test
after the disk gate is restored.

## Explicit nonclaims

This research does not prove or authorize C2A acceptance, C2B1 implementation, Rocks persistence,
VM or container containment, a Linux collector artifact, a live Engram store, store copying,
filesystem-read-only observation, crash or power-loss durability, correction execution, daemon,
MCP, adapter, provider, deletion propagation, every hidden close/flush failure, host-side VZ swap or
compression behavior, secure disk erasure, protection from trusted-capable host/guest principals,
complete heap erasure or flagship completion.
