# Native strict successor Stage C2B1 — sealed-Memory design review

## Disposition

The C2B1 design is implementation-safe within its frozen Memory-only boundary. This review accepts
the design for later implementation only after every prerequisite stated in the design is green.
It does not accept an implementation, a persistent datastore, a live Engram store or the flagship
goal.

The exact reviewed design is:

```text
evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B1_MEMORY_ACQUISITION_FROZEN_2026-09-06.md
SHA-256 04f1be24c758f4a7f52baafab2d9ff27b9fd59241eecc48545deb3e5365600cb
```

All three final reviewers verified that the file had the same SHA-256 before and after review and
made no edits:

| Review | P0 | P1 | P2 | Verdict |
|---|---:|---:|---:|---|
| C2A integration/conformance | 0 | 0 | 0 | implementation-safe |
| C2B1 frozen-spec audit | 0 | 0 | 0 | implementation-safe |
| C2B1 adversarial red team | 0 | 0 | 2 | implementation-safe |

The final review round performed no build, network, provider or datastore operation.

## Closed findings

The repaired design now freezes and correctly distinguishes:

- nine initialization and 18 replacement responses, each exactly a successful empty native array;
- native `NONE`, native `NULL`, nonempty arrays and every other write-output value as failures;
- a deterministic fifth-insert failure inside the test replacement transaction, after nine deletes
  and four inserts, followed by an exact observation of the complete prior generation;
- a checked conservative borrowed pre-serialization bound and the later exact C2A meter;
- deterministic pre-dispatch errors, uncertain post-write outcomes, read engine failures and
  malformed successful read responses; and
- syntax-aware negative-trait and sole initialization/collection-path gates.

The response and rollback contracts match pinned SurrealDB 2.6.0 behavior: `RETURN NONE` suppresses
row collection into an empty native array; a non-array/non-object single-expression insert fails;
and the explicit-transaction executor cancels the transaction on that statement failure.

## P2 implementation evidence obligations

The red team left two non-blocking evidence obligations. C2B1 implementation acceptance must make
them explicit rather than relying on broad tests:

1. Exercise every concrete serialization branch, every enum and optional-field shape, maximal
   control-character escaping and extremal finite number spellings to prove the conservative
   borrowed meter never undercounts and does not falsely reject any C2A-valid generation.
2. Record a complete table mapping every typed-ingress failure to its exact named C2B1 categorical
   error, including invalid timestamps and numbers, duplicate IDs, `id`/`record_id` collisions and
   projection inconsistency.

These are acceptance-evidence refinements, not authority or implementation blockers. Any failure
to prove them during implementation reopens the design disposition.

## Prerequisite gate still open

C2B1 implementation remains unauthorized until C2A passes its one deferred
`native_stale_preparation` integration test and receives its accepted evidence record. The test
must use only the frozen external build target after the mandatory disk reserve is positive.

At the final pre-review disk check:

```text
available space                         341,708 KiB
mandatory reserve                    19,427,004 KiB
repository target/debug                     absent
/private/tmp/engram-claude-host-action-target-20260904-01
                                      20,781,808 KiB
```

The cache directory remained untouched. Its deletion still requires exact user confirmation for
that exact path. No C2A test or C2B1 implementation may start before the reserve is restored and
the frozen identities are revalidated.
