# Native flagship remaining-evidence execution boundary V7

Date: 2026-09-10
Status: frozen append-only design delta; advisory review plus an exact future authority request only;
zero implementation, qualification, authentication, Docker, provider, pilot, cleanup, or deletion
authority

## 1. Predecessor and exact delta

V7 rejects and supersedes the remaining lifecycle defects in exact V6:

```text
evals/native_memory_pilot_v1/NATIVE_FLAGSHIP_REMAINING_EVIDENCE_EXECUTION_BOUNDARY_V6_FROZEN_2026-09-10.md
SHA-256  55a7c066901ee8f866303cf8bb9db11ade602c82190f3cb6e3cfba819fb28aa3
LF       429
bytes    26055
```

V6 stays immutable and non-executable. V7 replaces V6 Sections 2–3, the P/U1 canonicalization,
credential and sanitation clauses of Sections 4–7, the watchdog/cleanup/auth-terminal clauses of
Section 8, and their gates. All other V6 clauses remain by the exact hash above; through V6, the
V5 protocol, schedule, scorer, no-Schema-15-replay rule, flagship semantics, outcome labels,
no-candidate-Python cut, and nonclaims remain. Conflict is resolved only in favor of V7.

## 2. No composite D or bootstrap cycle

There is no D file, review manifest, I0 artifact, or pre-I executable. Exactly:

```text
D_digest = SHA256(exact raw V7 design bytes)
```

The three fresh blind reviews are separate advisory G0 evidence and never enter authority. For each
review, the trusted coordinator performs V6's no-follow pre/post byte-identity sandwich over V7 and
the protocol, supplies those identities to the reviewer, and captures the exact output. A reviewer
uses its available read-only tools and need not run a shell or compute a hash. Before I, review
bytes remain authenticated task-host evidence, not a locally manufactured authority artifact.

The coordinator computes, without publishing a file, one exact evidence-set digest:

```text
G0_digest = SHA256(
  ASCII "engram-v7-g0-advisory-evidence-v1\0" ||
  design_identity || protocol_identity ||
  semantic_review || operational_review || claude_review || adjudication
)
```

Each item is `u64be length || canonical payload`. An identity payload is, in order, canonical
absolute path, device/inode/uid/gid/mode/nlink/byte length, and raw SHA-256 for both the pre and post
observations. A review payload is, in order, fixed role enum, source system, stable session ID or
literal `unavailable`, model/effort, raw prompt SHA-256, raw output SHA-256, output byte length, and
the two design/protocol identity payloads. Adjudication is `u32be finding_count`, then findings in
review order and source ordinal, each encoded as severity enum, raw finding digest, disposition
enum, evidence text, followed by `u32be unresolved_p0` and `u32be unresolved_p1`; both must be zero.
Text uses shortest-form UTF-8 with a u64be length; enums are u16be; integers are unsigned big-endian.
A duplicate, unknown enum, unstable identity, or unavailable stable session ID blocks G0.

Authentic A0 (Section 3) directly authorizes the trusted coordinator to create one private root and
publish I. The following minimal pre-I publication rule is fixed by V7 bytes, not by future I:
stable no-follow open of the trusted parent; absent-child `mkdirat(0700)`; held child descriptor;
`openat(O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600)`; bounded full write; file fsync; regular-file,
owner, mode and `nlink=1` fstat; close; no-follow reopen; bounded readback/SHA-256/identity check;
close; parent fsync. Any failure stops. It is used only for the root and I. I is a canonical binary
manifest with domain `engram-v7-implementation-freeze-v1\0`, version/hash IDs, and ascending
UTF-8-keyed length-prefixed records for every V6 I field and exact path-keyed source/build input.
No record is implicit; duplicate or unknown keys reject I.

After I exists, its exact sealer stores audit-only A0/review/coordinator-adjudication receipts. I
binds and independently recomputes G0_digest from the authenticated task-host evidence; its sealed
G0 receipt records the same canonical payload. I cannot self-certify or change that evidence.
The receipts cannot retroactively authorize I. V7 explicitly permits read-only hashing/review and
presentation of the exact A0 template; only authentic A0 permits private-root/I publication.

## 3. Authentic literal A0 and bounded provider-free construction

All post-A0 artifacts live below this initially absent private root:

```text
base  /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01
B     /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01/build
Q     /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01/qualification
E     /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01/execution
P     /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01/plan
audit /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01/audit
Q-prefix  engram-native-flagship-v7-stale-safety-20260910-01-qual-
E-prefix  engram-native-flagship-v7-stale-safety-20260910-01-exec-
```

The user must send exactly the coordinator-rendered A0 bytes, including final LF and no wrapper:

```text
ENGRAM_NATIVE_FLAGSHIP_V7_A0_V1
project=engram
design_sha256=<D-lowercase-64-hex>
g0_evidence_sha256=<G0-lowercase-64-hex>
protocol_sha256=282875027ba8fc5732edfca9a05b58a07dcdce12851156575052592cf8c68fc8
base=/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01
build_root=/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01/build
qualification_root=/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01/qualification
plan_root=/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01/plan
audit_root=/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01/audit
implementation_destination=/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01/build/implementation-freeze-v1.bin
execution_root_must_remain_absent=/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v7-stale-safety-20260910-01/execution
qualification_docker_prefix=engram-native-flagship-v7-stale-safety-20260910-01-qual-
build_max_wall_seconds=7200
build_max_child_process_starts=4096
build_max_written_bytes=4294967296
build_max_regular_files=500000
build_max_docker_build_requests=1
build_max_docker_pull_requests=0
qualification_max_wall_seconds=7200
qualification_max_child_process_starts=4096
qualification_max_written_bytes=2147483648
qualification_max_container_creates=128
qualification_max_container_starts=128
qualification_max_container_removes=128
qualification_max_network_creates=128
qualification_max_network_removes=128
qualification_max_volume_creates=0
qualification_max_image_builds=0
qualification_max_image_pulls=0
sealed_artifact_max_count=10000
source_time=<YYYY-MM-DDTHH:MM:SS.NNNNNNNNNZ>
expires_at=<YYYY-MM-DDTHH:MM:SS.NNNNNNNNNZ>
allow=create_private_base_B_Q_plan_audit_roots;publish_one_exact_I_at_implementation_destination;build_only_in_B_target;one_no-pull_image_build;provider-free_fake-provider_qualification_only;Q-prefix_container_network_create_start_stop_remove;publish_Q_P_R4_and_audit_receipts
deny=live_credentials;live_provider;E_creation;adapter_or_settings_change;volume_create;image_pull;prune;wildcard_or_cross-epoch_delete
END_ENGRAM_NATIVE_FLAGSHIP_V7_A0_V1
```

This grammar, constants, decimal rules, lowercase hex, nanosecond UTC format, line order, and exact
operation strings are the pre-I A0 parser; A0 has no I field. Let declared source time be `Sd`, the
authenticated host-event time `Sh`, coordinator receipt times `R0`/`C0`, and expiry `X`, all integer
nanoseconds; C is `mach_continuous_time` converted with checked `u128` and captured timebase. A0 is
valid only when:

```text
Sd <= Sh <= Sd + 120_000_000_000
Sh <= R0 <= Sh + 120_000_000_000
Sh < X <= Sh + 21_600_000_000_000
A0_continuous_not_after = C0 + (X - R0)  # checked
```

Before and after every A0 mutation, the coordinator checks the same boot UUID, non-regressed real
and continuous clocks, `now_real < X`, `now_continuous < A0_continuous_not_after`, exact target and
prefix, E absence, positive disk reserve, and every cumulative numeric counter. Post-check also
rehashes/reopens the target and inventories Docker objects. Equality at expiry fails. A crash may
perform only exact Q-prefix containment/cleanup within A0; it never broadens or resumes a build.

A0 is the authentic external user event, not its local receipt. Only after I is published does I's
sealer create the audit-only receipt binding source system, task/event IDs, authenticated role,
exact raw message, `Sd/Sh/R0/C0/X`, boot UUID, D/G0/protocol, and mutation journal. At first use and
after every process restart, the coordinator must re-read that same authenticated external event by
stable task/event IDs and byte-compare its role/message/time; the local receipt alone is never
authority. If the event cannot be re-read, no new mutation is allowed and the stage abandons except
for already-authorized exact-object containment. AI output, existing goal text, or a fabricated
receipt is not A0.

## 4. Exact P/R4 and canonical U1

V6's exact 30 rows remain: teaching lane orders 1–12; Codex activation 1, 3, 5, 8, 10, 12; then
evaluation 1–12. P literally enumerates every row and all digests, limits, allowlists, expiries,
prompts, fixtures, auth/state assignments, Docker names, and artifact paths. Independent R4
reconstructs P and the complete U1 projection before any U1 request; E stays absent.

U1 retains V6's fixed decisions but replaces indexed sections with this unambiguous grammar:

```text
ENGRAM_NATIVE_FLAGSHIP_V7_U1_V1
project=engram
digest_count=<N>
digest.<0000..N-1>.<name>=<lowercase-64-hex>
limit_count=<N>
limit.<0000..N-1>.<name>=<unsigned-decimal>
allowset_count=<N>
allowset.<set-index>.<name>.item_count=<M>
allow.<set-index>.<name>.<0000..M-1>=<base64url-no-pad>
expiry_count=<N>
expiry.<0000..N-1>.<name>=<YYYY-MM-DDTHH:MM:SS.NNNNNNNNNZ>
source_time=<YYYY-MM-DDTHH:MM:SS.NNNNNNNNNZ>
decision.provider_execution=accept
decision.exact_30_admissions_no_replay=accept
decision.ephemeral_container_network_cleanup=accept
decision.six_codex_auth_dispositions=exact_delete_or_exact_provider_revocation
ack.claude_admissions=12
ack.claude_usd_ceiling=not_enforceable
ack.post_acceptance_conversation_revocation=not_a_claimed_control
END_ENGRAM_NATIVE_FLAGSHIP_V7_U1_V1
```

Names/sets are ascending raw UTF-8; indices are four-digit, zero-based, contiguous; values have no
whitespace. A zero-item allowlist has `item_count=0` and no `allow` row. `<empty>` is invalid.
Every P/R4 value appears exactly once; extra, missing, duplicate, placeholder, or misordered bytes
fail. The authenticated user event must equal the rendered bytes including final LF.

For U1 use the same `Sd/Sh/R0/C0` formulas, but require `Sh < X <= Sh+86_400_000_000_000` where X
is U1's single epoch expiry and derive `U1_continuous_not_after=C0+(X-R0)` with checked arithmetic.
Each claim requires the cached boot UUID, `now_real >= R0`, `now_continuous >= C0`,
`now_real < X`, and `now_continuous < U1_continuous_not_after`. Forward jumps may expire early;
backward time, boot change, or equality terminates. After U1, conversational revocation is not an
authoritative control; delivered signal/FD EOF remains best-effort containment only. Before any
new post-restart claim, the coordinator must re-read the same authenticated external U1 event by
stable task/event IDs and byte-compare role/message/time. If unavailable, the receipt cannot stand
in for authority: no new claim is permitted and only J/U1-bound containment or cleanup may run.

## 5. Credential eligibility and exact terminal disposition

V6's exact Claude and Codex model/provider/reasoning/config identity gates remain. A selected
external auth broker is auth-only byte-transparent forwarding: one claim-bound client and one
allowlisted upstream; credential injection only at the transport boundary; request/response bodies
and all non-auth bytes forwarded unchanged. It performs no caching, retry, redirect following,
connection reuse, body inspection, request/response mutation, persistence, or content logging;
each downstream request has exactly one fresh upstream attempt. The broker endpoint, credential,
FD, and process are unreachable by every model-driven tool. Headers/bodies never enter evidence.
Failure closes the admission. Consume-and-unmount remains the only other eligible Codex mechanism;
same-process built-in Read must be proven unable to obtain the synthetic credential before U1.

Exactly six auth disposition receipts exist in Codex lane order 1, 3, 5, 8, 10, 12, even for a
never-created copy. Before unlink, an O_EXCL intent binds the exact copy identity/path and proves no
open FD, bind mount, container reference, or other evaluator path can recreate it. Allowed results
are `NOT_CREATED`, `EXACT_FILE_DELETED`, `EXACT_DERIVED_PROVIDER_CREDENTIAL_REVOKED`, or
`LIVE_SECRET_CONTAINMENT_FAULT`. A created copy must then prove held-identity deletion, parent fsync,
absence and no recreation; provider revocation qualifies only for a per-copy derived capability
and must prove the source credential unchanged. The original source-auth file is never a cleanup
target. Mode, path quarantine, and expiry make a copy evaluator-rejected only; they do not revoke
the credential and cannot yield success. If deletion or safe derived revocation is unproven, the
last result applies. Intent/result receipts contain no credential bytes or reusable digest.

After the last claimed/partial H, define in that lane order:

```text
HA[0] = H[last]
HA[j] = SHA256("engram-v7-auth-disposition-v1\0" || HA[j-1] || lane_label || receipt_digest)
```

No successful or partial epoch-terminal artifact may be published before all six exact receipts
and HA[6] exist. Every final and partial report binds HA[6]. Cleanup-only restart may finish these
U1-authorized dispositions but can never create/start provider work. A live-secret result forces a
containment-fault partial report, never a completed comparison.

## 6. Pre-persistence secret boundary and fault cleanup

V7 selects fresh per-admission encrypted staging; no plaintext-disk or tmpfs-only alternative is
permitted. Docker root/fixtures are read-only and log driver is `none`; every writable path is
served only by the I-bound encryption process. It uses XChaCha20-Poly1305 with a fresh 32-byte
`getrandom` key and fresh 16-byte nonce prefix per admission; nonce is
`prefix || u64be(record_counter)`, starting at zero with checked increment and no reuse. Frames are
at most 1 MiB and contain domain `engram-v7-staging-frame-v1\0`, version/kind, counter, canonical
path and offset, plaintext length, nonce, and ciphertext+16-byte tag. AAD is the ordered raw
`P || M || J || path || offset || length || counter` encoding. Unknown frames or authentication
failure fault.

The key exists only in locked, non-dumpable memory, is absent from argv/env/FD inheritance/control
frames, and is explicitly zeroized before normal exit. Core dumps are disabled. Only ciphertext
may reach a host file, named volume, writable layer, page cache, or crash residue. On process crash
the key dies with the address space; ciphertext is exact-object quarantined until cleanup and is
never resumed. Q proves every writable path, including stdout/stderr capture, temp, native memory,
Engram data, client config, and tool output, crosses this boundary. Failure makes the host
ineligible. V7 claims only controlled durable projections and key-loss confidentiality, not
physical erasure of ciphertext or storage blocks.

Only the same lane's last sanitizer-approved native/Engram generation is read-only input; each
phase writes a fresh tmpfs generation. Thus V5/V6 native flags, private homes, phase locality,
no-cross-lane controls, and provider-side-memory confounding labels remain, while all newly retained
state passes the sanitizer before durable promotion.

Normal termination decrypts through bounded locked buffers and sanitizes all staged bytes/metadata
before durable T or state promotion, then zeroizes the key. On a secret/canary hit or incomplete
scan, no T is created. A canonical sanitized fault receipt records only category, location class,
count, and reason—never bytes or a reusable digest—then the key is destroyed. U1/P/M/J already
authorize exact-name cleanup; immediately after watchdog object creation/binding, the resulting
exact IDs activate that capability independently of T. The watchdog can therefore remove exact
container/network/ciphertext staging after a safe fault or peer crash. K binds the result. Missing
removal/not-found proof leaves encrypted residue quarantined and the epoch indeterminate; no raw
secret bytes become durable.

Clean staged data alone may be copied to a new O_EXCL/fsynced/readback-verified generation. Failure
leaves the prior sanitized generation current. The I-bound seal from V6 applies to every post-I
artifact. It proves same-boot local process-crash consistency only—not power-loss, reboot, Docker
Desktop VM/storage, or undetectable snapshot-rollback durability. M-without-claim recovery remains
same-boot/deadline-bound; post-claim restart is containment/cleanup-only.

## 7. V7 watchdog frame, creator ownership, and cleanup states

Only the watchdog invokes Docker create, bind/inspect, start, stop/kill, disconnect, remove, and
not-found APIs. The coordinator requests transitions and never creates or adopts an object. Every
frame is exactly 356 bytes:

```text
0:8    magic 45 4e 47 52 56 37 00 00 ("ENGRV7\0\0")
8:2    version u16be = 1
10:2   kind u16be
12:8   sender-local sequence u64be
20:4   admission ordinal u32be
24:4   phase u32be: teaching=1, activation=2, evaluation=3
28:8   continuous deadline_ns u64be
36:32  raw P SHA-256
68:32  raw M SHA-256
100:32 raw J-CLAIM SHA-256
132:32 raw cleanup-capability SHA-256
164:32 SHA-256(container name)
196:32 raw container-ID SHA-256; zero before creation
228:32 SHA-256(network name)
260:32 raw network-ID SHA-256; zero before creation
292:32 raw T-or-sanitized-fault SHA-256; zero before evidence bind
324:32 raw K SHA-256; zero before K bind
```

Kinds are `HELLO=1`, `READY=2`, `CREATE_NETWORK=3`, `NETWORK_CREATED=4`,
`CREATE_CONTAINER=5`, `CONTAINER_CREATED=6`, `START=7`, `STARTED=8`, `HEARTBEAT=9`,
`TERMINAL_INPUT=10`, `CANCEL=11`, `STOPPED=12`, `BIND_EVIDENCE=13`, `EVIDENCE_BOUND=14`,
`CLEANUP=15`, `REMOVED=16`, `VERIFY_NOT_FOUND=17`, `NOT_FOUND=18`, `TERMINAL=19`,
`TERMINAL_ACK=20`, and `FAULT=21`. The only normal transitions are:

```text
S0 C:HELLO            -> W:READY              -> S1
S1 C:CREATE_NETWORK   -> W creates/binds; W:NETWORK_CREATED   -> S2
S2 C:CREATE_CONTAINER -> W creates/binds; W:CONTAINER_CREATED -> S3
S3 C:START            -> W starts; W:STARTED  -> S4
S4 C:HEARTBEAT        -> W:HEARTBEAT          -> S4
S4 C:TERMINAL_INPUT/CANCEL or W deadline/exit -> W contains; W:STOPPED -> S5
S5 C:BIND_EVIDENCE(T or safe fault + capability) -> W:EVIDENCE_BOUND -> S6
S6 C:CLEANUP          -> W exact removal; W:REMOVED            -> S7
S7 C:VERIFY_NOT_FOUND -> W exact inspect; W:NOT_FOUND          -> S8
S8 C:TERMINAL(K)      -> W verifies K; W:TERMINAL_ACK          -> END
```

Only the shown owner may initiate. Every reply repeats identities/deadline; sequences strictly
increase; zero fields become nonzero only at their listed state. Invalid state/direction/identity,
late frame, or EOF faults. On coordinator loss, watchdog uses the J-bound cleanup capability,
emits/uses only a safe control fault, contains and removes exact bound objects, and never starts a
new one. Missing cleanup/not-found evidence is `SPENT_INDETERMINATE`.

V6's I-bound guest PID-1/subreaper and attainable `/proc` census plus exact Docker Desktop inspect/
event/not-found proof remain; inaccessible host-cgroup proof is not claimed. Q must prove hostile
`setsid`/double-fork containment through the complete V7 cleanup automaton.

## 8. Gates and freeze state

```text
G0  freeze V7; coordinator-sandwich three blind reviews; unresolved P0/P1 must be zero
G1  receive exact authentic A0; create private base/B/Q; publish I using V7 bootstrap rule
G2  I-seal A0/review audits; provider-free Q of exact I and every V7 boundary
G3  materialize exact 30-row P and complete U1 projection
G4  independently reconstruct P/projection; seal R4; require pristine E absence
G5  receive byte-exact authentic U1; seal cache; re-attest boot/time/identity/absence
G6  create E and only U1-listed auth topology
G7  teaching 1–12; durable teaching receipt and strict one-hour V6 retention anchor
G8  activation 13–18; evaluation 19–30; per admission V7 claim/watchdog/sanitize/cleanup
G9  finish six auth dispositions and HA[6]; only then publish completed or partial terminal result
G10 update the flagship audit and independently assess every still-unproven completion criterion
```

V6's atomic same-boot 60-second CLAIM/ABANDON rule and exact retention inequalities remain, with
V7's U1 time formula controlling epoch expiry. Any failure is fail-closed; completed Schema-15 is
never replayed; a changed boundary requires a successor and new applicable authority.

At freeze, V7's private root/prefixes, A0/I/Q/P/R4/U1, E/auth, Docker objects, provider outputs,
M/J/T/K/H/HA, audit receipts, and reports are absent. V6 and predecessors remain untouched;
repository `target/debug` and the forbidden combined-output path remain absent; user-owned changes
are unstaged.

This document authorizes only read-only advisory review and presentation of the literal A0 request.
It authorizes no publication, implementation, build, test, authentication access/copy, Docker
mutation, provider/pilot run, adapter/daemon change, cleanup, staging, commit, or deletion. Those
require the exact applicable authentic event above.
