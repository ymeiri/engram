# Native strict successor Stage C2A — accepted semantic-validator evidence

Date: 2026-09-06 (Asia/Jerusalem)

Status: **accepted** after the frozen source-firewall repair and its complete validation sequence.

## Outcome and exact claim boundary

Stage C2A is implemented and accepted as a private, provider-free, pure validator over an
already-materialized bounded persisted-state snapshot. It proves strict input limits, secret-first
rejection, persisted-record integrity, correction-transition semantics, deterministic target/scope
projection and one-shot keyed sanitized audits for the exact frozen schemas.

C2A accepts no datastore, path, configuration, connection or reader capability. It does not prove
store acquisition, locality, network isolation, atomicity, concurrency consistency, production key
minting, pre/post chronology, causal correction application, complete deletion, daemon or transport
authority, provider or authentication isolation, VM/runner authority, public CLI/MCP support,
artifact execution or flagship success.

## Frozen and accepted identities

- Frozen C2A design:
  `5a3599038198ba95e1c6ad7421b4ba2ee703b2b96021c04a34dca196952e27bc`
- Frozen source-firewall repair:
  `0b527f29bfb5341e5698c420903f2c30630ac6e9505d8bb37a8edb6c2d6211a0`
- `engram-eval/src/native_successor_semantic.rs`:
  `2fe9b2f8e3d2ae8f483b3b93e38f3f4a7d508f033bb1e1edd8892348de1a6f75`
- `engram-eval/src/lib.rs`:
  `2eea0ad9fcafdd9cc7f8a9e813a41e594e293a853a453b482c2ec41cd1064a74`
- `Cargo.lock`:
  `8741698619a41dcce8f58aae3609e6ee48929d6c2e91b955caf0ed586920572e`
- Compiler: `rustc 1.93.0 (254b59607 2026-01-19)`

The exact semantic references also retain their frozen identities:

```text
engram-core/src/id.rs
  b7ee04c068128f48a5df9ffea355dcdca66817ff54c10b65c7e2b197021e6953
engram-core/src/memory.rs
  26f9f2e7f8920b1328220c1408a8a6f638e7aba8dc3f61d75d4d83cb547a911c
engram-core/src/repository.rs
  ec2917ebb76e8dbac10a5a953d1f5b7b74aa352c03f008dc831205a1fbdf3f01
engram-core/src/work.rs
  ffbb62e50d9084717680bf076f0d9473b3c6e6499cdbe024a5171ce7551466c6
engram-store/src/secret.rs
  0db4abdabbfe2ddd57fb7f4ef94694b29c0b367a679eaa45989e50cfcbb74f92
engram-store/src/repos/memory.rs
  c285cb5b8359e2768d2f031f12e9ff3e87717879b59d17b194cd15ae2f3ae8ca
engram-index/src/memory.rs
  7ece3d1c85c325a2d6e0cd5ba9dcaebd77d32d5d5240bffec2a42375018886b4
engram-index/src/repository.rs
  85cccf12cec5af20af3ec87ef3ffe53d34fca0a6fb5309ecc58b8d40505ab13a
engram-store/src/repos/repository.rs
  20834be588301dc669822b25f919df9c557c82dbc47363ce04cd9b3f5f1f344e
```

These hashes identify only the accepted C2A implementation. The worktree contains unrelated,
user-owned changes; none was staged, committed or rewritten for this acceptance.

## Accepted invariants

### Bounded raw input and deterministic semantics

- The private validator accepts only the exact target, prior-binding and nine-table virtual input
  shape frozen by C2A.
- Exact table, row, aggregate byte, node, depth, object-key, vector, scalar and checked-arithmetic
  limits fail closed before semantic authority can be returned.
- Strict schemas reject missing and unknown fields, malformed IDs, enums, timestamps and numeric
  domains, invalid `NONE`/null/omission placement, duplicate logical identities and inconsistent
  denormalized projections.
- Canonical ordering, framing and keyed MAC domains are deterministic under insertion permutations
  and tied timestamps.

### Identity, scope and lifecycle

- Project, repository, remote, checkout, path, component and optional task selectors are
  conjunctive. Split-brain, missing, competing, orphaned and cross-repository topology rejects.
- Only applicable active memory reaches the relay projection. Out-of-scope records remain bound
  into the store MAC but cannot silently enter the project or relay MAC.
- Pending and applied correction states require the exact pair, marker, provenance, scope,
  lifecycle and moved prior-binding rules. Forbidden chains, cycles, repeated roles, competing
  superseders and invalid timestamp orderings reject.
- Incomplete or malformed forget receipts are terminal-invalid under the frozen precedence.

### Secret, key and capability boundary

- The complete bounded input is scanned for every frozen secret family before any record or state
  MAC is computed.
- Successful audits and all formatted errors contain no raw content, names, paths, remotes,
  evidence text, writer strings, secret canaries or content-derived `_sha256` fields.
- The MAC key and pads are move-only private values with zeroizing drops. C2A exposes only the
  deterministic test constructor; production key minting remains deferred to C2B.
- The module is private, has no production caller and has no filesystem, environment, Git, clock,
  process, thread, async/runtime, network, provider, authentication, datastore, daemon, logging or
  artifact-writing capability.
- The independently reviewed production prefix is sealed at
  `11747afaddc14fd01d7da5e63addd56a9fae7eaa3ad70fb2154e0212f2dfc286`.
  A non-circular normalized whole-source seal covers the complete file, including any suffix, at
  `30ef7f171e4292b7cb7f306ea2a2439f82ab2970afd5da5272e839817ed51ce0`.

## Validation evidence

All compiler output was placed under
`/private/tmp/engram-stage-b-20260906`; repository `target/debug` stayed absent.

- Focused C2A semantic suite, parallel: 81 passed, 0 failed.
- Focused C2A semantic suite, serial: 81 passed, 0 failed.
- Exact repaired source-firewall test: 1 passed, 0 failed.
- Full provider-free `engram-eval` library suite, parallel: 375 passed, 0 failed.
- Full provider-free `engram-eval` library suite, serial: 375 passed, 0 failed.
- Full `engram-eval --tests` matrix:
  - library: 375 passed, 0 failed;
  - native correction foundation: 66 passed, 0 failed, 1 explicit installed-binary gate ignored;
  - native isolation foundation: 25 passed, 0 failed, 4 explicit enforcement gates ignored;
  - native stale preparation: 1 passed, 0 failed; and
  - native VM foundation: 17 passed, 0 failed.
- Strict all-target Clippy with warnings denied: passed.
- Formatting and whitespace checks: passed.
- Frozen source, lockfile and semantic-reference hashes: matched.
- Compiler pin: matched.
- Final disk reserve: positive and above the mandatory reserve.

The deferred integration command was:

```sh
env \
  ORT_LIB_LOCATION=/private/tmp/engram-onnxruntime-1.20.0.eTyfOQ/extracted/onnxruntime-osx-arm64-1.20.0 \
  ORT_PREFER_DYNAMIC_LINK=1 \
  DYLD_LIBRARY_PATH=/private/tmp/engram-onnxruntime-1.20.0.eTyfOQ/extracted/onnxruntime-osx-arm64-1.20.0/lib \
  CARGO_TARGET_DIR=/private/tmp/engram-stage-b-20260906 \
  cargo test --offline -p engram-eval --test native_stale_preparation
```

It passed 1/1. The complete focused, library, integration and Clippy commands used the same local
runtime and external target, and Cargo ran with `--offline`.

## Build-prerequisite provenance and recovery history

The pinned dependency graph compiles `fastembed 4.9.1 -> ort 2.0.0-rc.9 -> ort-sys
2.0.0-rc.9` even though the C2A integration test does not initialize embeddings. A cold build first
stopped before C2A because the `ort-sys` build script attempted its own download from
`parcel.pyke.io` and received HTTP 403. Cargo's offline flag cannot govern a dependency build
script's direct HTTP client.

The build was supplied instead with Microsoft's official GitHub release asset:

```text
release       https://github.com/microsoft/onnxruntime/releases/tag/v1.20.0
asset id      202999152
asset name    onnxruntime-osx-arm64-1.20.0.tgz
asset size    7,875,127 bytes
archive SHA   2bcfaafa9ff0a3a94f78e3af2f135ffde5bb2d79b08e83a50dbc450b0d20ddae
dylib SHA     d8be733cb8dd097cfe2b21e069a7462b5ff561625141d9c4b98d866f15bfb852
version       1.20.0
commit        c4fb724e810bb496165b9015c77f402727392933
architecture arm64
API version   20
deployment   macOS 13.3
```

The archive had one expected root, only regular files/directories plus one root-contained relative
library symlink, no traversal path, the exact unversioned-to-versioned library link and no
unexpected non-system dylib dependency. The dylib exports `OrtGetApiBase`; the linked integration
test loaded it from the isolated local path.

GitHub publishes no SHA-256, detached signature or attestation for this historical asset. The
release is not immutable, its annotated tag is unsigned, and the dylib has an ad-hoc signature
without a TeamIdentifier. Therefore the recorded SHA-256 is trust-on-first-use evidence over the
official HTTPS release channel plus exact internal version/commit/architecture consistency; it is
not claimed as publisher-signed binary provenance. The raw archive is retained under its private
temporary parent for reproducibility during this acceptance.

An earlier attempt was also prevented from starting when disk fell below the mandatory reserve.
Neither prerequisite failure reached or failed C2A semantics.

## Independent agent audits, repair and residual hardening

Three delegated agent reviews independently read the exact frozen design and final source. They
are technical agent audits, not human/manual-review authority, and all met the frozen acceptance
threshold of P0=0 and P1=0:

- semantic and exact-contract audit: P0=0, P1=0, P2=0;
- source-firewall audit: P0=0, P1=0, P2=2; and
- adversarial lifecycle/secret audit: P0=0, P1=0, P2=2.

The four P2 observations reduced to three unique proof-hardening themes:

1. The source firewall uses a denylist and manually enumerated peer files rather than a positive,
   self-closing source inventory.
2. Secret rejection tests do not directly instrument a zero-MAC-invocation counter.
3. Prior/current binding tests use grouped mutation coverage rather than a literal per-field
   mutation matrix.

One later fresh audit conservatively escalated the first theme to P1 because the denylist test did
not close over all possible source bytes. Acceptance was withheld. The separately frozen repair
added a test-only seal for the independently audited production prefix plus a self-normalized seal
for the complete source file. The latter replaces its sole embedded expected digest with a fixed
64-byte placeholder before hashing, so it covers prefix, tests and suffix without a hash fixed
point. The production prefix, `lib.rs`, lockfile and semantic behavior did not change.

Three new delegated agent reviews then read the exact repaired source, original design and repair.
Each reported P0=0, P1=0 and P2=0 and independently reproduced the full, prefix and normalized
hashes. These were repair/conformance audits, not human/manual-review authority.

The repair closes the source-firewall observation. Two earlier nonblocking proof-hardening themes
remain conservatively recorded: secret rejection lacks a direct zero-MAC-invocation counter, and
binding tests use grouped coverage rather than a literal every-field mutation matrix. The frozen
acceptance threshold is P0=0 and P1=0; this record does not claim those stronger formulations.

## Deliberate remaining gates

C2A is a semantic oracle, not a runnable native-memory successor. The already frozen C2B1 slice
must next prove a sealed, bounded, one-statement acquisition from an evaluator-owned Memory store
and the sole production OS-CSPRNG key mint, while rerunning all C2A goldens. C2B2 must separately
prove persistent evaluator-owned RocksDB containment and close/reopen behavior. C2B3 must bind a
real correction journey's pre-state, exact operator action and post-state without comparing MACs
across fresh keys. Confirmed deletion, daemon/adapter isolation, provider execution and the final
Codex/Claude comparative journey remain later gates.

The full Engram flagship goal therefore remains active.
