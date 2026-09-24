# Native strict successor Stage C2B2 — persistent containment research review

## Disposition

The C2B2 persistent-containment research record is review-safe as non-authorizing sequencing and
future-freeze input. It is not a C2B2 design freeze, implementation, acceptance record or runtime
authority.

The exact reviewed record is:

```text
evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2_PERSISTENT_CONTAINMENT_RESEARCH_2026-09-06.md
SHA-256 00735349dd10c8c18414a4e611fc5030a82ae014c9ac95bfd9872b3f9427ece0
```

Three independent reviewers verified the same SHA-256 before and after their final reviews and
made no edits:

| Review | P0 | P1 | P2 | Verdict |
|---|---:|---:|---:|---|
| C2B2 inherited-requirements audit | 0 | 0 | 0 | review-safe |
| C2B2 host-isolation audit | 0 | 0 | 0 | review-safe |
| C2B2 SurrealDB/RocksDB audit | 0 | 0 | 0 | review-safe |

The final review round performed no build, provider, network, VM, container or datastore operation.

## Accepted research direction

The future C2B2 freeze should use stock pinned direct SurrealDB Core through four bounded process
roles in one dedicated VZ guest: writer, live-lock contender, post-drop release probe and final
semantic reader. The guest has no host-directory shares, uses a fixed private data disk, and places
collector processes inside exact cgroup and private-network boundaries.

The record requires:

- a bounded canonical wire decoder before the inherited C2B1 typed preflight;
- an explicit inherited-versus-superseded C2B1 contract;
- graceful handle-drop proof while the writer process remains alive;
- independent reader reopen and complete C2A canonical comparison with a fresh key;
- distinct guest-local and host-local move-only causal witnesses;
- acceptance only after a final live challenge, cleanup and stable absence;
- structural-only persisted receipts; and
- honest exclusion of hidden close errors, crash/power durability and trusted-principal attacks.

A pinned SurrealDB patch remains only a fallback. Native macOS, the current shared Colima instance,
a live or copied Engram store, and same-container tmpfs cannot substitute for the full claim.

## Sequencing and current gate

C2B2 freeze work may begin only after accepted C2A and C2B1 evidence exists. The immediate
executable step remains C2A's deferred `native_stale_preparation` integration test.

At the final revalidation:

```text
available space                       1,052,664 KiB
mandatory reserve                    19,427,004 KiB
repository target/debug                     absent
/private/tmp/engram-claude-host-action-target-20260904-01
                                      20,781,808 KiB
```

The cache remained untouched. Deleting it still requires exact path-specific user confirmation.
