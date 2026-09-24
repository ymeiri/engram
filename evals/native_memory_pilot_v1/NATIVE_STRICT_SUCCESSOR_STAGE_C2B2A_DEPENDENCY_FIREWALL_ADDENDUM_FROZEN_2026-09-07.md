# Native strict successor Stage C2B2a — dependency-firewall addendum

## Disposition

This is a frozen candidate addendum, not an accepted implementation or acceptance record. It must
receive an independent review bound to its exact SHA-256 before it can authorize a C2B2a build or
run. It does not accept a payload binary, guest image, run plan, VM run, RocksDB result, persistence
claim, C2B2b, harness behavior or the flagship Engram goal.

This addendum narrowly supersedes only:

- the package/dependency interpretation of lines 352–357;
- the dependency-name interpretation of lines 1336–1337, while preserving without qualification
  the `ROCKSDB_COMPILE`, external-input, C/C++ flag, discovery and build-script rules at lines
  1334–1341; and
- the upstream dependency/source-presence interpretation of the firewall requirements at lines
  1383 and 1400.

Those references are to the accepted 1,451-line C2B2a freeze:

```text
evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_CALIBRATION_CONTAINMENT_FROZEN_2026-09-07.md
SHA-256 85312b0286080ecbbab94b5236003a42f281b1122565016d96bb20f9572063e3
lines   1451
```

Every other frozen requirement remains unchanged. The upstream stock-Core archive may contain only
the exact dormant structure bound below, but the payload-owned source firewall, source-reachability
firewall and final-binary firewall remain mandatory. This addendum waives no entropy, filesystem,
query, binary, environment, seccomp, reproducibility, guest, cleanup or audit gate. If stock Core
cannot satisfy any independent gate, the freeze's narrow-patch fallback remains mandatory.

## Why clarification is necessary

For stock `surrealdb-core 2.6.0`, the literal interpretation that HTTP-, JWKS- or ML-adjacent package
names and source structures must be absent from the complete archive or lockfile is impossible even
when Core is built with `default-features = false` and only `kv-rocksdb`. Stock Core unconditionally
activates generic protocol, local authentication and numerical packages. Its all-target lockfile
also contains target-inactive WebAssembly records.

Cargo lock membership, target activation, feature activation, compilation, link retention, source
reachability, query reachability and effective runtime capability are distinct claims. The frozen
firewall is therefore interpreted as an exact target-graph and effective-capability firewall, not
as an imprecise package-name substring ban.

## Exact source and resolution authority

The stock Core input is bound as follows:

| Input | SHA-256 |
|---|---|
| `surrealdb-core-2.6.0.crate` | `c48e42c81713be2f9b3dae64328999eafe8b8060dd584059445a908748b39787` |
| extracted normalized `surrealdb-core-2.6.0/Cargo.toml` | `b51344d84c1b3550ba82e43dd85c708cefcc780a4906ff03fe43d2e1e221e8db` |
| extracted `surrealdb-core-2.6.0/Cargo.toml.orig` | `93cbee9909d7cf16f93149b38b3a755ce54959c4749c7c2fd7026ae2fc9afe09` |

The candidate payload inputs at the time of this addendum are:

| Input | SHA-256 |
|---|---|
| `engram-eval/native-c2b2a-payload/Cargo.toml` | `3b28236b23a2b1116bcbbd696d9d6c0c78a707b6a2de791933c73277e5fb7ff8` |
| `engram-eval/native-c2b2a-payload/Cargo.lock` | `40a484a61cd15e2555e0a18bd532a8229d78ab4144945b2fe56984c271e7f19b` |

Those candidate hashes are evidence, not permanent authorization: any later payload edit requires a
new exact hash and review. The collector manifest may directly depend only on its exact five
declared packages: `getrandom`, `libc`, `sha2`, optional `surrealdb-core`, and optional `tokio`.

Pinned Cargo 1.93.0 target-filtered resolution for `aarch64-unknown-linux-musl`, with
`--locked --offline --no-default-features --features collector`, must activate exactly
`kv-rocksdb` on `surrealdb-core`. The complete active graph, not the illustrative table below, is
closed by this canonical identity:

```text
schema       c2b2a-target-active-graph-v1
target       aarch64-unknown-linux-musl
cargo        1.93.0 (083ac5135 2025-12-15)
nodes        392
edges        948
rows         1344
bytes        225759
SHA-256      21f27c741fa7eddc881b5917626f14008464503a027f3ae7a9849c9b01f5b9a1
```

The canonical byte stream begins with four LF-terminated rows:

```text
schema=c2b2a-target-active-graph-v1
target=aarch64-unknown-linux-musl
cargo=1.93.0 (083ac5135 2025-12-15)
root=engram-native-c2b2a-payload@0.0.0
```

For every target-active Cargo resolve node, sorted by the node reference defined below, it then
contains one LF-terminated node row. For every `dep_kinds[]` record emitted beneath that node by the
already target-filtered Cargo metadata, sorted by dependency name, destination reference, kind and
target, it immediately contains one LF-terminated edge row. The serializer does not evaluate a
target expression a second time; it preserves every emitted target string verbatim, including
records whose target text names another platform. Every sort is ascending raw UTF-8 byte order.

```text
node<TAB>reference<TAB>registry-checksum-or-"-"<TAB>comma-sorted-active-features
edge<TAB>source-reference<TAB>dependency-name<TAB>destination-reference<TAB>kind<TAB>target
```

A reference is `name@version|source`; the sole root path source is normalized exactly to
`path:engram-native-c2b2a-payload`, and registry source strings are preserved verbatim. A missing
dependency kind is `normal`; a missing target is `*`. The final row also ends in LF. Checksums come
from the exact lockfile. Any row, count, byte-length or digest mismatch is terminal and requires a
new review.

The following high-risk active rows explain unavoidable stock-Core structure; they are not a
substitute for or an exhaustive listing of the canonical graph:

| Package | Version | Registry checksum | Exact active features |
|---|---:|---|---|
| `async-graphql` | 7.2.1 | `1057a9f7ccf2404d94571dec3451ade1cb524790df6f1ada0d19c2a49f6b0f40` | `dynamic-schema` |
| `http` | 1.5.0 | `918d3568bebf352712bc2ef3d46a8bcf1a75b373be6539de198e9105cbbf9ce0` | `default,std` |
| `jsonwebtoken` | 9.3.1 | `5a87cc7a48537badeae96744432de36f4be2b4a34a05a5ef32e9dd8a1c169dde` | `default,pem,simple_asn1,use_pem` |
| `object_store` | 0.12.5 | `fbfbfff40aeccab00ec8a910b57ca8ecf4319b335c542f2edcd19dd25a1e2a00` | `default,fs,walkdir` |
| `linfa-linalg` | 0.1.0 | `56e7562b41c8876d3367897067013bb2884cc78e6893f092ecd26b305176ac82` | `default,iterative,rand` |
| `ndarray` | 0.15.6 | `adb12d4e967ec485a5f71c6311fe28158e9d6f4bc4a447b474184d0f91a8fa32` | `approx,default,std` |
| `ndarray-stats` | 0.5.1 | `af5a8477ac96877b5bd1fd67e0c28736c12943aba24eda92b127e036b0c8f400` | none |

Their accepted meanings are narrow:

- `http` supplies typed message/header structure, not a client, server or transport;
- `jsonwebtoken` supplies local JWT/PEM cryptography, not remote JWKS retrieval;
- `object_store` is local-only and may expose its in-memory, buffered, prefix, registry, throttle
  and filesystem implementations, with no cloud, HTTP, AWS, Azure or GCP feature;
- `async-graphql` supplies schema/protocol structure, not a listener or client; and
- `linfa-linalg`, `ndarray` and `ndarray-stats` supply numerical/vector-index structure, not
  SurrealML model execution.

The exact active Rocks closure remains:

| Package | Version | Exact active features |
|---|---:|---|
| `surrealdb-rocksdb` | `0.24.0-surreal.1` | `bindgen-runtime,bzip2,default,lz4,snappy,zlib,zstd` |
| `surrealdb-librocksdb-sys` | `0.17.3+10.6.2` | `bindgen-runtime,bzip2,bzip2-sys,libz-sys,lz4,lz4-sys,snappy,static,zlib,zstd,zstd-sys` |

The payload manifest directly requests Tokio features `rt,sync,time`. Cargo feature unification in
the exact canonical graph activates Tokio features
`bytes,default,fs,io-std,io-util,macros,rt,rt-multi-thread,sync,time,tokio-macros`.
`net`, `process` and `signal` remain absent. Both requested and unified sets are frozen; neither may
be described as a general minimal-Tokio claim.

The following rows are target-inactive lock sentinels, not an exhaustive list of every WebAssembly
record. Each must be absent from the filtered Linux graph. The exact whole-lock hash and canonical
active-graph digest, rather than this sentinel table, close the two sets:

| Package | Version | Registry checksum |
|---|---:|---|
| `ws_stream_wasm` | 0.7.5 | `6c173014acad22e83f16403ee360115b38846fe754e735c5d9d3803fe70c6abc` |
| `js-sys` | 0.3.104 | `0e0c1080212aad755ea003d18543e8768dd432c48819efd73a7bf1e39b7a5a3a` |
| `wasm-bindgen-futures` | 0.4.77 | `6b7777d5cc23d0e91404e53ce2d5e8ec7acae3026b16233dba62cd3246457950` |

## Denied capabilities and activation

The exceptions above authorize no HTTP client or server, socket transport, TLS, native WebSocket,
remote JWKS retrieval, scripting engine, SurrealML model execution, cloud object-store backend,
provider adapter or provider credential path.

The active Linux graph must match the exact canonical digest above. In addition, the sentinel
packages `reqwest`, `hyper`, `hyper-util`, `hyper-rustls`, `hyper-tls`, `rustls`, `native-tls`,
`openssl`, `tokio-rustls`, `tokio-tungstenite`, `tungstenite`, `surrealml-core`, `rquickjs`,
`rquickjs-core`, `rquickjs-macro`, high-level `surrealdb`, `surrealkv`, `foundationdb`,
`surrealdb-tikv-client` (Core dependency key `tikv`), `indxdb` and `surrealcs` must be absent. Core
features `http`, `jwks`, `ml`, `scripting`, `kv-mem`, `kv-surrealkv`, `kv-fdb`, `kv-tikv`,
`kv-indxdb`, `kv-surrealcs` and their implicit dependency features must be absent. These names are
diagnostic sentinels; the canonical graph identity and capability/source checks are the fail-closed
authority for every other package and path.

Dependency evidence is necessary but insufficient. Effective capability remains denied only by the
complete conjunction of:

1. fixed, hash-bound query ASTs with no transport, remote-authentication, ML, scripting, GraphQL,
   RPC, cloud object-store or dynamic-function path;
2. datastore construction with `Capabilities::none()`;
3. a payload-owned source firewall that introduces no denied path;
4. the cfg-selected target dependency graph above;
5. the accepted static collector ELF and exact environment;
6. containment installed before the first Core reference; and
7. live seccomp evidence that every frozen network syscall returns `EPERM`.

Dormant upstream source or a structural type crate is not by itself an effective capability. Any
reachable transport, provider, JWKS-fetch, model-execution or scripting path; any successful network
syscall; or any denied feature/package in the filtered Linux graph is terminal.

## Required machine evidence

Before acceptance, a provider-free checker must fail closed unless it proves all of the following:

1. the original freeze SHA-256 and 1,451-line count, plus this separately reviewed addendum's exact
   SHA-256;
2. the exact payload manifest/lock identities and every package/version/checksum tuple above;
3. pinned Cargo 1.93.0 metadata from the repository root using:

   ```text
   cargo +1.93.0 metadata \
     --manifest-path engram-eval/native-c2b2a-payload/Cargo.toml \
     --locked --offline --no-default-features --features collector \
     --filter-platform aarch64-unknown-linux-musl --format-version 1
   ```

4. exact canonical active-graph rows, counts, byte length and digest, exact Core, exception, Rocks
   and requested/unified Tokio feature sets, plus absence of every denied sentinel;
5. the exact lock-only WebAssembly rows above and their absence from the filtered Linux graph;
6. the freshly extracted Core archive and both manifest hashes, with exact mappings
   `http → reqwest`, `jwks → reqwest`, `ml → surrealml` and `scripting → js` proven inactive, and
   every separately listed denied KV feature and backend package proven absent;
7. payload-owned fixed queries and `Capabilities::none()` with no denied query/source path;
8. exact environment absence for proxy, certificate, JWKS, object-store, cloud and provider
   variables;
9. accepted-ELF static PIE identity, absent interpreter and empty `NEEDED` set, plus denied
   implementation import/symbol/string scans as corroborating evidence; and
10. installed-filter negative probes returning `EPERM` for all 18 frozen network syscalls in both
    bootstrap and steady states, plus the exact pre-steady syscall trace.

The current foundation's direct-dependency check alone is insufficient: it does not prove the
target-filtered graph, source reachability, accepted ELF or effective capability. No C2B2a
acceptance may rely on this addendum until all ten requirements are implemented, independently
reviewed and satisfied for one exact candidate identity.
