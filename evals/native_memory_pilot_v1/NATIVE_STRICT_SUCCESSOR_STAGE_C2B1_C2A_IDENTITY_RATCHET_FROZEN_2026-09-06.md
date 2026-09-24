# Native strict successor Stage C2B1 — frozen C2A identity ratchet

Date: 2026-09-06 (Asia/Jerusalem)

## Status and purpose

This identity ratchet is frozen before C2B1 implementation. It supplements, and does not replace,
the frozen C2B1 Memory-acquisition design. C2A was accepted only after a test-only whole-source
firewall repair. The C2B1 design predates that repair and pins the pre-repair semantic source, while
its required `mod acquisition;` declaration necessarily changes both repaired source seals.

This document updates only those identities and authorizes their exact mechanical transition. All
C2B1 semantic, ownership, datastore, query, entropy, test and nonclaim requirements remain exactly
as frozen in the original C2B1 design.

## Frozen inputs

```text
C2B1 frozen design
  04f1be24c758f4a7f52baafab2d9ff27b9fd59241eecc48545deb3e5365600cb

C2A original design
  5a3599038198ba95e1c6ad7421b4ba2ee703b2b96021c04a34dca196952e27bc

C2A source-firewall repair
  0b527f29bfb5341e5698c420903f2c30630ac6e9505d8bb37a8edb6c2d6211a0

C2A accepted evidence
  2c583accccf77c51e912a10b5184ad5fef96a16aebca0f005019f228caf05b2f

Accepted pre-child semantic source
  2fe9b2f8e3d2ae8f483b3b93e38f3f4a7d508f033bb1e1edd8892348de1a6f75

Accepted pre-child production prefix
  11747afaddc14fd01d7da5e63addd56a9fae7eaa3ad70fb2154e0212f2dfc286

Accepted pre-child normalized source
  30ef7f171e4292b7cb7f306ea2a2439f82ab2970afd5da5272e839817ed51ce0

Unchanged lib.rs
  2eea0ad9fcafdd9cc7f8a9e813a41e594e293a853a453b482c2ec41cd1064a74

Pre-C2B1 evaluator manifest
  5238c63cb31eb420ad27ae50c44e92cfbb39bdab1c895dade1cf066770c480c6

Unchanged workspace manifest
  ec61531f8ffd401211f649c7b91e28a4ee5f0fa1a0fe2f199c63a841c9724f5a

Pre-C2B1 lockfile
  8741698619a41dcce8f58aae3609e6ee48929d6c2e91b955caf0ed586920572e
```

## Exact parent transition

Only these three parent-source changes are authorized before adding the separately frozen child:

1. Immediately after `use zeroize::Zeroize;`, add:

   ```rust
   mod acquisition;
   ```

2. In the existing production-prefix assertion, replace only:

   ```text
   11747afaddc14fd01d7da5e63addd56a9fae7eaa3ad70fb2154e0212f2dfc286
   ```

   with:

   ```text
   c67ec74b936ce861e53e52d110b4635e4ba6fdfc9a3b0d3a93357315ca6d4b34
   ```

3. In the normalized whole-source assertion, replace only:

   ```text
   30ef7f171e4292b7cb7f306ea2a2439f82ab2970afd5da5272e839817ed51ce0
   ```

   with:

   ```text
   152678c4bb765f948c8fe3d96ed49428e89e4ca472322733916da948bfd70036
   ```

Every other byte of `engram-eval/src/native_successor_semantic.rs` remains unchanged. The resulting
parent identities must be:

```text
production prefix
  c67ec74b936ce861e53e52d110b4635e4ba6fdfc9a3b0d3a93357315ca6d4b34

normalized complete source
  152678c4bb765f948c8fe3d96ed49428e89e4ca472322733916da948bfd70036

full parent source
  2f9909b47c81238f3754fee954c7dcefa01ccdee28df1fcd30fbf94c6c718391
```

The normalized value is computed exactly as in the accepted repair: replace the sole embedded
normalized-source digest with 64 periods, then SHA-256 the complete source bytes. These values were
precomputed from the accepted source plus only the three transitions above.

## Interaction with the child

The declaration resolves only the exact private child authorized by the C2B1 design:

```text
engram-eval/src/native_successor_semantic/acquisition.rs
```

The child may not modify the parent through generated source, include indirection or macro
expansion. No other parent declaration, import, type, visibility, call site, test, constant or
literal may change. The parent still has no production caller for the acquisition entry in C2B1.

The original C2B1 source boundary, two exact direct dependencies, derived lockfile update and
unchanged `lib.rs`/workspace manifest remain authoritative. Their post-implementation hashes must
be recorded in C2B1 acceptance evidence.

## Required verification

1. Reproduce every frozen input hash before editing.
2. Apply only the exact three parent transitions above plus the original C2B1-authorized child,
   evaluator-manifest and derived-lockfile changes.
3. Reproduce all three post-transition parent hashes before running C2B1 tests.
4. Run both repaired source-firewall assertions and every original C2A test during C2B1 focused,
   full and integration validation.
5. Treat either seal failure as source-boundary failure, not as a value to update opportunistically.
6. Include this ratchet in each of the three final C2B1 exact-source audits, with P0=0 and P1=0.

## Nonclaims

This ratchet adds no datastore, acquisition, persistence, filesystem, network, environment, clock,
randomness, process, thread, provider, authentication, daemon, runner, adapter, relay,
correction-action, deletion or public-interface authority. It does not itself implement C2B1 or
broaden any C2A/C2B1 claim.
