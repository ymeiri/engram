# Native strict successor Stage C — provider-free daemon, relay, and semantic design

Date: 2026-09-06 (Asia/Jerusalem)

## Status and authority boundary

**Rejected for implementation after independent security and feasibility review.** This document
is retained as an evidence trace, not as current authority. The repaired, narrower authority is
`NATIVE_STRICT_SUCCESSOR_STAGE_C1_FROZEN_2026-09-06.md`.

This was a design candidate for a private, provider-free, non-runnable successor slice. It did not
add a CLI command or public run surface, execute an AI provider, create authentication copies,
start a VM, modify live adapters or settings, or authorize a flagship claim.

The rejection found no P0 within the provider-free scope, but found material P1 contradictions in
mutable-directory identity, partition containment, activation, semantic completeness, observer
mutation and time, secret-independent intent binding, relay transport, artifact recovery, embedder
startup, inherited Git subprocesses, token handling, and reverse dependency isolation. No code may
be implemented from this document.

Stage B remains the accepted execution substrate. Stage C may extend its private core and add an
evaluator-owned Engram daemon, exact phase relay, and live semantic collector. It must not import,
call, adapt, or accept authority from the historical `native_runner`, `native_pilot`,
`native_correction`, `native_isolation`, or `native_vm` execution paths. Historical code may inform
tests only.

The implementation remains non-runnable from outside `engram-eval`. The only public successor API
continues to be the sanitized structural document audit in `native_successor.rs`.

## Smallest module boundary

```text
engram-eval/src/native_successor_core.rs      closed-world authority and lifecycle
engram-eval/src/native_successor_engram.rs    evaluator-owned daemon and strict MCP client
engram-eval/src/native_successor_relay.rs     exact downstream phase programs
engram-eval/src/native_successor_semantic.rs  exhaustive typed semantic snapshots
engram-eval/src/native_successor.rs           unchanged sanitized structural facade
```

The three new modules and the extended core remain private in `lib.rs`. Compile- and source-level
tests reject a public start/run/spawn/token/live-witness/raw-record surface and reject imports from
historical native execution modules.

## Closed-world plan authority

Successor plans use exact tagged payload structs for family and phase, never an arbitrary map.
Every common phase binding contains only:

- exact `document_kind`, execution ID, protocol SHA-256, and tagged phase;
- exact tagged host-runtime binding;
- canonical artifact-directory binding and preparation time;
- phase-specific predecessor receipt digests;
- canonical phase-working-directory binding; and
- either an absent Engram boundary or one exact typed Engram boundary.

An executable binding contains canonical absolute path, SHA-256, version, byte length, mode, and a
device/inode identity digest. A directory binding contains canonical absolute path,
device/inode/effective-UID/mode identity digest, immutable manifest SHA-256, and, where applicable,
the expected Git HEAD and normalized remote.

The Engram boundary contains:

- the pinned Engram executable binding;
- pinned empty daemon working-directory and owner-private Engram-home bindings;
- one opaque partition;
- distinct numeric daemon and relay loopback ports;
- health schema 3, MCP contract 4, MCP protocol `2024-11-05`;
- full-profile tool count 32 and tool digest
  `714e07ffd5fcc5e615ef3eeb6df2e479801495a834bc0f6fa7e0672a68410422`;
- exact phase-policy ID and digest; and
- exact semantic-query-policy ID and digest.

The currently observed installed Engram contract is version 0.2.3 at executable SHA-256
`d45cc2e3ab4a928f6390ec0c5f5a2a0fce4bd8485a0a44159f6e87b52a6da8b8`. A later frozen plan must
bind its selected executable explicitly; this observation is not a wildcard or compatibility
rule.

Plans never serialize argv, environment maps, numeric timeouts or reserves, arbitrary tool/action
arrays, daemon or relay capabilities or hashes, authentication paths or bytes, or host settings.
One private `derive_phase_authority` mapping derives the exact launch, clean environment, limits,
and ordered deadline schedule from the family, phase, host binding, and policy identity. A future
production preparation edge accepts only the descriptor-bound typed plan and a process-local
invocation confirmation; caller-supplied launch, timeout, output cap, cleanup reserve, disk reserve,
or tool list is forbidden. Arbitrary launches remain test-only.

The deadline schedule has distinct monotonically ordered boundaries for readiness, action response,
post-action semantic collection, daemon/session cleanup, and final durable receipt creation.
Teardown begins while positive cleanup and receipt reserves remain. The implementation freezes the
exact numeric mapping as code constants and records its digest in intent evidence; acceptance must
publish the exact mapping and tests before a runnable adapter can depend on it.

## Evaluator-owned Engram daemon

The private boundary has consuming states equivalent to:

```text
PreparedEngramBoundary
  -> EvaluatorOwnedEngramDaemon
  -> LiveEngramDaemonSeal
  -> TerminalEngramBoundary
```

After the phase intent is durably committed, the evaluator generates independent 256-bit daemon
and relay capabilities from the operating-system CSPRNG. Secret values are zeroized, have no
`Clone`, `Debug`, or Serde implementation, never appear in error text, traces, artifacts, hashes,
or reports, and are checked by marker scans over every serialized artifact.

The evaluator launches exactly:

```text
<pinned-engram> serve --http --port <daemon-port> --project <opaque-partition>
```

The child uses the pinned empty working directory, null standard input, bounded standard output and
standard error, and a cleared environment containing only:

```text
ENGRAM_HOME=<pinned owner-private home>
ENGRAM_DAEMON_TOKEN=<fresh daemon capability>
DISABLE_TELEMETRY=1
PATH=/usr/bin:/bin:/usr/sbin:/sbin
```

On macOS, live inspection may additionally accept only the exact effective-UID-derived
`__CF_USER_TEXT_ENCODING` value injected by the platform. No other inherited variable is allowed.

The evaluator retains executable, working-directory, Engram-home, and process authority. Before
and after readiness it verifies:

- direct-child PID equals process-group leader;
- exact executable, argv, working directory, and effective environment;
- executable and directory identities and hashes;
- health PID equals the owned child;
- exact service, version, build identity, health schema, MCP contract, protocol, tool digest, and
  bearer-auth requirement;
- storage readiness and positive plan-derived reserve; and
- a separate authenticated strict MCP initialization and full `tools/list` digest.

The ordinary CLI proxy and `connect-existing-only` are not used. They are intentionally broader,
support recovery behavior, and prove transport rather than evaluator ownership. The provider never
receives the daemon URL, port, capability, home, or partition.

## Exact phase relay

The evaluator terminates downstream MCP itself. The private relay has consuming states equivalent
to `PreboundNativeRelay -> ActiveNativeRelay -> TerminalNativeRelayReceipt`. It uses one private
upstream session and at most one downstream session.

The downstream control plane accepts one strict JSON-RPC object per request. It rejects batches,
recursive duplicate keys, trailing bytes, `_meta`, oversized input, reused or non-scalar request
IDs, reconnects, retries, reinitialization, session confusion, and every unexpected notification or
method. Exact order is initialize once, initialized notification once, locally synthesized
`tools/list` once, then the phase program. Synthesized schemas set `additionalProperties: false`.
Every rejected transition terminal-poisons the phase. The relay never automatically retries or
reinitializes upstream.

Every accepted argument object must equal the protocol-derived object exactly. Only underscore
action spellings are valid; case-folded or hyphenated aliases are rejected. Upstream results are
strictly parsed, locally validated, minimized, and bounded before any downstream response.

The exact programs are:

| Family/phase | Downstream program |
| --- | --- |
| stale native-only teaching | no relay |
| stale Engram/both teaching | exactly one `memory/add`; no `orient` |
| stale activation | no relay |
| stale Engram/both evaluation | exactly one `memory/procedure_match` |
| correction proposal | minimized `orient`, one proposal, then exact active tagged list |
| correction operator | no provider relay; compiled evaluator client only |
| correction retrieval | minimized synthetic `orient`, then exact active tagged `memory/list` |

For stale evaluation, the entire procedure query is frozen. The exact base object is:

```json
{
  "action": "procedure_match",
  "query": "<frozen exact query>",
  "limit": 1,
  "scope": {
    "relevance_mode": "local",
    "cwd": "<exact evaluation cwd>"
  }
}
```

Exact `conditions` appear only when the frozen case requires them. Raw upstream `orient` is never
exposed during stale evaluation because it can reveal stale memory independently of applicability.
If host compatibility later requires orientation, the relay may synthesize an identity-only result
from the live pre-action snapshot at most once; it never forwards raw `orient` content.

Correction proposal arguments equal the existing exact nine-part semantic contract: action,
obsolete ID, title, content, writer harness/provider/model, one exact file-evidence object, and one
exact local Atlas scope. Correction operator uses the compiled evaluator sequence: wrong-scope
rejection; Atlas proposal list/get and active list; durable journal; exactly one apply or a
zero-send recovery observation; fresh daemon restart; active list/get; shutdown. Correction
retrieval uses only minimized identity plus the exact active-tagged list.

Before forwarding any `add`, `propose_correction`, or `apply_correction`, the evaluator creates and
fsyncs an owner-only exclusive dispatch journal bound to plan SHA-256, intent SHA-256, ordinal, and
exact request digest. An ambiguous response is never resent. Journal existence freezes the
mutation outcome for offline resolution in a fresh successor.

## Exhaustive live semantic collector

The collector uses a separate authenticated evaluator session that is never exposed through the
relay. It parses strict typed domain objects, rejects unknown fields and duplicate stable IDs,
normalizes and sorts records, and hashes a canonical representation. Raw response bytes exist only
in bounded zeroizing memory.

Persisted evidence contains only sanitized stable IDs, status, counts, individual record hashes,
and aggregate hashes. Raw semantic content, source excerpts, capabilities, URLs, home/partition
paths, and unredacted responses are never persisted or exposed publicly.

Coverage includes every state class capable of influencing an allowed call:

- all MemoryItems in the evaluator-exclusive store, not only tagged rows;
- pending replacements;
- every correction proposal and exact proposal/obsolete/replacement tuple;
- projects, repositories, checkouts, and project links; and
- immutable fixture/source bindings plus the time inputs used for freshness and expiry.

Bounded list responses whose `count` only reflects returned rows do not prove completeness. The
collector uses exhaustive no-limit reads under a strict streaming byte cap in the known-small
exclusive store, with bounded proposal enumeration and exact-ID inspection. If a full-profile API
cannot exhaust one influencing table, collection returns `semantic_coverage_unsupported`; no
complete or flagship claim is inferred. A private typed snapshot API may be considered only if
tests prove the existing full-profile surface cannot provide exhaustive coverage.

Required boundaries are:

- stale S0: pre-teach, empty relevant Engram state;
- stale S1: post-teach/pre-verify, exactly one unverified candidate and no native-only marker;
- stale S2: immediately pre-evaluation, the same ID with exact active, verified, scoped,
  source-bound, and frozen freshness/expiry state;
- stale S3: post-reap, canonical state equal to S2 and the match result bound to both;
- correction C0: exact seed;
- correction C1: post-proposal/pre-operator;
- correction C2: post-apply and fresh restart, before retrieval; and
- correction C3: post-retrieval, canonically equal to C2.

Raw RocksDB directory hashes are diagnostic only because health and read activity may change
storage files.

## Lifecycle and containment corrections

Stage C extends the private lifecycle with:

```text
NativeOfflineExecutionState =
  Pristine
  | IntentOnlyFrozen
  | TerminalPairTimingUnproven
  | TerminalPairClaimsLate
  | InvalidResidue
```

A successful final durable write mints a non-cloneable live timely-terminal seal only when the
final monotonic deadline check passes. Offline receipt presence is terminal and prevents replay,
but can prove only `TerminalPairTimingUnproven`; an explicit late marker yields
`TerminalPairClaimsLate`. Offline artifacts never recreate live or timely authority.

Standard-input delivery performs a final monotonic check after the last successful write. The
typestate pre-binds the relay, spawns the evaluator-owned daemon, performs synchronous readiness
and pre-action collection, completes every future child-spawn edge, and only then permits relay or
output collector threads. No spawn method exists after `AllChildrenSpawned`.

Shutdown order is: reject new relay ingress, close downstream and upstream sessions, perform the
post-action semantic snapshot, terminate the daemon group, prove stable group absence, reap the
leader, then create the terminal receipt. Stage C keeps relay and collector work in the evaluator;
the pinned daemon is its only production child.

Local process groups cannot contain a descendant that deliberately escapes with `setsid` or an
equivalent mechanism. An adversarial test must demonstrate this limitation and every local audit
must report `strong_containment=false`. Strong provider evidence remains blocked on mount-free VM,
namespace/cgroup-equivalent teardown, or another independently proven containment boundary.

Retained descriptors, descriptor-relative operations, exclusive create, owner-only modes, exact
UID/mode/link/ACL checks, closed-world entries, and an evaluator-held lock detect ordinary races.
They do not protect against a malicious same-UID process; every local audit states that limitation.

Normal transitions enter cleanup before the reserved deadline. If cleanup nevertheless begins
after expiry, the guard signals once and attempts nonblocking reap, returns cleanup-unproven, mints
no terminal receipt or seal, and permanently forbids replay. A runnable adapter remains blocked
until a dedicated lane supervisor or equivalent proves eventual leader reap without extending the
authorization deadline.

## Required provider-free tests

- family, phase, host, and resource-policy substitution; callers cannot select argv, timeout,
  reserve, binary, working directory, output cap, environment, or tool list;
- binary/path/hash/version/length/mode/device/inode and directory/tree/Git substitution before and
  after live observation;
- exact daemon argv, clean environment, PID/PGID, health, auth, storage, and full MCP contract;
  port collision, early exit, wrong PID/hash/contract/auth/storage, and output overflow;
- capability redaction and proof that a relay capability cannot authenticate to the direct daemon;
- exact relay schemas, order, cardinality, IDs, arguments, and minimized results; every forbidden
  tool/action/alias, batch, duplicate, extra key, widened scope, changed ID/path/tag, appended
  query, reconnect, retry, malformed upstream result, and session-confusion case;
- mutation dispatch ambiguity proving zero retry;
- proof that stale raw-orient content is unreachable;
- exhaustive collector overflow and incomplete-enumeration failure; ordering normalization,
  duplicates, changed-record hashes, and pending-replacement coverage;
- exact S0-S3 and C0-C3 delta/equality rules;
- final standard-input byte crossing the response deadline;
- late durable receipt classified terminal/no-replay but never timely success;
- proof that no relay or collector thread exists before the last spawn edge;
- `setsid` escape producing only non-strong, non-authorizing local evidence;
- expired cleanup producing no receipt, with a test supervisor proving eventual leader reap;
- artifact/home/cwd swap, extra entry, symlink, hardlink, permission, ACL, and same-UID mutation;
- marker scans over every serialized result; and
- compile/public-surface and source-dependency firewalls.

## Gates after Stage C

Stage C acceptance, if achieved, authorizes only later design work. Before a provider-run facade,
the project must still prove non-durable Claude relay-capability delivery, exact isolated Codex and
Claude host configuration, exhaustive semantic coverage, provider spawn/fork ordering, terminal
hygiene, and VM or different-UID containment for any strong claim. A new concrete protocol and run
plan require their own review, freeze, attestation, authentication readiness, provider-free audit,
disk reserve, and retention gates. No historical plan may be replayed or repaired.
