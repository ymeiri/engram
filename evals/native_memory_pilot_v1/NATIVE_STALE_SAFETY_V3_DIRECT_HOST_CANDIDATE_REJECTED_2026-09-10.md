# Native stale-safety v3 direct-host candidate rejection — 2026-09-10

Status: immutable provider-free rejection evidence; grants no provider, credential, deletion,
installation, or pilot-execution authority

## Rejected candidate

`protocol-native-stale-safety-v3-base-schema-10-direct-host-file-cache.json`

SHA-256: `ba2d148596ebbd55690c216bdc0e1350d7e62e38c2963f96e795bab2f039871a`

The candidate is non-executable for four independently sufficient reasons:

1. Its missing-source base case omitted schema-10
   `evaluation_prerequisite_state="missing"`, so the current deserializer defaulted the case to
   `unchanged` and generic validation failed.
2. Its missing-source verification lifetime was prose rather than a materialized, digest-bound
   value. This could not prove evaluation completed while the verification was strictly unexpired.
3. Its signal table omitted the missing-source/native `SAFE_INCONCLUSIVE` row and used
   `boundary:evidence_insufficient` in one expiry row even though current source/time evidence, not
   native-marker presence, determines the boundary signal.
4. The checked-in family-v1 typed parser silently discarded the candidate's new nested v3
   contracts, so those contracts were neither validated nor included in the typed protocol
   snapshot.

No provider was invoked and no lane was prepared from this candidate. The file remains unchanged
and must never be repaired or executed.

## Forward-only successor

`protocol-native-stale-safety-v3-r2-base-schema-10-direct-host-file-cache.json`

SHA-256: `b080d19bc1ef6f0d2813201cb240338fd14be19d3f3b56806fd53a23563ef650`

The successor moves all v3-only case semantics under the typed stale extension, restores the base
missing-state field, materializes the missing-source verification lifetime as 2,592,000 seconds,
keeps the expiry case at 300 seconds, uses exact case boundary signals for every admissible result,
and contains both native clean-absence `SAFE_INCONCLUSIVE` rows. It is still non-executable until
the typed family-v3 implementation and adversarial provider-free qualification pass and a fresh
run plan binds the resulting binaries and contracts.
