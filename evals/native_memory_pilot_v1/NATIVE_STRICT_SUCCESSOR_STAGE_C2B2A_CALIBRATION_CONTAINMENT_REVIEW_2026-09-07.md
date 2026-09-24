# Native strict successor Stage C2B2a — calibration-containment freeze review

## Disposition

The C2B2a calibration-containment design is implementation-safe within its exact frozen boundary.
This review authorizes only the provider-free source implementation and evidence work enumerated in
the freeze. It does not accept an implementation, payload binary, guest image, run plan, VM run,
RocksDB result, persistence claim, C2B2b, harness behavior or the flagship goal.

The exact accepted design is:

```text
evals/native_memory_pilot_v1/
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_CALIBRATION_CONTAINMENT_FROZEN_2026-09-07.md
SHA-256 85312b0286080ecbbab94b5236003a42f281b1122565016d96bb20f9572063e3
lines   1451
```

All three final reviewers re-read the complete 1,451-line artifact, verified the same SHA-256 before
and after review, made no edits and returned no implementation blocker:

| Independent review | P0 | P1 | Verdict |
|---|---:|---:|---|
| adversarial protocol and lifecycle red team | 0 | 0 | implementation-safe |
| RocksDB, wire and measurement audit | 0 | 0 | implementation-safe |
| Lima/VZ, cgroup, disk and cleanup audit | 0 | 0 | implementation-safe |

The final attestation round performed only local file reads and hashes. It started no build, network,
provider, VM, container, adapter, daemon or datastore operation.

## Review ratchet

Every non-final reviewed SHA was rejected rather than silently repaired in place. The material
review checkpoints were:

- `b314f55217a647ee5313ecfe97fc94331e2c3b1b3eb534e61cebac111a2a2a60` — rejected:
  incomplete nonce, terminal, measurement and VM-boundary exactness.
- `895491148fbb628f072ba0722fb9c0e0b82d5aef115c86d576fe97ae994bc337` — rejected:
  no executable instance/characterization selector and incomplete negative paths.
- `94b76559b44b02a8668e341aefd4bcf91870a9bbcd31a14e75ce7df46d8ec153` — rejected:
  an inner namespace claimed an unobservable outer PID; p39 and FD exceptions were incomplete.
- `ee79b425df4861ee8cf824bedb3ff129f1933d2256c1922bcea96994c787854c` — rejected:
  one literal probe-17/18 entry-stub contradiction.
- `85312b0286080ecbbab94b5236003a42f281b1122565016d96bb20f9572063e3` — accepted:
  three independent P0=0/P1=0 attestations.

No verdict for an earlier SHA transfers to a later SHA. Only the final digest above is accepted.

## Closed blockers

The accepted freeze now makes these previously ambiguous or impossible boundaries exact:

- a closed big-endian HostStart phase/class/ordinal selector chooses the non-acceptance
  characterization automaton or one of ten acceptance instances without a plan/argv override;
- one C01 guest runs three fresh-filesystem lock-byte characterizations, emits 21 separately retained
  measurements and cannot satisfy any acceptance count;
- trusted PID-1 role-init and the probe-39 launcher remain outside the workload OOM subtree, while
  only collector tasks atomically enter bounded role cgroups;
- every process-creating `clone3` call, exit signal, pidfd and `waitid` option is frozen;
- probe 39 has inner-only identity packets, root-owned outer-PID derivation, exact SCM_RIGHTS
  cardinality, half-closes, a post-Continue trigger and a race-free reparent/reap automaton;
- probes 17 and 18 have one exact FD-5 pre-exec negative exception and are rejected after the audited
  entry stub but before Start, fixture construction or Core;
- `/data`, `/diag` and `/tmp` are writable only under frozen UID/GID/mode values, and every writer
  begins with `/data/store` absent;
- lock matching is exact byte equality, and writer-witness, contender and release-probe failures have
  deterministic post-acknowledgement categories;
- pre-Start measurements, aggregate-only PID/thread claims, multi-call probe selectors and probe-37
  signal evidence are fully represented by the frozen protocol; and
- write-dispatch and acknowledgement flags reset per active item and cannot inherit authorization
  from an earlier filesystem or case.

## Implementation boundary still open

Implementation must remain inside the freeze's exact source/package list and preserve the existing
workspace manifest, workspace lockfile, evaluator manifest, `native_vm.rs`, C2A and C2B1 sources.
All Cargo output must use the already designated external target; repository `target/debug` must
remain absent. User-owned worktree changes must not be staged, committed or overwritten.

Before any VM run, the implementation must satisfy all twenty provider-free evidence requirements,
two-root deterministic payload reproduction, final ELF/source/dependency firewalls, three fresh
exact-source/binary audits, a separately frozen image/kernel/tool/run plan and all disk/identity/ACL
gates. C01 must complete before the ten-instance acceptance phase. A failed or changed identity
burns the affected evidence exactly as the freeze specifies; no completed lane or attempt may be
replayed or repaired.
