# Native flagship remaining-evidence execution boundary V6

Date: 2026-09-10
Status: frozen design boundary; non-authoritative advisory review only; zero implementation,
qualification, authentication, Docker, provider, pilot, cleanup, or deletion authority

## 1. Append-only disposition and inherited closure

This compact successor rejects and supersedes the execution lifecycle in exact V5:

```text
evals/native_memory_pilot_v1/NATIVE_FLAGSHIP_REMAINING_EVIDENCE_EXECUTION_BOUNDARY_V5_FROZEN_2026-09-10.md
SHA-256  193727a7710cd16703c686ebf33e6c81c00afcf0268d6432fe07747981624949
LF       454
bytes    26978
```

V5 remains immutable and non-executable. Fresh review found that V5 still lacked a concrete
post-review design identity, authority for provider-free construction, exact-plan user acceptance,
boot-bounded preclaim recovery, pre-persistence secret handling, complete host identity and auth
isolation, durable receipt rules, attainable liveness evidence, and exact retention semantics.
Review-environment inability to execute a hashing command is not an artifact defect; Section 2
assigns byte identity to the trusted coordinator without asking a reviewer to use a shell.

Except where Sections 2–13 replace a V5 clause, V6 incorporates V5's exact Schema-15 no-replay
disposition, exact stale-safety protocol and 30-admission schedule, flagship semantics, scorer
semantics, isolation intent, outcome classes, report labels, no-candidate-Python cut, and nonclaims
by the V5 digest above. A conflict is resolved only in favor of V6. No V5 authority, lifecycle,
recovery, authentication, persistence, timing, or receipt clause survives by implication when V6
replaces it.

The prospective human-readable review and canonical design-manifest paths are:

```text
evals/native_memory_pilot_v1/NATIVE_FLAGSHIP_REMAINING_EVIDENCE_EXECUTION_BOUNDARY_V6_ADVISORY_REVIEW_2026-09-10.md
evals/native_memory_pilot_v1/NATIVE_FLAGSHIP_REMAINING_EVIDENCE_EXECUTION_BOUNDARY_V6_DESIGN_MANIFEST_V1.bin
```

Both must be absent at this freeze. Reviews are advisory defect evidence, never user authority.

## 2. Canonical post-review design manifest D

`D` means one concrete binary manifest, not the phrase “design plus reviews.” It is created only
after three fresh blind reviews of the exact V6 and protocol bytes and only when the coordinator's
adjudication has zero unresolved P0 and P1 findings. Otherwise D remains absent and V6 is rejected.

For each reviewer, the trusted coordinator performs the byte-identity sandwich: no-follow open,
`fstat`, bounded read and SHA-256 of V6 and the protocol before dispatch; supplies those verified
identities in the prompt; asks the reviewer to use its available read-only file tools; captures the
exact returned UTF-8 bytes; then repeats the no-follow `fstat`, bounded read and SHA-256. Pre/post
path, device, inode, owner, group, mode, link count, length, and digest must match. The protocol gets
the same sandwich. A reviewer is never required to run a shell or compute a hash. Each review
receipt binds the coordinator-verified pre/post identities, reviewer role, source system, stable
session ID or literal `unavailable`, model/effort, exact prompt digest, and exact output bytes.

D uses this encoding:

```text
ASCII "engram-v6-design-review-manifest-v1\0"
u16be version = 1
u16be hash algorithm = 1 (SHA-256)
u32be input count = 6
then six records in the exact order below
```

Every record is `u16be kind || u64be payload_length || payload`. Nested text fields are
`u64be length || shortest-form UTF-8`; binary digests are exactly 32 raw bytes. The six inputs are:

1. kind 1: repository-relative V6 path, byte length, and raw SHA-256;
2. kind 2: repository-relative protocol path, byte length, and raw SHA-256;
3. kind 3: Codex semantic-review receipt;
4. kind 4: Codex operational-review receipt;
5. kind 5: isolated Claude-review receipt; and
6. kind 6: coordinator adjudication, binding the preceding receipt digests, every finding ordinal,
   its severity and disposition, and exact unresolved counts.

The protocol identity remains:

```text
evals/native_memory_pilot_v1/protocol-native-stale-safety-v1-schema-10-file-cache.json
SHA-256  282875027ba8fc5732edfca9a05b58a07dcdce12851156575052592cf8c68fc8
LF       332
bytes    10404
```

Real P0/P1 findings cannot be “repaired” by adjudication: they require an append-only successor.
Adjudication may mark a finding not applicable only with an exact V6/protocol clause and evidence.
`D_digest = SHA256(exact D bytes)`. D is sealed once with Section 11's durable primitive. Its
human-readable rendering is derived evidence and cannot substitute for D.

The dependency chain is now acyclic and concrete:

```text
D -> A0 -> I -> Q -> P -> R4 -> U1 -> M[i] -> J[i] -> T[i] -> K[i] -> H[i]
```

I binds D and all six D input digests; Q and P bind D/A0/I; R4 binds P; U1 binds every preceding
digest; every M binds P/R4/U1; a claim-kind J binds M; and H genesis binds the complete prefix.
Hash identity still creates no authority.

## 3. Two authentic authority events, with no future-plan delegation

V6 itself authorizes only advisory review. After D, one authentic user-role event `A0` is required
to authorize the bounded provider-free construction stage. A0 names exact D/protocol identities,
project `engram`, V6 roots and qualification prefix, the implementation-freeze destination, an
absolute expiry, and the closed operation list: write/build only under B; fake-provider and
provider-free Docker qualification only under Q; exact Q-object cleanup; P/R4 generation; and no
credential access, E creation, live provider traffic, adapter/settings change, or other deletion.
The coordinator seals the authenticated host event as a canonical `A0` receipt using the V5 U
event encoding with domain `engram-v6-provider-free-construction-a0-v1\0`. A0 cannot authorize
provider work and cannot be synthesized from AI prose or existing goal intent.

Only after exact I, Q, P, and R4 exist may the user send `U1`. There is no authority to delegate a
future plan. The coordinator renders the exact message below from the independently attested P;
the authenticated user event must equal those bytes, including the final LF, with no wrapper:

```text
ENGRAM_NATIVE_FLAGSHIP_V6_USER_ACCEPTANCE_V1
project=engram
digest.<four-digit-index>.<canonical-name>=<lowercase-64-hex>
limit.<four-digit-index>.<canonical-name>=<unsigned-decimal>
allow.<four-digit-set-index>.<canonical-set-name>.<four-digit-item-index>=<base64url-no-pad>
expiry.<four-digit-index>.<canonical-name>=<RFC3339-UTC>
decision.provider_execution=accept
decision.exact_30_admissions_no_replay=accept
decision.ephemeral_container_network_cleanup=accept
decision.codex_auth_terminal_disposition=delete_exact_files_or_quarantine_on_failure
ack.claude_admissions=12
ack.claude_usd_ceiling=not_enforceable
ack.post_acceptance_conversation_revocation=not_a_claimed_control
END_ENGRAM_NATIVE_FLAGSHIP_V6_USER_ACCEPTANCE_V1
```

The indexed sections contain every digest, numeric/resource/process/deadline limit, path/network/
tool/model/auth/cleanup allowlist item, and absolute or derived expiry in P and R4. Names and sets
are ascending UTF-8 byte order; indices are contiguous from `0000`; allowlist item order is raw
UTF-8 byte order; complex values are raw bytes encoded as base64url without padding. Empty sets are
represented by an explicit `.0000=<empty>` item. R4 independently proves projection completeness.
Any omission, duplicate, reordering, placeholder, extra byte, parse ambiguity, or mismatch blocks
U1. The canonical U1 audit cache additionally binds authenticated role, task/thread/event IDs,
source timestamp, exact raw message, boot identity, real/continuous clock samples, and every
prefix digest. It is sealed at:

```text
A0  /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v6-stale-safety-20260910-01/authority/provider-free-construction-a0-v1.bin
U1  /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v6-stale-safety-20260910-01/authority/provider-execution-u1-v1.bin
```

Before U1 the user has an exact go/no-go. After U1, V6 claims no authoritative conversational
revocation channel. Already-delivered OS signals or control-FD EOF are best-effort stop inputs: no
later claim is created, exact active objects are contained, and a partial disposition is emitted.
Failure to prove containment is `SPENT_INDETERMINATE`; cancellation never authorizes replay.

## 4. Fresh namespace and authority separation

V6 uses only:

```text
base  /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v6-stale-safety-20260910-01
B     /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v6-stale-safety-20260910-01/build
Q     /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v6-stale-safety-20260910-01/qualification
E     /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v6-stale-safety-20260910-01/execution
Q Docker prefix  engram-native-flagship-v6-stale-safety-20260910-01-qual-
E Docker prefix  engram-native-flagship-v6-stale-safety-20260910-01-exec-
```

B, Q, E, and authority are distinct owner-private children. Q/E share no object, writable volume,
claim, or evidence chain. E and its Docker prefix remain absent through R4 and until authentic U1
passes. A0 allows only its exact B/Q/P/R4 operations; U1 allows only its exact E epoch. Repository
`target/debug` remains absent and implementation uses `B/target`. V4/V5 namespaces stay absent.

## 5. I, provider-free Q, exact P, and independent R4

I retains every V5 frozen implementation field and additionally binds A0; the canonical D schema;
the exact A0/U1 parsers and U1 renderer; the time sources; preclaim-choice schema; sanitizer and
promotion logic; guest PID-1 supervisor; every receipt schema; and exact Codex identity fields.
Changing any selected mechanism after U1 requires a successor and new authority.

Q runs only after A0. It is provider-free and uses synthetic credentials, fake provider endpoints,
canaries, hostile Read/Bash/double-fork fixtures, injected clocks/boot IDs, crashes at every durable
boundary, and exact Docker Desktop observations. It proves exact I mechanisms without accessing a
live credential or model. Q never selects or binds a future P.

P is one canonical, fully materialized plan, not a runtime recipe. Its ordered table is exactly:

- ordinals 1–12: teaching for protocol `stale_safety.lanes` order 1–12;
- ordinals 13–18: Codex activation for lane orders 1, 3, 5, 8, 10, and 12; and
- ordinals 19–30: evaluation for lane orders 1–12.

Every row contains literal phase/host/case/arm/repetition/lane IDs; exact prompt and fixture byte
digests; source/command-tool contract; native/Engram state inputs and outputs; auth assignment;
model/provider/reasoning/effective-config identity; deterministic object names; limits; deadlines;
expected absences; M/J/T/K/phase basenames; and predecessor chain/retention requirements. Nothing
is derived after U1 except the P-enumerated dynamic observations permitted in M.

`R4` is produced before U1 by a separately invoked, I-bound provider-free verifier path that
receives no plan-generator state. It independently parses the protocol and I, reconstructs all 30
rows and the complete U1 projection, byte-compares them with P, rehashes every referenced input,
checks B/Q/E separation and pristine E absence, and seals an R4 receipt. Shared code is limited to
I-bound canonical decoders and SHA-256. Failure or missing coverage rejects P; no user execution
acceptance is requested. U1 and every M bind `R4_digest`.

## 6. Exact host/model identity and structurally isolated authentication

Claude retains V5's date-versioned immutable model rule. Codex now has the same standard: I and P
bind exact Codex CLI/provider client and code-mode companion hashes, immutable model identifier,
provider/backend route, reasoning effort, full effective config, feature/memory flags, tool surface,
and expected response identity fields. Each admission attests the same values before and after.
An alias, unavailable identity, config drift, or provider response mismatch makes the host
ineligible or terminates the epoch; it cannot be relabeled as comparable evidence.

`chatgpt_file_cache` is structurally eligible only if Q first proves, with a synthetic cache and
fake provider, one exact I-selected boundary:

1. credential consume-and-unmount: the trusted bootstrap consumes the cache, closes every FD,
   unmounts/revokes its path, and proves it absent before the first model-directed tool action; or
2. external broker: the cache remains exclusively outside the agent container/process namespace
   and only a claim-bound provider transport, unreachable by every model-driven tool, is exposed.

Same-process built-in Read access is not dismissed as a child-sandbox problem. Q must make the
synthetic secret unreadable through built-in Read, Bash, MCP, environment, FD, `/proc`, socket,
proxy, DNS, crash output, and native-memory paths. If neither boundary passes, Codex is ineligible
before U1 and the 30-admission U1 must not be requested. Claude's selected helper/login route passes
the equivalent structural tests. There is no direct-API or weaker-isolation fallback.

If the qualified Codex boundary needs V5's six lane-local cache copies, U1 lists those exact paths
and authorizes their terminal disposition. Each copy has the epoch expiry in U1 and is unusable
after it. After terminal evidence, the disposer unlinks only those exact held-parent identities,
fsyncs each parent, proves absence, and emits a content-free deletion receipt. Failure moves the
exact file into an owner-only, mode-`0000`, non-reusable quarantine named in P and emits an
incomplete cleanup receipt; it grants no future use or deletion. Any deletion absent exact U1 is
forbidden. Broker topology and zero-copy topology must be represented literally in P/U1.

## 7. Durable teaching anchor and exact time semantics

The Darwin execution host identity is the raw 16-byte UUID parsed from
`sysctlbyname("kern.bootsessionuuid")`; no fallback string or wall-clock boot inference is allowed.
Elapsed time uses `mach_continuous_time` with the captured `mach_timebase_info` numerator and
denominator and checked `u128` conversion to nanoseconds. This clock includes system sleep. Lack,
parse error, arithmetic overflow, regression, or boot-UUID change is terminal and cannot fall back
to `CLOCK_MONOTONIC` or wall time.

After teaching ordinals 1–12 have complete T/K/H evidence, the evaluator seals a teaching-complete
receipt binding H[12]. Only after its fsync/readback succeeds does it sample boot UUID, continuous
time, and UTC real time and seal a retention-anchor receipt. Activation may start only when:

```text
current_boot_uuid == anchor_boot_uuid
current_continuous_ns >= anchor_continuous_ns + 3_600_000_000_000
current_realtime_ns >= anchor_realtime_ns + 3_600_000_000_000
```

Both comparisons are inclusive, so the exact one-hour boundary passes. Continuous time prevents a
forward wall jump from shortening retention; the real-time condition turns a backward wall jump
into a terminal stop; suspend time counts; boot change, VM restore, time regression, chain/cursor
regression, or detected storage rollback produces a partial terminal disposition and no later
claim. An undetectable trusted-filesystem snapshot rollback remains outside V6's threat claim and
is stated as such in reports. Activation/evaluation M records bind the anchor digest and samples.

At U1-cache creation, the coordinator similarly binds boot UUID plus real/continuous samples and
derives the continuous epoch expiry from U1's absolute UTC expiry. Every later claim requires the
same boot, non-regressed clocks, real time before the absolute expiry, and continuous time before
the derived expiry. A forward clock jump may expire early but can never extend authority.

## 8. Boot-bounded M-to-claim decision

P fixes `m_to_claim_window_ns = 60_000_000_000`. M binds the current raw boot UUID,
`m_observed_continuous_ns`, and:

```text
m_claim_not_after_ns = min(
  m_observed_continuous_ns + 60_000_000_000,
  u1_epoch_not_after_continuous_ns
)
```

The addition is checked. After M is durably published and before any Docker object/output exists,
one single P-bound O_EXCL decision slot `J[i]` is created. Its mutually exclusive kind is:

- `CLAIM`: all V5 C fields plus M/R4/U1 digests, boot UUID, current clock samples, and the strict
  proof `m_observed <= now < m_claim_not_after`; or
- `ABANDON_UNCLAIMED`: M digest, prior H, observed boot/clocks/expiry, absence evidence, and exactly
  one reason: boot mismatch, clock regression/unavailable, deadline reached (including equality),
  U1 expired, identity drift, M validation failure, or expected object/output not absent.

For `CLAIM`, `C[i]` means the exact J bytes and digest. Only that kind permits watchdog/Docker/
provider work. For `ABANDON_UNCLAIMED`, no C exists semantically, no provider work or later J is
allowed, and the epoch ends in a partial terminal disposition. The shared slot prevents a claim and
abandonment race.

M-without-J may receive its one first decision after process restart only if the same boot UUID,
strict deadline, U1 expiry, all byte identities, and all expected absences still pass. After reboot
or deadline, the only valid decision is durable abandonment. A crash while sealing J is either one
fully verified record or an indeterminate terminal epoch; J is never reconstructed or overwritten.

V6 chain genesis and claimed-admission links are:

```text
H[0] = SHA256(
  ASCII "engram-native-flagship-v6-chain-genesis-v1\0" ||
  D_digest || A0_digest || I_digest || Q_digest || P_digest || R4_digest || U1_digest
)
H[i] = SHA256(
  ASCII "engram-native-flagship-v6-admission-chain-v1\0" ||
  H[i-1] || M[i]_digest || J_CLAIM[i]_digest || T[i]_digest || K[i]_digest
)
```

An abandonment does not extend the success chain; its partial disposition binds H[i-1], M, J, and
every never-claimed ordinal.

## 9. Pre-persistence sanitizer and state promotion

Docker uses log driver `none`; daemon logs and container log files are forbidden evidence paths.
Provider stdout/stderr, tool traces, generated files, and mutable native/Engram state first live
only in an I-qualified bounded ephemeral staging boundary. Durable lane state is mounted read-only
as input; each admission writes a fresh staging generation. Credentials are never part of staged
state.

After proven process termination and before any T, transcript, log, trace, native-memory, Engram
datastore, or result promotion, the exact sanitizer scans all staged bytes and metadata for the
full in-memory credential/canary set, forbidden paths, environment/FD disclosures, and bounded
encoded variants selected in I. A hit or incomplete scan blocks promotion. Secret-bearing bytes
remain in ephemeral quarantine only until exact container/staging cleanup; durable evidence records
only category, location class, byte count, and redaction reason—never the bytes or a reusable
credential digest. Cleanup failure is a containment fault and preserves only non-secret inspect
evidence.

Only a complete clean scan permits canonical sanitized evidence and intended state to be copied
into a new O_EXCL generation, fsynced, read back, hashed, and atomically selected as the next
lane-local generation. Failure leaves the previous sanitized generation current. This ordering is
part of Q's crash matrix. Raw model data is never executed, parsed as code, or promoted around the
sanitizer. Canary persistence anywhere outside the deliberate U1-listed auth topology is a
containment fault.

## 10. Exact watchdog automaton and attainable termination evidence

I freezes the complete frame bytes from V5 with V6 magic/digests and this only legal automaton:

```text
S0  coordinator HELLO       -> watchdog READY            -> S1
S1  coordinator BIND_NETWORK -> watchdog NETWORK_BOUND   -> S2
S2  coordinator BIND_CONTAINER -> watchdog CONTAINER_BOUND -> S3
S3  coordinator STARTED     -> watchdog HEARTBEAT         -> S4
S4  coordinator HEARTBEAT   -> watchdog HEARTBEAT         -> S4
S4  coordinator TERMINAL    -> watchdog verifies stop and sends TERMINAL -> S5
S4  coordinator CANCEL      -> watchdog contains object and sends TERMINAL -> S5
S5  coordinator TERMINAL_ACK -> both close cleanly         -> END
ANY local deadline/peer EOF/invalid frame -> watchdog contains bound object, sends FAULT -> END
ANY watchdog FAULT/invalid frame/EOF -> coordinator requests containment, seals fault -> END
```

Only the displayed sender may initiate a transition. Sender-local sequence numbers are strictly
increasing; replies repeat all identities and the continuous deadline; duplicate, skipped,
out-of-state, wrong-direction, or late frames fault. No Docker network precedes S1, no container
precedes S2, and no start precedes S3. I binds heartbeat and every transition timeout.

V6 does not demand an inaccessible macOS view of Docker Desktop's Linux cgroup. Instead the exact
guest supervisor is container PID 1 and a subreaper. It owns the harness process group, forbids
daemonization/namespace creation, kills the group on exit/cancel/deadline, reaps children, and
emits a fixed control-channel `/proc` census showing no remaining descendant before it exits. Q
must prove this with hostile double-fork and `setsid` fixtures. The watchdog then requires exact-ID
Docker inspect `Running=false`, `Pid=0`, `RestartCount=0`, restart policy `no`, expected exit/event
sequence, no network endpoint, and finally exact-ID not-found after U1-authorized removal. If the
guest census or attainable Engine observations are unavailable or inconsistent, termination is
unproven and the epoch is `SPENT_INDETERMINATE`. Compromised Docker/kernel escape resistance
remains an explicit nonclaim.

## 11. One durable sealing primitive for every artifact

D, A0/U1 caches, I, Q, P, R4, M, J, sanitizer/promotion receipts, T, cleanup authorization, K,
phase/retention receipts, H checkpoints, partial dispositions, and final reports all use one
I-bound primitive: held verified `0700` parent descriptor; absent-child check; `openat` with
`O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC` and mode `0600`; bounded canonical full write; file fsync;
regular-file/owner/mode/link-count `fstat`; close; no-follow reopen; bounded readback/hash and
identity verification; close; parent fsync. No artifact is overwritten, repaired, adopted,
reconstructed, or silently omitted. A partial write, link/rebind, duplicate, failed fsync, or
readback mismatch is terminal.

T may be sealed only from sanitizer-approved inputs and binds the sanitizer/promotion receipts.
Only then may the exact cleanup authorization be sealed for the C/T-bound stopped container,
network, and ephemeral staging identities. K binds that authorization and exact cleanup result.
Qualification cleanup derives only from A0; execution cleanup and auth deletion derive only from
U1. Wildcards, prune, image/volume/evidence deletion, and cross-epoch action remain forbidden.

## 12. Progression, reporting, and exact gates

V5's `SEMANTIC_PASS`, `SEMANTIC_FAIL`, `CONTROLLED_HARNESS_FAILURE`, `CONTAINMENT_FAULT`,
`INELIGIBLE`, 30-complete-admission report label, partial-report separation, scoring, budget
nonclaims, and no-replay rules remain. `ABANDON_UNCLAIMED` is an additional terminal no-provider
class; it never becomes a semantic observation or successful completion.

The gate order is now:

```text
G0  freeze V6; coordinator-sandwich three blind reviews; seal D only with zero unresolved P0/P1
G1  receive authentic bounded A0; create B; freeze exact I
G2  provider-free Q of exact I/A0, including auth, sanitizer, clock, crash, watchdog, and Docker
G3  materialize exact 30-row P with all prompts, fixtures, assignments, limits, allowlists, expiry
G4  independently reconstruct/compare P and U1 projection; seal R4; require pristine E absence
G5  receive exact authentic U1 for D/A0/I/Q/P/R4; seal U1 cache; re-attest expiry/boot/absence
G6  create E; provision only U1-listed auth topology; re-attest isolation/disk/absence
G7  teaching 1–12 via M/J-claim/watchdog/sanitize/T/cleanup/K/H; seal teaching + retention anchor
G8  after strict one hour, activation 13–18; then evaluation 19–30, with all gates per admission
G9  reconcile 30 complete claimed outcomes and publish only the unreviewed comparison
G10 update the flagship audit and independently assess every still-unproven completion criterion
```

Any failure is fail-closed. No completed Schema-15 admission is replayed. No V4/V5 work is adopted.
A provider-bearing or post-U1 indeterminate failure spends the epoch; a preclaim abandonment ends
it without claiming provider success or cost. A changed design/I/P/auth/cleanup boundary requires
an append-only successor and new applicable authority.

V6 makes only bounded local-control and automated-evidence claims. It does not claim hard provider
request/retry/token/USD enforcement, server-side native-memory isolation, malicious-coordinator or
compromised kernel/Docker/client/provider resistance, undetectable snapshot-rollback resistance,
instant conversational revocation, statistical power, product superiority, human review,
production readiness, or flagship completion.

## 13. Freeze state and authority

At freeze, the V6 review/D path, namespace, B/Q/E/authority roots, A0, I, Q, P, R4, U1, M/J/T/K/H,
auth copies, Docker objects, provider outputs, phase receipts, and reports are absent. V5 and its
predecessors remain untouched. Repository `target/debug` and the forbidden combined-output path
remain absent. User-owned worktree changes are preserved; nothing is staged or committed.

This document authorizes only non-authoritative advisory review. It authorizes no implementation,
build, test, authentication access/copy, Docker mutation, provider/pilot run, live adapter/daemon
change, cleanup, publication, staging, commit, or deletion.
