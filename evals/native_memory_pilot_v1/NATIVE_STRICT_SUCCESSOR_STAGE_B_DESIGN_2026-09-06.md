# Native strict successor Stage B — accepted provider-free substrate

Date: 2026-09-06 (Asia/Jerusalem)

## Status and claim boundary

This design was implemented and independently accepted as a provider-free, non-runnable substrate
on 2026-09-06. It is not a frozen protocol or run plan, exposes no provider-run entry point,
creates no authentication copies, and authorizes no daemon, VM, provider, or live-settings
operation. Exact implementation hashes, validation results, claim boundaries, and remaining gates
are recorded in `NATIVE_STRICT_SUCCESSOR_STAGE_B_ACCEPTED_2026-09-06.md`.

The historical native runner is categorically outside this design. It accepts serialized launch
arguments, inherits ambient environment, and does not retain process-group authority. A strict
successor must not call, import, adapt, or accept its plan, approval, preparation, or execution
types.

Both prerequisite foundations have now been repaired and independently accepted:

- native correction passed 54 focused provider-free tests and its exact installed-binary gate
  (1 passed, 0 failed, in 746.72 seconds), including real isolated seed, proposal, exactly-once
  apply, interrupted recovery without replay, restart persistence, and terminal cleanup;
- connect-existing-only passed its full CLI, focused Store/MCP, strict Clippy, format, and
  independent audit gates. Its accepted claim remains deliberately narrow: it proves bounded,
  persistence-safe observation of an existing transport, not daemon semantic identity or provider
  authority.

These acceptances authorize only the provider-free, non-runnable implementation slice below. They
do not authorize a provider run or weaken any later daemon-identity, VM, relay, or semantic-proof
gate.

## Exact module boundary

The smallest shared substrate adds:

```text
engram-eval/src/native_execution.rs       private owned-process lifecycle
engram-eval/src/native_successor_core.rs  private successor authority and receipts
engram-eval/src/native_successor.rs       public sanitized structural facade
```

`native_document`, `native_correction`, `native_isolation`, and `native_vm` remain private. Only
`native_successor` may be public. Its initial API may validate and structurally audit successor
documents, but it must not expose a run function or CLI command.

Public values must exclude argv, environments, tokens, canaries, raw semantic records, auth
material, file descriptors, `Child`, process-group IDs, VM witnesses, and execution seals.

## Exact document envelope

The document firewall accepts only these sibling families at version 1:

```json
{
  "family": "native_stale_isolated_v1",
  "schema_version": 1,
  "payload": {
    "document_kind": "protocol"
  }
}
```

```json
{
  "family": "native_correction_v1",
  "schema_version": 1,
  "payload": {
    "document_kind": "protocol"
  }
}
```

Every protocol, run plan, execution intent, terminal receipt, audit, and report uses the same outer
envelope and an exact `document_kind` inside its payload. Unknown family, wrong version or kind,
extra outer key, non-object payload, recursive duplicate key, malformed marker, symlink, hardlink,
replacement, empty file, or oversized file is terminal. There is never historical fallback.

The loader retains the accepted file handle, canonical identity, and SHA-256, then consumes itself
into one typed payload without rereading by pathname. The correction template migrates into the
payload; its legacy top-level form is rejected rather than accepted as compatibility. The stale
family receives a fresh payload type and does not wrap a historical stale protocol.

## Process-local authority and lifecycle

`NativeExecutionSeal` is private and derives none of `Clone`, `Debug`, `Serialize`, or
`Deserialize`. No JSON boolean, approval record, persisted receipt, or VM structural audit can mint
or reconstruct it.

The internal lifecycle is consuming and one-way:

```text
PreparedExecution
  -> IntentCommittedExecution
  -> RunningExecution
  -> TerminalExecutionReceipt
```

Only `IntentCommittedExecution` can spawn. Before any runtime, auth-status, VM, daemon, proxy, or
provider subprocess, it must:

1. consume one bounded, descriptor-bound plan load;
2. validate exact empty phase state, residue, disk reserve, paths, and invocation confirmation;
3. derive launch, configuration, and cleared environment internally from family and phase;
4. create-new and fsync the execution intent plus digest sidecar; and
5. mint a live process-local seal from fresh observations.

Every child is wrapped immediately in one shared WNOWAIT-reserved process-group guard. One absolute
deadline bounds delivery, execution, collection, group termination, stable descendant absence,
leader reap, and terminal receipt creation. Success and failure both create-new and fsync exactly
one terminal receipt. A success, terminal failure, or intent without receipt forbids replay; an
interrupted phase is `FrozenIncomplete` and requires a fresh successor.

The correction operator retains its stricter rule: its exact apply request is preceded by a
fsynced dispatch journal, and any existing dispatch is never resent. A later read-only resolution
may report already-applied state without claiming dispatch attribution.

## Phase-specific surfaces

Host configuration, proxy filtering, and trace validation must all enforce the same policy:

| Family/phase | Engram surface |
| --- | --- |
| stale teaching, native arm | none |
| stale teaching, Engram arm | only `memory:add` |
| stale activation | none |
| stale evaluation | only `orient` and `memory:procedure_match`; exactly one memory call |
| correction proposal | `orient`, `memory:propose_correction`, and bounded proposal inspection |
| correction operator | evaluator-only exact reads and one journaled apply; no provider |
| correction retrieval | `orient` and bounded `memory:list`; no mutation |
| semantic collector | evaluator-only authenticated canonical read plan |

The existing six-tool agent profile and full profile do not establish these surfaces. Host
allowlists are defense in depth, not server-side authority. Provider execution remains blocked
until a phase-filtered relay or equivalent evaluator-owned transport enforces the exact action
allowlists.

## Evaluator-owned Engram boundary

For each Engram-bearing phase, the evaluator spawns and retains the exact HTTP daemon as its direct
process-group child with a cleared environment containing only the frozen `ENGRAM_HOME`,
`ENGRAM_DAEMON_TOKEN`, `DISABLE_TELEMETRY=1`, and `PATH`. The launch is derived internally as:

```text
engram serve --http --port <port> --project <partition>
```

Connect-existing-only supplies bounded transport evidence, never process identity authority. Its
health check performs a reversible committed datastore write/delete canary, proxy traffic may
mutate according to the selected profile, and proxy shutdown deletes its MCP session. Therefore it
may be used only against the evaluator-owned isolated child/store and under a phase-constrained
tool surface.

The semantic collector authenticates to that exact daemon, verifies the full contract, performs a
bounded typed canonical query plan, and records canonical record hashes/counts immediately before
provider spawn and immediately after provider reap. Live authority and offline audit are distinct:
offline reporting validates receipt-bound chronology and hashes, but never recreates a live seal.

## VM boundary

A VM structural audit remains non-authorizing. A fresh, non-cloneable, non-serializable VM witness
is rechecked and consumed by exactly one phase. The guest then performs its own fresh checks and
mints its own process-local seal before owning guest daemon, proxy, and provider children. A host
seal is never serialized into the guest.

Strong Claude flagship evidence remains mount-free-VM-only with in-guest subscription login. Local
Claude is limited to an explicitly labelled `confounded_exploratory` result.

## Accepted provider-free implementation slice

With both prerequisites independently accepted, the implemented and accepted slice is:

1. extend `native_document` with the exact two-family, version-one, consuming successor loader;
2. move, rather than copy, the accepted correction process guard into `native_execution`;
3. add the private intent/seal/terminal-receipt/no-replay state machine in
   `native_successor_core`;
4. add the sanitized validation/audit facade in `native_successor`, with no run API;
5. wire private modules in `lib.rs`; and
6. add provider-free compile-time and adversarial tests.

Required tests include exact envelope/kind substitution, historical fallback prevention,
descriptor/path attacks, intent-before-child observation, pre-intent spawn prevention, terminal
receipt uniqueness, intent-only no replay, residue invalidation, cleared environment, bounded
stdin/stdout/stderr, timeout/output overflow, malformed output, late descendants, `ECHILD`, `EPERM`,
enumerator failure, panic unwind, and public-facade secret-field exclusion.

Independent acceptance found no P0 or P1 issue within this non-runnable claim boundary. Work may
therefore continue to a separately designed and reviewed provider-free slice for exact daemon
ownership, phase-filtered relay profiles, live semantic collection, and closed-world plan-derived
limits. Correction/isolation adaptation, fresh VM consumption, a concrete frozen protocol, and a
provider-run facade remain later, separately gated work.
