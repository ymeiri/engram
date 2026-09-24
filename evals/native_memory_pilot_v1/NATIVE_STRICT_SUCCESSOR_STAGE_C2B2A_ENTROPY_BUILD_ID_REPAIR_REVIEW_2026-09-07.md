# Native strict successor Stage C2B2a — entropy and build-ID repair review

Date: 2026-09-07 (Asia/Jerusalem)

## Disposition

The exact entropy/build-ID repair design below is accepted as implementation-safe within its
stated pre-implementation boundary:

```text
evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_REPAIR_FROZEN_2026-09-07.md
SHA-256 4a4dec13b99c88694625b52538b246323f69465741300c7cb9b8394cbda83e02
lines   2653
bytes   179776
```

This review authorizes only the bounded source implementation, support-file creation, archive
provisioning and provider-free qualification work described by that exact digest. It does not
accept an implementation, generated support file, edited payload, archive bundle, source-universe
manifest, launch-influence manifest, Cargo target graph, ELF, build ID, Linux runner, VM result,
RocksDB result, persistence result, C2B2a acceptance, harness behavior or the Engram flagship goal.

The authority here is an agent-observed exact-artifact review, not human or manual-review
provenance. It cannot manufacture human approval, transfer a verdict to changed bytes or waive a
later implementation, build/source, runner or runtime review gate.

## Independent exact-SHA review

Three independent reviewers re-read the complete 2,653-line artifact, verified the same SHA-256,
line count and byte count before and after review, made no edits and returned no blocker:

| Independent review | P0 | P1 | Verdict |
|---|---:|---:|---|
| full lifecycle, security, authority and evidence review | 0 | 0 | accept/freeze |
| controlling Cargo closure and host-validation review | 0 | 0 | accept/freeze |
| source-universe, xattr, tool/system-seal and trampoline review | 0 | 0 | accept/freeze |

The reviewers independently reproduced or checked the material new facts:

- Cargo commit `083ac5135f967fd9dc906ab057a2315861c7a80d` has root Git tree
  `ad1967c7c91181b65043fc8792002d1ff07a63ed` and exactly 4,460 recursive non-root
  entries: 1,560 trees and 2,900 blobs, with blob modes split as 2,774 mode-`100644`, seven
  mode-`100755` and 119 mode-`120000`;
- all 33 non-exhaustive semantic-anchor paths match their frozen byte counts, LF line counts and
  SHA-256 identities;
- the complete 29,013-byte `jobserver 0.1.34` archive matches SHA-256
  `9afb3de4395d6b3e67a780b6de64b51c978ecf11cb9a462c66be7d4ca9039d33`, and its
  `src/lib.rs` and `src/unix.rs` anchors match the identities in the freeze;
- the whole-source authority plus independent whole-universe sink inventory closes the former
  partial Cargo child-process derivation model;
- typed jobserver values remain explicitly non-authoritative, while source-created
  `CARGO_MAKEFLAGS` is no longer confused with forbidden ambient input;
- `CARGO_CACHE_RUSTC_INFO=0`, exact zero-result `.rustc_info.json` scans and the no-query-occurrence
  nonclaim close cross-invocation compiler-query cache influence;
- `cargo clippy` and `cargo fmt` external-subcommand chains remain inside one controller-launched
  substantive invocation and one active-home rotation lifecycle;
- requested-at-exec tables constrain permitted transitions without claiming a complete per-PID,
  descriptor, exec or loader-preserved environment history; and
- source universes remain preflight-only evidence, add no runtime child/controller input and add no
  run-evidence pathname.

The existing post-sandbox environment trampoline, rotating `HOME`/`CARGO_HOME`, foundation output
and parent-state controls, direct-invocation cwd, canonical xattr ordering and bounds, hardlink
accounting, disjoint tool/system roots and sole target-absolute AArch64-musl symlink exception
remain coherent under the replacement design.

## Review ratchet

No verdict transferred across edits. In particular, candidate SHA-256
`64a0506be7c27e65f65383b9af05c72ba38b46c9cbf9e7ac79c305fbbc1efb04`
(2,492 lines, 166,021 bytes) remains rejected and authorizes nothing. Two reviews found no P0/P1,
but the controlling host-validation review found one P1: its seven-file Cargo environment source
set omitted reachable dynamic-path, compiler-query, jobserver, test and external-subcommand sites.

The accepted digest replaces that incomplete claim with complete pinned-tree byte authority, a
conservative per-command launch-influence manifest, an independently generated whole-universe sink
inventory, full external-tool/build-script source universes, typed runtime-only values and explicit
cache-state handling. All three fresh reviewers evaluated the replacement bytes; earlier clean
verdicts were not reused.

## Local checkpoint evidence

At acceptance, the following protected repository inputs retained their frozen SHA-256 identities:

| Input | SHA-256 |
|---|---|
| workspace `Cargo.toml` | `ec61531f8ffd401211f649c7b91e28a4ee5f0fa1a0fe2f199c63a841c9724f5a` |
| workspace `Cargo.lock` | `0eb924bb9008bb417cb9990081df8429520131aa6b3f7367866c5b805ec474de` |
| `engram-eval/Cargo.toml` | `4176c6a4886492f7d5adaa0bdc3134d7e04bfc8d1536de4d5eb8d4b8985ce2bb` |
| `engram-eval/src/native_vm.rs` | `7236583a702a85998e1be5a4c32950df239d32a384d4bf82e2b9dc961c90b69a` |
| `engram-eval/src/native_c2b2a.rs` | `3501336994fa7ede67c51c1fd0b7b6dc0e94595dce058d434d63edadd0122904` |
| `engram-eval/src/bin/native-c2b2a.rs` | `f2c3704cae113330674cea0d1724ee7befd6c5c8c548e7963289b55c6f90ac27` |
| `engram-eval/tests/native_c2b2a_foundation.rs` | `4f7bd87691c18d5ce2d82634415ad6635cd08d0e6fa7ad92388daaf21cfa1638` |

The retained RocksDB repair patch remains:

```text
engram-eval/native-c2b2a-payload/patches/
surrealdb-librocksdb-sys-0.17.3-rocksdb-getentropy.patch
SHA-256 c82a6d05411362963eb8049cc6a0ecb563562bd1d35e14bfe892f39db936fcfb
lines   64
bytes   1565
```

Its patched-tree reverse dry run exited zero and a forced forward dry run exited nonzero, proving
the retained tree already contains exactly the patch rather than accepting an accidental second
application. Repository and nested `target/debug` directory count was zero. The filesystem reported
259 GiB available at the final local checkpoint. Formatting/trailing-whitespace validation of the
accepted freeze was clean.

The review round performed only local/official-source reads, hashes, non-mutating source and patch
checks and agent analysis. It started no build, provider, authentication, VM, container, daemon,
adapter, datastore, payload or live Engram operation.

## Next boundary

Implementation must remain inside the accepted freeze's exact edit and support-file boundary. It
must preserve user-owned worktree changes, the protected identities above and the absent repository
`target/debug` state. Before any Cargo build, implementation preflight must bind and independently
review the controller/checker/provisioner implementation, complete source universes, canonical
source and sink manifests, binary provenance, exact tool/system roots, environments, profiles and
all other preflight gates at `P0=0`/`P1=0`.

Provider-free build/source identity and review come only after that preflight. A Linux runner freeze
and review come only after build/source acceptance. No VM or Linux probe may start before those
separate gates, and no later stage may claim C2B2a or flagship completion without its own exact
evidence and review.
