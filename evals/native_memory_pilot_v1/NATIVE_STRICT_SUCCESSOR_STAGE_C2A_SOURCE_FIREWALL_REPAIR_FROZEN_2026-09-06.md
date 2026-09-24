# Native strict successor Stage C2A — frozen source-firewall repair

Date: 2026-09-06 (Asia/Jerusalem)

## Status and reason

This narrow repair is frozen before implementation. A fresh independent exact-source audit agreed
that the current C2A production prefix contains no forbidden capability, but classified its
regression firewall as P1 because the test is a partial denylist rather than a closing proof over
the reviewed bytes. Other audits classified the same weakness as P2 or found no issue. Acceptance
is conservatively withheld until this repair is implemented and re-audited.

The repair adds exact test-only seals over the existing independently audited production prefix
and the complete semantic source under one non-circular normalization. It does not change or
reinterpret the original C2A semantic design.

## Frozen inputs

```text
C2A design
  5a3599038198ba95e1c6ad7421b4ba2ee703b2b96021c04a34dca196952e27bc

Pre-repair semantic source
  4da0ce228b5127587a72f65d19efd58ddbc7d6e7d00dc98a41117ff18dbce4b5

Reviewed production prefix
  11747afaddc14fd01d7da5e63addd56a9fae7eaa3ad70fb2154e0212f2dfc286

Precomputed normalized post-repair complete source
  30ef7f171e4292b7cb7f306ea2a2439f82ab2970afd5da5272e839817ed51ce0

Precomputed post-repair full source
  2fe9b2f8e3d2ae8f483b3b93e38f3f4a7d508f033bb1e1edd8892348de1a6f75

Unchanged private lib wiring
  2eea0ad9fcafdd9cc7f8a9e813a41e594e293a853a453b482c2ec41cd1064a74

Unchanged lockfile
  8741698619a41dcce8f58aae3609e6ee48929d6c2e91b955caf0ed586920572e
```

The production prefix is exactly the bytes before the single sentinel
`#[cfg(test)]\nmod tests {`. Its hash was independently reproduced from the pre-repair source.

## Exact authorized implementation

Only `engram-eval/src/native_successor_semantic.rs` may change. Inside the existing test
`security_firewall_tests::production_source_has_no_forbidden_capability_or_runnable_surface`,
immediately after `let production = production_source();`, add exactly:

```rust
let production_sha256 = lower_hex(&Sha256::digest(production.as_bytes()));
assert_eq!(
    production_sha256,
    "11747afaddc14fd01d7da5e63addd56a9fae7eaa3ad70fb2154e0212f2dfc286",
    "reviewed C2A production prefix changed"
);

const SOURCE_SEAL_PLACEHOLDER: &str = concat!(
    "................................",
    "................................"
);
const REVIEWED_NORMALIZED_SOURCE_SHA256: &str =
    "30ef7f171e4292b7cb7f306ea2a2439f82ab2970afd5da5272e839817ed51ce0";
assert_eq!(
    SEMANTIC_SOURCE
        .matches(REVIEWED_NORMALIZED_SOURCE_SHA256)
        .count(),
    1,
    "reviewed C2A normalized source digest must occur exactly once"
);
let normalized_source = SEMANTIC_SOURCE.replacen(
    REVIEWED_NORMALIZED_SOURCE_SHA256,
    SOURCE_SEAL_PLACEHOLDER,
    1,
);
assert_eq!(
    lower_hex(&Sha256::digest(normalized_source.as_bytes())),
    REVIEWED_NORMALIZED_SOURCE_SHA256,
    "reviewed C2A complete source changed"
);
```

The existing denylist, private-declaration assertions, no-caller peer inventory and all other tests
remain unchanged. The implementation uses only the already imported `Sha256`, `Digest` and the
existing test helper `lower_hex`; no dependency, feature, manifest or lockfile change is allowed.

The post-repair production-prefix SHA-256 must remain
`11747afaddc14fd01d7da5e63addd56a9fae7eaa3ad70fb2154e0212f2dfc286`. Replacing the single
embedded normalized-source digest with exactly 64 periods must yield SHA-256
`30ef7f171e4292b7cb7f306ea2a2439f82ab2970afd5da5272e839817ed51ce0`. The post-repair full
semantic-source SHA-256 must be
`2fe9b2f8e3d2ae8f483b3b93e38f3f4a7d508f033bb1e1edd8892348de1a6f75`.

## Exact claim

The prefix assertion proves that the production prefix exercised by the existing denylist is
byte-identical to the prefix independently inspected under the original C2A allowed-source and
capability boundary. The normalized whole-source assertion additionally covers every byte before,
inside and after the outer test module while excluding only its own expected 64-hex digest. The
digest must occur exactly once and is replaced with a fixed 64-byte placeholder before hashing, so
the check is non-circular. Together with unchanged module privacy, current peer inventory,
dependency lock and accepted compiler, appending or changing source can no longer pass merely by
avoiding the finite forbidden-token list.

It does not claim generic Rust parsing, macro-expansion analysis, dependency reachability analysis,
malicious-test resistance, supply-chain authenticity or future-source safety without rerunning the
test and exact-source audits. Any source change requires a new reviewed normalized-source hash
rather than silently updating this seal.

## Required acceptance sequence

1. Reproduce the pre-repair full-source and production-prefix hashes.
2. Apply only the exact test-only insertion above.
3. Prove the production-prefix, normalized complete-source and full-source hashes match the three
   post-repair values frozen above.
4. Run the exact source-firewall test.
5. Run the focused C2A suite in parallel and serial modes.
6. Run the full provider-free library and integration suites, including the deferred
   `native_stale_preparation` test.
7. Run strict all-target Clippy, formatting and whitespace checks under the approved external
   target while repository `target/debug` remains absent and disk reserve remains positive.
8. Obtain three fresh independent exact-source audits with P0=0 and P1=0 against the repaired full
   source, unchanged production-prefix hash, normalized complete-source hash, frozen C2A design and
   this repair.
9. Update the C2A acceptance record with this repair hash, the repaired source hash and audit
   results; remove its withheld status only after every preceding gate passes.
10. Before C2B1 implementation, freeze a narrow downstream identity ratchet that replaces the
    C2B1 design's pre-repair semantic-source pin with the repaired accepted pin. Because C2B1's
    already frozen `mod acquisition;` declaration necessarily changes both seals, that ratchet must
    also authorize exactly two corresponding test-literal replacements: the reviewed-prefix digest
    and normalized complete-source digest. It must pin pre/post full-source, production-prefix and
    normalized-source hashes and leave every other C2A production and test byte unchanged.

No C2B1 source or dependency edit is authorized by this document.

## Nonclaims

This repair adds no semantic validation, acquisition, datastore, filesystem, environment, network,
clock, randomness, process, thread, async/runtime, provider, authentication, daemon, runner,
adapter, relay, correction-action, deletion or public-interface capability. It does not close any
C2B acquisition, persistence, correction-journey or flagship gate.
