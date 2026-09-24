# Native strict successor Stage C2B2a — dependency-firewall addendum review

## Disposition

The exact dependency-firewall addendum below is accepted as a narrow normative clarification of
the already accepted C2B2a design:

```text
evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DEPENDENCY_FIREWALL_ADDENDUM_FROZEN_2026-09-07.md
SHA-256 f31df61c5f4def5270b3aa5915aeeb74fe59cfde24d9e7566d5c9a24c61977a8
lines   222
```

It supersedes only the dependency/source-presence interpretations that it names. It does not edit
or replace the original 1,451-line freeze, whose accepted identity remains:

```text
SHA-256 85312b0286080ecbbab94b5236003a42f281b1122565016d96bb20f9572063e3
```

This review accepts a design boundary only. It does not accept the current implementation, a
payload binary, transitive-source patch, guest image, run plan, VM run, RocksDB result, persistence
claim, C2B2b, harness behavior or the Engram flagship goal.

The authority here is explicitly an agent-observed exact-artifact review, not human or manual
approval. It permits continued provider-free implementation under the standing user authorization;
it cannot manufacture human-review provenance or waive any later execution gate.

## Independent review

One independent exact-SHA red team re-read the original freeze, the candidate addendum, the nested
manifest and lockfile, target-filtered Cargo 1.93 metadata, and the extracted Core 2.6.0 manifests.
The final review returned P0=0 and P1=0.

It independently reproduced the addendum's complete canonical active-graph identity:

```text
target       aarch64-unknown-linux-musl
nodes        392
edges        948
rows         1344
bytes        225759
SHA-256      21f27c741fa7eddc881b5917626f14008464503a027f3ae7a9849c9b01f5b9a1
```

The same review verified:

- Core's sole active feature is `kv-rocksdb`;
- every structural package/version/checksum/feature row is exact;
- requested and unified Tokio feature sets are distinct and exact;
- lock-only WebAssembly rows are explicitly sentinels rather than an exhaustive list;
- the named transport, TLS, native-WebSocket, remote-JWKS, scripting, ML, provider and non-Rocks
  backend packages/features are absent from the filtered active graph;
- the complete graph digest, rather than a package-name heuristic, fails closed on every unreviewed
  graph change; and
- payload-owned source, source-reachability, final-ELF, entropy, environment, seccomp and runtime
  capability gates remain mandatory.

## Review ratchet

No verdict transferred across edits. The material review identities were:

| Candidate SHA-256 | P0 | P1 | Disposition |
|---|---:|---:|---|
| `615af1f434084f0c027b1f0be70e4cf6c30369fb499d9d3de55ed3134c251a95` | 2 | 4 | rejected: contradictory source scope and an open graph policy |
| `9608dc9238cf959840bf52f1967d3a8424382d2ac30203d8a171456656324982` | 0 | 2 | rejected: canonical edge wording and feature mapping were incomplete |
| `da76822f439d43477c1861dc17687b918469186392d2b027722b654bbdebbcd4` | 0 | 1 | rejected: non-Rocks backend package identities were not explicit |
| `f31df61c5f4def5270b3aa5915aeeb74fe59cfde24d9e7566d5c9a24c61977a8` | 0 | 0 | accepted as the narrow clarification |

## Remaining implementation boundary

The addendum's ten machine-evidence requirements are not yet all implemented. In particular, a
direct-manifest test is not a substitute for reproducing the canonical target graph, checking
source/query reachability, inspecting one exact accepted static ELF, and executing the installed
seccomp negative probes. The separate stock-RocksDB random-device fallback blocker also remains
outside this addendum and requires its own frozen successor before any accepted build.

All other original provider-free evidence, deterministic-build, fresh-source, guest, runtime and
three-audit gates remain open. No provider, VM, daemon, adapter or datastore operation was authorized
or performed by this review.
