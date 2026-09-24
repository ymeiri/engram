# Native strict successor Stage B — accepted implementation evidence

Date: 2026-09-06 (Asia/Jerusalem)

## Outcome and exact claim boundary

Stage B is implemented and independently accepted as a provider-free, non-runnable native
successor substrate. The acceptance proves a strict document boundary, process-local execution
authority, one-way execution lifecycle, durable intent and terminal evidence, no replay, bounded
I/O, and a sanitized structural public facade.

It does not authorize or prove a provider run, provider authentication, daemon semantic identity,
phase-filtered relay enforcement, live semantic collection, VM authority, live adapter settings,
or flagship success. There is no public run entry point. No authentication cache was copied and no
daemon, VM, provider, or live-settings operation occurred in this slice.

## Accepted implementation identity

- `engram-eval/src/native_document.rs`:
  `8cd13f7ac7b7f09ee19153bb01d15e7ad84963c5414fafed75ef5c5f4606c1dd`
- `engram-eval/src/native_execution.rs`:
  `9eb8ef99292e45a318ad13dc0ac5392a3b52106ef24adbcce851b2116a8a5574`
- `engram-eval/src/native_successor_core.rs`:
  `d450a2326960c916c34c152dbfa71d5d809cc75e65a1211e114c59ab2eb9cd77`
- `engram-eval/src/native_successor.rs`:
  `22239a6c66edba75ede83cbac3b2f4f05b12c620dac2bcac1a19ccccbbea0a07`
- `engram-eval/src/lib.rs`:
  `e5f7c403a87c043503ff91037be92b21dd6bd1c0f744d9ee069d990607d66b78`
- `engram-eval/src/native_correction.rs`:
  `2cddf80f9e21ceb71c0949093ac92e329d32358b3e78ecc468acce1ad6d22f51`
- `engram-eval/tests/native_correction_foundation.rs`:
  `6686287d70363b0c3421ac25d04bf0a02e6a1933408af43b3f99c03509c9a61b`

These hashes identify the accepted implementation only. The design document is intentionally
updated to reference this evidence record, so its post-acceptance hash is recorded separately
after documentation validation.

## Accepted invariants

### Strict document boundary

- Exactly two successor families are accepted: `native_stale_isolated_v1` and
  `native_correction_v1`, each at schema version 1.
- Protocol, run-plan, execution-intent, terminal-receipt, audit, and report documents require the
  exact three-key outer envelope and exact typed document kind.
- Recursive duplicate keys, historical fallback, unknown family/version/kind, symlinks,
  hardlinks, path replacement, truncation, growth, group/other-writable files, and macOS extended
  ACLs are rejected.
- The bounded loader retains the accepted descriptor, canonical identity, metadata, SHA-256, and
  bytes, then consumes that snapshot into one typed payload without rereading by pathname.
- Its public audit exposes only sanitized structural results.

### Execution authority and lifecycle

- The accepted correction process guard was moved into one shared execution boundary rather than
  copied.
- `Prepared -> IntentCommitted -> Running -> Terminal` is a private consuming lifecycle; only an
  intent-committed state can spawn.
- Execution identity, phase, and artifact directory come from the typed run-plan payload rather
  than independently chosen caller fields.
- The process-local seal is non-cloneable, non-serializable, and cannot be reconstructed from a
  document or receipt.
- Intent and receipt documents plus digest sidecars are created exclusively with owner-only mode,
  flushed durably, retained, and revalidated through directory-relative descriptors.
- Existing residue, intent-only interruption, terminal state, receipt collision, or sidecar
  mismatch prevents replay. Terminalization is attempted once even across unwind paths.
- Every process-group operation and owned pipe operation checks the owning process ID. A forked
  foreign drop cannot signal the owner's group.
- Success, terminal failure, output overflow, timeout, and cleanup uncertainty are distinct typed
  outcomes. Cleanup uncertainty leaves intent-only evidence and never manufactures a terminal
  receipt.
- Standard input, standard output, standard error, and combined output are bounded. One absolute
  deadline spans execution, collection, group termination, stable descendant absence, reap, and
  terminal evidence creation.

### Public boundary

- `native_document`, `native_execution`, `native_successor_core`, correction, isolation, and VM
  internals remain private.
- `native_successor` exposes only structural validation/audit. It exposes no provider, auth, VM,
  daemon, process, or run capability and no secret-bearing field.

## Validation evidence

- Focused successor document tests: 7 passed, 0 failed.
- Focused execution-guard tests: 13 passed in the library suite and 13 passed through the
  path-mounted correction suite.
- Focused successor-core tests: 20 passed, 0 failed.
- Full provider-free `engram-eval` library suite: 240 passed, 0 failed.
- Native correction foundation: 66 passed, 0 failed, 1 ignored.
- Native isolation foundation: 25 passed, 0 failed, 4 ignored.
- Native stale preparation: 1 passed, 0 failed.
- Native VM foundation: 17 passed, 0 failed.
- Strict Clippy with warnings denied: passed.
- Formatting and whitespace checks: passed.
- Exact installed-binary correction gate: 1 passed, 0 failed, 66 filtered, in 742.06 seconds.
- Independent stable-snapshot audit: no P0 or P1 finding; Stage B accepted within this exact
  provider-free, non-runnable scope.
- Repository `target/debug`: absent before and after validation; external compiler storage was
  used.

## Deliberate remaining gates

Before any runnable successor facade is considered, the next slices must close these boundaries:

1. Make launch, binary identity and hash, working directory identity, timeout, disk reserve, and
   tool surfaces closed-world values derived from the typed plan and phase.
2. Make offline lifecycle audit explicitly distinguish a late-but-durable receipt pair; the mere
   existence of intent and receipt files must never imply live or on-time authority.
3. Add a final response-deadline check after the last standard-input write.
4. Forbid process forks after collectors start or replace thread-owned inherited pipe state with
   an owner-held pollable design before any runnable/forking adapter is admitted.
5. Do not treat a process group as containment for deliberately daemonizing descendants. Direct
   provider execution remains blocked until a VM, cgroup-equivalent boundary, or exact trusted
   non-daemonizing binary contract closes that gap.
6. Provision evaluator-exclusive execution directories and ownership before live execution;
   retained descriptors do not eliminate all same-UID concurrent mutation.
7. Address the expired-deadline cleanup edge in which a killed leader can remain a zombie until
   evaluator exit. This edge is non-authorizing and leaves no live process group, but it must be
   closed before runnable acceptance.
8. Keep the successor payload implementation internal. External crates cannot implement it today;
   any future crate-internal extension must remain explicitly reviewed.

The next authorized activity is design and provider-free validation of the smallest daemon-owned,
phase-filtered relay and live semantic-collector slice. Provider execution remains separately
blocked.
