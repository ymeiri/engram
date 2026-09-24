# Native flagship remaining-evidence execution boundary V8

Date: 2026-09-10
Status: frozen append-only design delta; advisory review plus presentation of one exact future A0
request only; zero implementation, qualification, authentication, Docker, provider, pilot,
cleanup, or deletion authority

## 1. Exact predecessor and delta scope

V8 rejects and supersedes the remaining defects in exact V7:

```text
evals/native_memory_pilot_v1/NATIVE_FLAGSHIP_REMAINING_EVIDENCE_EXECUTION_BOUNDARY_V7_FROZEN_2026-09-10.md
SHA-256  a65e1ebe680669eec8318e953cf888c366685ab8af5b3b6e3cfba819fb28aa3
LF       366
bytes    22193
```

V7 stays immutable and non-executable. V8 replaces V7's G0/A0 canonicalization and pre-I recovery;
U1 name/expiry grammar; encrypted-staging framing; broker equivalence; auth-disposition cleanup;
cleanup capability; watchdog frame, fault, and recovery states; and related gates. Every other V7
clause remains by the exact predecessor identity above. Through V7, the exact protocol, 30-row
schedule, scorer, no-Schema-15-replay rule, flagship semantics, outcomes, no-candidate-Python cut,
and nonclaims remain. Conflict is resolved only in favor of V8.

## 2. Fully canonical G0 evidence

As in V7, `D_digest = SHA256(exact raw V8 bytes)`; there is no D file or I0. Reviews remain advisory
and never create authority. The trusted coordinator computes:

```text
G0_digest = SHA256(
  ASCII "engram-v8-g0-evidence-v1\0" ||
  L(design_identity) || L(protocol_identity) ||
  L(semantic_review) || L(operational_review) || L(claude_review) || L(adjudication)
)
L(x) = u64be(byte_length(x)) || x
```

An identity is `L(canonical_absolute_path) || u64be(device) || u64be(inode) || u32be(uid) ||
u32be(gid) || u32be(mode&07777) || u32be(nlink) || u64be(byte_length) || raw_sha256`, repeated in
pre then post order. A review is `u16be(role) || L(source_system) || L(stable_session_id) ||
L(model) || L(effort) || raw_prompt_sha256 || raw_output_sha256 || u64be(output_length) ||
L(pre_design) || L(post_design) || L(pre_protocol) || L(post_protocol)`. Roles are semantic=1,
operational=2, Claude=3. Empty/`unavailable` stable IDs and unknown enums are invalid.

For review role `r`, zero-based finding ordinal `n`, and severity P0=0/P1=1/P2=2:

```text
finding_digest = SHA256(
  ASCII "engram-v8-review-finding-v1\0" ||
  u16be(r) || u32be(n) || u16be(severity) || L(title) || L(body) || L(evidence)
)
```

Adjudication is `u32be(count)`, then findings in role/ordinal order as
`u16be(role)||u32be(ordinal)||u16be(severity)||raw_finding_digest||u16be(disposition)||L(evidence)`,
then `u32be(unresolved_p0)||u32be(unresolved_p1)`. Dispositions are unresolved=0,
inapplicable-with-clause=1, duplicate-of-prior=2, accepted-nonblocking-P2=3. P0/P1 may use only 0,
1, or 2; P2 may use 0, 1, 2, or 3. Both unresolved counts must be zero. All text is shortest-form
UTF-8; integer overflow, duplicate, omitted field, reordering, or unstable pre/post identity blocks
G0. I later revalidates and seals the exact evidence but cannot self-certify or alter it.

## 3. Canonical A0 event and fail-stop pre-I construction

V8 uses this initially absent private namespace:

```text
base /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v8-stale-safety-20260910-01
B    /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v8-stale-safety-20260910-01/build
Q    /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v8-stale-safety-20260910-01/qualification
P    /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v8-stale-safety-20260910-01/plan
audit /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v8-stale-safety-20260910-01/audit
E    /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v8-stale-safety-20260910-01/execution
Q-prefix engram-native-flagship-v8-stale-safety-20260910-01-qual-
E-prefix engram-native-flagship-v8-stale-safety-20260910-01-exec-
I-destination /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v8-stale-safety-20260910-01/build/implementation-freeze-v1.bin
```

The exact A0 message is V7's ordered literal template with each exact ASCII token `V7` replaced by
`V8` and each exact token `v7` replaced by `v8`, plus these lines immediately after
`qualification_max_image_pulls=0`:

```text
global_max_concurrency=1
build_max_cpu_seconds=14400
build_max_peak_rss_bytes=2147483648
qualification_max_cpu_seconds=14400
qualification_max_peak_rss_bytes=2147483648
plan_max_written_bytes=67108864
plan_max_regular_files=256
audit_max_written_bytes=67108864
audit_max_regular_files=4096
docker_max_reported_logical_content_bytes=4294967296
docker_physical_write_ceiling=unavailable_not_enforced
operation_metadata_max_wall_seconds=60
operation_child_build_max_wall_seconds=7200
operation_docker_build_max_wall_seconds=3600
operation_qualification_action_max_wall_seconds=300
operation_plan_or_audit_seal_max_wall_seconds=60
operation_term_grace_seconds=10
operation_kill_grace_seconds=10
```

It also contains `g0_evidence_sha256=<G0>`, the one exact I-destination above, exact B/Q/P/audit
roots, E-must-remain-absent, Q-prefix actions/counts, source time and expiry, and V7's closed
allow/deny set. It contains no I digest. The Docker byte value is a post-operation Engine-reported
logical-content eligibility bound, not a physical daemon/cache/layer-write ceiling. A0 authorizes
exactly one no-pull build request; the coordinator inventories pre/post image, manifest, config,
layer, BuildKit-cache and reported-size identities and reports physical writes as unavailable. It
does not hide them inside `build_max_written_bytes`.

Let source metadata be authenticated source system, host ID, task/thread ID, nonempty stable event
ID, authenticated role `user`, host source timestamp `Sh`, and exact raw A0 message. Let `R0` be
receipt `CLOCK_REALTIME` ns, `C0` converted `mach_continuous_time` ns, `X` parsed expiry, and B the
raw 16-byte boot-session UUID. V7's declared-source constraints remain and additionally require
`R0 < X`. With text encoded as L above:

```text
A0_event_digest = SHA256(
  ASCII "engram-v8-a0-authenticated-event-v1\0" ||
  L(source_system) || L(source_host_id) || L(task_thread_id) || L(event_id) || L("user") ||
  u64be(Sh) || L(exact_raw_message) || u64be(R0) || u64be(C0) || B ||
  raw_D_digest || raw_G0_digest || raw_protocol_digest || u64be(X) ||
  u64be(C0 + (X - R0))
)
```

All arithmetic is checked; timestamps are unsigned Unix ns. The later I-sealed A0 receipt binds
this digest and source locator only and is audit evidence, never authority. Every use, including
after restart, must re-read and byte-verify the same authenticated event; otherwise abandon.

Before I is sealed, construction is deliberately fail-stop. Base absence is required at first
mutation; one child mutation runs at a time. For operation start C, its active deadline is
`min(C + operation_kind_limit_ns, A0_continuous_not_after)`. A pre-I watchdog owns the child process
group, TERM/KILL deadlines, counters, CPU/RSS sampling, and terminal observation. Any coordinator,
watchdog, child, output-observation, clock, or counter interruption after base creation consumes A0:
no construction resumes, no I is adopted, and only exact Q-prefix/process containment permitted by
A0 may run. A fresh design epoch and authentic A0 are then required. Equality at a deadline fails.

Before/after each mutation, enforce V7 clock/boot/path/prefix/E-absence/disk checks plus the active
deadline, concurrency=1, cumulative process/file/byte/object counters, CPU, peak RSS, plan/audit
separate bounds, and Docker inventory. Unknown accounting fails closed except the explicitly
unavailable physical Docker-byte nonclaim.

## 4. U1 grammar and source-time closure

V8 replaces V7's U1 indexed grammar with:

```text
ENGRAM_NATIVE_FLAGSHIP_V8_U1_V1
project=engram
source_time=<YYYY-MM-DDTHH:MM:SS.NNNNNNNNNZ>
epoch_expires_at=<YYYY-MM-DDTHH:MM:SS.NNNNNNNNNZ>
digest_count=<N>
digest.<index>.<name>=<lowercase-64-hex>
limit_count=<N>
limit.<index>.<name>=<unsigned-decimal>
allowset_count=<N>
allowset.<set-index>.<name>.item_count=<M>
allow.<set-index>.<name>.<item-index>=<base64url-no-pad>
decision.provider_execution=accept
decision.exact_30_admissions_no_replay=accept
decision.ephemeral_container_network_cleanup=accept
decision.six_codex_auth_dispositions=exact_delete_or_exact_provider_revocation
ack.claude_admissions=12
ack.claude_usd_ceiling=not_enforceable
ack.post_acceptance_conversation_revocation=not_a_claimed_control
END_ENGRAM_NATIVE_FLAGSHIP_V8_U1_V1
```

`index`, `set-index`, and `item-index` are exactly four lowercase decimal digits. A name is 1–128
ASCII bytes matching `[a-z][a-z0-9_]*(\.[a-z0-9_]+)*`; there is no escaping. Names and decoded item
bytes are strictly ascending and unique. Counts are canonical decimal without sign/leading zero
except `0`. A zero-item set has its count line and no item line. `epoch_expires_at` is the sole U1
expiry field: every absolute execution/auth/cleanup expiry in P/R4 must equal it; relative deadlines
appear only as limits. R4 reconstructs a bijection: every P/R4 digest, limit and allowlist occurs
exactly once in U1 and U1 contains no extra value. The authenticated event must equal rendered
bytes including final LF. V7's `Sd/Sh/R0/C0`, `R0<X`, same-boot, checked continuous-expiry, strict
pre-expiry and restart re-read formulas apply unchanged.

## 5. Broker application equivalence

An auth broker is eligible only if I/Q prove application equivalence. It rejects any inbound
`Authorization`, `Proxy-Authorization`, provider-key header, Cookie, duplicate Host/`:authority`,
conflicting Content-Length, trailer, upgrade, or unknown hop-by-hop field. It resolves only P's
hostname/port through P's exact resolver configuration and pinned answer set, uses exact SNI, and
verifies P's certificate-chain/SPKI identities. DNS/TLS drift fails; redirects are not followed.

For request and response, canonical application bytes are method/status, scheme, exact decoded
path+query bytes, strictly ordered lowercased end-to-end header-name/value pairs after OWS
normalization, and streaming body length+SHA-256. Upstream/downstream representations must match
after only: injecting the one auth field; replacing Host/`:authority` with the P-bound upstream;
TLS record changes; and removing/recreating `Connection`, `Proxy-Connection`, `Keep-Alive`,
`Transfer-Encoding`, `TE`, `Trailer`, `Upgrade`, `Proxy-Authenticate`, and `Proxy-Authorization` or
Content-Length/chunk framing without changing decoded body bytes. No other application mutation is
permitted. The broker performs no cache, retry, redirect, connection reuse, body parse, response
rewrite, persistence, or content logging; streaming hashes are discarded after equality evidence.
Its endpoint, credential, process and FDs are unreachable by model-driven tools. Inability to prove
all of this makes that host ineligible.

## 6. Metadata-encrypted staging

V7's XChaCha20-Poly1305 key/nonce lifetime remains, but no model-controlled filename, path,
directory shape, mode, time, xattr, link target, or other metadata may appear in a backing filename,
header, AAD, log, or durable index. Every object receives a fresh random 16-byte token; backing
names are only `o-` plus its lowercase 32-hex token. The token-to-path and all metadata map is itself
an encrypted kind-1 record. Directory and data frames use only the token.

The exact canonical header is:

```text
ASCII "engram-v8-staging-header-v1\0" || u16be(version=1) || u16be(kind) ||
u32be(admission_ordinal) || raw_P || raw_M || raw_J || raw_16_byte_object_token ||
u64be(record_counter) || u64be(plaintext_offset) || u32be(plaintext_length) || raw_24_byte_nonce
AAD = u64be(header_byte_length) || header
frame = AAD || u64be(ciphertext_byte_length) || ciphertext_and_16_byte_tag
```

Kinds are encrypted-metadata-map=1 and encrypted-content=2. Plaintext length is at most 1 MiB;
ciphertext length must equal plaintext length+16. Unknown kind, duplicate token/counter/nonce,
counter wrap, malformed length, or plaintext metadata leakage faults. All writable paths, native/
Engram state and captured streams remain behind this boundary; normal sanitation/promotion and
crash key destruction retain V7 semantics. V8 claims controlled projections and computational
key-loss confidentiality, not physical storage erasure or cross-boot durability.

## 7. Cleanup capability, auth receipts, and expiry survival

Before Docker objects, compute the non-authoritative cleanup intent:

```text
X0 = SHA256(
  ASCII "engram-v8-cleanup-intent-v1\0" || raw_P || raw_M || u32be(ordinal) || u32be(phase) ||
  L(container_name) || L(network_name) || raw_image_config_digest ||
  u32be(allowed_operation_mask) || raw_U1_event_digest
)
```

The mask permits only inspect=1, disconnect=2, TERM=4, KILL=8, remove-container=16,
remove-network=32, verify-not-found=64. P/M bind X0 and J-CLAIM binds it before creation. After the
watchdog creates both objects it seals:

```text
X1 = SHA256(
  ASCII "engram-v8-cleanup-id-binding-v1\0" || raw_X0 ||
  L(container_name) || raw_container_id_sha256 || raw_container_inspect_sha256 ||
  L(network_name) || raw_network_id_sha256 || raw_network_inspect_sha256
)
```

Exact IDs/labels/config must match P/M/J. X1 activates cleanup immediately, independently of T or
sanitizer success. U1's exact cleanup authority survives `epoch_expires_at` solely for X0/X1-bound
containment, auth disposition, and evidence sealing; expiry still forbids create/start/provider.

P binds schema digest and these six exact auth receipt basenames:

```text
auth-dispositions/lane-0000000000000001-v1.bin
auth-dispositions/lane-0000000000000003-v1.bin
auth-dispositions/lane-0000000000000005-v1.bin
auth-dispositions/lane-0000000000000008-v1.bin
auth-dispositions/lane-000000000000000a-v1.bin
auth-dispositions/lane-000000000000000c-v1.bin
```

Each canonical receipt binds P/U1, lane/path, pre-unlink intent, copy-created boolean, no-FD/mount/
container/recreation observations, result enum, source-auth unchanged observation, real/continuous
times, and evidence digests—never credential bytes/digest. V7's exact deletion or safe derived-
credential revocation rule, six-receipt HA chain, live-secret fault, source-auth exclusion, and
terminal-publication wait remain. Cleanup-only restart may finish them after U1 expiry.

## 8. Watchdog ownership, abnormal faults, and recovery actor

V8 replaces the frame with exactly 388 bytes. V7 offsets 0–131 retain their meanings with V8 magic
`45 4e 47 52 56 38 00 00`; then:

```text
132:32 raw X0
164:32 raw X1; zero until both objects are bound
196:32 SHA-256(container name)
228:32 raw container-ID SHA-256; zero before create
260:32 SHA-256(network name)
292:32 raw network-ID SHA-256; zero before create
324:32 raw T-or-sanitized/control-fault SHA-256; zero before evidence bind
356:32 raw K SHA-256; zero before K bind
```

V7's normal state machine remains with shifted fields, watchdog-only Docker creation/binding, and
X1 becoming nonzero in `CONTAINER_CREATED`. Abnormal states are exact:

```text
F0 fault, no object bound -> seal safe control fault + no-object K -> END
F1 fault, any object bound -> watchdog use X0/X1 to inspect/disconnect/TERM/KILL -> F2
F2 stopped/proven or indeterminate -> seal safe fault evidence -> F3
F3 remove exact bound objects -> verify exact-ID not-found -> seal K -> END
R0 watchdog death -> recovery actor O_EXCL recovery claim -> inspect exact J/P names only -> R1
R1 no object -> seal no-object K -> END
R1 exact labels/config/ID match -> derive/verify X1; disconnect/TERM/KILL/remove -> R2
R1 ambiguity/mismatch -> seal indeterminate K without mutation -> END
R2 verify exact-ID not-found; seal K -> END
```

The I-bound recovery actor is a separate mode with an API allowlist limited to inspect/list-exact-
name, disconnect, stop, kill, remove, and not-found. It cannot create, start, restart, exec, copy,
attach, adopt, pull, build, or contact a provider. It reads only P/M/J/X0 and durable binding frames;
if an ID receipt is missing it may resolve the deterministic exact name only after labels/image/
config match. One O_EXCL recovery claim prevents concurrent actors. Recovery is cleanup-only even
on the same boot and even before U1 expiry; it never resumes an admission.

Coordinator death leaves the watchdog responsible for F1–F3. Watchdog death invokes R0–R2. If
both die, a later same-boot actor may perform only R0–R2 after re-reading the authenticated U1 event;
if it cannot, it may still use the already durable J/X0 as narrowly preauthorized emergency
containment but may not create other evidence beyond safe fault/K receipts. Unknown state is
`SPENT_INDETERMINATE`, never replay.

## 9. Gates, durability, and freeze

V7's gates remain with G0 using Section 2; G1 requiring exact A0 event digest and fail-stop pre-I;
G2 qualifying resource accounting, broker, encrypted metadata, X0/X1, all fault/recovery states;
G4 checking U1's bijection/single expiry and six receipt names; and G9 waiting for HA[6]. Any
failure is fail-closed and any changed boundary requires V9 plus new applicable authority.

All O_EXCL/fsync/readback claims are same-boot local process-crash consistency only. They do not
prove power-loss, reboot, Docker Desktop VM/cache/layer, filesystem/hardware, or snapshot-rollback
durability. Pre-I interruption is irrevocably fail-stop; M-without-claim remains same-boot and
deadline-bound; post-claim restart is exact containment/cleanup only.

At freeze, the V8 namespace/prefixes, A0/I/Q/P/R4/U1, E/auth, Docker objects, provider outputs,
M/J/X0/X1/T/K/H/HA, audit receipts, and reports are absent. V7 and predecessors remain untouched;
repository `target/debug` and the forbidden combined-output path remain absent; user-owned changes
remain unstaged.

This document authorizes only read-only advisory review and presentation of exact future A0 bytes.
It authorizes no publication, implementation, build, test, authentication access/copy, Docker
mutation, provider/pilot run, adapter/daemon change, cleanup, staging, commit, or deletion.
