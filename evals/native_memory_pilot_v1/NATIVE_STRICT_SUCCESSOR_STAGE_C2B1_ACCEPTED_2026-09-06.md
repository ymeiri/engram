# Native strict successor Stage C2B1 — accepted sealed-Memory acquisition evidence

Date: 2026-09-06–07 (Asia/Jerusalem)

Status: **accepted** for the exact sealed C2B1 source identity below after full provider-free
validation and three independent exact-source audits with P0=0 and P1=0.

## Outcome and exact claim boundary

Stage C2B1 is implemented and accepted as a private acquisition boundary over one fresh,
evaluator-owned SurrealDB `Memory` datastore. It accepts one closed, statically typed complete
generation; performs bounded preflight, complete secret screening and projection-collision
rejection before mutation; writes all nine tables in one explicit transaction; performs one fixed
native collection query; normalizes only the frozen native value subset; mints one move-only MAC
key from the operating-system CSPRNG; and hands the resulting snapshot to the accepted C2A
semantic validator.

This acceptance proves the Memory-only acquisition and handoff slice. It does not prove a
filesystem-backed store, persistence across close/reopen, process or VM containment, a real
correction journey, deletion propagation, live Codex or Claude integration, provider execution,
authentication isolation, daemon or adapter behavior, or the full flagship outcome.

## Frozen and accepted identities

| Artifact | SHA-256 |
|---|---|
| Frozen C2B1 design | `04f1be24c758f4a7f52baafab2d9ff27b9fd59241eecc48545deb3e5365600cb` |
| Frozen C2B1 design review | `56ee4d0b7dbd99d5742d6a5a80e1a0665a379d0dc44a54c9e092ecce497d73e4` |
| Frozen C2A identity ratchet | `88f7bca22e70557da93794aeb32a633d11fe72cc62c42c5a440169023f90b039` |
| Accepted C2A record | `2c583accccf77c51e912a10b5184ad5fef96a16aebca0f005019f228caf05b2f` |
| `engram-eval/src/native_successor_semantic/acquisition.rs` | `895fa4188bcaeebc907ef0f014f7ed5053ad6b413017ac6402c52f217bcb6d9a` |
| Normalized whole-child self-seal | `57eb30ec185af8f6cfac3369e5c13128609e95ff14d88cf245fe62c7c032f899` |
| C2A parent, including private child declaration | `2f9909b47c81238f3754fee954c7dcefa01ccdee28df1fcd30fbf94c6c718391` |
| C2A parent production prefix | `c67ec74b936ce861e53e52d110b4635e4ba6fdfc9a3b0d3a93357315ca6d4b34` |
| C2A normalized whole-source seal | `152678c4bb765f948c8fe3d96ed49428e89e4ca472322733916da948bfd70036` |
| `engram-eval/src/lib.rs` | `2eea0ad9fcafdd9cc7f8a9e813a41e594e293a853a453b482c2ec41cd1064a74` |
| `engram-eval/Cargo.toml` | `4176c6a4886492f7d5adaa0bdc3134d7e04bfc8d1536de4d5eb8d4b8985ce2bb` |
| Workspace `Cargo.toml` | `ec61531f8ffd401211f649c7b91e28a4ee5f0fa1a0fe2f199c63a841c9724f5a` |
| `Cargo.lock` | `0eb924bb9008bb417cb9990081df8429520131aa6b3f7367866c5b805ec474de` |

The normalized child seal replaces its single embedded expected digest with 64 zeroes before
hashing, avoiding a hash fixed point while covering the complete source. The exact firewall
requires one unique declaration and rederives this value during the test.

Only `acquisition.rs` changed for the final repair rounds. The parent, public facade, manifests,
lockfile and frozen records retained the identities above. The wider worktree contains unrelated,
user-owned changes; none was staged, committed or rewritten for this acceptance.

## Accepted invariants

### Closed typed ingress and prewrite rejection

- Production accepts only the private move-only `C2CompleteGeneration` with the target and the
  eight frozen domain families. It accepts no arbitrary serializer, JSON/native map, query, AST,
  binding map, raw bytes, backend, path or callback.
- An allocation-free first pass enforces table, vector, scalar and checked upper bounds. A bounded
  exact/lower-bound pass then accounts for every serialized field and gathers source-string secret
  evidence. Derived scope and lowercase projections are scanned only after complete accounting.
- Exact C2A table, row, global-byte, node, depth, key, vector and scalar limits are enforced over
  the prospective complete generation before datastore construction.
- Per-family duplicate IDs and C2A-equivalent logical identities are rejected before JSON/native
  conversion: folded project names; exact and folded task name/Jira selectors by project;
  normalized repository remotes; checkout paths; repository/component paths; correction pairs and
  obsolete/replacement roles; and project-name-resolved link tuples `(project ID, repository ID,
  optional component ID)`. Repository/component reference and topology validity remains C2A's
  responsibility.
- Limits precede complete secret evidence, and both precede typed-invalid or projection-invalid
  results. Invalid timestamps and non-finite values are classified without allowing them to hide a
  secret or an exact cap violation.

### Restricted write, read and native normalization

- The child parses and source-checks exactly one initialization literal and one read literal before
  constructing the datastore. Production creates exactly one `Datastore::new("memory")`, applies
  `Capabilities::none()`, and uses the fixed owner session, namespace and database.
- Initialization is one explicit transaction. It requires exactly nine successful empty-array
  statement results. Any dispatched write failure or malformed outcome is
  `EngineOutcomeUncertain` and consumes the owner.
- Collection is one fixed, one-statement query returning exactly one outer object with the exact
  nine table arrays. No raw store, session, query, native value or second handle escapes.
- The collector computes the complete table mask, dry-runs all definitive limits, scans every
  supported native string/key (including row `Thing` table and string ID), records unsupported
  variants without inspecting their internals, and accumulates malformed-row state. The accepted
  order is `LimitExceeded` → `SecretMaterial` → `UnsupportedNativeValue` → `InvalidResponse`.
- Missing, wrong or noncanonical row IDs, stored `record_id` collisions, non-object rows and wrong
  timestamp representations cannot preempt later higher-priority evidence. Invalid/lower-bound
  accounting can never normalize, mint a key or call C2A.
- Normalization admits the frozen JSON-native subset, the exact row `Thing` and top-level
  `Datetime` exceptions, and schema-authorized native `NONE`. `NONE` becomes JSON `null` only for
  exact nullable members or is omitted only for exact optional members; it rejects everywhere
  else. The normalizer never stringifies, repairs, defaults or silently drops a required or
  unknown value. Dry-run and normalized C2A accounting must agree exactly.

### Key and authority lifecycle

- Production contains exactly one `getrandom::fill(&mut key.0)` call, after all acquisition gates.
  The key is move-only and zeroizing; partial entropy failure returns only `EntropyUnavailable`.
- Exactly one C2A invocation can produce the sanitized validated snapshot. Pre-C2A failures and
  prewrite typed collisions prove zero C2A invocation.
- The owner and state markers implement none of `Clone`, `Copy`, `Debug`, `Display`, `Serialize` or
  `Deserialize`. Production has one initialization entry, one collection entry, no retry surface
  and no `Arc<Datastore>`. The child source directly references no environment, filesystem,
  network, process, provider, logging or time-now API and constructs no runtime, task, process or
  explicit thread.
- Those source-boundary claims do not erase dependency behavior: `Datastore::new("memory")` may
  use OS randomness and the system clock, and SurrealDB Memory uses a process-global commit pool.
  C2B1 therefore does not claim absence of dependency threads, immediate dependency-worker
  teardown, timing isolation, or safe recovery/retry after an acquisition future is canceled.
- Test-only replacement, native-injection, fixed entropy and cancellation seams are source-gated
  out of the production projection and individually bounded.

## Mandatory serializer and category evidence

The borrowed-meter evidence covers the concrete serializer rather than relying on a generic
fuzzing claim:

| Area | Explicit evidence |
|---|---|
| String framing | NUL/control worst-case escaping and maximum-width Unicode at every scalar cap |
| Numbers | Production `f32`, `i32` and `u32` extremes; `f64` extremes; finite spelling and non-finite classification |
| Time | No fraction, every fractional width 1–9, positive/negative whole-minute offsets, invalid year and seconds-bearing offset branches |
| Enums | Every unit and data-carrying variant for memory kind/status, origin, harness, evidence kind, project/task state and priority, repository provider and link role |
| Optionals | Both shapes for target task selector, scope IDs/names, model/writer provenance, evidence, archive metadata, procedure prerequisite source, verification evidence, procedure expiry, correction markers, project/task/repository/checkout/component/link fields |
| Scope/projections | Every `MemoryScope`, repository remote selection, local-path fallback and empty fallback; Unicode-expanding lowercase projections |
| Generations | Pending and applied proposals, zero/one/two-sided optional component provenance, rich and sparse complete generations, and exact proposal-row punctuation |

The categorical evidence binds origin and phase to the outward error:

| Origin / phase | Accepted outward category |
|---|---|
| Typed table, row, aggregate or arithmetic cap | `LimitExceeded` |
| Typed or derived secret | `SecretMaterial` |
| Invalid typed timestamp/number/domain shape | `InvalidGeneration` |
| Duplicate typed ID or logical/link projection collision | `InvalidGeneration` before native ingress, zero C2A calls |
| The same projected duplicate/project/link fixtures at full C2A | `InconsistentProjection`, proving semantic parity |
| Test-only JSON→native probes: caller `id` plus generated `record_id`, invalid record ID or forbidden number | `InvalidGeneration` |
| Static literal/hash/AST mismatch | `QueryContractInvalid` |
| Datastore construction or read-process failure | `EngineFailure` |
| Any uncertain post-write outcome | `EngineOutcomeUncertain` |
| Malformed supported native response after higher-priority scans | `InvalidResponse` |
| Unsupported native variant after complete supported-string scan | `UnsupportedNativeValue` |
| OS entropy failure | `EntropyUnavailable` |
| Post-handoff C2A semantic failures | Exact named C2A categories, including `InvalidRecord`, `InconsistentProjection`, `IncompleteDeletion`, `AmbiguousIdentity`, `ScopeMismatch` and `AppliedProvenanceUnproven` |

Post-handoff raw `name_key` tampering separately remains `InconsistentProjection`; the prewrite
mapping does not erase the distinction between typed ingress and an inconsistent collected store.

## Validation evidence

All Cargo commands used `--offline --locked`, a clean explicit environment, the retained local ONNX
Runtime, and `CARGO_TARGET_DIR=/private/tmp/engram-stage-b-20260906`. Repository `target/debug`
remained absent.

- Exact child/parent/peer/dependency/source firewall: 1 passed, 0 failed.
- Focused C2B1, twice parallel: 41 passed, 0 failed, 1 intentional ignored child each run.
- Focused C2B1, twice serial: 41 passed, 0 failed, 1 intentional ignored child each run.
- Focused inherited C2A, twice parallel and twice serial: 81 passed, 0 failed each run.
- Full provider-free `engram-eval` library, parallel: 416 passed, 0 failed, 1 ignored.
- Full provider-free `engram-eval` library, serial: 416 passed, 0 failed, 1 ignored.
- Full `engram-eval --tests` matrix:
  - library: 416 passed, 0 failed, 1 ignored;
  - binary test target: 0 tests;
  - native correction foundation: 66 passed, 0 failed, 1 installed-binary gate ignored;
  - native isolation foundation: 25 passed, 0 failed, 4 enforcement gates ignored;
  - native stale preparation: 1 passed, 0 failed; and
  - native VM foundation: 17 passed, 0 failed.
- Explicit serial `native_stale_preparation`: 1 passed, 0 failed.
- Strict all-target Clippy with warnings denied: passed.
- Formatting and `git diff --check`: passed.

The sole ignored C2B1 test is the poison-environment child. Its non-ignored wrapper executes that
exact child in a subprocess with poisoned `SURREAL_*` variables, requires the same bounded
semantic success with no widened ingress/query behavior, and passed in every focused/full run.

## Build and execution provenance

The clean acceptance process explicitly set and attested the basic host variables plus Cargo/Rustup
homes, offline and external-target controls, and the retained ORT/DYLD locations. It contained no
`SURREAL_*`, `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS` or `getrandom_backend` setting. No repository or
user Cargo config file existed at the four checked standard paths.

```text
rustc 1.93.0 (254b59607 2026-01-19)
cargo 1.93.0 (083ac5135 2025-12-15)
target aarch64-apple-darwin
host OS Mac OS 26.4.1
```

The direct C2B1 dependencies are pinned exactly:

```text
getrandom = "=0.3.4"
surrealdb-core = { version = "=2.6.0", default-features = false, features = ["kv-mem"] }
```

The cached crate archives reproduce the lockfile checksums:

```text
getrandom-0.3.4.crate
  899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd
surrealdb-core-2.6.0.crate
  c48e42c81713be2f9b3dae64328999eafe8b8060dd584059445a908748b39787
```

The complete evaluator dependency graph also contains broader `surrealdb` features through the
existing `engram-store` dependency. C2B1 does not claim that the entire test binary lacks those
symbols; its authority boundary is the private source firewall, direct `surrealdb-core` Memory
constructor, fixed ASTs, `Capabilities::none()` and non-escaping owner. On this Apple target,
`getrandom 0.3.4` selected the `getentropy` backend; the exact library-test executable imports
`_getentropy` from `libSystem`. No custom, RDRAND or RNDR backend configuration was present.

The retained C2A ONNX prerequisite was revalidated without downloading or replacing it:

```text
archive SHA-256  2bcfaafa9ff0a3a94f78e3af2f135ffde5bb2d79b08e83a50dbc450b0d20ddae
dylib SHA-256    d8be733cb8dd097cfe2b21e069a7462b5ff561625141d9c4b98d866f15bfb852
architecture     arm64
install name     @rpath/libonnxruntime.1.20.0.dylib
export           _OrtGetApiBase
```

Its provenance limitations remain exactly those recorded in the accepted C2A report: the official
historical GitHub release supplied no publisher SHA-256, detached signature or attestation, so the
retained digest is trust-on-first-use evidence, not a publisher-signed binary claim.

The final no-run artifact inventory was:

| Target | SHA-256 |
|---|---|
| Library test `engram_eval-8e0a400b4acd689d` | `02b70a5c96b7cb2c1977316afd52c0fe132d099a2f302f84c2ba000aedc58090` |
| Native correction `native_correction_foundation-862d2f5c20f868eb` | `c882e2117e6ed45ef130e1ea0b18ea3ba6daff0c40af1a4021f0c937dd018ae0` |
| Native VM `native_vm_foundation-ca33caaf2ed2238d` | `a9934a45d0fce5d573a8bb63e6d2b50d89f84a9e4d91be2019db48bc5620ea5b` |
| Binary test `engram_eval-489138d1ba5858a2` | `282b7d7e455425fc03e565cc1493baafa747454b2d168981237cbd396f8dc533` |
| Native stale preparation `native_stale_preparation-bd3922c03e8d6804` | `177ae0f7a82a45df1ddcc28995d1eba2670ddea4758e0aa79be417981aedbb2c` |
| Native isolation `native_isolation_foundation-b69922f19d3ae565` | `34250294ba0f9aa3b2666164236ff6cc83ba79ab30b7954bff6004cc2c6784da` |
| Debug evaluator binary | `68ca98d3d0ac1a17e51d60ae74637a94b7b9d04778aa198f8b7a485d9f18866a` |

All are arm64 Mach-O executables. The library-test executable links the retained ONNX Runtime and
only expected system frameworks/libraries outside the build graph.

## Host sandbox recovery evidence

The first broad validation attempts did not establish a product failure. The host `/private/tmp`
has an inherited `_dd-agent` read/search ACL. Existing native isolation/security tests correctly
reject a temp root carrying that ACL, after which barrier-based sibling tests wait for an artifact
that can never be published.

Two retained samples capture that host condition:

```text
/private/tmp/engram_eval-8e0a400b4acd689d_2026-09-06_221322_ssjG.sample.txt
  41b0a2f4e98c64f30703bfec30f062c4e178bcdb3cc2ba6348f810a4e03b1988
/private/tmp/engram_eval-8e0a400b4acd689d_2026-09-06_2313_serial.sample.txt
  10ca3b955677d9b0e93b559bbf52ec59f9c0a22aa9ebb0d48051deeea43ca565
```

Acceptance therefore used retained owner-only, ACL-free directories:

```text
HOME      /private/tmp/engram-c2b1-acceptance-home.oEhtvT
TMPDIR    /private/tmp/engram-c2b1-test-tmp.gFSI1B
evidence  /private/tmp/engram-c2b1-acceptance-evidence.HvkTKS
```

The first ACL-free temp root inherited group `wheel`, which made one setgid fixture unavailable to
the non-wheel user. Changing only that isolated temp root's group to the user's `staff` group made
the exact fixture and the full matrix pass. No product assertion was weakened or skipped. The host
failure and its correction are environmental evidence, not a C2B1 pass/fail substitution.

The final acceptance-time observation was 318,482,516 KiB available, above the mandatory
19,427,004 KiB reserve. Repository `target/debug` was absent.

## Independent audits and repair history

Three final delegated reviews independently read the exact accepted source, frozen design/review
and C2A ratchet. Each reproduced the source and normalized hashes and reported P0=0, P1=0, P2=0.
These are technical agent audits, not human/manual-review authority.

Acceptance was deliberately withheld through multiple earlier candidates:

- `cd743d…`: test-only generation/seam authority was too broad; mixed read errors had the wrong
  precedence; secret versus invalid time/number handling and proposal row counts were incomplete.
- `7e16db…`: derived lowercase/scope secrets and exact-cap precedence remained incomplete.
- `beb914…`: semantic repairs were green, but strict Clippy exposed test-only warnings.
- `8dd572ae7a0a112466e542a728341a64167639c2d4871dbe6d48699a0c3825ee`:
  incomplete mandatory serializer and typed-category evidence remained.
- `63f184e33a53672d26d9c7808a2a4bdcf2927d6777aee5e05aff18920a51c6b2`:
  timestamp/projection evidence, prewrite logical collisions and malformed-native precedence were
  still incomplete.
- `927c151534ab816bd3484cfa8731ea869afdf478dbd2be8f128a9c4060862393`:
  behavior was green, but the same typed collision fixtures had not yet proven their parent-C2A
  versus child-prewrite categories.

The accepted `895fa418…` source closes the final proof gap without changing production bytes from
`927c151…`: duplicate-ID, folded project-name and duplicate-link fixtures are independently
recreated; their projected snapshots reach C2A as `InconsistentProjection`, while typed ingress
rejects them before mutation as `InvalidGeneration` with zero C2A calls.

## Deliberate remaining gates

C2B1 remains an in-memory acquisition slice, not a runnable persistent native-memory successor.
The researched C2B2 direction must next prove evaluator-owned persistent containment, no host
shares or ambient network, fixed disk bounds, exclusive writer/contender behavior, close/reopen
durability and a fresh-reader journey. C2B3 must then bind a real correction journey's pre-state,
exact operator action and post-state without comparing fresh-key MACs.

Confirmed deletion, adapter/provider isolation and the final controlled Codex/Claude comparison
remain later gates. No live adapter, setting, daemon, authentication cache, provider or user
datastore was accessed or changed for C2B1. The full Engram flagship goal therefore remains active.
