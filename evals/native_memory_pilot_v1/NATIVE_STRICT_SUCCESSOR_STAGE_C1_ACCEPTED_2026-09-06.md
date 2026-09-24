# Native strict successor Stage C1 — accepted implementation evidence

Date: 2026-09-06 (Asia/Jerusalem)

## Outcome and exact claim boundary

Stage C1 is implemented and independently accepted as a private, provider-free, read-only,
non-runnable structural-policy and artifact-classification slice. It proves exact phase-policy
derivation, strict synthetic relay parsing, bounded retained-descriptor artifact observation,
conservative lifecycle classification, and deterministic categorical failure under the frozen
race schedules.

It does not authorize or prove a run plan, live Engram call, semantic snapshot, daemon, listener,
transport, provider run, authentication, VM, mutation, replay, repair, activation, live adapter
setting, or flagship success. No authentication cache was copied and no daemon, provider, VM,
network, process, thread, async task, environment mutation, or live settings operation occurred in
this slice.

## Frozen design identity

- Original structural-policy design:
  `482dcf894be38640dd46908e2ca572c1d978c7dfd6c17ecca460119685909f98`
- Nonblocking retained-descriptor race repair:
  `3871b2bbb226060624fe1eb5e7c13268e6b0ca0c2c8a65ba593f78755e2be89c`
- Mandatory post-read safety repair:
  `9d969461cad6c5ea7b4b1a35d46b737882194628e88101845383da6af5f35e43`
- Handle-first error-precedence repair:
  `113b63f92e6e234400ab7e25096efe65d0bc096d33d0075885a76a6d7d194dea`

The four documents are cumulative. No later repair weakens an earlier invariant.

## Accepted implementation identity

- `engram-eval/src/native_successor_policy.rs`:
  `b01abc4c69580277155301b4cd1b67875eeb2836ed4690f0b3bd9cb2617ab677`
- `engram-eval/src/native_successor_relay.rs`:
  `a77d6742e552391fabdbdd9a9df443fa2efefe513337aa2ac1f94bad3e267e64`
- `engram-eval/src/native_document.rs`:
  `2236bf9f74624c1007c626ab6a16d0b51fe4c753c6ca4b99b7f7272d2739c0c6`
- `engram-eval/src/native_successor_artifact.rs`:
  `476df3e21d17ed9a00b9ae82dc51c943543e64ee6737792db194baa380324f2f`
- `engram-eval/src/lib.rs`:
  `b59d9ba4f1c69160f7c48e92af24faa1621264e90c74a123c8ab64bbde9c8666`

These hashes identify the accepted implementation only. Repository files contain user-owned work
outside this slice; nothing was staged or committed.

## Accepted invariants

### Policy and relay structure

- Exact lexical newtypes reject rather than normalize invalid input.
- One exhaustive family/phase table derives the exact frozen policy ID, policy digest, partition,
  synthetic relay shape, mutation-journal presence, and artifact order.
- The relay is a single-owner in-memory JSON-RPC sequence automaton over synthetic calls only. It
  has no real tool mapping, transport, provider, daemon, callback, or executor.
- The strict duplicate-rejecting JSON parser preserves malformed, duplicate-key, and trailing-data
  categories without broadening the public document surface.

### Artifact observation and error precedence

- The artifact directory is retained through a directory descriptor and every accepted entry is
  retained through its original file descriptor for both observation sweeps.
- Entry open uses `O_RDONLY|O_NONBLOCK|O_NOFOLLOW|O_CLOEXEC`; FIFO substitution cannot block or
  consume a sentinel byte, symlinks are not followed, and hard links fail the single-link rule.
- Exact directory-relative name sets are observed before and after the descriptor sweep, with no
  more than the eight frozen names.
- Every bounded entry read is exact-length plus EOF. First-sweep bytes are read twice; the retained
  descriptor is reread in the second sweep; exact bytes and SHA-256 must remain equal.
- One shared boundary checks retained-handle metadata and ACL before any fallible pathname lookup,
  then checks non-following pathname safety and full identity. It runs post-open, after both
  first-sweep reads, around the inter-read seek, before and after the second-sweep reread, and
  around its seek.
- Read and seek failures are captured until the mandatory safety boundary runs. An observed unsafe
  handle therefore reports `UnsafeEntry` even when it was renamed away or a read was shortened.
- Directory revalidation likewise checks its retained handle and ACL before path metadata. An
  unsafe directory renamed away reports `UnsafeDirectory`.
- Directory unsafety has precedence around enumeration and every entry operation; other operation
  and directory failures remain categorical.
- Identity binds device, inode, length, modification seconds/nanoseconds, and change
  seconds/nanoseconds. Same-length overwrite with restored modification time still rejects.

### Scope and capability boundary

- The classifier returns only the six frozen structural states or six categorical read errors.
- Errors expose no raw operating-system error, pathname, file name, or input bytes.
- Production performs bounded read-only filesystem observation on macOS. Every other platform
  returns `UnsupportedPlatform` before filesystem I/O.
- Production contains no execution, provider, authentication, daemon, network, process,
  environment, thread, synchronization, async, mutation, repair, or retry capability.
- All synchronization and filesystem mutation used to exercise races is compiled only in tests.

## Rejection and repair record

C1 was not accepted on its first successful test run. Independent review found that a FIFO could
be substituted between metadata observation and a blocking open, and that dropped entry
descriptors allowed a mixed-time chain. The frozen race repair added nonblocking opens, descriptor
retention, and a complete second sweep.

The first repaired implementation was also rejected: a read error could bypass the mandatory
post-read safety boundary, directory path failure could mask an unsafe retained directory, and the
source gate plus ctime regression were incomplete. The frozen post-read repair made validation
unconditional, fixed directory precedence, strengthened source gates, and restored mtime in the
ctime-only regression.

The next implementation was rejected because entry and directory validators still performed
fallible pathname observations before classifying already-observed unsafe retained handles, and
first-sweep read errors could still propagate too early. The frozen handle-first repair introduced
the single shared entry boundary, handle-first directory ordering, captured read/seek results, and
the required compound regressions. Only the exact final hashes above are accepted.

An earlier, superseded implementation snapshot also produced one transient macOS `EPERM` in
`production_verification_failure_cleanup_proves_descendant_absence` during a broad suite. That test
then passed five focused reruns and later full parallel and serial suites. The incident is recorded
for honesty; it did not recur on the accepted source and was outside the C1 classifier itself.

## Validation evidence

- Focused artifact classifier: 29 passed, 0 failed; then the same 29 tests passed in each of ten
  consecutive repetitions.
- Combined C1 successor modules: 76 passed, 0 failed.
- Full provider-free `engram-eval` library suite, parallel: 294 passed, 0 failed.
- Full provider-free `engram-eval` library suite, serial: 294 passed, 0 failed.
- Full `engram-eval --tests` matrix:
  - library: 294 passed, 0 failed;
  - native correction foundation: 66 passed, 0 failed, 1 ignored;
  - native isolation foundation: 25 passed, 0 failed, 4 ignored;
  - native stale preparation: 1 passed, 0 failed;
  - native VM foundation: 17 passed, 0 failed.
- Strict Clippy with warnings denied: passed.
- Formatting and whitespace checks: passed.
- Repository `target/debug`: absent before and after validation; external compiler storage was
  used.
- Final disk reserve: approximately 60 GiB available.

## Independent final audits

Three fresh audits independently read the exact final artifact source and all four frozen designs:

- security-boundary audit: accepted, P0=0, P1=0, P2=0;
- exact design-conformance audit: accepted, P0=0, P1=0, P2=0; and
- adversarial race/error-precedence audit: accepted, P0=0, P1=0, P2=0.

The reviews specifically exercised FIFO and symlink substitution, unsafe retained objects renamed
away, short reads combined with unsafe mode, byte mismatch combined with unsafe mode, same-name
replacement, whole-set mutation, ACL changes, ctime-only changes, descriptor/path mixing, and
capability leakage. No reproducible finding remained against the frozen claim.

## Deliberate remaining gates

C1 does not make the native successor runnable. The next separately designed and reviewed slice
must establish an atomic, bounded, validated semantic snapshot from persisted Engram state without
yet admitting the current live daemon. Later slices must still harden evaluator-owned daemon
isolation, offline embeddings, daemon authentication and Git-child environment handling,
inherited-listener ownership, phase-filtered relay enforcement, exact plan-derived launch values,
VM authority, and provider execution before any flagship claim is possible.
