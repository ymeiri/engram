# Native strict successor Stage C2B2a — entropy and build-ID repair freeze

Date: 2026-09-07 (Asia/Jerusalem)

## Status and authority

This is a pre-implementation, provider-free repair design. It is not an implementation, payload
identity, build record, binary acceptance, VM result, RocksDB result, C2B2a acceptance or flagship
completion record.

After an independent review binds this document's exact SHA-256 and reports `P0=0` and `P1=0`,
this document authorizes only the bounded source edits, support-file creation, archive provisioning
and provider-free qualification builds defined below. It authorizes no VM, provider,
authentication, datastore, live adapter, live Engram store or user-data execution.

The post-edit manifest, lockfile, Cargo configuration, support files, archive manifest, target
graph, patched source tree, final ELFs and build IDs do not exist as accepted identities yet. A
separate exact build/source identity record and independent `P0=0`/`P1=0` review must bind them and
the cross-built probe before a Linux runner freeze may be prepared. That reviewed runner freeze is
then required before any VM or Linux probe starts. A subsequent exact runtime-qualification record
and review must bind all three probe results before C2B2a acceptance. No value derived by an
unreviewed qualification build or runtime is authority.

Every clause and exact identity in the accepted C2B2a freeze and reviewed dependency-firewall
addendum that is not explicitly superseded here remains unchanged. This document narrows and does
not silently waive an entropy, source, dependency, query, environment, binary, containment,
seccomp, reproducibility, cleanup, disk, audit or nonclaim gate; every deliberate scope
substitution is enumerated below.

## Repair-review ratchet

Candidate SHA-256 `897c197dd1d9fcb436fa7a9e6db6c65d94b567e0f9c5dda05806d6ae2fc6ded5`
was rejected and authorizes nothing. Independent reviews found a still-reachable RocksDB
`/proc/sys/kernel/random/uuid` entropy path, a missing mandatory dependency-test edit, no complete
pre-repair payload authority, same-owner source mutation through an overbroad write sandbox,
inaccurate Cargo/rustc modeling, an impossible ring occurrence gate and incomplete archive, tool,
path, signal and core-dump controls. No verdict for that SHA transfers to this revision.

Candidate SHA-256 `64a0506be7c27e65f65383b9af05c72ba38b46c9cbf9e7ac79c305fbbc1efb04`
(2,492 lines, 166,021 bytes) was also rejected and authorizes nothing. Two exact-SHA reviews
returned `P0=0`/`P1=0`, but the controlling host-validation review returned `P0=0`/`P1=1`: its
seven-file Cargo child-environment source table omitted reachable dynamic-path, compiler-query,
jobserver, test and external-subcommand derivation sites. This revision replaces that purported
closed table with the complete pinned Cargo Git tree, exact reviewed anchors and a mandatory
machine-readable per-command reachability/mutation manifest. No clean verdict for the rejected SHA
transfers to this revision.

## Bound predecessor authorities

```text
C2B2a calibration and containment freeze
  evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_CALIBRATION_CONTAINMENT_FROZEN_2026-09-07.md
  SHA-256 85312b0286080ecbbab94b5236003a42f281b1122565016d96bb20f9572063e3
  lines   1451

C2B2a calibration and containment review
  evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_CALIBRATION_CONTAINMENT_REVIEW_2026-09-07.md
  SHA-256 22ac94877a58d14391591afd452e8ac51a2a9c5828d7a2f32639a2aad3c7a011

Dependency-firewall addendum
  evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DEPENDENCY_FIREWALL_ADDENDUM_FROZEN_2026-09-07.md
  SHA-256 f31df61c5f4def5270b3aa5915aeeb74fe59cfde24d9e7566d5c9a24c61977a8
  lines   222

Dependency-firewall addendum review
  evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DEPENDENCY_FIREWALL_ADDENDUM_REVIEW_2026-09-07.md
  SHA-256 2812bcb480ce70427b580519b580e4928c59c9fc6b5c6512f3b6192a8a8f93cd
```

The following protected inputs remain byte-for-byte unchanged:

| Input | SHA-256 |
|---|---|
| workspace `Cargo.toml` | `ec61531f8ffd401211f6492c7b91e28a4ee5f0fa1a0fe2f199c63a841c9724f5a` |
| workspace `Cargo.lock` | `0eb924bb9008bb417cb9990081df8429520131aa6b3f7367866c5b805ec474de` |
| `engram-eval/Cargo.toml` | `4176c6a4886492f7d5adaa0bdc3134d7e04bfc8d1536de4d5eb8d4b8985ce2bb` |
| `engram-eval/src/native_vm.rs` | `7236583a702a85998e1be5a4c32950df239d32a384d4bf82e2b9dc961c90b69a` |
| `engram-eval/src/native_c2b2a.rs` | `3501336994fa7ede67c51c1fd0b7b6dc0e94595dce058d434d63edadd0122904` |
| `engram-eval/src/bin/native-c2b2a.rs` | `f2c3704cae113330674cea0d1724ee7befd6c5c8c548e7963289b55c6f90ac27` |
| `engram-eval/tests/native_c2b2a_foundation.rs` | `4f7bd87691c18d5ce2d82634415ad6635cd08d0e6fa7ad92388daaf21cfa1638` |

The pre-repair nested inputs are evidence for the permitted delta, not post-repair identities:

| Input | Pre-repair SHA-256 |
|---|---|
| `engram-eval/native-c2b2a-payload/Cargo.toml` | `3b28236b23a2b1116bcbbd696d9d6c0c78a707b6a2de791933c73277e5fb7ff8` |
| `engram-eval/native-c2b2a-payload/Cargo.lock` | `40a484a61cd15e2555e0a18bd532a8229d78ab4144945b2fe56984c271e7f19b` |
| payload `.cargo/config.toml` | `cbe42bd9dbe49e8dd350166544e3cfbdc372a4a9abbd196d41f55b55d0f341ec` |
| payload `rust-toolchain.toml` | `d5d0e289ce24dcad9cbe5620576217d1f57c8f5248cc14001a249816534ec8ab` |

The captured complete pre-repair payload—not merely the four rows above—is the pre-repair
authority. Its canonical stream excludes the payload root itself, begins with the schema row below,
then contains one row for every descendant directory or regular file in raw-UTF-8 path order.
Directories use
canonical mode `0555`; regular files use `0555` iff any execute bit is set and otherwise `0444`.
It has exactly 23 LF-terminated rows, 1,839 bytes and SHA-256
`c6e6de9d4b8c6cd7a162341f43506bce15154c1d54e19ff6aad40f2c5c67f8d2`:

```text
schema=c2b2a-pre-repair-payload-tree-v1
directory<TAB>0555<TAB>.cargo
file<TAB>0444<TAB>346<TAB>cbe42bd9dbe49e8dd350166544e3cfbdc372a4a9abbd196d41f55b55d0f341ec<TAB>.cargo/config.toml
file<TAB>0444<TAB>108142<TAB>40a484a61cd15e2555e0a18bd532a8229d78ab4144945b2fe56984c271e7f19b<TAB>Cargo.lock
file<TAB>0444<TAB>1237<TAB>3b28236b23a2b1116bcbbd696d9d6c0c78a707b6a2de791933c73277e5fb7ff8<TAB>Cargo.toml
directory<TAB>0555<TAB>patches
file<TAB>0444<TAB>980<TAB>3c0127f47f2f63990d1c3387d991bc24afd1ec8d4969a1f87f5060b8da03d09d<TAB>patches/surrealdb-librocksdb-sys-0.17.3-rocksdb-getentropy.patch
file<TAB>0444<TAB>127<TAB>d5d0e289ce24dcad9cbe5620576217d1f57c8f5248cc14001a249816534ec8ab<TAB>rust-toolchain.toml
directory<TAB>0555<TAB>src
directory<TAB>0555<TAB>src/bin
file<TAB>0444<TAB>1180<TAB>ec5ff01fc4ab16a334fc5f30b53741971c817ecf58a54e4b5fddf34058a1a88e<TAB>src/bin/collector.rs
file<TAB>0444<TAB>1301<TAB>4ceca71496337fa07565ef3df3c2b628dbb3e09e9a36f26266dc8183ad1aeff6<TAB>src/bin/supervisor.rs
file<TAB>0444<TAB>45528<TAB>0c9ead1aefc00a722819481c23bf33fa064df7bfcff641ae45b96692ef4dabe8<TAB>src/contract.rs
file<TAB>0444<TAB>38934<TAB>cdf143a561274512c510e0734d7f65f7d99c229e2a42978e5a8dafce512eece2<TAB>src/fixtures.rs
file<TAB>0444<TAB>181<TAB>7b02babd0aa1f881fcfb9642196aafe6ed1181a63bb2bf8d272c18750153cac9<TAB>src/lib.rs
file<TAB>0444<TAB>97347<TAB>cbd747e0d35ccdcfc78972eb804fc8700c3076d8935efc57f949e651e39786c7<TAB>src/protocol.rs
file<TAB>0444<TAB>12054<TAB>499b3986e4439c994cccab8dc4e43af636708853dddc8ad7062f3c236155101e<TAB>src/rocks.rs
file<TAB>0444<TAB>29180<TAB>e11d04fb371860bc3d40991ec8a9f48c9702878c72531d9b3d71adba49aab251<TAB>src/seccomp.rs
directory<TAB>0555<TAB>tests
file<TAB>0444<TAB>21846<TAB>90a66276fc5109bb358f368a8dd271d603549d8a7ee8036b5f8c4129fb4a9af4<TAB>tests/contract.rs
file<TAB>0444<TAB>33247<TAB>96ef921788663ace466666cb2ad42b1f74d214631e3e6ecb0c1bdd6680253dfc<TAB>tests/protocol.rs
file<TAB>0444<TAB>18649<TAB>791290945222d9b567d83ddf1886c0a56bedded1f7852a31a03890a1fb2f5e09<TAB>tests/rocks.rs
file<TAB>0444<TAB>13305<TAB>80a7e604882f13204b118716eb094af664d99eec75567b1f5ed335719f4241e5<TAB>tests/seccomp.rs
```

`<TAB>` above is the document's exact escape token for one byte `0x09`; it is not part of the
canonical bytes. This escaped rendering and its row/byte/digest triple were captured before the
patch repair and are the sole baseline authority. The reviewed controller first replaces every
exact token with
one tab, reconstructs these frozen bytes and verifies the triple, then compares candidate paths,
modes, sizes and hashes against these rows with only the named delta allowed. It must never hash the
then-current checkout and call that a baseline. This freezes the repaired protocol and seccomp bytes
instead of allowing a later A/B copy to invent its own baseline.

The old 980-byte patch is also independently recoverable as the exact first 980 bytes of the bound
replacement patch; those bytes hash to the old row digest above, and the following byte begins the
second file's diff header. The reviewed controller must prove that prefix relation before accepting
the baseline
or candidate delta.

## Exact supersession boundary

This document supersedes only these predecessor statements:

1. The phrase "stock-Core backend" at calibration-freeze lines 14–15 is qualified as stock
   `surrealdb-core 2.6.0` with one exact, archive-derived and reviewed
   `surrealdb-librocksdb-sys 0.17.3+10.6.2` source patch.
2. The source-file list at calibration-freeze lines 306–330 is expanded only by the changed patch
   and six new build-support files below; their one parent directory is an additionally authorized
   structural path.
3. The five-direct-package statement at addendum lines 57–66 is expanded by the exact optional
   `getrandom02` dependency below.
4. The pre-repair nested manifest, lock and configuration identities are replaced only after a
   later exact build/source identity record binds their post-edit bytes.
5. The target-active graph v1 identity at addendum lines 68–109 is replaced by the graph v2
   construction below after its exact identity is independently reviewed.
6. The payload build-flag statement at calibration-freeze lines 534–539 is expanded by one GNU
   SHA-1 build-ID linker option and the complete fresh-root remap below.
7. The package/source firewall at calibration-freeze lines 1321–1341 and addendum lines 191–217
   is expanded by exact patched-source and three-version entropy evidence. Its denials are
   unchanged.
8. The ambiguous "full provider-free evaluator matrix" and strict-check phrases at
   calibration-freeze lines 1401–1402 are replaced by the complete repair-scoped C2B2a matrix
   defined under `host-validation`: ten nested payload test invocations, two direct 42-test
   foundation runs, the direct no-argument host-foundation build/run, seven exact Clippy
   invocations, rustfmt and controller whitespace/source checks. This repair makes no claim that
   unrelated `engram-eval` or workspace targets were rebuilt or rerun. Earlier evidence for those
   unchanged targets is contextual only and cannot satisfy a repair gate.

The workspace sources, host/payload protocols, fixtures, query ASTs, schedule, resource envelope,
environment table, error classifier, cleanup order, seccomp programs and runtime plan are not
changed by this repair.

## Exact bounded edit and support boundary

Only these five existing files may differ from the pre-repair stream. The patch must already match
the replacement digest bound below and is immutable after review; only the other four receive an
implementation edit after this document is independently accepted:

```text
engram-eval/native-c2b2a-payload/Cargo.toml
engram-eval/native-c2b2a-payload/Cargo.lock
engram-eval/native-c2b2a-payload/.cargo/config.toml
engram-eval/native-c2b2a-payload/tests/contract.rs
engram-eval/native-c2b2a-payload/patches/
  surrealdb-librocksdb-sys-0.17.3-rocksdb-getentropy.patch
```

Only these seven structural support paths—one directory and six files—may be added:

```text
engram-eval/native-c2b2a-payload/build-support/
engram-eval/native-c2b2a-payload/build-support/archive-manifest-v1.tsv
engram-eval/native-c2b2a-payload/build-support/root-cargo-config.toml
engram-eval/native-c2b2a-payload/build-support/controller.py
engram-eval/native-c2b2a-payload/build-support/provision.py
engram-eval/native-c2b2a-payload/build-support/entropy-failure-probe.cc
engram-eval/native-c2b2a-payload/build-support/entropy-failure-probe.rs
```

The executable/source boundary above excludes only these enumerated non-input audit records:

```text
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_REPAIR_FROZEN_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_REPAIR_REVIEW_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_IMPLEMENTATION_PREFLIGHT_REVIEW_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_IDENTITY_FROZEN_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_IDENTITY_REVIEW_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_LINUX_RUNNER_FROZEN_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_LINUX_RUNNER_REVIEW_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_RUNTIME_QUALIFICATION_FROZEN_2026-09-07.md
evals/native_memory_pilot_v1/
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_RUNTIME_QUALIFICATION_REVIEW_2026-09-07.md
```

They cannot be read by Cargo, compilers, build scripts or runtime probes. A pre-implementation
worktree snapshot distinguishes pre-existing user-owned changes from this repair; those changes are
preserved, and no new mutation outside the bounded payload paths and exact audit-record paths is
attributed to this work.

No vendored crate tree is committed to the repository. No `[patch]` table, path dependency, git
dependency, local-registry source or Cargo path package is permitted. The provisioned build uses a
Cargo directory source derived from exact registry archives. Cargo metadata must retain the
registry identity for every registry node; the standalone payload remains the sole path node.

The rejected one-file patch is retained only in the pre-repair stream above. The replacement
two-file patch input is bound now:

```text
patch SHA-256
  c82a6d05411362963eb8049cc6a0ecb563562bd1d35e14bfe892f39db936fcfb
patch lines / bytes
  64 / 1565
upstream surrealdb-librocksdb-sys 0.17.3+10.6.2 crate archive SHA-256
  db194f1cf601bb6f2d0f4cbf0931bc3e5a602bac41ef2e9a87eccdfb28b7fed2
upstream rocksdb/env/unique_id_gen.cc SHA-256
  c689cd2a10dfe02bfb94f0b92d8730ee158a8a8ae7d5066beaf1facbc1232310
patched rocksdb/env/unique_id_gen.cc SHA-256
  44fcb9e1535c23ed9f0017e25be197fa0c365bf7a8a340aafa540526e485a4a5
upstream rocksdb/port/port_posix.cc SHA-256
  a34341c2e48b59a21037709c5cd34382991c4684a1db4f39e1ce73697ab2b62f
patched rocksdb/port/port_posix.cc SHA-256
  70daf26d27666616456679eb63fecdb5f703606b4583da6af00e53e8b9c29bec
```

The first patched file removes `<random>` and `std::random_device` and retains a 24-byte
`std::array<unsigned int, 6>`. The pinned AArch64-musl ABI must independently prove
`CHAR_BIT == 8` and `sizeof(unsigned int) == 4`; “192 bits” describes the requested byte
capacity, not an entropy-strength guarantee. When the existing `exclude_random_device` guard is
false, it performs one source-level `getentropy` call for that array and calls `std::abort()` on any
nonzero result. It introduces no retry or fallback. The source-level call count is not a claim that
pinned libc performs only one kernel syscall.

The second patched file removes `<fstream>` and the sole
`/proc/sys/kernel/random/uuid` open from Linux `port::GenerateRfcUuid`. The function clears its
output and returns `false` without a syscall. This intentionally activates RocksDB's existing
`Env::GenerateUniqueId` fallback, which calls `GenerateRawUniqueId(..., true)` and therefore reaches
the patched getentropy track while excluding the now-disabled port-UUID track. Direct raw-ID users
with `exclude_port_uuid=false` also see a deterministic false/no-I/O port track after the patched
getentropy track. No uniqueness fallback is removed: the 24-byte kernel-random track and existing
environment-details track remain inputs to RocksDB's raw-ID hash.

The production call graph is a required sealed proof, not an inference from the patch. Its universe
is exactly the pinned `rocksdb_lib_sources.txt` compilation set plus headers reachable in the actual
build; tool, benchmark and test sources outside that set are separately listed as excluded. Within
that universe it must enumerate every caller of `Env::GenerateUniqueId`, including fresh-database
identity and empty file-identity generation, prove that any wrapper delegates to that method, and
show the exact fallback edge from the patched `port::GenerateRfcUuid == false` result to
`GenerateRawUniqueId(..., true)`. It must separately enumerate every direct production caller of
`GenerateRawUniqueId` and state the `exclude_port_uuid` value at each call. Both patched translation
units must appear in the pinned RocksDB compilation list. Object and final-ELF evidence must then
show that the `GenerateRfcUuid` symbol's disassembly and relocation slice has no call edge to a
file-open or stream primitive and that its object has no UUID-path literal; other functions in the
same `port_posix.cc` translation unit are not falsely required to have no file-I/O imports. The
`unique_id_gen.cc` object must contain the intended `getentropy` and abort edges, and the final
collector ELF must contain no `/proc/sys/kernel/random/uuid` literal. A production fresh-database
trace must additionally show no `/proc/sys/kernel/random/uuid` or `/dev/*` access after the declared
bootstrap boundary.

## Exact manifest edit

The post-review manifest edit is limited to replacing the collector feature with this exact TOML:

```toml
collector = [
    "dep:getrandom02",
    "dep:surrealdb-core",
    "dep:tokio",
]
```

and adding this exact dependency table after the existing `sha2` entry and before
`[dependencies.surrealdb-core]`:

```toml
[dependencies.getrandom02]
package = "getrandom"
version = "=0.2.17"
default-features = false
features = ["linux_disable_fallback"]
optional = true
```

No Rust dependency expression, import, extern-crate declaration, crate path or executable code path
may reference or resolve through `getrandom02`; its sole purpose is Cargo feature unification. Exact
token occurrences are permitted only in the manifest and the named contract test's dependency and
TOML-string assertions. The final graph must show the direct alias edge and exactly one
`getrandom 0.2.17` node. Existing transitive users retain `default` and `std`, so the expected
unified feature set includes `default,linux_disable_fallback,std`. The exact final set is
machine-derived and must be sealed in the build/source identity record; the requested alias
feature must not be
mislabeled as the complete set.

Cargo 1.93.0 may update the standalone lock only through the fresh offline lock-derivation root
defined below. A structural comparison must prove that the sole resolution change is the payload
root's direct dependency edge to the already locked `getrandom 0.2.17`. No package version,
registry source or registry checksum may change. The repository update must use the exact
Cargo-produced bytes under the active repository editing rules and then receive an exact-SHA review.

The lock-writer source authority is Cargo repository commit
`083ac5135f967fd9dc906ab057a2315861c7a80d`, exact path
`src/cargo/ops/lockfile.rs`: 9,105 bytes, 254 LF-terminated lines and SHA-256
`abb5bba6e027e56abab733c1d8d529143dde313bbd0f8f6e1f8910c5010138d8`. Its
`write_pkg_lockfile` implementation at lines 44–107 opens the existing `Cargo.lock` with
`open_rw_exclusive_create`, calls `set_len(0)` and writes the serialized bytes to that same file. It
does not create a temporary lockfile or rename a replacement. The later build/source identity
record must bind
these source bytes, the executed Cargo binary and its toolchain provenance; version text alone is
insufficient.

The Cargo child-process authority is the complete tracked source tree at the same Cargo repository
commit. Its root Git tree object is `ad1967c7c91181b65043fc8792002d1ff07a63ed` and its recursive
non-root inventory has exactly 4,460 entries: 1,560 trees, 2,900 blobs and zero commit/gitlink
entries. The blobs comprise exactly 2,774 mode-`100644` files, seven mode-`100755` files and 119
mode-`120000` symlink blobs. Git SHA-1 object names are provenance, not the sole integrity gate.
Implementation preflight must independently acquire and bind that commit and tree, reject an
untracked overlay, inventory truncation or unexpected entry type, and emit a complete canonical
SHA-256 source-universe manifest. That manifest begins with these three exact LF-terminated rows:

```text
schema=c2b2a-cargo-source-universe-v1
commit<TAB>083ac5135f967fd9dc906ab057a2315861c7a80d
root-tree<TAB>040000<TAB>ad1967c7c91181b65043fc8792002d1ff07a63ed
```

Each following row is exactly one of:

```text
tree<TAB>six-digit-git-mode<TAB>git-sha1<TAB>lowercase-hex-raw-path
blob<TAB>six-digit-git-mode<TAB>decimal-size<TAB>git-sha1<TAB>sha256<TAB>lowercase-hex-raw-path
```

Those 4,460 rows are LF-terminated and sorted by raw path bytes, producing exactly 4,463 rows in the
complete stream. Every non-root entry occurs exactly once. Symlink blobs use the exact link-text
blob bytes rather than a followed checkout target. The manifest's exact byte count and SHA-256 are
later-bound implementation-preflight values; its entry and mode counts must equal the constants
above. The Git identities, canonical SHA-256 manifest and executed Cargo binary's source provenance
must all agree; the commit name, Git SHA-1 or version text alone is insufficient. The controller
and checker sources embed the independently reviewed source-universe and launch-influence digests;
every applicable existing phase `result.json` repeats those expected digests. The preflight source
universe is evidence used to establish binary provenance, not a runtime child or controller input,
and this rule creates no additional run-evidence pathname. Runtime integrity is enforced by the
already required pre/post tool/executable seal; source-to-binary provenance drift requires a new
preflight review and blocks the run.

The following exact LF-terminated files are mandatory human-reviewed semantic anchors. This table
is deliberately a non-exhaustive lower bound and does not assert that an unlisted file is
irrelevant:

| Cargo-relative path | Bytes | Lines | SHA-256 |
|---|---:|---:|---|
| `src/cargo/core/compiler/compilation.rs` | 18,973 | 513 | `0728c3d56fcb231620b9414bdae8f8892c280f7fa9126c303915f74d60313c57` |
| `src/cargo/core/compiler/mod.rs` | 98,293 | 2,559 | `bfc01e95e463fea7d667126f61ed3ade3f39c303d1bc28dc390f9243e81c621c` |
| `src/cargo/core/compiler/custom_build.rs` | 58,951 | 1,411 | `8537d274d4fac08675e6abe4375633e5e4b13b5e8ba038bdf43613308d9ad53c` |
| `src/cargo/core/compiler/artifact.rs` | 5,389 | 122 | `5988955e5514984303dc65ee93bbc4f37d30a7f1d6a1a38ece374909a4517f79` |
| `src/cargo/core/compiler/build_config.rs` | 11,590 | 331 | `00867ca2a03eb743257039d73e2faa1bf08ed40677a10991ef16bc12cbb72d11` |
| `src/cargo/core/compiler/build_context/mod.rs` | 5,007 | 151 | `14407baef65f03a036ac3033beabab1ed0f0fcbb2d38d1e4f2605e6cc8b5ae25` |
| `src/cargo/core/compiler/build_context/target_info.rs` | 43,373 | 1,119 | `69a8ad188d3905d3527e19d76898d637380d3de1e98ff810264859631c4b1a72` |
| `src/cargo/core/compiler/build_runner/mod.rs` | 32,758 | 777 | `47b785e2c580468306e4a08ef0ae46fd43cfe7813630cecfb6263b9e576ac4f1` |
| `src/cargo/core/compiler/build_runner/compilation_files.rs` | 37,823 | 924 | `3dc1c00117290855c2a0dbb4ef174c3eaf0efd394306ab28984819597f76a072` |
| `src/cargo/core/compiler/compile_kind.rs` | 9,332 | 243 | `08be11b7cac306657ecf7f3a45c3c2f2ac0a1605d460418ea8789cf7e8793d0e` |
| `src/cargo/core/compiler/layout.rs` | 14,188 | 388 | `9d510a5f31599924637b24ea6e9583c2e667dbb615f9be089aa96486426c81e9` |
| `src/cargo/core/manifest.rs` | 38,107 | 1,212 | `b7ffc9c8fa7065645ffc5fd3523f8f3024ea30ade0bd2f0ff1540c77f0fb0386` |
| `src/cargo/ops/cargo_compile/mod.rs` | 39,983 | 1,033 | `5cb0b5c11f463f1d7b94cb1ab167482695badb731a2b2d2dddf3145cebcd7390` |
| `src/cargo/ops/cargo_test.rs` | 13,522 | 437 | `1ecdfa10f871408fc17235d76f690b7d2517e48557730bbab2be3088ee932a14` |
| `src/cargo/util/rustc.rs` | 14,035 | 392 | `bdc3b21d37cb07c472cb1e59964f99f91734071b75257f842eb8467ba00ecb93` |
| `src/cargo/util/workspace.rs` | 5,536 | 150 | `a8ab44391e698a50d1b8f57a33ff6205866560e7f49520c1334aa8db2f419f83` |
| `crates/cargo-util/src/process_builder.rs` | 24,760 | 709 | `225dad38fa80928269fcd572d55ff21384c365b043c39c6a6090434d995263ae` |
| `crates/cargo-util/src/paths.rs` | 38,519 | 1,067 | `3c87a15cf681449d61b9452a4ebc4fb2f093d6dfa42bf4fdb64c5918b8efcb9e` |
| `crates/home/src/lib.rs` | 5,150 | 150 | `809fccdf4045c6131d8ff558f9dccb3a74bd0011571a7fec8ef27d402535a085` |
| `crates/home/src/env.rs` | 4,054 | 114 | `7d4d59640e66f75f7e2fd36708ce2f893f27dddc2add8b5bbccae92d5d2d6594` |
| `src/bin/cargo/cli.rs` | 30,487 | 797 | `9648dbe8147e7475ca14b453aba92f0bca9a7741abb9a66d65ee0becc9cae560` |
| `src/bin/cargo/main.rs` | 14,807 | 442 | `09d697445aca0a9c80be088052645e4bb14bd2ba2b916083ef62b1bcb28ed4c8` |
| `src/bin/cargo/commands/mod.rs` | 3,177 | 133 | `8478ae1963efff4987b3c7b8c6d64c527f238df9493e517b32a20aae592dbf81` |
| `src/bin/cargo/commands/build.rs` | 3,421 | 82 | `7bb9b097faac8e608ff07fcac3e49ec046726c6cb495e1eecc6da1c4c9fffda6` |
| `src/bin/cargo/commands/test.rs` | 4,486 | 115 | `e7d9f3882c4c72f9f00088993b8564985137585238e1551c28024ba540de5413` |
| `src/cargo/util/context/mod.rs` | 100,721 | 2,590 | `84589b0311db9a6f39d1c027fb79e5c6a25f86dd7ba3c5d312a93fa3dfa6d323` |
| `src/cargo/util/context/config_value.rs` | 10,832 | 311 | `7e5b78c61141896f54ddafebcbfc5db4b71c66eaab33e3da2976a5d6a65bb059` |
| `src/cargo/util/context/environment.rs` | 8,576 | 193 | `d2d47d2f2d5f5eab242921b8b90971d89b8fbd8628faa07b2bb2f8022e1d8815` |
| `src/cargo/util/context/key.rs` | 6,357 | 201 | `9e7d6af98f4c98a6855263ba3d974fb11fcf2245db97e9688f0942813ad8784e` |
| `src/cargo/util/context/de.rs` | 29,749 | 852 | `5478e14ee8237e7d3291dc6e4521edb42cd93c271827aa40a3c877bef5a571fa` |
| `src/cargo/util/context/path.rs` | 5,436 | 166 | `aef43baa2e3d1c763c6671c9e8fdf4bb9a5194ffabfb159b019b0dee96fedde4` |
| `src/cargo/util/context/target.rs` | 10,542 | 248 | `e7c8abef97c0af703dc311555d57fbe455292ee5245d6e39d9174b8cfda72d6c` |
| `src/cargo/util/context/schema.rs` | 13,579 | 511 | `f5856fdcd9ffeeff433825699c4be5f60d7a0493bbcfb1ae03489d36c0e0e73c` |

These anchors cover the known direct and value-supplying paths for `fill_env` and
`fill_rustc_tool_env`; dynamic-library variable selection, inherited-value splitting and path
joining; `prepare_rustc`, primary-package, SBOM and target-temp additions; environment snapshot and
configuration discovery; Cargo-home and tool resolution; compiler query construction and cache
inputs; build layout, job count and BuildRunner state; build-script `OUT_DIR`, target/host/profile,
feature, cfg, rustc/rustdoc/rustflags and jobserver setup; artifact and package variables; test-child
construction; built-in and external-subcommand dispatch; and `ProcessBuilder`'s ordered
set/remove/inherit/exec merge. In particular, `src/bin/cargo/main.rs::execute_subcommand` adds
`CARGO`, may attach a jobserver and exec-replaces Cargo with `cargo-clippy` or `cargo-fmt`;
`target_info.rs` applies config, removes `RUSTC_LOG` and constructs compiler queries; and
`cargo-util/src/paths.rs` selects, reads, splits and joins `DYLD_FALLBACK_LIBRARY_PATH` on macOS.

Implementation preflight must emit a conservative canonical per-command launch-influence manifest
over the complete Cargo tree. It is rooted in every exact fetch, metadata, build, test, Clippy and
format command handler in this freeze. Each entry binds the source-universe digest, repository,
exact source path and SHA-256, callable and byte/line interval, applicable frozen command and phase,
exact reachability chain, activation condition, ordered transform and disposition. Its closed sink
classes cover executable resolution, selection, spawn and exec-replace; argv and arg-zero changes;
environment read, clear, derive, set, remove and inherit; cwd; configuration, manifest, CLI and
environment discovery, merge and precedence; `HOME`, `CARGO_HOME`, `PATH` and dynamic-library-path
derivation; target, compiler, linker and build-output selection; jobserver creation, import,
configuration and environment/fd propagation; and build-script-emitted environment and native
paths. Compiler-query cache lookup, keying, creation and reuse are separate sink classes. Entries
are ordered by source universe, raw path bytes, starting byte, callable, sink class, command and
phase.

The analysis conservatively includes macro-expanded, generated and dynamically selected sites. A
site is either modeled or has a source-grounded proof that the exact cfg, feature, argv,
configuration and environment make it unreachable; uncertainty is treated as reachable. A second
whole-universe sink inventory is generated independently and must be exactly equal to the modeled
plus proved-unreachable manifest inventory. An unknown or extra sink, dynamic edge, variable,
configuration source, mutation site or precedence rule is terminal. A broad source-tree hash or a
callgraph-selected subset alone cannot satisfy this semantic gate. The exact source universes,
manifest bytes, inventory bytes and both checkers require independent implementation-preflight
review with `P0=0` and `P1=0` before any build command.

A manifest transform contains exact bytes wherever source plus preflight inputs determine them.
Values that Cargo allocates internally at runtime, including jobserver descriptor numbers and the
corresponding generated `CARGO_MAKEFLAGS` bytes, use a closed typed placeholder with exact source
derivation and applicability rather than an invented value. Such a placeholder is non-authoritative
and cannot become an identity or acceptance fact. The active build-script/helper source audit must
prove that no accepted byte, path or decision depends on an opaque value; otherwise the gate blocks.
No static model claims exact post-exec descriptor numbers or a complete descendant descriptor
history.

For every compiler-query cache branch, the manifest binds the lookup/key/write/reuse semantics and
the exact pre- and post-invocation state of the applicable target-cache paths, including any
`.rustc_info.json`. A fresh `HOME` or `CARGO_HOME` does not imply a fresh shared target cache. A
later command may reuse a sealed query result rather than spawn a compiler query, and command mode
alone never proves query occurrence. Any cached input not produced and sealed inside the same
authorized root or not admitted by the exact phase prestate is terminal.

Every controller-launched Cargo-family outer environment in this freeze—acquisition fetch, either
metadata command, qualification build, payload test, Clippy or format—contains exact
`CARGO_CACHE_RUSTC_INFO=0`. The exec-replaced external subcommand and its nested Cargo/tool chain
inherit that value under the reviewed manifest. Before and after every applicable Cargo-family
invocation, the controller performs a no-follow recursive basename scan of its authorized
target/build root and requires exactly zero entries of any type named `.rustc_info.json`; the zero
count is a required observation in that phase's existing `result.json`, not a new evidence leaf.
In-process query caching inside one Cargo process remains source-modeled and is not evidence that a
query exec occurred.

The same preflight binds the complete 29,013-byte `jobserver 0.1.34` crate archive with SHA-256
`9afb3de4395d6b3e67a780b6de64b51c978ecf11cb9a462c66be7d4ca9039d33`, not merely its package name
or two anchor files. Its exact source anchors include `src/lib.rs`, 27,241 bytes, 704 LF-terminated
lines and SHA-256 `9b82f32a87deae5bd5a0ed36cdab3a195f9419c6befa9f77a682f262b9158722`,
and `src/unix.rs`, 21,681 bytes, 634 LF-terminated lines and SHA-256
`353cc1b9653bc30e889522dcf31c9f0d1ae120ce4c1248dfc647ea8435171cad`.
The full extracted source tree and every linked platform module are bound by an archive-rooted
canonical tree manifest under the same path/mode/size/SHA-256 principles. The controller's top-level
environment has no `CARGO_MAKEFLAGS` or inherited jobserver. Cargo is nevertheless expected to
create its own reviewed jobserver when the ambient one is absent and to add `CARGO_MAKEFLAGS` plus
its source-defined inheritable descriptors at applicable child boundaries. Only that
manifest-derived transition is permitted; absence from the top-level environment is not a claim
that the variable is absent from Cargo descendants.

For `cargo clippy` and `cargo fmt`, the closure continues through the full exact source trees and
binary provenance of the sealed `cargo-clippy`, `clippy-driver`, `cargo-fmt` and `rustfmt`
components. Their complete reachable exec, argv, environment and cwd transitions receive separate
canonical manifests under the same schema and review gate; they cannot be attributed to Cargo's
source tree. The same rule covers every exact locked build script or helper allowed to spawn a
compiler or other child. A build-script or tool grandchild is never attributed to Cargo's source
model alone. Unavailable source, an incomplete source universe or unproved source-to-sealed-binary
provenance blocks the run.

Static source analysis defines the only permitted launch transitions; it does not prove that a
particular exec occurred or capture a kernel history of post-exec environments. Occurrence still
requires the enforced sandbox/exec closure, process-group EOF and reap, fingerprints, outputs and
artifact evidence already required by this freeze. Any fact that would require complete historical
per-PID observation remains blocked. An actual post-exec loader-preserved environment value may
become an identity fact only when separately established for the exact executable boundary by the
reviewed diagnostic/evidence rules; the requested environment map alone is insufficient.

One controller substantive invocation includes the outer Cargo process, an exec-replaced
`cargo-clippy` or `cargo-fmt` process when applicable, and every nested tool it launches. Only the
controller's outer entry uses the fixed `/usr/bin/env -i` trampoline and receives newly created
`home/active` and `cargo-home/active` roots. Those same active roots remain in force until the whole
process group reaches EOF, is reaped and is archived; an internal exec or nested Cargo launch does
not rotate or reuse a different active root.

At each qualification or host-validation top-level Cargo exec, the rotated `CARGO_HOME` is newly
empty. The only discoverable Cargo configuration files are the already sealed root and payload
`.cargo/config.toml` files, which have
no `[env]`, `include` or credential directive. The pinned context/config authority must prove that
no archived home, ambient home, CLI `--config` or later child write can enter the already loaded
configuration. Documentation or verbose transcript text alone is insufficient.

The sole permitted `tests/contract.rs` edit is inside
`standalone_manifest_and_lock_preserve_the_frozen_package_firewall`. It changes the exact direct
dependency expectation to:

```rust
[
    "getrandom",
    "libc",
    "sha2",
    "getrandom02",
    "surrealdb-core",
    "tokio",
]
```

It also requires the exact alias stanza above and adds this one identity to the existing lock tuple
table:

```rust
(
    "getrandom",
    "0.2.17",
    "ff2abc00be7fca6ebc474524697ae276ad847ad0a6b3faa4bcb027e9a4614ad0",
),
```

No other test line or assertion may change. The test must prove that the alias is optional,
default-features is false, its only requested feature is `linux_disable_fallback`, and the
collector feature contains `dep:getrandom02` exactly once before the two existing entries.

## Exact payload configuration edit

The existing target rustflags retain their order and values. Append only:

```toml
    "-C",
    "link-arg=-Wl,--build-id=sha1",
```

immediately after the existing `"link-arg=-static-pie"` pair and before the closing array. The
linker and archiver are unchanged. The configuration continues to carry the sole Rust entropy cfg:

```text
--cfg getrandom_backend="linux_getrandom"
```

The added build-ID option applies to both supervisor and collector. A build ID is a reproducible
ELF note, not a source-integrity seal; SHA-256 remains the binary identity.

## Trusted controller and evidence boundary

Except for the explicitly non-authoritative host invoker and system environment-clearing entry
below, every host-side reference in this document to the controller, verifier, launcher, checker or
out-of-sandbox collector means the exact reviewed `build-support/controller.py`; it is not an
ambient script or manual procedure. The controller and `provision.py` are invoked with the same
later-bound sealed Python as `<exact-python> -I -B -S <exact-script> <exact-argv>`. Before either
script's first invocation, a
separate implementation-preflight review must bind their source SHA-256 values, all five
hand-authored support-file hashes, the patch hash, deterministic expected edit deltas, exact mode
and argv grammar, focused tests and `P0=0`/`P1=0`.
The controller, provisioner or an output produced by either cannot review itself.

The required top-level environment-clearing entry executable is the exact preflight-bound
`/usr/bin/env`, with this argv prefix:

```text
/usr/bin/env -i LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=0 HOME=/var/empty \
  __CF_USER_TEXT_ENCODING=0x1F6:0x0:0x0 \
  <exact-python> -I -B -S <exact-controller.py> <exact-controller-argv>
```

Thus the controller starts with exactly those six variables. The fixed CoreFoundation value is the
observed encoding for the preflight-bound invoking UID 502; preflight must re-prove both the UID and
that the selected Python preserves this supplied value rather than injecting or rewriting one.
`PATH`, `TMPDIR`, every `PYTHON*`
variable, credential, provider, proxy and dynamic-loader variable are absent from the controller
process. The exact sealed interpreter path is used directly; the controller may use only explicit
paths and arguments bound by its closed mode. Preflight binds `/usr/bin/env`'s canonical system path
and identity and verifies its `-i` semantics, verifies `/var/empty` is root-owned and not writable
by the invoking user, and proves controller behavior does not read or write that directory.

A non-authoritative host invoker starts that exact argv array with cwd `/var/empty`, umask `0077`,
fd 0 as read-only `/dev/null`, fds 1 and 2 as distinct nonseekable one-way diagnostic pipes, and all
fds greater than 2 close-on-exec. Its implementation, ancestry and captured diagnostics cannot
satisfy a gate. At the controller's first initialization boundary—after the sealed interpreter,
fixed reviewed imports and reviewed controller bytes have loaded, and before any other repository,
build-tool, root or evidence access—it independently verifies exact argv/environment/cwd/umask and
enumerates fds. It requires
only 0, 1 and 2, verifies their modes/types and distinct identities, and resets umask `0077`; a
mismatch is terminal. Controller-child output evidence uses new controller-owned pipes and never
the invoker's diagnostics. Thus only the observed entry state, not a claim about the invoker, is
authoritative.

The ambient environment, descriptors and initial controller-script read visible to the host
invoker, system entry or Python loader before that boundary are not claimed as evidence. Whether
the non-authoritative invoker used an intermediate shell is also not an evidence fact; only the
verified entry state is. The absence of a malicious
same-UID race between preflight, launch and postflight is an explicit host trust assumption. The
controller rehashes its script and complete Python/tool closure at entry and exit, and the
independent review rehashes them afterward, but this provider-free unprivileged design does not
resist an administrator or adversarial controlling UID that swaps and restores its trust anchor.
Requiring that stronger local-adversary property blocks this run pending a separately reviewed
signed or privileged bootstrap.

After this design review, the five hand-authored support files are installed first while the
pre-repair payload `Cargo.toml`, `Cargo.lock`, payload configuration and dependency test remain
unchanged and the generated archive manifest remains absent. The implementation-preflight review
then occurs. Only after it passes may controller `acquisition-setup` make the pre-edit seed copy.
The exact static manifest, payload-configuration and dependency-test edits occur only after that
seed is sealed. This ordering gives the controller an authorized reviewed implementation without
allowing the seed to contain post-edit dependency bytes.

The controller has a closed mode table for acquisition setup/fetch, its internal policy probe, the
six provisioner modes, lock derivation, graph metadata/finalization, root A/B supervisor and
collector builds, probe cross-build, source audit, root A/B artifact audit, host validation and
pre-runtime finalization.
There is no VM, probe-runtime, provider, adapter, datastore or user-data mode. It uses no shell,
`eval`, command string or PATH-based executable lookup. Child argv are arrays whose executable is
an exact absolute hash-bound path; root, phase and A/B selection are validated closed-enum inputs.
It alone creates the bounded roots, seed copies, pipes, process groups, deadlines, launch records,
evidence tree records and canonical in-memory manifest reconstruction assigned below.
`provision.py` retains only its six non-subprocess extraction/copy/seal modes.

Every evidence directory has only create-new regular files beneath one exact phase component from
this set:

```text
acquisition-setup acquisition-fetch acquisition-stage lock-payload lock-source lock-derive
graph-metadata graph-finalize archive-seal qualification-A-payload qualification-A-source
qualification-A-supervisor qualification-A-collector qualification-B-payload
qualification-B-source qualification-B-supervisor qualification-B-collector probe-build-A
probe-build-B source-audit artifact-audit-A artifact-audit-B host-validation-payload-tests
host-validation-foundation-test-compile host-validation-foundation-test-run
host-validation-foundation-bin-compile host-validation-foundation-bin-run
host-validation-lints pre-runtime-finalize
```

The evidence root for each phase is exact:

- `acquisition-setup`, `acquisition-fetch`, `acquisition-stage`, `archive-seal`,
  `graph-finalize`, `source-audit`, every `host-validation-*` phase and
  `pre-runtime-finalize` use `<acquisition-root>/evidence/<phase>`;
- `lock-payload`, `lock-source`, `lock-derive` and `graph-metadata` use
  `<acquisition-root>/lock-root/evidence/<phase>`; and
- every `qualification-A-*` and `probe-build-A` phase uses `<root-A>/evidence/<phase>`, while
  every `qualification-B-*` and `probe-build-B` phase uses `<root-B>/evidence/<phase>`; and
- `artifact-audit-A` and `artifact-audit-B` use their corresponding A/B evidence roots.

No phase may create or write an evidence leaf outside the exact root mapping above.
`graph-finalize` remains acquisition evidence even though it runs after both qualification
build-output trees and their post-command payload/source trees are sealed. The controller alone may
open no-follow and read already sealed cross-root evidence expressly listed by a phase:
`graph-finalize` reads the lock-root `graph-metadata` stdout and invocation plus its listed A/B
build inputs; each artifact audit reads both sealed graph-finalize TSV/digest leaves plus its
corresponding root's inputs; and `pre-runtime-finalize` reads the complete allowed evidence-leaf
inventory. No other cross-root evidence read is allowed, and no child may read evidence. The A/B
shared evidence
parents and whole roots remain controller-writable until their later artifact-audit phases are
sealed. For each applicable phase, the only controller evidence basenames are `invocation.json`,
`stdout.bin`, `stderr.bin`, `sandbox.sb`, `sandbox-denials.log`, `result.json`,
`policy-probe-invocation.json`, `policy-probe-stdout.bin`, `policy-probe-stderr.bin`,
`policy-probe-sandbox.sb`, `policy-probe-result.json`,
`pre-payload-tree.tsv`, `post-payload-tree.tsv`, `payload-tree.sha256`, `pre-source-tree.tsv`,
`post-source-tree.tsv`, `source-tree.sha256`, `expected-prepatch-source-tree.tsv`,
`expected-prepatch-source-tree.sha256`, `pre-tool-tree.tsv`, `post-tool-tree.tsv`,
`tool-tree.sha256`, `archive-staging-tree.tsv`, `archive-staging-tree.sha256`,
`archive-bundle-tree.tsv`, `archive-bundle-tree.sha256` and `executable-bundle-v1.tsv`.
The only additional host-validation tree leaves are `foundation-deps-tree.tsv`,
`foundation-deps-tree.sha256`, `foundation-test-tree.tsv`, `foundation-test-tree.sha256`,
`foundation-bin-tree.tsv` and `foundation-bin-tree.sha256` in the exact applicable
`host-validation-foundation-*-compile` phase directories defined below.
`graph-finalize/target-active-graph-v2.tsv` and
`graph-finalize/target-active-graph-v2.sha256` are the only additional graph-finalization paths. A
mode may omit inapplicable leaves but may create no unlisted evidence pathname. Every leaf is
create-new mode `0600`, written, fsynced and closed. Each leaf is finally re-opened no-follow and
hashed, after which neither content nor metadata may change.

Every `.json` evidence leaf uses one canonical serializer: UTF-8 without BOM; exactly one compact
JSON value and one trailing LF; no insignificant whitespace; object keys in the phase schema's
declared order; arrays in observed execution order; and only booleans, unsigned 64-bit integers,
arrays, objects and UTF-8 strings. Floats, signed numbers, null and duplicate/unknown keys are
forbidden. A quote is encoded as `\"`, a backslash as `\\`, and each U+0000 through U+001F control
scalar—including every U+001F in `CARGO_ENCODED_RUSTFLAGS`—as lowercase `\u00xx`; the short JSON
escapes are forbidden. Every other scalar is emitted directly as UTF-8 and may not be replaced by a
Unicode escape. Every `result.json` has ordered top-level keys `schema`, `phase`, `valid`,
`failures` and `observations`; every policy-probe result uses the same shape with schema
`c2b2a-policy-probe-result-v1`. `failures` is empty exactly when `valid` is true. Closed per-phase
observation keys and their types are part of implementation preflight, and an absent required key
or extra key is terminal.

For payload, source and tool trees, `pre-*.tsv` is the exact canonical stream immediately before
the phase's policy probe and `post-*.tsv` is the stream immediately after the phase's final
substantive child. An input or output that does not yet exist is inapplicable rather than
represented by an empty stream. The controller also recomputes every applicable tree immediately
before and after every individual child; the corresponding ordered invocation element binds each
row count, byte count and SHA-256. These intermediate streams need not receive additional
pathnames, but every intermediate identity that is required stable must equal the retained pre/post
bytes exactly.
`policy-probe-invocation.json` binds the same identities around its one probe child. Thus the
singleton tree basenames remain sufficient for a multi-child probe-build phase without weakening
the per-launch comparisons.

The three foundation subtree streams exclude their selected subtree root and begin respectively
with exactly one of these LF-terminated rows:

```text
schema=c2b2a-foundation-deps-tree-v1
schema=c2b2a-foundation-test-tree-v1
schema=c2b2a-foundation-bin-tree-v1
```

Each then uses the directory/file row grammar, four-octal-digit modes, decimal-size encoding,
raw-UTF-8 relative-path ordering and final-LF rule of the directory-source tree. The selected root
must contain only directories and link-count-one regular files; a symlink, hardlink, sparse file,
device, socket, FIFO, extended attribute, nontrivial ACL, setuid/setgid/sticky bit or path rejected
by the directory-source path grammar is terminal. Final `deps` regular files are mode `0444`; the
single final `test-build` and `bin-build` executable is mode `0555`; and any descendant directory
is mode `0555`. Each selected subtree root is excluded from its stream and sealed mode `0500`.
The controller stores the final dependency and test streams in
`host-validation-foundation-test-compile`, and the final binary stream in
`host-validation-foundation-bin-compile`. Later host-validation phases recompute the applicable
streams from the filesystem rather than trusting those leaves; the controller may no-follow read
only these exact earlier sealed same-root leaves to require byte identity.

Each `payload-tree.sha256`, `source-tree.sha256`, `expected-prepatch-source-tree.sha256`,
`tool-tree.sha256`, `archive-staging-tree.sha256`, `archive-bundle-tree.sha256` and
`foundation-{deps|test|bin}-tree.sha256` leaf is an exact LF-terminated ASCII record:

```text
schema=c2b2a-tree-digest-v1
kind=<payload|source|expected-prepatch-source|tool|archive-staging|archive-bundle|foundation-deps|foundation-test|foundation-bin>
rows=<decimal-row-count>
bytes=<decimal-byte-count>
sha256=<lowercase-64-hex>
```

It describes the final retained stream for that kind. Counts are unsigned base ten with no leading
zero except zero; row count includes the schema row and every row ending in LF. Angle-bracket and
vertical-bar forms above declare alternatives and are never literal accepted values.

Each containing phase directory becomes mode `0500` immediately after that phase's final evidence
write. Its shared parent `evidence` directory remains mode `0700` until every phase assigned to
that exact evidence root has been created and sealed; only then does that shared parent become mode
`0500`. In particular, the acquisition evidence parent remains writable to the controller until
`pre-runtime-finalize` is sealed. No child receives an evidence descriptor or evidence-path read
permission. Before the phase's policy
probe, the controller creates distinct, previously absent mode-`0700`
`<owning-root>/tmp/<phase>-policy-probe` and `<owning-root>/tmp/<phase>` directories. The first is
the probe's `TMPDIR`; the second is the `TMPDIR` shared only by that phase's ordered substantive
children. The substantive policy denies the probe subtree, and every later phase denies both plus
all other sibling temporary subtrees. The probe subtree is sealed after the probe; the substantive
subtree is sealed after the phase's final child. Neither is ever a later input. Temporary files are
non-authoritative and their names or contents cannot satisfy a gate.

Qualification and host-validation `home` and `cargo-home` are controller-only rotation parents,
not environments themselves. Here qualification means the two qualification provisioner phases,
the four `qualification-{A|B}-{supervisor|collector}` builds and `probe-build-{A|B}`; artifact
audits are excluded and use `/var/empty`. Immediately before every policy-probe or substantive
child in those phase families, the controller requires both exact `active` children to be absent,
creates them
mode `0700`, and verifies each is an empty no-follow directory with the invoking uid/gid, no xattr
and no nontrivial ACL. The effective `HOME` and `CARGO_HOME` name only those two `active`
directories. The phase profile permits child reads and content writes only inside the two active
roots and has overriding denials for entry or metadata mutation of either parent or active-root
literal. It also denies creation or mutation of exact `cargo-home/active/config`,
`cargo-home/active/config.toml`, `cargo-home/active/credentials`,
`cargo-home/active/credentials.toml`, and every entry beneath `home/active/.cargo`; the controller
requires all five surfaces absent before and after the child. Every archive and peer root is
unreadable and unwritable to the child.

After the required EOF drain and complete process-group reaping for each child, and before any
later child, the controller verifies the same active root identities, ownership, mode, xattr and
ACL state, changes each root to mode `0500`, and renames
it without replacement under its same parent. The archive basename is exactly
`<phase>-policy-probe` for the probe or `<phase>-invocation-<ordinal>` for a substantive child,
where the ordinal is unsigned base ten with no leading zero except zero. The next child receives
new inode-distinct empty `active` roots at the same two paths. Archived contents are retained as
diagnostic evidence but are never a later input. After the last applicable child, `active` is
absent; the controller binds the parents' exact archive-name and device/inode inventory and changes
the parents to mode `0500` only after `probe-build-A` or `probe-build-B` for its corresponding
qualification root, and only after `host-validation-lints` for host validation.
At creation and around every child, both parents must retain one exact device/inode, the invoking
uid/gid, mode `0700`, empty xattrs and no nontrivial ACL; their final state differs only by mode
`0500` and the exact accumulated archive inventory.

The applicable invocation's `ambient_roots` array has exactly two objects in kind order `home`,
`cargo-home`. Each has ordered keys `kind`, `active_path`, `archive_path`, `device`, `inode`, `uid`,
`gid`, `pre_mode`, `pre_entries`, `post_mode`, `post_entries`, `archive_mode`. Numeric values are
unsigned; the pre/post root identity and ownership are identical; `pre_mode` and `post_mode` are
`0700`; `pre_entries` is zero; `post_entries` is the exact immediate-child count; and
`archive_mode` is `0500`. The trusted controller records the post state before its chmod/rename and
then verifies the same identity at the archive path. A reused inode, readable archive, surviving
active path, child-writable parent/root metadata, or later read of an archive is terminal. This
rotation, rather than an assumption that shared writable homes remain benign, excludes one
invocation's Cargo configuration or ambient state from every later invocation.

The unprivileged macOS boundary intentionally makes no claim of a complete kernel-trusted per-PID
exec, argv, environment, descriptor or allowed-file-access history. DTrace, Endpoint Security,
`fs_usage`, `ktrace` and audit facilities that require administrator privileges are not inputs. The
authority is instead the exact reviewed controller, exact outer argv, bootstrap and effective
environment, inherited-fd record,
hash-bound Seatbelt policy and executable/read/write/network closure, negative policy probes,
sealed source/tool inputs, pre/post payload/source/tool trees and phase-specific artifact seals.
Process output and best-effort
sandbox-denial logs are corroboration only. A child fact may satisfy a gate only when it is
independently derivable from pinned Cargo/build-script source and the enforced executable closure,
or corroborated by a sealed artifact or two-root result. A fact that genuinely requires complete
historical per-PID observation remains blocked pending a separately reviewed privileged runner; it
cannot be inferred from `-vv`.

Every sandboxed launch has this exact outer argv grammar, with no shell and no profile-file read:

```text
/usr/bin/sandbox-exec -p <exact-rendered-policy-bytes> /usr/bin/env -i \
  <name=value>... <substantive-executable> <substantive-argv>...
```

The rendered UTF-8 policy is one argv element with no NUL and at most 131,072 bytes. The controller
writes identical bytes to the phase's applicable `sandbox.sb` or `policy-probe-sandbox.sb`, fsyncs
and hashes that evidence leaf, and requires byte identity with the in-memory argv element before
launch. The controller passes `sandbox-exec` only the exact six-variable controller bootstrap
environment. The reviewed `/usr/bin/env` runs inside Seatbelt, clears that bootstrap with `-i`,
sets the effective phase environment from one `name=value` argv element per variable in raw UTF-8
name order, then directly execs the substantive executable. Every effective phase environment also
contains the fixed `__CF_USER_TEXT_ENCODING=0x1F6:0x0:0x0`; later environment tables omit it only
to avoid repetition. Names are nonempty ASCII identifiers with no equals sign; names and values
contain no NUL; and each name occurs exactly once. `/usr/bin/sandbox-exec`, `/usr/bin/env`, the
complete outer argv, decoded assignment table and separately decoded substantive argv are bound in
invocation evidence. Every Python, Cargo, metadata, compiler, linker, audit or test command shown
below is a substantive argv. A launch that bypasses either fixed outer executable is terminal.

This post-sandbox environment construction is mandatory because the protected
`/usr/bin/sandbox-exec` strips inherited `DYLD_*` variables on this host. Implementation preflight
must reproduce both sides of the read-only diagnostic: an inherited
`DYLD_FALLBACK_LIBRARY_PATH` is absent in the inner process, while the same exact value supplied as
an `/usr/bin/env -i` assignment is present in the substantive user-path executable. The diagnostic
binds the selected `/usr/bin/env` and `sandbox-exec` identities and exact argv; it is feasibility
evidence, while the reviewed wrapper grammar is the launch authority.

Every substantive Seatbelt phase default-denies `process-exec` and allows exact `/usr/bin/env`
solely as the fixed entry trampoline above. A provisioning phase otherwise allows only the exact
sealed Python entry executable and `provision.py` performs no child exec. Fetch and
metadata phases
allow only the exact sealed Cargo, rustc and rustdoc paths required by their pinned source-defined
queries. An A/B qualification or probe build additionally allows the exact sealed compiler,
linker, archiver, clang/LLVM and audited helper paths plus Cargo-generated `build-script-build`
executables under that root's
exact target directory. The latter is a closed path class derived from the locked target-active
package set, and its realized path/hash/source/fingerprint set is sealed after each command; every
other target-tree executable class is denied. An artifact-audit phase permits process execution of
only its exact sealed cross-readelf, cross-objdump, cross-nm and cross-strings paths. Each
host-validation phase permits only its narrower executable class: payload tests permit the sealed
Cargo, rustc, rustdoc, compiler/linker helpers, locked build scripts and Cargo-generated test
executables required by the ten listed test invocations; each direct compile permits only sealed
rustc plus its exact sealed Apple linker/compiler helper chain; each direct run permits only its
one already sealed generated executable; and lints permit the sealed Cargo, cargo-clippy,
clippy-driver, rustc, compiler/linker helpers, locked build scripts, cargo-fmt and rustfmt required
by the eight listed commands. A host-validation phase denies every executable assigned only to a
different host-validation phase. No substantive phase allows an unlisted shell or
interpreter; its sole interpreter exception is the exact sealed Python entry for a provisioning
phase, while the internal policy probe is the separately bounded auxiliary exception below. Every
Cargo/build phase denies Python. No phase allows an unlisted package manager or external
downloader. Only the substantive acquisition-fetch sandbox and its exact process group receive a
network allow. Seatbelt cannot condition that allow on executable identity, so the permitted Cargo
query children inherit the same address capability; pinned Cargo/rustc source, the executable
closure and sealed outputs must establish that those query forms do not use it. No phase allows an
executable from the repository, Cargo home, ambient user paths or the peer root. The policy's exact
allow rows and every
realized executable identity require preflight or build/source identity review as applicable.

Immediately before the first sandboxed substantive child of each phase, the controller validates
that phase's exact Seatbelt policy with one negative-probe child. A phase with multiple
controller-launched children, including a probe-build phase, performs only this one probe and must
reverify the identical substantive-policy bytes before every later child. Both the substantive and
probe policies are rendered
independently from reviewed closed templates. For a provisioning phase, the probe retains the
substantive policy's existing sealed-Python permission and differs only by substituting reviewed
`controller.py` for `provision.py` in the script-read rule and substituting the probe's distinct
temporary subtree for the substantive phase's temporary subtree. For every non-Python phase, the
probe instead adds the exact
sealed Python executable and runtime-read closure, reviewed `controller.py`, and that probe-temp
permission while removing the substantive phase-temp permission. The acquisition-fetch probe also
removes every DNS and network allow from the substantive fetch policy. The controller parses and
compares the two generated rule sets and records exactly the applicable class-specific delta; an
extra allow rule is terminal. It invokes the probe without a shell as:

```text
<exact-python> -I -B -S <exact-controller.py> policy-probe --phase <closed-phase-id> \
  --canary-root <exact-path> --tcp-host 127.0.0.1 --tcp-port <decimal-port> \
  --forbidden-exec /usr/bin/true [--denied-read <exact-path>]...
```

The probe environment is built from empty and contains only `LANG=C`, `LC_ALL=C`, `TZ=UTC`,
`SOURCE_DATE_EPOCH=0`, `__CF_USER_TEXT_ENCODING=0x1F6:0x0:0x0`, the owning root's exact `HOME`, and
`TMPDIR=<owning-root>/tmp/<phase>-policy-probe`; every other variable, including `PATH` and every
`PYTHON*` variable, is absent. The descriptor, pipe, byte-ceiling, deadline, process-group and reap
rules are the same as for any other controller child, with a 60-second deadline. For a
qualification or host-validation probe, `HOME` is that family's current `home/active`; otherwise it
is the already named owning-root `home` directory.

The optional `--denied-read` values are emitted in raw UTF-8 path order from the phase template and
may name only the existing repository, peer root, evidence root, sibling temp roots and other
phase-specific denied read surfaces. They are non-destructive read probes; no actual payload,
source, configuration, repository or peer-root path is ever used for a destructive probe.

Before the probe, the controller creates a phase-specific owner-only canary directory at exact
`<owning-root>/policy-canaries/<phase>`. It contains fixed relative entries named `read`, `write`,
`truncate`, `chmod`, `unlink`, `rename-source`, `rename-destination`, `hardlink-source`,
`create-parent` and `directory-mode`. The last two are empty mode-`0700` directories; every other
entry is a distinct mode-`0600`, link-count-one regular file containing exactly the 23 ASCII bytes
`c2b2a-policy-canary-v1\n`. Paths `hardlink-destination` and `symlink-destination` are absent. Thus
ordinary owner permissions permit every attempted operation,
while the probe policy denies the complete canary root. The controller seals its entry set, modes,
inodes, bytes and hashes before launch.

The controller also holds an active loopback TCP listener at the recorded ephemeral port; its
descriptor is not inherited. In fixed order, the probe first exercises successful create, read,
write, truncate, chmod, rename, unlink, hardlink, symlink and directory-mode operations on distinct
fixtures created solely beneath its own probe-temp root, then removes every fixture and proves that
root empty. It next must observe `EACCES` or `EPERM`, without retry, for the corresponding denied
read, existing-file write, truncate, chmod, unlink, rename-over, hardlink, symlink, create-new and
directory-mode operations against the exact canary entries. It then requires the same denial for
`/usr/bin/true` `execve`, the loopback connect and every ordered `--denied-read` value. An
unexpectedly successful exec exits zero and is terminal.

From outside the sandbox, the controller proves the complete canary tree remained byte-, metadata-,
inode- and entry-identical. It remains sealed, is included in terminal root accounting, is never a
later child input and cannot satisfy an acceptance gate. The canonical probe result's
`observations` value is an array with one ordered object for every positive and negative case. Each
object has exactly the ordered keys `operation`, `target_class`, `expected_result`,
`observed_result` and `errno`; `errno` is zero on success and the observed unsigned `EACCES` or
`EPERM` value on a required denial. A missing, duplicate, reordered or extra case is terminal. The
closed operation/target/result strings and phase-to-denied-read expansion are
implementation-preflight inputs. Static policy comparison, not a destructive attempt against
authoritative inputs, remains the authority for the path-specific write denials.

The controller writes the probe policy, exact invocation, framed raw streams and canonical result
only to the five `policy-probe-*` evidence leaves for that substantive phase. Each probe stream has
exactly one frame under the substantive-stream encoding, and `policy-probe-invocation.json` is a
one-element array under the same invocation schema. A substantive launch is
forbidden unless every mandatory positive and negative result passes. The controller then launches
with the separately precomputed substantive-policy bytes and re-verifies their runtime evidence
hash; it never derives the substantive policy by deleting probe rules. Preflight binds the renderer,
templates and substitution grammar, while phase evidence binds the actual canonical root/path
substitutions and rendered bytes. Static comparison proves that
Python and `/usr/bin/true` remain denied by Cargo, build and metadata policies; the dynamic probe is
corroboration of
the remaining default-deny behavior, not authority to widen a policy.

The implementation-preflight record cannot be read by Cargo, compilers, build scripts or probes and
is not a payload input.

## Canonical tool and system-input seal

The implementation-preflight review freezes a finite, non-overlapping set of canonical absolute
tool/input roots before the first controller launch. It must cover the complete Rust 1.93.0
toolchain and AArch64-musl target, cross compiler/binutils/sysroot/headers, LLVM/libclang, the exact
Python interpreter/framework/standard library and every other non-system helper or dynamic library
that an allowed executable can resolve. A separate exact list covers only fixed Apple
system-runtime, certificate, DNS and Seatbelt inputs that are not user-writable. No later command
may expand either list; a missing input blocks the run and requires a newly reviewed preflight.

The controller computes the tool stream before and after every child invocation. It covers each
declared root itself and excludes no descendant. It begins with the LF-terminated row
`schema=c2b2a-tool-closure-tree-v1`. Root declarations use this row:

```text
root<TAB>actual-mode<TAB>decimal-device<TAB>decimal-inode<TAB>decimal-uid<TAB>decimal-gid<TAB>label<TAB>canonical-absolute-path
```

They are sorted by raw UTF-8 label then path. Each root's
extended-attribute rows immediately follow its declaration in raw attribute-name-byte order. All
declared-root descendants follow, sorted by raw UTF-8 bytes of `label/relative-path`; each
descendant's extended-attribute rows immediately follow its primary row in raw attribute-name-byte
order. System primary rows come last, sorted by raw UTF-8 canonical absolute path, with each
system object's attribute rows immediately following it in the same name order. Rows use exactly
one of:

```text
root<TAB>actual-mode<TAB>decimal-device<TAB>decimal-inode<TAB>decimal-uid<TAB>decimal-gid<TAB>label<TAB>canonical-absolute-path
directory<TAB>actual-mode<TAB>label/relative-path
file<TAB>actual-mode<TAB>decimal-size<TAB>sha256<TAB>label/relative-path
hardlink-file<TAB>actual-mode<TAB>decimal-size<TAB>sha256<TAB>decimal-members<TAB>group-sha256<TAB>label/relative-path
symlink<TAB>decimal-target-bytes<TAB>target-sha256<TAB>lowercase-target-hex<TAB>label/relative-path
target-absolute-symlink<TAB>decimal-target-bytes<TAB>target-sha256<TAB>lowercase-target-hex<TAB>label/relative-path
root-xattr<TAB>decimal-name-bytes<TAB>name-sha256<TAB>lowercase-name-hex<TAB>decimal-value-bytes<TAB>value-sha256<TAB>lowercase-value-hex-or-dash<TAB>label
xattr<TAB>decimal-name-bytes<TAB>name-sha256<TAB>lowercase-name-hex<TAB>decimal-value-bytes<TAB>value-sha256<TAB>lowercase-value-hex-or-dash<TAB>label/relative-path
system-directory<TAB>actual-mode<TAB>canonical-absolute-path
system-file<TAB>actual-mode<TAB>decimal-size<TAB>sha256<TAB>canonical-absolute-path
system-xattr<TAB>decimal-name-bytes<TAB>name-sha256<TAB>lowercase-name-hex<TAB>decimal-value-bytes<TAB>value-sha256<TAB>lowercase-value-hex-or-dash<TAB>canonical-absolute-path
```

Labels, canonical root paths, relative paths and system paths are nonempty UTF-8 with no tab, LF,
CR, NUL, other C0 byte or DEL. Labels are globally unique. Canonical root paths are globally
unique, no-follow directories and pairwise nonoverlapping. System paths are globally unique; none
equals or falls beneath a declared non-system root and no declared non-system root falls beneath a
system path. An ancestor relationship between two system paths is accepted only when the ancestor
is an explicitly required traversal directory and that exact relationship is frozen by preflight.
`actual-mode` is the exact four-octal-digit `st_mode & 07777` value. Decimal integers are unsigned
base ten with no leading zero except zero; hash encodings follow the directory-source grammar.

A `symlink` target is hashed and hex-encoded from its exact filesystem bytes, must be relative,
acyclic and resolve inside the same declared root. There is exactly one permitted
`target-absolute-symlink`: it belongs to the preflight-bound AArch64-musl cross-sysroot root, its
relative path is exactly `lib/ld-musl-aarch64.so.1`, and its 12 target bytes are exactly
`/lib/libc.so` (SHA-256
`f769c070866e6abea14c41ca09fcb6abbcee32a21980539a380de81fbeb9f013`, lowercase hex
`2f6c69622f6c6962632e736f`). For target-namespace closure only, the leading slash is stripped and
the target maps to the same root's `lib/libc.so`, which must have its own regular-file primary row.
The controller reads the link itself no-follow, never resolves it as host `/lib/libc.so`, and no
host child receives a path whose host resolution traverses that link. This exception preserves
required Linux target-image metadata; it grants no host read or execution permission. Every other
absolute, broken, cyclic or escaping link is terminal, and the named exception is terminal if its
raw bytes, mapped member or pre/post identity differs. A regular
file with link count one uses `file`. A regular file with link count greater than one uses
`hardlink-file` and is accepted only when an exhaustive device/inode scan of all declared
non-system roots finds exactly `st_nlink` member paths, every member is beneath those roots and
every member has identical mode, size and bytes. The canonical group stream begins with
`schema=c2b2a-hardlink-group-v1\n`, then has one `path<TAB>label/relative-path\n` row per member in
raw UTF-8 path order. `decimal-members` is that row count and `group-sha256` hashes the complete
group stream; each member row carries the same two values. A link outside the declared roots,
cross-device inconsistency, unaccounted member or pre/post topology change is terminal. Devices,
sockets, FIFOs, sparse files, setuid/setgid/sticky bits and nontrivial ACLs are terminal in
non-system roots.

Extended-attribute names are nonempty raw byte strings of at most 255 bytes with no NUL. Their
lowercase hex and SHA-256 encode those exact bytes; names need not be UTF-8. Attribute values are
arbitrary byte strings of at most 16,777,216 bytes. Their lowercase hex field is `-` exactly when
the value is empty and otherwise encodes every value byte; its decimal length and SHA-256 must
agree. The complete closure may contain at most 1,048,576 emitted `root-xattr`, `xattr` and
`system-xattr` row occurrences and 1,073,741,824 checked aggregate value bytes, summing
`decimal-value-bytes` once for every emitted row occurrence, including every repeated hardlink
member row. The complete canonical tool stream may contain at most 4,294,967,296 bytes. All row,
value and stream arithmetic is checked before allocation or encoding. The controller enumerates
and reads every attribute with no-follow
semantics against the same verified filesystem-object identity used for its primary row. An
unreadable value, duplicate name, overflow or mismatch between enumeration and read is terminal.
No attribute is accepted merely by name: implementation preflight freezes the complete canonical
attribute row set and exact values, including any observed `com.apple.provenance` rows. Absence is
represented by zero attribute rows for that object. A later missing, added, changed or reordered
row is terminal; no attribute is removed, normalized or synthesized.

A system-file
row must be a regular file and a system-directory row must be a directory. Each system path must
be root-owned, not writable by the invoking user or group, resolve to the reviewed canonical path,
and match its preflight and pre/post metadata or content seal. A writable or changed system row is
terminal; placement on the signed system volume is not assumed.

The final row is LF-terminated. The stream's row count, byte count and SHA-256 are identity values.
Every pre/post stream must be byte-identical. This is a content and topology seal, not a claim that
same-owner installed tools are filesystem-immutable; Seatbelt denies child writes, and any observed
concurrent or pre/post mutation is terminal. The controller itself runs under the exact
preflight-bound Python closure, which is part of every pre/post seal. No tool root is copied,
normalized or modified by this repair.

## Exact archive acquisition phase

Acquisition is separate from qualification building. Before any acquisition root or file is
created, the controller sets and records process umask `0077`. It creates one new owner-only
acquisition root and new `home`, `cargo-home`, `target`, `tmp`, `policy-canaries`, `seed-payload`,
`lock-root` and `evidence` directories. Within `lock-root` it creates new `home`, `cargo-home`,
`target`, `tmp`, `policy-canaries` and `evidence` directories. The provisioner-output
leaves—`archive-staging`, `manifest-authority`, `sealed-archives`, `lock-root/.cargo`,
`lock-root/payload` and `lock-root/cargo-source`—must remain
absent until their named create operation below. Controller output `host-validation` must likewise
remain absent until its named phase. The acquisition root and every created directory beneath it are
verified mode `0700`, contain no symlink and must not exist before the witnessed create operation.
Only the explicit sealing transitions below may later make a directory read-only. A create-new
directory and regular-file probe must observe modes `0700` and `0600` before removal.
Cargo-generated entries must remain owner-only under the recorded umask.

The canonical create-new acquisition-root path must match exactly:

```text
^/private/tmp/engram-c2b2a-acquisition-[A-Za-z0-9._-]+$
```

It contains no whitespace, control byte, colon or equals sign, every component is opened
no-follow, and the nested lock root and every acquisition path remain beneath it.

After this design and the implementation-preflight review pass and before the manifest edit, the
controller creates `seed-payload/Cargo.toml` and `seed-payload/Cargo.lock` as create-new byte copies
of the exact
pre-repair inputs bound above. Each is regular, link-count one and inode-distinct from its source;
their bytes and hashes are reverified, their modes become `0400`, the directory mode becomes `0500`
and all three mtimes become Unix epoch zero. The already reviewed hand-authored support files must
remain unchanged. The prescribed manifest, payload-configuration and dependency-test edits may then
be installed under the active repository editing rules; the bound patch remains unchanged, the
repository lock remains at its pre-repair bytes and `archive-manifest-v1.tsv` remains absent.
Acquisition generates that manifest and lock derivation generates the only candidate lock. This
explicit order prevents either generated file from being guessed or treated as authority before it
exists.

The seed predates the alias edit and avoids claiming that a not-yet-generated candidate lock is
already frozen. With an acquisition environment built from empty, the exact absolute Cargo 1.93.0
binary from a later hash-bound read-only toolchain runs exactly once:

```text
<sealed-toolchain>/bin/cargo fetch -vv --locked --manifest-path seed-payload/Cargo.toml
```

The command has no `--target`, so it fetches the whole pre-repair lockfile rather than a host- or
Linux-filtered subset. Its cwd is the acquisition root. `HOME` and `CARGO_HOME` name the fresh
directories in that root; `CARGO_TARGET_DIR` names its fresh `target`; and `TMPDIR` is exactly
`<acquisition-root>/tmp/acquisition-fetch`. The remaining exact entries are
`CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse`, `LANG=C`, `LC_ALL=C`, `TZ=UTC`,
`SOURCE_DATE_EPOCH=0`, `RUST_BACKTRACE=0`, `CARGO_INCREMENTAL=0`,
`CARGO_CACHE_RUSTC_INFO=0`, `CARGO_BUILD_JOBS=1`, the exact absolute `PATH`, and exact absolute
`RUSTC` and `RUSTDOC` executables inside the same sealed toolchain. These named entries are the
complete environment. The toolchain tree, `PATH` and
executable closure are later bound in the build/source identity record. Nothing resolves through
ambient `/Users/yuval.meiri/.cargo` or a rustup proxy, and no `+toolchain` argv is used.
Every unlisted variable is absent. Cargo 1.93.0, rustc 1.93.0, their raw version outputs and
executable SHA-256 values are bound before the command.

Every controller child launch—including acquisition fetch, a provisioner mode, either lock-root
metadata command, an A/B Cargo build or an A/B probe cross-build—starts with an exact inherited
descriptor table: fd 0 is a read-only descriptor for `/dev/null`; fds 1 and 2 are distinct
nonseekable one-way pipes to a trusted out-of-sandbox collector; and every fd greater than 2 is
closed before the sandbox wrapper executes. The collector alone creates the phase's mode-`0600`
`stdout.bin` and `stderr.bin` evidence leaves with create-new descriptors and never exposes those
descriptors to a child. Each leaf is a canonical sequence of one frame per substantive
controller-launched child in invocation order: an unsigned eight-byte big-endian raw-stream length
followed by exactly that many stream bytes, including a zero-length frame when applicable.
`invocation.json` is a canonical JSON array with one object per frame pair. Each object has exactly
these ordered keys: `ordinal`, `argv`, `outer_environment`, `environment`, `cwd`,
`sandbox_sha256`, `pre_trees`, `ambient_roots`,
`started_monotonic_ns`, `ended_monotonic_ns`, `wait_status`, `stdout_bytes`, `stdout_sha256`,
`stderr_bytes`, `stderr_sha256` and `post_trees`. `ordinal` is a zero-based unsigned integer; `argv`
is the complete exact nonempty outer string array. `outer_environment` is the six-variable
controller bootstrap table supplied to `sandbox-exec`; `environment` is the effective table
decoded from `/usr/bin/env -i` assignment argv. Both are arrays of objects with ordered keys
`name`, `value`, sorted by raw UTF-8 name bytes. `pre_trees` and `post_trees` are arrays in this
closed kind order: `payload`, `source`, `expected-prepatch-source`, `tool`, `archive-staging`,
`archive-bundle`, `foundation-deps`, `foundation-test`, `foundation-bin`. Their objects have
ordered keys `kind`, `rows`, `bytes`, `sha256`; an inapplicable set is an empty array.
`ambient_roots` uses the closed rotation schema below and is empty for inapplicable phases.
`wait_status` is exactly either an object with ordered keys `kind`, `code`, where
`kind` is `exit`, or one with ordered keys `kind`, `signal`, `core_dumped`, where `kind` is
`signal`; all numeric values are unsigned. The collector
drains each pair through EOF, finalizes its frame lengths before the evidence leaves are fsynced and
closed, then hashes the completed leaves.
The controller verifies its outer table, ordered effective-environment assignments and full argv
immediately before spawn, and implementation preflight binds
the exact Python/OS `dup`, close-on-exec, empty-pass-fd and process-group launch semantics. It does
not claim an injected child-side descriptor verifier. Any descriptor alias, inherited
directory/archive/source/tool descriptor, socket, terminal, lock or other writable fd is terminal.
The design does not claim a complete history of descriptors created after exec.
Pinned Cargo source defines the only permitted jobserver behavior; an ambient jobserver or
`CARGO_MAKEFLAGS` input is forbidden, pipe EOF and process-group reaping must show that no launched
descendant remains, and any gate requiring more exact descriptor history stays blocked.
Process-originated transcript text remains untrusted and cannot alone prove an argv, environment,
file access or success. Such a fact requires the independently enforced and sealed evidence classes
defined in the controller boundary above.

Each pipe has a checked 268,435,456-byte ceiling and the two streams have a checked
536,870,912-byte combined ceiling per invocation. Acquisition fetch has a 3,600-second deadline,
each lock-root metadata command 900 seconds, each provisioner mode 3,600 seconds, each Cargo build
7,200 seconds and each A/B probe cross-build 3,600 seconds.
The trusted controller launches a new process group; on a byte overflow or deadline it sends
`SIGTERM`, waits at most five seconds, sends `SIGKILL` to the remaining group, drains/closes both
pipes, reaps every descendant it owns and records a terminal failed invocation. No partial output or
artifact from that invocation may be reused or replayed.

This fetch is the sole network-permitted Cargo command. It compiles no package, runs no build script
and produces no acceptance evidence. Its profile allows Cargo to execute only the exact pinned
rustc for target, version and capability queries. The permitted query forms are derived from pinned
Cargo source and corroborated by the bounded transcript; no complete per-PID argv/environment
history is claimed. If another executable or query-dependent artifact is required, the gate blocks.
The initial logical request origins are exactly `https://index.crates.io:443` and
`https://static.crates.io:443`. Before launch, the trusted controller binds
the configured DNS resolver endpoints and complete resolved A/AAAA address set for those names. The
sandbox permits only those resolver endpoints and resolved addresses on TCP 443 and denies every
other network destination. Exact Cargo/TLS hostname verification and the bound certificate closure
remain mandatory for every HTTPS connection. Seatbelt's address gate is not URL/content authority:
the registry source and
lockfile archive checksums are the final byte authority. Its write profile permits only acquisition
`home`, `cargo-home`, `target` and `tmp/acquisition-fetch`; it denies pathname writes to `evidence`
and content and metadata writes to `seed-payload` and every other acquisition entry. Its read
profile permits only the seed files, exact sealed toolchain, later-bound system runtime/certificate
closure and its writable subtrees. The exact sandbox profile,
endpoint-resolution evidence, read set and command transcript require the later build/source
identity record and review. Pinned Cargo enables HTTPS redirect following; this unprivileged freeze
does not claim a complete redirect chain or make redirect absence an identity fact. A redirect that
requires an unlisted destination address is denied, while any redirect that remains within the
address gate is non-authoritative. No credential, token, proxy secret or provider value exists in
the process, and only a byte stream matching the exact lockfile archive checksum may enter the
sealed bundle. This design does not claim that an unobserved redirect retained the initial scheme
or logical origin. An alternate registry source, git dependency, credential request, unaccounted
read or write outside an allowed subtree is terminal.

That redirect behavior is bound to Cargo commit `083ac5135f967fd9dc906ab057a2315861c7a80d`,
`src/cargo/sources/registry/http_remote.rs`: 35,126 bytes, 900 LF-terminated lines and SHA-256
`0c5bf92429b7fcb5c47d4ec29f4d09dfb9702702de7325f042ce125a79ddbd9f`; its line 634 calls
`follow_location(true)`. The later tool/source identity must bind those exact source bytes. This is
an explicit limitation, not evidence that a redirect did or did not occur.

After fetch, acquisition-stage mode runs exactly once and derives the whole-lock archive set from
the exact seed lockfile. It copies each selected `.crate` file byte-for-byte from the fresh
acquisition `CARGO_HOME` into
`archive-staging` using create-new destination files, never rename, clonefile, reflink or hardlink.
Each destination must be a regular file with link count one and a distinct device/inode tuple from
its source and every peer. The cache directory and source archive are each opened once with
directory-relative no-follow descriptors; copying, hashing and pre/post identity checks use that
same source descriptor and one create-new destination descriptor, never a pathname reopen. It
verifies the registry source, exact single-component archive basename formed as
`<lock-name>-<lock-version>.crate`, byte length and SHA-256 against the seed lock and generates the
sole provisional manifest at exact path `manifest-authority/archive-manifest-v1.tsv`. The basename
may contain no `/`; the cache lookup opens that one component relative to the already-open cache
directory. `manifest-authority` and its file are created mode `0700`/`0600`; the file is fsynced,
closed, re-opened once with no-follow, rehashed, normalized to mode `0400` and epoch-zero mtime, and
never mutated. Its directory becomes mode `0500` with epoch-zero mtime. Every later use permits
read-only access to this exact non-evidence path and verifies the same descriptor identity and
digest; no child may read any other acquisition-root sibling outside its phase closure.
After the complete set and manifest compare, staging files become mode `0400`, its directory becomes
mode `0500` and mtimes become Unix epoch zero. After the provisioner exits, the controller
independently constructs the canonical staging-tree stream and SHA-256 and writes only the two
named acquisition-stage evidence leaves. That stream excludes the staging root and begins with the
LF-terminated row
`schema=c2b2a-archive-staging-tree-v1` and then uses the directory/file grammar, decimal-size
encoding and raw-UTF-8 ordering defined for the directory-source tree below. No later step may
mutate or add a staging entry.

Lock-payload mode runs exactly once, creating the independent candidate copy with the still-
pre-repair lock and injecting the byte-exact provisional manifest at its one authorized support
path. Lock-source mode then runs exactly once to create the independent directory source from that
manifest and staging set. The checker proves every unchanged path against the complete frozen
pre-repair stream and allows only the five changed-file and seven new-structural-path deltas above,
plus the exact root Cargo configuration below. It verifies that candidate delta before Cargo runs.
With network denied and the lock environment and write-confinement rules below, cwd
`lock-root/payload` executes exactly:

```text
<sealed-toolchain>/bin/cargo metadata --offline --no-default-features --features collector \
  --filter-platform aarch64-unknown-linux-musl --format-version 1
```

That metadata command starts from an empty environment containing exactly `LANG=C`, `LC_ALL=C`,
`TZ=UTC`, `SOURCE_DATE_EPOCH=0`, `RUST_BACKTRACE=0`, `CARGO_INCREMENTAL=0`,
`CARGO_CACHE_RUSTC_INFO=0`, `CARGO_BUILD_JOBS=1`, the exact sealed `PATH`, exact absolute `RUSTC`
and `RUSTDOC`, and `HOME`, `CARGO_HOME` and `CARGO_TARGET_DIR` naming the fresh directories beneath
`lock-root`; `TMPDIR` is exactly `<lock-root>/tmp/lock-derive`. `ROCKSDB_COMPILE`, encoded Rust
flags and C/C++ flags are absent because metadata performs no package compilation. Every unlisted
launch variable is absent.
The lock profile denies network and all writes by default, then permits only `home`, `cargo-home`,
`target`, `tmp/lock-derive` and an existing exact `payload/Cargo.lock`. For that file it permits the
read/write open, exclusive lock, truncation and content write required by the pinned implementation.
Although Cargo's open helper is create-capable, `Cargo.lock` is precreated and verified while its
parent is mode `0555`; the sandbox denies creation, rename, unlink, link, symlink, chmod and
directory-metadata changes throughout `payload`. It denies every other payload, directory-source
and root-config content or metadata write. The enforced profile plus independently computed pre/post
directory-entry and tree seals must prove that no temporary or other payload entry, rename or
unlink occurred and that the sole content delta is `Cargo.lock`; transcript or denial text is only
corroboration. It default-denies reads except the exact payload, exact
`lock-root/.cargo/config.toml`, directory source, sealed toolchain, later-bound system runtime
closure and its writable subtrees. Negative probes cover the denied surfaces before Cargo runs.

This is the only Cargo command allowed to update a lockfile. It compiles no package and invokes no
build script, but Cargo 1.93.0 may run exact pinned `rustc -vV`, host `rustc --print=...` and target
`rustc --target aarch64-unknown-linux-musl --print=...` capability queries. Their allowed forms and
environment constraints are derived from pinned Cargo source, bounded by the executable/read
profile and corroborated by captured output; exact executable identity is retained.
The resulting exact Cargo-produced lock bytes must differ only by the root direct edge described
above. Its registry package tuples must be identical to the seed lock. The trusted checker
deterministically reconstructs the candidate manifest bytes in memory by joining those final lock
tuples and checksums to the sizes reverified from the same sealed staging archive descriptors. It
hashes the reconstruction and requires byte identity with the sealed provisional manifest. It
creates no post-lock manifest file; equality designates the already sealed provisional bytes as the
final manifest. Any other delta, manifest difference or archive not already covered by the
whole-lock fetch is terminal.

After that comparison passes, the checker normalizes the existing `Cargo.lock` to mode `0444` and
epoch-zero mtime, computes the final lock-root payload stream, and then executes exactly once from
the same cwd and exact from-empty environment, with only the phase-specific TMPDIR substitution:

```text
<sealed-toolchain>/bin/cargo metadata --locked --offline --no-default-features \
  --features collector --filter-platform aarch64-unknown-linux-musl --format-version 1
```

For this second command only, `TMPDIR` is `<lock-root>/tmp/graph-metadata`. This graph-metadata
profile retains the same read closure and mutable `home`, `cargo-home`, `target` and that exact
phase-specific temporary subtree but denies every content, metadata and directory-entry write to
`payload`, `cargo-source` and the root configuration. It must exit zero without changing the final
lock-root payload or directory-source stream. Its bounded stdout is parsed, structurally checked
against the final lock and sealed directory source, and retained with its stderr and invocation
evidence as the
sealed node-and-edge input. Graph v2 is materialized only after the later bundle, repository and
A/B seal values exist. The earlier lock-derivation output cannot satisfy the graph gate. Both
metadata commands run no build script or package compilation; any additional lock-root Cargo or
metadata command is terminal.

Only after the graph-metadata command and its immutability checks pass does archive-seal mode create
`sealed-archives` from the final, designated provisional manifest descriptor and independently
copied staging archives. Neither the
archive-seal mode nor either build root regenerates the manifest. The manifest and every archive
are create-new byte-stream copies, never a rename, hardlink, clonefile or reflink. Every source and
destination must be regular and link-count one, and each destination must have a device/inode tuple
distinct from its source and every peer. Bytes, lengths and hashes are reverified before sealing.

Only after the complete bundle seal passes are the exact Cargo-produced lock and that final manifest
transferred to their bounded repository paths under the active repository editing rules; this is
the repository publication point, not a Git commit. The already-installed payload `Cargo.toml`,
payload configuration, dependency test, patch and hand-authored support files must still match their
pre-command hashes. The provisional/final, in-memory reconstructed, bundle, repository and later
A/B payload manifest representations must be byte-identical and have one SHA-256; only the
in-memory reconstruction is not a file copy.

The sealed bundle contains only the manifest and its selected archive files. Directories become
mode `0500`; regular files become mode `0400`; mtimes become Unix epoch zero. Its canonical stream
excludes the bundle root itself and begins `schema=c2b2a-archive-bundle-tree-v1`, followed by the
same directory/file row grammar, decimal-size encoding and raw-UTF-8 path ordering used for the
directory-source tree below. The final row is LF-terminated; its SHA-256 is the bundle seal. The
acquisition cache, index and transcript remain research evidence, never source authority.
Qualification-root provisioning uses the exact support inputs and dedicated sealed bundle; all
qualification builds use only its provisioned source. Both have network denied and never read the
acquisition Cargo home or ambient `/Users/yuval.meiri/.cargo`.

## Archive manifest

`archive-manifest-v1.tsv` is LF-terminated UTF-8 with no BOM, empty field or duplicate row. No
field contains a C0 byte `0x00`–`0x1f`, DEL `0x7f` or backslash; tab and LF exist only as the one
field separator and row terminator defined here. Its first row is exactly:

```text
schema<TAB>c2b2a-archive-manifest-v1
```

Every remaining row has these seven tab-separated fields:

```text
package<TAB>name<TAB>version<TAB>registry-source<TAB>archive-basename<TAB>size<TAB>sha256
```

Rows use the exact Cargo package name and version, the exact registry source string, decimal bytes
without leading zeroes, and lowercase 64-hex SHA-256. They are sorted by raw UTF-8 bytes over
`name`, `version` and `registry-source`. There is exactly one row for every registry package in the
post-edit standalone lockfile. Each SHA-256 equals that package's lockfile registry checksum.
`archive-basename` is exactly the single UTF-8 component obtained by concatenating that row's
`name`, `-`, `version` and `.crate`; it contains no `/` and is never interpreted as a path.

The provisioner accepts one explicit owner-controlled archive directory. It performs no search,
download, index update, subprocess package discovery or network operation. Each expected archive
is opened once relative to an already-open directory descriptor with create-safe/no-follow flags.
That one descriptor must identify a regular, single-link file; its basename, byte length and
SHA-256 must match the manifest. The descriptor is rewound and passed directly to `tarfile`; the
archive is never reopened by pathname. Pre-hash and post-extraction `fstat` device, inode, mode,
link count, size, mtime and ctime must match. Missing archives, extra selected archives, hardlinks,
symlinks, nonregular files, changed files or duplicate identities are terminal.

The bounded archive-source limits are:

```text
manifest bytes                 1,048,576
manifest package rows              1,024
one compressed archive bytes     268,435,456
all compressed archive bytes   4,294,967,296
one archive members                200,000
all archive members              2,000,000
all realized source entries      2,100,000
one expanded regular file bytes   536,870,912
one expanded archive bytes      4,294,967,296
all expanded archive bytes     17,179,869,184
path components                       128
UTF-8 path bytes                      4,096
```

Every sum and counter uses checked arithmetic. The realized-source-entry count includes explicit
members, create-new implicit parent directories and generated checksum files across the complete
directory source. A limit equality is allowed; an exceedance or overflow is terminal.

## Safe extraction and directory-source seal

`provision.py` uses only Python's standard library and never calls a shell, Cargo, tar, patch, git,
curl or another subprocess while sealing or extracting archives. A trusted launcher sets umask
`0077` and invokes it directly as:

```text
<exact-python> -I -B -S <exact-provision.py> <exact-reviewed-argv>
```

The executable, raw version output, framework/stdlib tree, imported-module hashes, argv, exact
environment and umask probe are bound. Every provisioner environment is built from empty and has
exactly these seven entries after phase substitution:

```text
LANG=C
LC_ALL=C
TZ=UTC
SOURCE_DATE_EPOCH=0
__CF_USER_TEXT_ENCODING=0x1F6:0x0:0x0
HOME=<owning-root>/home
TMPDIR=<owning-root>/tmp/<phase>
```

`<owning-root>` is the acquisition root for acquisition-stage and archive-seal, the nested
lock-root for lock-payload and lock-source, and the selected qualification root for either
qualification mode. For either qualification mode, the effective `HOME` value is instead the
current `<owning-root>/home/active` rotation root. `PYTHONHOME`, `PYTHONPATH`, `PYTHONUSERBASE`,
user-site initialization, `PATH`
and every other variable are absent; output order may not depend on hash-table iteration. The
provisioning sandbox denies network and has exactly six mutually exclusive profiles:

1. **Acquisition-stage** reads only the exact seed lock, the explicit fresh Cargo cache directory,
   selected archive descriptors, support/Python/system closure and fresh destination. It creates
   only `archive-staging` and `manifest-authority`; its sole scratch is the controller-precreated
   empty `tmp/acquisition-stage`.
2. **Lock-payload** reads only the exact repository candidate, frozen pre-repair stream,
   exact manifest-authority file, root configuration and support/Python/system closure. It creates
   only `lock-root/payload` and `lock-root/.cargo/`, injects the provisional manifest at its
   authorized support path, and creates
   `lock-root/.cargo/config.toml` from the exact support bytes.
   Its sole scratch is the controller-precreated empty `lock-root/tmp/lock-payload`.
3. **Lock-source** reads only the read-only staging set, byte-exact manifest-authority file, patch,
   support/Python/system closure and fresh destination. It creates only `lock-root/cargo-source`;
   its sole scratch is the controller-precreated empty `lock-root/tmp/lock-source`.
4. **Archive-seal** reads only the read-only staging set, final manifest-authority file and
   support/Python/system closure. It creates only `sealed-archives`; its sole scratch is the
   controller-precreated empty `tmp/archive-seal`.
5. **Qualification-payload** reads only the exact final repository candidate, frozen pre-repair
   stream, root configuration and support/Python/system closure. It creates only one selected A/B
   `payload`, that root's `.cargo/` and `.cargo/config.toml` from the exact support bytes. Its sole
   scratch is the matching controller-precreated empty `tmp/qualification-{A|B}-payload`.
6. **Qualification-source** reads only the sealed archive bundle, final manifest, exact patch and
   support/Python/system closure. It creates only one selected A/B `cargo-source`; its sole scratch
   is the matching controller-precreated empty `tmp/qualification-{A|B}-source`.

One invocation selects exactly one profile and may create only that profile's enumerated durable
outputs and scratch descendants; it cannot create an output assigned to another profile. Every
named durable output leaf must be absent, while the exact phase TMPDIR must exist and be empty. The
provisioner creates durable directories initially mode `0700` and regular files mode `0600`, and
fails if an output exists or an exact pre-opened ancestor is not the expected bound directory. No
ancestor or output may be a symlink. The later per-output sealing rules set final modes and mtimes.
The same descriptor, no-follow, hash and pre/post identity rules apply to fresh-cache, staging and
sealed archive inputs. A failed or partial mode invalidates its complete parent root, which is
never repaired or reused.

Before each lock-source or qualification-source provisioner launch, the controller independently
reconstructs the complete expected prepatch directory-source stream directly from the authenticated
archive descriptors and exact manifest. It repeats all member-type, path, collision, count, size,
mode and checksum validation below; hashes every regular member; constructs the upstream
`.cargo-checksum.json` bytes; and emits canonical normalized rows without materializing or mutating
a source tree. It writes that stream only as the phase's
`expected-prepatch-source-tree.tsv` plus matching digest record and binds its row count, byte count
and digest in the probe/substantive invocation evidence. Each of the three source
phases recomputes it from its own opened archive descriptors; no prior stream is copied or reused.

After the provisioner returns, the controller constructs the physical postpatch stream and requires
its complete row delta from the reconstructed prepatch stream to be exactly the two named Rocks C++
files plus that crate's generated `.cargo-checksum.json`; every directory and other file row must
be byte-identical. `post-source-tree.tsv` and `source-tree.sha256` bind that result. The provisioner
may have a transient writable extraction while applying the patch, but that transient tree and its
unobserved state are never called a seal or accepted as evidence.

Before writing an archive member, it must prove all of the following:

1. the archive SHA-256 and compressed length match the manifest;
2. the member is a regular file or directory, never a link, device, FIFO, socket or sparse entry;
3. the POSIX path is valid UTF-8, relative, contains no empty, `.` or `..` component, no backslash,
   no C0 byte `0x00`–`0x1f`, no DEL byte `0x7f` and exactly one top-level directory whose
   component equals the manifest row's exact `name-version` concatenation;
4. the normalized destination remains beneath the create-new output directory;
5. no path, Unicode-default-case-folded path or NFC-normalized path collides with an earlier
   member;
6. the member and aggregate checked bounds above remain satisfied; and
7. every destination is created without following a link and without overwriting an inode.

An archive member named `.cargo-checksum.json` anywhere beneath the crate root is terminal; only the
provisioner may create that file. Extracted directories are initially created mode `0700` and
regular files mode `0600`, while the archive execute-bit class is retained as metadata for later
normalization. Regular-file bytes are streamed while hashing. The exact operation order is:
validate/extract all members from the same archive descriptor; apply the exact Rocks patch where
applicable; generate the Cargo checksum file; normalize modes and mtimes; then seal. No read-only
file is patched or
rewritten. During normalization every directory becomes mode `0555` and every mtime becomes Unix
epoch zero. A regular file becomes mode `0555` if any owner, group or other execute bit was present
in its archive mode; otherwise it becomes mode `0444`. This preserves the executable class without
preserving archive write permissions. Ownership remains the invoking owner. Setuid, setgid, sticky
and every special file-type bit are terminal.

For every archive, the provisioner constructs one canonical compact `.cargo-checksum.json` in
memory before creating it: top-level key order is exactly `files`, then `package`; `package` is the
upstream archive checksum; and `files` maps every extracted regular path other than
`.cargo-checksum.json` to its lowercase SHA-256. File keys are UTF-8 POSIX paths relative to and
excluding the sole `name-version/` crate root, always use `/`, and are sorted by raw UTF-8 path
bytes. JSON has no insignificant whitespace, emits non-ASCII scalar values directly as shortest
UTF-8, escapes only JSON-required quote/backslash bytes, uses no `\u` escape and has one trailing
LF. The file is created once, receives mode `0444` and Unix epoch-zero mtime only in the later
normalization step.

For `surrealdb-librocksdb-sys 0.17.3+10.6.2`, the provisioner additionally:

1. verifies the exact upstream archive and both original C++ file hashes bound above;
2. verifies the exact patch SHA-256, exactly four `---`/`+++` file-header lines, exactly four hunk
   headers and exactly the two named paths;
3. applies its hunks in Python at their exact declared line positions with exact old context;
4. permits no fuzz, offset, reject, mode change, extra hunk or extra changed source file;
5. verifies both exact patched-file SHA-256 values bound above; and
6. replaces exactly those two in-memory `files` values with their patched hashes, retains every
   other file entry and the upstream `package` value, and only then creates
   `.cargo-checksum.json`.

Thus the patched crate differs from the archive extraction in exactly two C++ sources and the
generated checksum manifest. The upstream package checksum authenticates the input archive, not the
patched output. The patched directory-source tree seal authenticates the output.

The canonical tree stream excludes the directory-source root itself and begins with:

```text
schema=c2b2a-directory-source-tree-v1
```

It then contains one LF-terminated row per directory and regular file, sorted by raw UTF-8 relative
path bytes:

```text
directory<TAB>mode<TAB>path
file<TAB>mode<TAB>decimal-size<TAB>sha256<TAB>path
```

Modes are four octal digits. `decimal-size` is base-ten bytes with no sign or leading zero unless
the value is exactly `0`. The final row ends in LF. The provisioner makes the complete source tree
read-only and returns without writing a tree record. After it exits, the controller independently
walks the sealed output, constructs the stream and digest, and writes only the applicable
`post-source-tree.tsv` and `source-tree.sha256` evidence leaves. The build checker recomputes the
seal before and after every Cargo command. Any mutation is terminal.

## Independent payload-tree seals

Each qualification root receives an independent byte copy of the complete candidate payload. The
path set preserves every pre-repair payload path; bytes or modes may differ only at the five changed
files and seven new structural support paths above. Every relative path must satisfy the archive
path controls, including the C0/DEL denial, before it can enter a tab/LF-delimited canonical
stream. A missing pre-existing file or any other changed or added path is terminal.
Every lock/A/B payload copy operation uses create-new regular files and directories and never a
rename, symlink, hardlink, clonefile or reflink. Every copied regular file has link count one, and
its device/inode tuple differs from the repository source and every peer copy.

The canonical payload stream excludes the payload root itself and begins with:

```text
schema=c2b2a-payload-tree-v1
```

It then uses the directory/file row grammar and raw-UTF-8 ordering defined for the directory-source
tree. For the repository candidate, the canonical mode is `0555` for a directory and `0555` for a
regular file if any execute bit is set, otherwise `0444`. Each A/B copy is physically normalized to
those canonical modes and Unix epoch-zero mtimes. Therefore repository, A and B streams and digests
must be byte-identical even though the repository remains writable under its own ownership rules.

The controller writes each stream and digest only to that phase's applicable payload-tree evidence
leaves; no provisioner or build child writes a payload-tree record. The checker recomputes the
stream before and after every Cargo command and compares it to the repository candidate and the
peer root. A mutation, shared inode, symlink, mode-class difference or digest mismatch is terminal.
The exact payload-tree manifest byte count, row count and SHA-256 remain later build/source identity
values.

The lock-root payload has one temporary file-mode exception. Before metadata, its root is already
mode `0555`, `Cargo.lock` is mode `0600`, all other directory/file modes are the canonical
`0555`/`0444` values and every mtime is Unix epoch zero. The read-only parent plus lock sandbox, not
ownership alone, confines Cargo to in-place truncation and writing of the pre-existing lock. It
permits no temporary file, replacement, rename or directory mutation. Immediately after metadata
exits, the checker proves the sole content delta is the candidate lock and the directory-entry set
is unchanged, normalizes `Cargo.lock` to `0444`, restores its epoch-zero mtime, and computes the
final canonical payload stream with the same root exclusion, row grammar, mode projection and
ordering as the qualification payloads. That stream must be byte-identical to the repository stream
after the publication point and both later A/B streams; all four copies remain inode-independent.

## Cargo directory-source configuration

`root-cargo-config.toml` contains exactly these LF-terminated UTF-8 bytes, including one final LF:

```toml
[source.crates-io]
replace-with = "c2b2a-archive-source"

[source.c2b2a-archive-source]
directory = "cargo-source"

[net]
offline = true
```

It is copied byte-for-byte to `<fresh-root>/.cargo/config.toml`. Relative-source resolution is from
the root Cargo configuration and must name `<fresh-root>/cargo-source`. It contains no `[patch]`,
registry mirror, git source, credential, proxy, path dependency, target directory or rustflag. Its
observed SHA-256 is a later build/source identity value, not a placeholder accepted by this design.
Each lock/A/B copy and its `.cargo` parent are create-new, never linked, cloned or reflinked; the
regular file has link count one and a device/inode tuple distinct from the repository support source
and every peer copy. After creation the directory is mode `0555`, the file is mode `0444`, both have
Unix epoch-zero mtimes, and bytes/hash must remain unchanged before and after every Cargo command.

The qualification layout is exactly:

```text
<fresh-root>/.cargo/config.toml
<fresh-root>/payload/
<fresh-root>/payload/.cargo/config.toml
<fresh-root>/home/
<fresh-root>/cargo-home/
<fresh-root>/cargo-source/
<fresh-root>/dylib-empty/
<fresh-root>/target/
<fresh-root>/tmp/
<fresh-root>/policy-canaries/
<fresh-root>/evidence/
```

Cargo is invoked with current directory `<fresh-root>/payload`. It therefore reads both the
payload target configuration and the root source configuration. `CARGO_HOME` names the current
create-new `cargo-home/active` child, while `CARGO_TARGET_DIR` names the create-new persistent
`target` directory above.

Pinned Cargo metadata must retain
`registry+https://github.com/rust-lang/crates.io-index` for every registry package, including the
patched `surrealdb-librocksdb-sys` node. The payload root is the only path source. Any path, git,
local-registry or second registry source is terminal. The checker also proves exactly one active
Rocks sys node, every reverse edge reaches it, and no unused replacement exists.

## Target-active graph v2

The post-edit graph uses schema `c2b2a-target-active-graph-v2`. It preserves the addendum's Cargo
1.93.0, `aarch64-unknown-linux-musl`, locked/offline, collector-only metadata algorithm and its
node/edge row ordering. It adds these exact LF-terminated header fields before node rows:

```text
schema=c2b2a-target-active-graph-v2
target=aarch64-unknown-linux-musl
cargo=1.93.0 (083ac5135 2025-12-15)
root=engram-native-c2b2a-payload@0.0.0
manifest-sha256=<lowercase-64-hex>
lock-sha256=<lowercase-64-hex>
payload-config-sha256=<lowercase-64-hex>
root-cargo-config-sha256=<lowercase-64-hex>
pre-repair-payload-tree-sha256=c6e6de9d4b8c6cd7a162341f43506bce15154c1d54e19ff6aad40f2c5c67f8d2
rocks-patch-sha256=c82a6d05411362963eb8049cc6a0ecb563562bd1d35e14bfe892f39db936fcfb
archive-manifest-sha256=<lowercase-64-hex>
archive-bundle-tree-sha256=<lowercase-64-hex>
directory-source-tree-sha256=<lowercase-64-hex>
payload-tree-sha256=<lowercase-64-hex>
```

The angle-bracket forms above define field syntax only; they are not accepted values. No graph v2
identity exists until the separate build/source identity record replaces each syntactic field with
an observed digest and binds node, edge and total-row counts, byte length and final SHA-256.
After both qualification build/probe schedules and their output/post-command seals pass, controller
`graph-finalize` runs
exactly once without a subprocess or network. It reads only the sealed graph-metadata stdout, final
lock, repository and A/B payload/source streams, archive manifest/bundle seal, exact configuration
hashes and trusted A/B invocation results. It creates only the two named graph-finalization evidence
files, canonicalizes the locked metadata node/edge rows using the inherited addendum algorithm,
fills every observed header, and records row count, byte length and SHA-256. Any unavailable header,
input mismatch, unlisted read/write or second finalization is terminal.
`target-active-graph-v2.sha256` is not a bare digest. It contains exactly these LF-terminated ASCII
rows, with unsigned base-ten values and no leading zero except for zero itself:

```text
schema=c2b2a-target-active-graph-v2-digest-v1
rows=<decimal-row-count>
bytes=<decimal-byte-count>
sha256=<lowercase-64-hex>
```

The row count includes every LF-terminated row in `target-active-graph-v2.tsv`; the byte count and
digest cover that complete file exactly. The angle-bracket forms are syntax declarations and are
never accepted literal values.
The graph record also proves the final lock-root/repository/A/B payload streams and digests all
equal one another, the independently provisioned lock-root/A/B directory-source streams and
digests all equal one another, and both build roots consumed the one sealed archive-bundle digest.
Equality never permits shared payload or directory-source inodes.
It separately proves the candidate's unchanged rows match the captured pre-repair stream and that
the complete delta is exactly the five changed files plus seven new structural support paths.

The graph must re-prove every high-risk row, Rocks feature, requested/unified Tokio feature,
target-inactive WebAssembly sentinel and denied package/feature from the dependency addendum. It
must additionally bind the complete requested and unified feature sets for all three active
`getrandom` versions and the active ring row.

## Complete entropy matrix

The build/source identity record must bind each static row below from freshly extracted,
hash-bound source, actual rustc arguments, dep-info and final ELF. The later
runtime-qualification record alone binds the rows that require runtime syscalls, signals, core
state or tracer observations; it references the already reviewed source and probe-binary hashes
without backfilling them.

### Host entropy

The macOS host retains the two direct, separate 32-byte `getentropy` calls for the run nonce and
challenge. A nonzero, interrupted or failing return is terminal and not retried. Host evidence is
not Linux collector evidence.

### Supervisor entropy

The Linux supervisor retains one direct 16-byte `getrandom` call for each fresh filesystem UUID. A
short, interrupted or failing return is terminal and not retried.

### getrandom 0.2.17

The target graph must contain exactly one registry `getrandom 0.2.17`, checksum
`ff2abc00be7fca6ebc474524697ae276ad847ad0a6b3faa4bcb027e9a4614ad0`. The alias must unify
`linux_disable_fallback`. Actual rustc cfg and dep-info must include `linux_android.rs` and exclude
`use_file.rs`, `linux_android_with_fallback.rs`, `rdrand.rs` and `custom.rs`.

### getrandom 0.3.4

The target graph must contain exactly one registry `getrandom 0.3.4`, checksum
`899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd`. Actual rustc arguments
must contain `getrandom_backend="linux_getrandom"`. Dep-info must include only its Linux getrandom
backend and exclude file, fallback, custom, RDRAND and RNDR backends.

### getrandom 0.4.3

The target graph must contain exactly one registry `getrandom 0.4.3`, checksum
`300e883d756b2e4ec94e02791f39b04b522276138852cfc41d9fb7e904106099`. It is active through
`uuid 1.24.0`. Actual rustc arguments must contain `getrandom_backend="linux_getrandom"` and
dep-info must prove the same backend exclusions required for 0.3.4.

### ring 0.17.14

The graph may retain ring's active feature named `dev_urandom_fallback`; for this exact version it
is an empty feature declaration. The exact registry ring checksum is
`a4689e6c2294d81e88dc6261c768b63bc4fcdb852be6d1352498b114f61383b7`. Fresh-source evidence
must prove Linux `SystemRandom` delegates to the single repaired `getrandom 0.2.17` node and that
the feature name introduces no separate source path. The report must not falsely claim the named
feature is absent.

The inert-feature exception is valid only for this exact three-file source identity:

| Extracted ring 0.17.14 input | SHA-256 |
|---|---|
| normalized `Cargo.toml` | `b81a19b27cc744410b056eb1d1195ecd41e25a163acadfca3c6457e96b797d3f` |
| `Cargo.toml.orig` | `4c58550e0f323c33bff699c2c00ee7320f2e078b68e9eafc9b269f65784873de` |
| `src/rand.rs` | `fb8708fe62706fe7f7c7aad5f7df9f65ca19def5fabac9616375e9f1e532f4b8` |

A recursive exact-token scan must find exactly four `dev_urandom_fallback` occurrences: one
reference in the `default` feature list and one empty feature declaration in each of the two
manifests. It must find zero occurrences in every other archive member and no cfg, source or
build-script reference. Any file-hash, occurrence-set, default-list or empty-value mismatch
invalidates the exception and the candidate.

### Patched RocksDB entropy

Fresh source and object disassembly must prove the exact patched function requests 24 bytes from
pinned musl `getentropy`, takes the abort branch on any nonzero result, and contains no
`std::random_device` call or fallback. The patched `port_posix.cc` object must prove
`GenerateRfcUuid` clears output and returns false and that the function's own call/relocation slice
has no file-open or stream edge; unrelated file-I/O imports elsewhere in that object are allowed.
The `env.cc` object
must retain the false-result edge from `Env::GenerateUniqueId` to
`GenerateRawUniqueId(..., true)`. Pinned musl source, archive identity and disassembly must prove
successful `getentropy` reaches only Linux `getrandom` for this target. The build-listed C/C++
source occurrence set must contain no `/proc/sys/kernel/random/uuid`; an inert historical mention
outside the compiled source list is reported, not misclassified as executable reachability.

### Whole-collector denial

The source-reachability, dep-info, object, archive, symbol, string, relocation, disassembly and
runtime trace checks must jointly reject every retained `/dev/random`, `/dev/urandom`,
`/proc/sys/kernel/random/uuid`, `std::random_device`, custom backend, file fallback, RDRAND, RDSEED
or RNDR path. Negative strings alone are corroborating evidence, never authority. The accepted
collector may use only the pinned Linux getrandom syscall path.

## Complete encoded rustflags and path remapping

Every acceptance-candidate build sets `CARGO_ENCODED_RUSTFLAGS` to the complete ordered list below;
ASCII unit separator `0x1f` separates arguments:

```text
--cfg
getrandom_backend="linux_getrandom"
-C
relocation-model=pie
-C
link-arg=-static-pie
-C
link-arg=-Wl,--build-id=sha1
--remap-path-prefix=<fresh-root>=/c2b2a-root
```

`<fresh-root>` above is a normative substitution for the canonical, create-new absolute root
witnessed for that build, not an accepted literal or unresolved environment variable. The encoded
value has exactly nine arguments and eight `0x1f` separators, with no leading or trailing separator,
LF, CR or NUL. The acceptance build always supplies this complete value; a partial override of the
payload configuration is forbidden. Actual rustc invocations must prove every target dependency
received the entropy cfg and remap. The cfg selects the syscall-only backend in `getrandom` 0.3.4
and 0.4.3; it does not repair 0.2.17, which is closed only by the manifest alias feature above.

Each A/B qualification-root path is canonical, create-new, has no symlink component and matches
exactly one of:

```text
^/private/tmp/engram-c2b2a-A-[A-Za-z0-9._-]+$
^/private/tmp/engram-c2b2a-B-[A-Za-z0-9._-]+$
```

Thus a root contains no whitespace, control byte, colon or equals sign. Every varying payload,
home, Cargo-home, directory-source, target, temporary and evidence path is beneath it. The command
record stores the actual root and exact encoded bytes. `RUSTFLAGS`, `RUSTDOCFLAGS`,
`CARGO_BUILD_RUSTFLAGS`, every `CARGO_TARGET_*_RUSTFLAGS` and any second encoded-rustflags variable
are absent.

The following are the only additional target C/C++ flags this repair authorizes, each with the same
observed fresh-root substitution:

```text
-ffile-prefix-map=<fresh-root>=/c2b2a-root
-fdebug-prefix-map=<fresh-root>=/c2b2a-root
-fmacro-prefix-map=<fresh-root>=/c2b2a-root
-Werror=date-time
```

They are carried only through the exact target-specific `CFLAGS_aarch64_unknown_linux_musl` and
`CXXFLAGS_aarch64_unknown_linux_musl` environment variables. Each variable is exactly one
ASCII-space-separated string containing those four tokens in that order, after literal root
substitution; quoting, escaping, shell evaluation, a fifth token or empty token is forbidden. The
whitespace-free root makes that tokenization unambiguous. Generic `CFLAGS`, `CXXFLAGS`, `CPPFLAGS`
and `LDFLAGS`, and every other target-specific flag variable, are absent. No host or arbitrary C/C++
flag is permitted. `ROCKSDB_COMPILE=true` remains mandatory; external libraries, includes,
`pkg-config`, ambient compiler discovery and other build influence remain absent.

## Two-root hermetic qualification build

The provider-free qualification sets and records process umask `0077`, then creates two different
owner-only roots with create-new semantics. In each root it creates and verifies only `home`,
`cargo-home`, `target`, `tmp`, `policy-canaries`, `evidence` and `dylib-empty` directories at mode
`0700`; it verifies `dylib-empty` is empty and has no xattr or nontrivial ACL, then seals it mode
`0500` before the first qualification provisioner or Cargo invocation. The controller re-observes
its exact empty entries, device/inode, uid/gid, mode, xattr and ACL state before and after every
qualification child and in `pre-runtime-finalize`;
`.cargo`, `payload` and `cargo-source` remain absent until their exact qualification provisioner
modes run.
They share no payload or directory-source inode, Cargo home, target directory, incremental state,
build-script output, hardlink, clone, reflink or symlink. Each independently copies and seals the
payload and independently verifies, extracts, patches and seals the directory source from the one
read-only archive bundle.
Qualification-payload mode and then qualification-source mode run exactly once for root A and once
for root B before any Cargo command; no output from one root is copied into the other.

The launch environment is created from an empty environment. These values are exact after root
substitution:

```text
LANG=C
LC_ALL=C
TZ=UTC
RUST_BACKTRACE=0
CARGO_INCREMENTAL=0
CARGO_CACHE_RUSTC_INFO=0
CARGO_BUILD_JOBS=1
ROCKSDB_COMPILE=true
SOURCE_DATE_EPOCH=0
HOME=<fresh-root>/home/active
CARGO_HOME=<fresh-root>/cargo-home/active
CARGO_TARGET_DIR=<fresh-root>/target
TMPDIR=<fresh-root>/tmp/qualification-<A-or-B>-<supervisor-or-collector>
DYLD_FALLBACK_LIBRARY_PATH=<fresh-root>/dylib-empty
PATH=<later-bound-exact-absolute-path-list>
RUSTC=<sealed-rust-toolchain>/bin/rustc
RUSTDOC=<sealed-rust-toolchain>/bin/rustdoc
CARGO_ENCODED_RUSTFLAGS=<the-complete-encoded-value-above>
CC_aarch64_unknown_linux_musl=<exact-cross-gcc-path>
CXX_aarch64_unknown_linux_musl=<exact-cross-g++-path>
AR_aarch64_unknown_linux_musl=<exact-cross-ar-path>
CFLAGS_aarch64_unknown_linux_musl=<the-exact-four-token-value-above>
CXXFLAGS_aarch64_unknown_linux_musl=<the-exact-four-token-value-above>
CLANG_PATH=<exact-clang-executable>
LIBCLANG_PATH=<exact-directory-containing-one-selected-libclang-dylib>
BINDGEN_EXTRA_CLANG_ARGS_aarch64_unknown_linux_musl=<exact-target-and-sysroot-args>
```

For each Cargo command, the two TMPDIR selectors are replaced by its exact root letter and bin:
`qualification-A-supervisor`, `qualification-A-collector`, `qualification-B-supervisor` or
`qualification-B-collector`. The angle-bracket values are normative substitutions, not accepted
literal values. The later build/source identity record freezes exact `PATH` bytes, entry order and
SHA-256 plus the pre/post-identical canonical seal of the complete Rust 1.93.0 toolchain and target
sysroot. Every PATH entry
is absolute, nonempty and unique; resolution, realpath and SHA-256 of each invoked tool are
recorded. The first entries name a separately sealed tool bundle; none names ambient
`/Users/yuval.meiri/.cargo`. All Cargo commands invoke the absolute Cargo binary inside that sealed
tree directly. Rustup proxies, `cargo +toolchain`, `RUSTUP_HOME`, `RUSTUP_TOOLCHAIN`,
`RUSTC_WRAPPER` and `RUSTC_WORKSPACE_WRAPPER` are forbidden. `CARGO_HOME` remains fresh and contains
no registry authority. Cargo/rustc 1.93.0 raw version outputs, `rustc --print sysroot`, installed
target tree and direct executable hashes are bound before use.

The exact cross GCC/G++/ar paths must match the linker/archiver identities in payload Cargo
configuration after realpath resolution. `CLANG_PATH` and `LIBCLANG_PATH` eliminate ambient
libclang discovery; `LIBCLANG_PATH` names one sealed directory containing exactly one accepted
matching libclang dylib basename, and the exact executable, selected dylib dependency closure and
LLVM tree are sealed. `LLVM_CONFIG_PATH` is absent because no reviewed `llvm-config` executable is
installed. Pinned `clang-sys 1.9.1` source must prove that `LIBCLANG_PATH` selects the library
first, then its optional `llvm-config --includedir` probe searches the exact sealed `PATH`, finds no
entry named `llvm-config`, returns no include result and remains nonfatal. Preflight proves that
absence in every `PATH` directory; any executed `llvm-config` or different behavior is terminal.
The target bindgen value consists of exactly
`--target=aarch64-unknown-linux-musl --sysroot=<exact-cross-sysroot>` under the same whitespace-safe
token rules. The cross compiler, binutils, sysroot, headers, Python interpreter and standard-library
trees are independently sealed and hash-bound. Every compiler/helper child and every file it reads
must fall within a later-reviewed input closure.

Every unlisted launch variable is absent. In particular this includes generic or alternate Rust,
C/C++ and linker flags; `MAKEFLAGS`; registry/index overrides; all upper- and lower-case HTTP,
HTTPS, ALL and NO proxy names; certificate variables; credentials and tokens; provider variables;
ambient Cargo or Rust configuration names; generic `CC`, `CXX`, `AR`, `CFLAGS`, `CXXFLAGS`,
`CPPFLAGS`, `LDFLAGS`, `CPATH`, include/library/compiler search variables, `SDKROOT`,
`DEVELOPER_DIR`, generic bindgen flags, alternate clang/LLVM variables, every other `DYLD_*` and
`PKG_CONFIG_*`. Cargo-generated child variables such as `OUT_DIR`, `TARGET`, `HOST` and `NUM_JOBS`
and the source-created `CARGO_MAKEFLAGS` must derive only from the exact from-empty top-level
environment and the reviewed Cargo, jobserver, build-script and external-tool source manifests.
Only values independently reflected in sealed fingerprints, dep-info or outputs may become identity
facts; `-vv` is corroboration and no complete per-PID child-environment capture is claimed.

The nonempty qualification `DYLD_FALLBACK_LIBRARY_PATH` prevents pinned Cargo from appending
`$HOME/lib`, `/usr/local/lib` or `/usr/lib`. At every reachable Cargo `fill_env` boundary, the
reviewed child-environment derivation may prepend only the applicable sealed target output,
dependency, native-library and sysroot-library paths to that sentinel. The controller derives and
binds each permitted ordered path table from the complete reachability manifest, sealed
fingerprints and outputs. Every entry must be absolute and lie beneath the current qualification
target, the complete tool seal or the exact empty sentinel; a different, duplicate, relative or
outside-closure entry is terminal.

Every A/B qualification Cargo invocation and all of its rustc, compiler, linker and build-script
children run under one exact later-bound `/usr/bin/sandbox-exec` qualification profile. The two
lock-root metadata commands use their distinct phase-specific profiles defined above. The
qualification profile default-denies network, reads and writes and imports only a hash-bound
minimal host-system baseline. Within the fresh root,
read access is limited to traversal/metadata for the root itself and exact `.cargo/config.toml`,
`payload`, `cargo-source`, the current `home/active` and `cargo-home/active`, `dylib-empty`,
`target` and current
phase-specific `tmp` subtree;
`evidence` is explicitly unreadable. The only other reads are the exact sealed
Rust/Python/LLVM/cross-tool/system input closure and fixed system runtime files proven necessary.
Repository paths, ambient Cargo/Rust homes, the peer root and unrelated user paths are denied. The
enforced policy, rather than a purported complete access log, is the closure authority; any denial
or evidence of an outside read is terminal.

The qualification profile default-denies every persistent write, then permits content and metadata
writes only inside the current `<fresh-root>/home/active`, `cargo-home/active`, `target` and the
current phase-specific `tmp`; it denies entry or metadata writes to either rotation parent and
active-root literal. Pathname writes to `evidence` are denied and output reaches the trusted
collector only
through fds 1 and 2. It explicitly denies data and metadata writes, chmod, create, rename, unlink,
link and symlink operations against `<fresh-root>/.cargo/config.toml`, the complete `payload`, the
complete `cargo-source`, the fresh-root directory itself, the sealed archive bundle, peer root,
repository, toolchains and every other path.
Modes `0444`/`0555` are canonical normalization only, never an immutability claim. Before use, the
profile's parsed canonical rules must independently deny content-write, truncate, chmod, create,
rename-over, unlink, hardlink, symlink and directory-mode operations for the root config, payload
and directory source. Non-destructive actual-path read probes and the closed canary/probe-temp
matrix above corroborate those selectors without risking an authoritative file. Network,
forbidden-read, repository, ambient-home, peer-root and outside-root probes must fail. Exact
profile/import bytes, hashes, `/usr/bin/sandbox-exec` identity and probe transcripts are later
identity evidence. No A/B
Cargo command or build script may run outside that profile. Acquisition fetch and the two lock-root
metadata commands remain governed by their earlier exact profiles; only acquisition fetch has
network access.

Within each root, cwd is exactly `<fresh-root>/payload`. Root A executes these exact payload argv in
this order:

```text
<sealed-rust-toolchain>/bin/cargo build -vv --frozen --offline \
  --target aarch64-unknown-linux-musl \
  --release --no-default-features --features supervisor --bin supervisor
<sealed-rust-toolchain>/bin/cargo build -vv --frozen --offline \
  --target aarch64-unknown-linux-musl \
  --release --no-default-features --features collector --bin collector
```

After root A's second Cargo command and its seals pass, `probe-build-A` runs exactly once. At least
one wall-clock second after root A's two qualification builds, probe build and their output/evidence
seals complete, root B executes the same two Cargo argv in the same order with only the root
substitutions changed, followed exactly once by `probe-build-B`. The later root-A artifact audit is
expressly excluded from this ordering boundary. No
target filter, feature, bin, flag, environment value or order may be inferred or appended.
`--frozen` and explicit `--offline` are both mandatory. Payload and directory-source seals are
recomputed before and after each Cargo command. The enforced process-exec class, sealed Cargo and
build-script sources, Cargo fingerprints, produced artifacts and untrusted corroborating `-vv`
transcript must agree on the expected build-script set; no complete historical execution trace is
claimed.

Each probe-build mode uses byte-exact compiler/linker argv from the preflight-reviewed controller
table. Its environment is built from empty and contains exactly `LANG=C`, `LC_ALL=C`, `TZ=UTC`,
`SOURCE_DATE_EPOCH=0`, `HOME=<fresh-root>/home/active`, the exact later-bound `PATH`, and
`TMPDIR=<fresh-root>/tmp/probe-build-{A|B}` after root-letter substitution; every other variable is
absent. Every executable is nevertheless selected by its absolute reviewed path. Its sandbox
inherits the qualification read and exec closure, additionally reads only its
same-root production objects and fingerprints, and writes only that root's
`target/probe-build` and exact `tmp/probe-build-{A|B}`. Payload, directory source, root
configuration, evidence, repository,
peer root and network remain read-only or denied exactly as applicable. It compiles and links the
two exact qualification-only sources against the freshly built same-root patched production Rocks
objects, emits the probe-only link map beneath that exact `target/probe-build`, never executes the
AArch64 result on macOS, and then seals source, argv, object, link-map
and executable identities. A and B outputs and remapped evidence must be byte-identical. A missing
production edge, extra object, unreviewed argument or target write outside `probe-build` is
terminal.

The full command argv, cwd, exact environment, sandbox, tool paths, raw version-output hashes,
target sysroot, GCC/G++ 14.2.0, binutils 2.44, musl, linker, archiver, libclang, headers and all
build-script inputs are recorded. Repository and nested `target/debug` remain absent and all outputs
remain beneath the applicable fresh root.

Before a candidate identity may be recorded, both roots must produce byte-identical supervisor and
collector ELFs. Each ELF must have:

1. identical SHA-256 across roots;
2. AArch64 `ET_DYN` static PIE identity;
3. no `PT_INTERP` and an empty `DT_NEEDED` set;
4. exactly one GNU SHA-1 build-ID note with the same value across roots;
5. no raw fresh-root, Cargo-home, target, repository or temporary path;
6. exact dependency-file hashes or a separately frozen canonical depfile stream; and
7. all entropy and dependency firewalls above.

Both complete bundle manifests must also be byte-identical and contain only the supervisor,
collector and fixed metadata defined under the artifact-audit phases. Build IDs are reported
separately from ELF SHA-256.

The one-second separation detects only second-resolution wall-clock influence such as `__TIME__`;
it is not evidence against minute-, date- or coarser inputs. `-Werror=date-time`, the read sandbox,
exact controller-captured top-level launch state, pinned-source-derived child constraints and an
audited denial of build-script wall-clock use remain the authority against time-dependent output.

## Entropy failure qualification probes

`entropy-failure-probe.cc` and `entropy-failure-probe.rs` are qualification-only sources. Neither
is a Cargo payload bin, a member of either production ELF, or a deterministic-bundle member; neither
can satisfy a calibration count. Their exact source, compile/link argv, object and executable
identities belong only to the build/source identity record. Runtime identities belong only to the
later runtime-qualification record.

The C++ source exposes one fixed C ABI entry point with a fixed 36-byte output buffer and status.
That wrapper links the freshly patched production Rocks objects and calls the real
`ROCKSDB_NAMESPACE::Env::Default()->GenerateUniqueId()`. It does not copy, restate or link-wrap
`getentropy`, `GenerateRfcUuid`, `GenerateRawUniqueIdImpl` or an entropy branch. The exact source
and link map must prove the call uses production `env.cc`, observes the patched false/no-I/O
`GenerateRfcUuid`, enters the existing `GenerateRawUniqueId(..., true)` fallback and reaches the
same patched 24-byte getentropy track as fresh DB identity creation. A normal result is accepted
only if it is exactly 36 lowercase ASCII UUID characters, has hyphens at offsets 8, 13, 18 and 23,
version nibble `4` and variant nibble one of `8`, `9`, `a`, `b`.

The standalone Rust driver includes the exact candidate payload source with:

```rust
#[path = "../src/seccomp.rs"]
mod seccomp;
```

It must call that module's `bootstrap_program()`, `steady_program()` and `install_program()`
directly; a copied, generated or probe-specific production filter is forbidden. An unfiltered parent
creates one fresh owner-only probe directory and forks one fresh child per mode. Before any filter,
each child resets `SIGABRT` and `SIGSYS` to `SIG_DFL`, unblocks both and verifies neither pending;
sets both `RLIMIT_CORE` values to zero; sets and verifies `PR_SET_DUMPABLE=0`; and sets
`PR_SET_NO_NEW_PRIVS=1`. It then installs only the exact mode filters and makes exactly one call
through the C ABI wrapper. The parent uses `waitid` or `waitpid` and rejects stopped, continued or
wrong exit/signal states. Any setup error is a reported probe failure rather than an expected
signal.

Any syscall tracer is attached before the child becomes nondumpable and remains a passive observer:
it forwards every signal unchanged and never changes registers, memory or syscall results. After
filters are installed, the child writes one fixed byte to a dedicated
preopened marker pipe immediately before entering the C ABI wrapper. The accepted trace epoch begins
at that marker and ends only at wrapper return or terminal signal; startup and filter-installation
syscalls are reported separately and cannot contribute to the success-mode 24-byte count.

The separately reviewed Linux-runner freeze defines the exact commands and expected checks for
`/proc/sys/kernel/core_pattern`, `core_uses_pid`, `suid_dumpable`, the controlled writable search
root and a helper-invocation witness. The runtime-qualification record binds their observed values
and outcomes. Both the zero rlimit and verified nondumpable state are mandatory; neither
substitutes for the other. Every mode must leave zero core artifacts and invoke no core helper.

The three modes are exact. Every test filter kills a wrong architecture and uses unconditional
AArch64 `openat=56`, `openat2=437`, `getrandom=278` and `tkill=130` numbers rather than
target-header feature detection:

1. **Successful production/no-file discriminator.** The injector traps `openat` and `openat2` and
   allows every other correct-architecture syscall, including `getrandom`. The child then installs
   the exact bootstrap and exact steady programs. The wrapper must return a valid UUID and exit
   zero.
   Trace evidence must show successful getrandom operations whose cumulative returned bytes are 24,
   while permitting pinned libc's EINTR/short-read loop, and no open or random/proc pathname.
2. **Failure fallback discriminator, without production filters.** The sole injector returns
   `EPERM` for `getrandom`, traps `openat` and `openat2`, explicitly allows `tkill`, and allows
   every other correct-architecture syscall. Patched production identity must reach the real musl
   `getentropy` error and terminate by `SIGABRT` (6). An unpatched proc-UUID path or file fallback
   reaches a trapped open and terminates by `SIGSYS` (31), which is a terminal mismatch.
3. **Production failure composition.** The injector returns `EPERM` for `getrandom`, traps `openat`
   and `openat2`, and allows every other correct-architecture syscall so filter installation remains
   possible. The child then installs the exact bootstrap and exact steady programs. Real musl
   `getentropy` fails; the patched branch calls `abort`; pinned musl attempts `tkill`; unchanged
   steady seccomp traps it. The expected terminal result is `SIGSYS` (31), not `SIGABRT`, and trace
   or siginfo must identify trapped syscall 130 so an open trap cannot masquerade as the result.

For mode 3, the runtime-qualification record must also bind the non-obvious signal-mask edge:
pinned musl `raise()` blocks application signals before issuing `tkill(130)`, while the exact
runner kernel's
seccomp forced-signal path unblocks the generated `SIGSYS` and restores its default disposition.
Pinned musl and kernel source/ELF identities plus the observed passive trace must agree; a masked,
handled, suppressed or tracer-rewritten `SIGSYS` is terminal.

All filters use exact classic-BPF bytes and action values bound by the build/source identity;
runtime traces and outcomes are bound later by the runtime-qualification record. Source, object,
call-graph and trace evidence jointly prove the production wrapper reaches one Rocks
source-level `getentropy` call, no port/device file path and the stated outcome. Failure modes show
one failing `getrandom`; success counts cumulative bytes rather than asserting one kernel syscall.
The success discriminator proves the second patch hunk is runtime-reachable; the failure
discriminator distinguishes abort from either proc-UUID or device fallback; the full composition
proves the honest production signal outcome.

The production result is intentional fail-closed evidence. This repair does not add `tkill`, widen
bootstrap or steady seccomp, or make entropy failure acceptable. A production entropy failure is a
terminal failed run with bounded supervisor cleanup. The report must distinguish the source-level
`abort()` call from kernel-observed `SIGSYS` caused by the denied abort syscall.

If these probes require Linux execution, execution remains blocked until a separately reviewed
provider-free Linux qualification plan binds the exact runner. This document authorizes their
source and cross-build, not an implicit VM or emulator.

## Preserved containment and seccomp order

The collector still stacks the steady filter at its first application instruction, before fixture
construction, thread creation or any Core reference. Every Rocks and transitive entropy call must
occur only after that point. The sole successful-production entropy syscall remains `getrandom`,
already in the unchanged steady allowlist.

No bootstrap or steady filter row changes. All 18 frozen network syscalls must still return `EPERM`
in both states, and the exact pre-steady trace remains mandatory. Because `openat` is required by
RocksDB, seccomp alone cannot prove absence of a device fallback; source reachability, final ELF
and runtime pathname evidence remain required.

Any newly required production syscall, pre-filter entropy call, device open, successful network
syscall or runtime filter widening is terminal and requires a new reviewed freeze.

## Closed pre-runtime verification phases

After both probe-build phases and graph finalization pass, the controller runs exactly these phases
in order: `source-audit`, `artifact-audit-A`, `artifact-audit-B`,
`host-validation-payload-tests`, `host-validation-foundation-test-compile`,
`host-validation-foundation-test-run`, `host-validation-foundation-bin-compile`,
`host-validation-foundation-bin-run`, `host-validation-lints`, then
`pre-runtime-finalize`. No phase may be skipped, repeated or reordered. They deny network and may
not change the repository, payload, directory source, root configuration, sealed archives,
production objects or accepted A/B executables.

`source-audit` has no child process. Within a 1,800-second self-deadline, the controller reads only
the reviewed source universe, final graph and seals already authorized above. It deterministically
constructs the complete production caller graph and compiled/excluded source sets; verifies all
dependency, feature, source-occurrence, entropy-backend and forbidden-path rows; and writes its
canonical findings only to that phase's `result.json`. Every observation maps to one exact source
path, line span and source SHA-256. Text absence alone cannot satisfy an object or runtime gate.

Each artifact-audit phase uses a from-empty environment containing only `LANG=C`, `LC_ALL=C`,
`TZ=UTC`, `SOURCE_DATE_EPOCH=0`, `HOME=/var/empty` and its exact phase `TMPDIR`. Its Seatbelt
profile
reads only the corresponding root's sealed target artifacts, payload/source/link-map/fingerprint
inputs, exact audit tools and fixed system closure; it writes only its own temporary subtree and
controller evidence pipes. For each supervisor ELF, collector ELF, probe ELF and each exact Rocks
object selected by the probe link map plus Cargo fingerprints, in raw canonical path order, the
controller launches this fixed substantive-argv table using exact sealed AArch64 GNU-binutils paths:

```text
<cross-readelf> --wide --file-header --program-headers --dynamic --notes --symbols --relocs <file>
<cross-objdump> --disassemble --reloc --wide --demangle <file>
<cross-nm> --debug-syms --demangle --print-file-name <file>
<cross-strings> --all --bytes=4 --radix=x <file>
```

The selected Rocks object set must be nonempty, identical across A/B and contain the objects for
`unique_id_gen.cc`, `port_posix.cc` and `env.cc`; an ambiguous, unlinked or extra selected object is
terminal. Each child has a 1,800-second deadline and the common output ceilings. The controller
parses the framed outputs, independently hashes every input, enforces every ELF/build-ID/import/
symbol/string/relocation/disassembly firewall above and emits only the allowed artifact-audit phase
evidence leaves from the global inventory, including the executable bundle. Tool output is
corroboration tied to exact tool and artifact hashes, never an unbound assertion.

After all audit-tool children pass, the controller writes the create-new
`artifact-audit-{A|B}/executable-bundle-v1.tsv`. It is an LF-terminated UTF-8 stream with exactly
these 17 ordered rows after substituting observed lowercase digests, unsigned decimal byte counts
and 40-lowercase-hex GNU SHA-1 build IDs:

```text
schema=c2b2a-executable-bundle-v1
target=aarch64-unknown-linux-musl
profile=release
payload-tree-sha256=<lowercase-64-hex>
source-tree-sha256=<lowercase-64-hex>
target-active-graph-v2-sha256=<lowercase-64-hex>
root-cargo-config-sha256=<lowercase-64-hex>
archive-bundle-tree-sha256=<lowercase-64-hex>
tool-tree-sha256=<lowercase-64-hex>
supervisor.path=target/aarch64-unknown-linux-musl/release/supervisor
supervisor.bytes=<unsigned-decimal>
supervisor.sha256=<lowercase-64-hex>
supervisor.gnu-sha1-build-id=<lowercase-40-hex>
collector.path=target/aarch64-unknown-linux-musl/release/collector
collector.bytes=<unsigned-decimal>
collector.sha256=<lowercase-64-hex>
collector.gnu-sha1-build-id=<lowercase-40-hex>
```

The angle-bracket forms define syntax and are never accepted literally. The payload, source,
graph, configuration, archive and tool values come from independently recomputed sealed inputs,
not parsed tool prose. The artifact values come from the exact regular files plus the independently
parsed ELF notes. Root-relative paths deliberately omit A/B identity. The controller reopens and
hashes the completed leaf like every other evidence file; `pre-runtime-finalize` requires its A and
B bytes to be identical.

`host-validation-payload-tests` first creates exact `<acquisition-root>/host-validation` and its
`home`, `cargo-home`, `target` and `dylib-empty` subdirectories at mode `0700`; all must be absent.
It verifies `dylib-empty` is empty, has no extended attribute or nontrivial ACL and changes it to
mode `0500` before the phase policy probe. Every later host-validation child and
`pre-runtime-finalize` must re-observe that exact empty, read-only directory and its device/inode,
uid/gid, mode, xattr and ACL state. In this
section, `<host-validation-target>` means exactly `<acquisition-root>/host-validation/target`.
Its `payload-tests`, `lints` and `foundation` children must initially be absent. The six
`host-validation-*` phases read root A's sealed configuration, payload and directory source plus
only the protected repository inputs required by the two direct foundation compiles. For the direct
foundation test compile, that repository read closure is
exactly the foundation test; host `native_c2b2a.rs`; payload `contract.rs` and `protocol.rs`;
`native_successor_semantic/acquisition.rs`; `native_vm.rs`; the workspace and evaluator manifests
plus workspace lock; and the six `include_bytes!` documents named by the test. Each regular-file
identity and required ancestor traversal is sealed before launch; every other repository read is
denied. Every profile denies writes to root-A and repository inputs. All Cargo output is redirected
to the validation target, so repository and nested `target/debug` remain absent.

The host-validation Cargo environment is built from empty and contains exactly `LANG=C`,
`LC_ALL=C`, `TZ=UTC`, `SOURCE_DATE_EPOCH=0`, `RUST_BACKTRACE=0`, `CARGO_INCREMENTAL=0`,
`CARGO_CACHE_RUSTC_INFO=0`, `CARGO_BUILD_JOBS=1`, `ROCKSDB_COMPILE=true`, the current
validation-root `home/active` as `HOME`, the current `cargo-home/active` as `CARGO_HOME`, and
`CARGO_TARGET_DIR`, the phase `TMPDIR`, exact sealed `PATH`, `RUSTC`, `RUSTDOC`, Apple `CC`, `CXX`
and `AR`, exact `CLANG_PATH` and `LIBCLANG_PATH`, plus
`DYLD_FALLBACK_LIBRARY_PATH=<acquisition-root>/host-validation/dylib-empty`.
`CARGO_TARGET_DIR` is exactly
`<host-validation-target>/payload-tests` for payload tests,
`<host-validation-target>/lints` for lints and `<host-validation-target>` for each direct compile
or run. `TMPDIR` always names that exact phase's temporary subtree. `LLVM_CONFIG_PATH` and every
unlisted variable are absent; every other `DYLD_*` variable is absent. Every top-level
host-validation Cargo child, direct rustc child and controller-launched foundation executable is
launched through the post-sandbox empty-environment wrapper with exactly that effective
environment. Cross-target
Clippy commands instead use the exact root-A qualification environment with `HOME`, `CARGO_HOME`,
`CARGO_TARGET_DIR` and `TMPDIR` replaced by those exact host-validation values and the same exact
`DYLD_FALLBACK_LIBRARY_PATH` sentinel added; every other entry remains byte-identical. The
sandbox executable
closure contains only the reviewed Cargo, rustc, rustdoc, rustfmt, clippy-driver, `cargo-clippy`,
`cargo-fmt`, host/cross compiler tools, locked build
scripts, Cargo-generated test executables and the two exact direct-rustc foundation executables
beneath the validation target. The exact sealed paths and hashes of both Cargo subcommand binaries
and their source-defined exec chain are mandatory.

Cargo- or sealed-subcommand-spawned rustc, rustdoc, build-script and test descendants are not
claimed to receive a byte-identical copy of the top-level environment. The complete environment
maps requested at each modeled exec boundary are instead closed, ordered derivations from that
exact environment and the reviewed Cargo, external-tool, jobserver and build-script reachability
manifests above. In particular, Cargo may prepend only its source-derived in-target native
directories, applicable root output, dependency output and exact sealed sysroot target-lib
directory to the one inherited `dylib-empty` entry. Because the inherited value is present and
nonempty, `$HOME/lib`, `/usr/local/lib` and `/usr/lib` must not be appended. Every resulting
dynamic-library path is an absolute path and is either a preflight-enumerated member of the sealed
tool closure or a source-derived member of that phase's authorized target subtree, except for the
exact empty sentinel; each is explicitly present in that descendant's read closure. A duplicate,
relative, empty, outside-closure or differently ordered entry is terminal. The derivation also
binds the exact source-defined `CARGO`, `CARGO_MANIFEST_DIR`, `CARGO_MANIFEST_PATH`, `CARGO_PKG_*`,
crate/target/build-script and wrapper variables, each modeled cwd and the typed non-authoritative
jobserver handoff, including its `CARGO_MAKEFLAGS` placeholder.
The payload has no manifest-configured environment entry; every build-script-emitted environment
entry and native search path must be independently recovered from sealed Cargo fingerprints and
output and match the source-derived table. Implementation preflight freezes the exact ordered
derivation algorithm, closed variable schema and path classifier. After each Cargo build, the
controller constructs the complete requested environment and cwd table for every modeled exec
class and invocation from the reviewed manifests, sealed configuration, fingerprints and outputs;
the build/source identity record binds those tables. This source-derived table constrains permitted
transitions but does not prove exec occurrence or claim a kernel history of post-exec environments.
`-vv` output is only corroboration; an unmodeled variable, source mutation site or path is terminal.

Every host-validation profile positively allows reads only of the exact root-A payload, directory
source and root configuration; the current validation `home/active` and `cargo-home/active`, the
sealed `dylib-empty` sentinel,
that phase's authorized target subtree and temp; the exact sealed foundation dependency or
executable subtree needed by that phase; the phase-specific repository closure above when
applicable; and the sealed tool/system
closure. A phase cannot read a peer host-validation target subtree unless the schedule below
expressly names it. Required ancestor traversal is allowed, but evidence, unrelated root-A paths,
every peer root, ambient homes and all other repository paths remain unreadable to children. A
write permission does not stand in for this explicit read closure.

The six phases have separate, hash-bound Seatbelt profiles and separate policy probes:

1. Before `host-validation-payload-tests`, the controller creates empty
   `<host-validation-target>/payload-tests`. The profile permits writes only to validation `home`,
   `cargo-home` active roots, `payload-tests` and its phase temp; `lints` and `foundation` remain
   absent. After
   the final test child, the controller seals the complete `payload-tests` tree before dependency
   selection.
2. Before `host-validation-foundation-test-compile`, the controller creates and seals the exact
   `foundation/deps` closure below and creates empty `foundation/test-build`. The profile denies
   every write to `deps` and the rest of `target`, and permits content and directory-entry writes
   only beneath `test-build`, inside the two active ambient roots and in its phase temp. An
   overriding literal-root rule denies the
   compiler and every descendant any chmod, chown, ownership, flag, extended-attribute or ACL
   mutation of `test-build` itself. After the one compile, `test-build` is sealed. The
   `foundation` parent remains mode
   `0700` with exactly the two children `deps` and `test-build`; that state is bound below.
3. `host-validation-foundation-test-run` denies every persistent write, including all of
   `foundation`, and permits only the two active ambient roots and its phase temp. Both test-binary
   executions share that policy and
   preserve the exact two-child parent state.
4. Before `host-validation-foundation-bin-compile`, the controller revalidates the two-child parent
   state, creates empty `foundation/bin-build` as the sole directory-entry delta, verifies the
   exact three-child state and changes the parent mode to `0500` before the policy probe. The
   profile denies every write to `deps`, `test-build`, the parent and the rest of `target`, and
   permits content and directory-entry writes only beneath `bin-build`, inside the two active
   ambient roots and in its phase temp. An
   overriding literal-root rule denies the compiler and every descendant any chmod, chown,
   ownership, flag, extended-attribute or ACL mutation of `bin-build` itself. After the one compile,
   `bin-build` is
   sealed.
5. `host-validation-foundation-bin-run` denies every persistent write, including all of
   `foundation`, and permits only the two active ambient roots and its phase temp. It preserves the
   final parent state below.
6. Before `host-validation-lints`, the controller creates empty
   `<host-validation-target>/lints`. The profile permits writes only to validation `home`,
   `cargo-home` active roots, `lints` and its phase temp. It denies reads and writes beneath
   `payload-tests` and
   has an overriding denial for every content, metadata and directory-entry write beneath
   `foundation`; the final parent state must remain identical around every lint child.

The controller captures and stores the final `deps` tree after sealing it and requires it to remain
byte-, mode- and entry-identical around every substantive child beginning with the test compile.
The test compile alone is allowed the exact absent-output-to-one-sealed-executable delta in the
otherwise empty `test-build`. Its invocation binds the complete empty pre-stream and the post-child
stream containing exactly one link-count-one mode-`0700` executable. The controller's only
authorized post-child mutation is to set that file to mode `0555` and the selected subtree root to
mode `0500`, after which it stores the final stream. Every later child must preserve it. The binary
compile alone has the corresponding exact delta and controller-only sealing transition in the
otherwise empty `bin-build`; its invocation binds both streams and the controller stores the final
stream. Every later child must preserve that tree too. For every applicable stream, the controller
also records device/inode identities in that phase's closed `result.json` observations and requires
all pre-existing entries to retain them. Any broader delta, any policy that permits a protected
subtree write outside its one named compile, any later changed tree or any unsealed executable is
terminal.

Each foundation compile `result.json` observation also contains exactly one
`output_root_states` array with exactly three ordered objects. Each object has ordered keys
`stage`, `device`, `inode`, `uid`, `gid`, `mode`, `xattrs`, `extended_acl`. The `stage` values are
`pre_child`, `post_child`, `final`; all integer values are unsigned; `xattrs` is exactly an empty
array; and `extended_acl` is false. `pre_child` and `post_child` must be byte-identical apart from
their stage values and bind the invoking uid/gid, mode `0700` and one unchanged device/inode for
the selected output-root descriptor. The observation is taken immediately around the one compile,
before any controller metadata mutation. `final` preserves the same identity, uid/gid, empty
xattrs and absent ACL and differs only by mode `0500`, after the controller's named seal. Every
later phase and `pre-runtime-finalize` re-observes that exact final root state. A child-visible
policy that can mutate the literal root's metadata, or any pre/post/final mismatch, is terminal.

The parent inventory is separately exact. The test-compile `result.json` binds a
`foundation_parent_after_test` object with ordered keys `device`, `inode`, `uid`, `gid`, `mode`,
`xattrs`, `extended_acl`, `entries`. Its four numeric identity and ownership values are unsigned,
its uid/gid equal the invoking uid/gid, `mode` is the string `0700`, `xattrs` is an empty array,
`extended_acl` is false, and `entries` contains
exactly two objects in raw-name order: `deps`, then `test-build`. Each entry object has ordered keys
`name`, `device`, `inode`, `uid`, `gid`, `mode`, `xattrs`, `extended_acl`, `kind`; its numeric
identities and ownership are unsigned, its uid/gid equal the invoking uid/gid, its mode is `0500`,
its `xattrs` is an empty array, its `extended_acl` is false and its kind is `directory`. The
test-run phase requires byte-identical canonical state before and after both children.

The bin-compile `result.json` binds a `foundation_parent_states` array with exactly three ordered
objects named by `stage`: `before_bin_create`, `after_bin_create`, `final`. Each state otherwise
uses the parent-object fields above in the same order after `stage`. The first must be
byte-identical to `foundation_parent_after_test` after projecting away its `stage` field. The
second preserves the parent device/inode, ownership, empty xattrs and absent ACL, changes its mode
only to `0500`, preserves both existing child identities and metadata and adds exactly one
mode-`0700`, invoking-uid/gid, empty-xattr, ACL-free directory
named `bin-build`; its entry order is `bin-build`, `deps`, `test-build`. The final state preserves
that parent and entry identity/metadata set, with the sole mode change being `bin-build` to `0500`
after its
executable is sealed. The controller enumerates each state no-follow from one held parent
descriptor. Every later host-validation child and `pre-runtime-finalize` must re-observe the exact
final state before and after its work. An extra, missing, replaced or reordered entry, changed
identity, ownership, mode, xattr or ACL state, or a writable final parent is terminal.

The test-run, binary-run and lint `result.json` observations each contain a required
`foundation_parent_checks` array with one element per substantive child: respectively two, one and
eight elements in invocation order. Each element has ordered keys `ordinal`, `pre`, `post`;
`ordinal` matches the invocation ordinal and each state uses exactly the parent-object schema above.
For test-run both states equal `foundation_parent_after_test`; for binary-run and lints both equal
the bin-compile `final` state. `pre-runtime-finalize` binds one final re-observation plus the exact
hashes of every result carrying these states.

From cwd `<root-A>/payload`, the controller executes the following closed schedule. For each of the
five selectors below it runs the command first with final argv `-- --test-threads=1`, then with
final argv `-- --test-threads=4`; every other Cargo argument shown is literal and ordered:

```text
<cargo> test -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --no-default-features --lib
<cargo> test -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --no-default-features --test contract
<cargo> test -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --no-default-features --test protocol
<cargo> test -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --no-default-features --test seccomp
<cargo> test -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --no-default-features --features collector --test rocks
```

The expected passed-test counts per invocation are respectively 5, 15, 17, 12 and 11, with zero
failed, ignored or filtered tests. After the collector-backed test build, the controller enumerates
the sealed standalone Cargo fingerprints, dep-info and host artifacts. For each of `sha2 0.10.9`,
`serde 1.0.229` and `thiserror 2.0.20`, it selects one host library unit by exact registry source,
package/version, crate target, host compile kind, normal target-dependency role, non-test debug
profile and the feature set derived by pinned Cargo for the collector-backed `rocks` test. If more
than one byte-distinct unit has that complete semantic key, the raw-UTF-8-minimum artifact basename
is the mandatory tie-break; all rejected candidates and hashes remain evidence. Starting at those
three selected units, it recursively follows only the normal-library and proc-macro fingerprint
edges required for rustc metadata resolution, including the corresponding `thiserror-impl 2.0.20`
proc macro. Custom-build units and build-only dependencies are not copied; their fingerprints,
outputs and completed execution were already sealed and remain evidence. An unknown, missing or
ambiguous required edge is terminal. The implementation-preflight review must bind this parser and
selection algorithm to the pinned Cargo fingerprint format.

Every direct rustc invocation and every controller-launched foundation test or binary invocation
uses cwd exactly `<root-A>/payload`. That sealed directory and its required ancestor traversal are
already in each applicable profile's read closure; no direct invocation may select its output
directory, phase temp, repository root or ambient cwd instead. The canonical `invocation.json`
must contain that exact cwd for all five direct invocations: one test compile, two test runs, one
binary compile and one binary run.

At the start of `host-validation-foundation-test-compile` and before its policy probe, the
controller creates the absent `<host-validation-target>/foundation` parent plus its absent `deps`
and `test-build` children with create-new semantics. It copies only the selected `.rlib`,
proc-macro dynamic library and any normal-library metadata artifact actually required by that
self-consistent closure into `deps`.
It preserves unique basenames but never inode identity. A basename collision or other file type is
terminal. Every copy is regular, link-count one, byte-identical to its sealed source and hash-bound;
the directory contains no extra artifact. It becomes read-only before rustc. The direct `--extern`
paths below identify the selected copies, and transitive rustc resolution is confined to that one
directory. The controller then compiles the protected foundation test with this exact substantive
argv
after path substitution:

```text
<rustc> --edition=2021 --test \
  <repository>/engram-eval/tests/native_c2b2a_foundation.rs \
  --crate-name native_c2b2a_foundation --extern sha2=<exact-sha2-rlib> \
  --extern serde=<exact-serde-rlib> \
  --extern thiserror=<exact-thiserror-rlib> \
  -L dependency=<host-validation-target>/foundation/deps \
  --remap-path-prefix <repository>=/c2b2a-repository \
  -D warnings \
  -o <host-validation-target>/foundation/test-build/native_c2b2a_foundation
```

The selected proc-macro library must be in that exact dependency directory, and no other artifact
may satisfy resolution. Immediately before launch, the output path must be absent and the output
subtree must be empty. After exit, the controller must observe exactly the one new link-count-one
regular executable specified above. This is a pre/post state claim, not a claim that rustc or its
linker used `O_EXCL`, nor a complete history of transient files inside the write-confined subtree.
After the successful compile, the controller seals `test-build`, completes and seals the compile
phase evidence, then enters
`host-validation-foundation-test-run`. That phase runs the sealed test binary once with
`--test-threads=1` and once with `--test-threads=4`; each must report exactly 42 passed and zero
failed, ignored or filtered tests.

After sealing the test-run evidence, the controller enters
`host-validation-foundation-bin-compile`, creates the absent `foundation/bin-build` directory
before that phase's policy probe and compiles the protected no-argument host-foundation binary with
the same three direct `--extern` arguments, dependency search path, remap and warning denial,
substituting this exact source/output argv:

```text
<rustc> --edition=2021 <repository>/engram-eval/src/bin/native-c2b2a.rs \
  --crate-name native_c2b2a \
  --extern sha2=<exact-sha2-rlib> --extern serde=<exact-serde-rlib> \
  --extern thiserror=<exact-thiserror-rlib> \
  -L dependency=<host-validation-target>/foundation/deps \
  --remap-path-prefix <repository>=/c2b2a-repository \
  -D warnings -o <host-validation-target>/foundation/bin-build/native-c2b2a
```

That source, host module and their exact ancestor traversals are its whole repository read closure.
Immediately before launch, the output path must be absent and the output subtree must be empty.
After exit, the controller must observe exactly the one new link-count-one regular executable
specified above. This is the same bounded pre/post state claim and makes no `O_EXCL` or complete
transient-file-history claim. After the successful compile, the controller seals `bin-build`,
completes and seals the compile phase evidence, then enters
`host-validation-foundation-bin-run`. One no-argument execution of that sealed binary must exit
zero, emit exactly the five LF-terminated lines `foundation_valid=true`, `provider_free=true`,
`runtime_implemented=false`, `c2b2a_acceptance_proven=false` and
`flagship_goal_complete=false` in that order, and emit empty stderr.

After sealing the binary-run evidence, the controller enters `host-validation-lints` and runs these
exact substantive argv in order:

```text
<cargo> clippy -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --no-default-features --lib -- -D warnings
<cargo> clippy -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --no-default-features --test contract -- -D warnings
<cargo> clippy -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --no-default-features --test protocol -- -D warnings
<cargo> clippy -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --no-default-features --test seccomp -- -D warnings
<cargo> clippy -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --no-default-features --features collector --test rocks -- -D warnings
<cargo> clippy -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --target aarch64-unknown-linux-musl --release --no-default-features \
  --features supervisor --bin supervisor -- -D warnings
<cargo> clippy -vv --frozen --offline --manifest-path <root-A>/payload/Cargo.toml \
  --target aarch64-unknown-linux-musl --release --no-default-features \
  --features collector --bin collector -- -D warnings
<cargo> fmt --manifest-path <root-A>/payload/Cargo.toml --all -- --check
```

It then performs its own trailing-whitespace, final-LF, bounded-path and changed-file checks. Each
build, Clippy or test child has a 7,200-second deadline;
rustfmt has 300 seconds and each direct foundation compile/run has 1,800 seconds. A command absent
from the preflight-bound expanded argv table, any warning, count mismatch, source mutation or
unexpected executable is terminal.

These commands, and only these commands, are the complete repair-scoped C2B2a matrix named by this
supplement. They exercise every source changed or added by the repair plus the protected host
foundation that consumes the frozen contract. They are not evidence for unrelated evaluator or
workspace targets.

`pre-runtime-finalize` has no child process or network and runs once after all preceding phase
directories are sealed. Within 900 seconds it rehashes every input and every previously sealed
evidence leaf, explicitly excluding its own not-yet-created result; checks A/B and all gate-specific
equalities; proves the exact command/phase order; and writes its sole canonical `result.json`. Its
ordered `observations` object contains keys `gate_01` through `gate_21`; each
value contains only `passed=true` plus the sorted evidence-path/hash bindings that prove the
corresponding gate below. A missing binding, `false`, unknown key or evidence cycle is terminal.
Only after that result is sealed may the acquisition evidence parent become mode `0500` or a
build/source identity record be proposed.

## Provider-free verification gates

Before the exact build/source identity record may be proposed, all of these must pass:

1. exact predecessor document, review and protected-input hashes;
2. exact post-edit manifest, lock, payload configuration, dependency test, changed patch, root
   configuration and other five new support-file hashes;
3. structural proof against the pre-implementation snapshot that this work caused no mutation
   outside the bounded payload and audit-record paths while preserving user-owned changes;
4. witnessed seed-lock fetch, exact Cargo/tool identity and complete acquisition transcript;
5. exact lock-only root-edge delta and byte-identical provisional, in-memory reconstructed and
   final archive-manifest representations;
6. archive-manifest syntax, package coverage, sizes, checksums, sealed staging-tree identity and
   sealed-bundle tree identity;
7. final lock-root/repository/A/B payload-tree streams, digests, independent inodes and
   post-command immutability;
8. three independent safe extractions, independently reconstructed authenticated prepatch streams,
   exact postpatch seals and A/B post-build unchanged-tree seals;
9. exact root-config bytes and directory-source resolution retaining registry identities with the
   payload as the sole path node;
10. graph v2 rows, counts, bytes, SHA-256, all source seals, feature sets and denied sentinels;
11. all three `getrandom` backend proofs and all three exact inert-ring file identities;
12. both Rocks patched sources, complete production caller graph, `env.cc` fallback edge, object,
    musl, success-path and failure-branch proofs;
13. exact outer bootstrap and post-sandbox first-executable environments plus independently derived
    requested-at-exec child launch-state and effective Rust, C and C++ argument constraints with
    full remapping, typed non-authoritative jobserver values and no inferred query occurrence;
14. sandbox profile/tool and post-sandbox environment-trampoline identities, exact top-level
    inherited descriptors, pipe/process-group cleanup, network denial and peer/outside-root
    write-denial evidence;
15. exact A/B command order plus supervisor, collector, build-ID, static-PIE and bundle equality;
16. final dependency, symbol, string, relocation, disassembly and source-reachability firewalls;
17. both entropy-probe source identities plus byte-identical A/B object, link-map and executable
    build identities; Linux runtime mode results are expressly deferred;
18. every repair-scoped C2B2a unit/integration, protocol, fixture, bounds and negative-trait test;
19. strict Clippy, rustfmt and whitespace checks with no unreviewed warning;
20. all ten nested serial/parallel invocations, both direct 42-test repetitions, the direct
    host-foundation build/run and every strict check in the complete repair-scoped matrix; and
21. absent repository and nested `target/debug`, positive disk reserve and no source mutation.

After that identity receives an independent exact-SHA `P0=0`/`P1=0` review, it may authorize only
preparation of the exact provider-free Linux-runner freeze. The runner freeze itself requires a
separate exact-SHA `P0=0`/`P1=0` review before the runner starts. The three modes then execute once
under that exact runner. Their syscall/signal/core evidence, runner identity, probe-binary hashes
and all pre/post seals are bound in the runtime-qualification record and independently reviewed.
Three fresh independent exact-source/binary/runtime audits must report `P0=0` and `P1=0` before
C2B2a acceptance. No runtime observation can be backfilled into the earlier build/source identity.

The current direct-dependency check, a clean string scan, one compilation or one build root is
insufficient. No result may be repaired by replaying only its failed portion under a changed input.

## Research-only diagnostics

The following observations motivated this freeze but are not accepted identities and cannot satisfy
any gate:

```text
exploratory collector SHA-256
  f1b76a0cb1d569e611af74140898dcd6e0e639bd31752fb6ce8fa9a2fec016ab

patched diagnostic collector SHA-256
  a82c262ae1674865e7d6a298c81dd1a3cb502687649f2a85f3654270cf6d2a2b

patched diagnostic GNU build ID
  763559eab6700474303a6e84940b13fb98b09837
```

The diagnostic collector dead-stripped relevant final workload paths, preceded the
`getrandom 0.2.17` alias, and did not establish the final graph, source seal, supervisor identity or
runtime containment. Its byte equality is feasibility evidence only. Ambient Cargo registry trees
and any telemetry emitted by host instrumentation are likewise research observations, not build
authority.

The rejected one-file patch SHA-256
`3c0127f47f2f63990d1c3387d991bc24afd1ec8d4969a1f87f5060b8da03d09d` changed only
`unique_id_gen.cc`. It left the production `/proc/sys/kernel/random/uuid` path reachable through
`port_posix.cc`, so no build or probe made with it can satisfy entropy closure.

## Review and activation rule

This document must first receive an independent exact-SHA review with `P0=0` and `P1=0`. After
that review, implementation is limited to the exact edit/support boundary and procedures above.
The five hand-authored support files then require the separate implementation-preflight review
before any controller or provisioner invocation.

The resulting host candidate must then receive the separate build/source identity record containing
every observed post-edit/source/graph/build/probe-binary identity and an independent exact-SHA
review with `P0=0` and `P1=0`. Only that review may authorize preparation—not execution—of the
provider-free Linux-runner freeze. Only the independent exact-SHA review of that runner freeze may
authorize one runtime. The runtime-qualification record and its independent review must then bind
the three exact results. No earlier review alone authorizes a VM run or C2B2a acceptance.

Any source, patch, archive, manifest, lock, configuration, support file, tool, flag, graph, tree,
ELF, build ID, classifier or environment change invalidates downstream evidence and returns to a
new reviewed identity. No provider, datastore, live adapter, authentication, user-data or flagship
claim is authorized by this repair.
