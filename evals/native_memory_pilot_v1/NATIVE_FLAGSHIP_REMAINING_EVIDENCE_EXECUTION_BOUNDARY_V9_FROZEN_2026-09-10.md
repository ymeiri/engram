# Native flagship remaining-evidence execution boundary V9

Date: 2026-09-10
Status: frozen append-only design delta; advisory review plus presentation of exact future A0 bytes
only; zero implementation, qualification, authentication, Docker, provider, pilot, cleanup, or
deletion authority

## 1. Corrected predecessor closure

V9 rejects exact V8:

```text
evals/native_memory_pilot_v1/NATIVE_FLAGSHIP_REMAINING_EVIDENCE_EXECUTION_BOUNDARY_V8_FROZEN_2026-09-10.md
SHA-256  95d5226f9dc4fa94211b3cb4a8a4fab9c15c4b7a92d4895476a81e2195f4adfa
LF       335
bytes    18198
```

V8's predecessor line was malformed and therefore did not bind inherited V7. V9 directly binds:

```text
evals/native_memory_pilot_v1/NATIVE_FLAGSHIP_REMAINING_EVIDENCE_EXECUTION_BOUNDARY_V7_FROZEN_2026-09-10.md
SHA-256  a65e1ebe680669eec8318e953cf888c366685ab8af5b0a89e4d4df4544471759
LF       366
bytes    22193
```

V7/V8 remain immutable and non-executable. V9 incorporates V8's repairs except its bad inherited
identity and replaces only G0 timing, pre-I/image construction, encrypted framing, broker lifecycle,
X/recovery/watchdog handling, and related gates. All other V7/V8 clauses—including the exact
protocol and 30 admissions, scorer, A0/U1 event authority, auth dispositions, retention, outcomes,
no-Schema-15-replay, no-candidate-Python, and nonclaims—remain by the two exact hashes above.
New V9 domains/namespaces use literal `v9`/`V9`; no older artifact is adopted.

## 2. Exact G0 observation timing and encoding

`D_digest = SHA256(exact raw V9 bytes)`; no D file or I0 exists. Reviews run sequentially in fixed
semantic, operational, Claude order. A single identity observation is:

```text
L(canonical_absolute_path) || u64be(device) || u64be(inode) || u32be(uid) || u32be(gid) ||
u32be(mode&07777) || u32be(nlink) || u64be(byte_length) || raw_32_byte_sha256
L(x) = u64be(byte_length(x)) || x
```

The coordinator takes batch-pre design/protocol observations immediately before review 1's
per-review pre observations, and batch-post observations immediately after review 3's output is
captured and before any other filesystem read. For each review it takes per-review pre observations,
dispatches exactly that prompt, captures the complete output, then takes post observations before
the next review. All eight design and eight protocol values must be identical across batch and
per-review observations.

V8's review/finding/adjudication widths, enum values, order and digest preimages remain. The V9 G0
preimage replaces each ambiguous `identity` with
`L(batch_pre_observation)||L(batch_post_observation)` and each review with its exact separate
`L(pre_design)||L(post_design)||L(pre_protocol)||L(post_protocol)`. No pair is nested where a single
observation is expected. Unknown/trailing bytes, concurrent review, missing stable session ID, or
identity drift blocks G0.

## 3. No pre-I Docker or resumable construction

V9 uses only this fresh initially absent namespace:

```text
base /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v9-stale-safety-20260910-01
B/Q/P/audit/E are the corresponding build/qualification/plan/audit/execution children
Q-prefix engram-native-flagship-v9-stale-safety-20260910-01-qual-
E-prefix engram-native-flagship-v9-stale-safety-20260910-01-exec-
I /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v9-stale-safety-20260910-01/build/implementation-freeze-v1.bin
```

The exact A0 template is V8's template with only exact ASCII `V8`→`V9` and `v8`→`v9` replacement,
plus these replacements:

```text
build_max_docker_build_requests=0
qualification_max_image_builds=1
allow=create_private_base_B_Q_plan_audit_roots;publish_one_exact_I_at_implementation_destination;deterministic_offline_host_cargo_build_only_pre-I;provider-free_fake-provider_qualification_only;one_post-I_Q_no-pull_image_build;Q-prefix_container_network_create_start_stop_remove;publish_Q_P_R4_and_audit_receipts
deny=live_credentials;live_provider;E_creation;adapter_or_settings_change;pre-I_Docker_or_BuildKit;volume_create;image_pull;prune;wildcard_or_cross-epoch_delete
```

All unchanged literal lines and order remain. Before I, Docker/BuildKit socket access and daemon
mutation are forbidden. The only build child is env-cleared, locked/offline host Cargo with exact
toolchain/config/source/dependency identities, `CARGO_TARGET_DIR=B/target`, path remapping, one
process group and A0's active deadline/CPU/RSS/file/byte limits. Repository `target/debug` remains
absent. Any pre-I interruption consumes A0 and is cleanup-only exactly as V8 specifies.

I freezes the OCI recipe, ordered rootfs inputs, guest sources/toolchain, expected configuration,
and build verifier—not a future image digest. After I, provider-free Q makes exactly one no-pull
image build through I's bounded daemon-call protocol in Section 7. Q independently checks the
resulting OCI manifest/config/layer/platform identities against I, binds their digests and cleans
only unexpected builder/cache intermediates. P/R4/U1 bind the exact accepted image. A missing
BuildKit job ID, cancellation capability, object inventory, completion, quiescence, or cleanup
receipt makes Q ineligible; client-process death is not daemon-job termination.

V8's canonical `A0_event_digest`, including authenticated metadata, exact message, clocks and boot
ID, remains with the V9 D/G0/protocol and namespace. It additionally requires `R0<X`. The later
receipt only binds the event digest. Every new mutation re-reads the same authenticated event.

## 4. Metadata-encrypted frame correction

V9 permits bounded ciphertext length/frame-count leakage but no semantic filename/path/metadata.
One random 16-byte object token is unique per logical object and intentionally repeats across that
object's frames. `(object_token, object_frame_index)` is unique; frame index begins at zero and is
contiguous; plaintext offsets begin at zero and equal the prior offset plus prior length. A separate
admission-global nonce counter begins at zero, is unique and strictly increasing, and may not wrap.
The 24-byte nonce is exactly a fresh per-admission 16-byte random prefix followed by
`u64be(nonce_counter)`.

The V8 staging header is replaced by:

```text
header = ASCII "engram-v9-staging-header-v1\0" ||
  u16be(version=1) || u16be(kind) || u32be(admission_ordinal) ||
  raw_32_byte_P_digest || raw_32_byte_M_digest || raw_32_byte_J_digest ||
  u16be(token_length=16) || raw_16_byte_object_token ||
  u64be(object_frame_index) || u64be(nonce_counter) || u64be(plaintext_offset) ||
  u32be(plaintext_length) || raw_24_byte_nonce
AAD = u64be(byte_length(header)) || header
frame = AAD || u64be(ciphertext_length) || ciphertext_and_16_byte_tag
```

Kinds remain encrypted-metadata-map=1/content=2. The encrypted map contains every semantic name,
path, hierarchy, link, mode, time and xattr. Backing names contain only the opaque token. P limits
object count, frames/object, total frames, per-frame plaintext, and total ciphertext; only those
length/count values may persist unencrypted. Duplicate/gapped pair, offset, nonce, malformed length,
semantic leak, or bound breach faults before promotion.

## 5. Broker/helper as an independently contained sub-boundary

V8's broker rejects all request trailers rather than canonicalizing them and compares the exact raw
request-target octets—never a decoded/re-encoded path. Its permitted application difference remains
only one injected auth field. Transport may alter only P-bound Host/`:authority`, TLS records, the
enumerated hop-by-hop fields, and byte framing whose decoded body is identical. Duplicate/existing
auth, cookies, trailers, ambiguous length, DNS/SNI/certificate/SPKI drift, or any other difference
fails closed.

Before connecting upstream, the broker encrypted-buffers the entire bounded request, validates it,
and computes its canonical application digest. It encrypted-buffers and validates the entire
bounded upstream response before releasing any response byte downstream. Unknown/chunked length
must finish within P's encrypted byte/time bound; overflow closes without upstream/downstream
forwarding where not already causally possible. No mismatch is discovered after the affected bytes
are forwarded. Streaming is not a fidelity claim.

The content-free equality receipt binds P/main-J, broker manifest/claim, raw-target digest, canonical
request/response application digests and lengths, exact permitted-transformation bitmask, upstream
DNS/TLS identity digests, timestamps, and pass/fail; it contains no header/body/cookie/token bytes.
Failure emits only a sanitized fault receipt.

Every external broker/helper has its own P-enumerated manifest `MB`, O_EXCL claim `JB`, cleanup
intent `XB0`, actual PID/start/socket binding `XB1`, and watchdog. It has a fresh non-reused socket,
credential capability, and process per admission. Main J binds MB/JB/XB0; no upstream connection
precedes both claims. Terminal evidence proves process death, socket not-found, credential
zeroization, no FD/client/tool reachability, and XB cleanup. These receipts bind main T/K/H. Missing
proof is a live-secret containment fault; a shared/unwatched helper is ineligible.

## 6. Acyclic network/container cleanup capabilities

M does not contain X0. After durable M, compute:

```text
X0 = SHA256(
  ASCII "engram-v9-cleanup-intent-v1\0" || raw_P || raw_M || u32be(ordinal) || u32be(phase) ||
  L(network_name) || L(container_name) || raw_image_config || u32be(operation_mask) || raw_U1_event
)
```

Here `raw_P`, `raw_M`, `raw_image_config`, and `raw_U1_event` are their exact raw 32-byte SHA-256
digests, never artifact bytes.

J-CLAIM binds X0 one-way before objects. Mask bits are inspect=1, disconnect=2, TERM=4, KILL=8,
remove-container=16, remove-network=32, verify-not-found=64, remove-encrypted-staging=128. After the
watchdog creates the network it seals:

```text
X1N = SHA256("engram-v9-cleanup-network-v1\0" || raw_X0 || L(network_name) ||
              raw_network_id || raw_network_inspect)
```

X1N immediately authorizes only inspect/disconnect/remove/not-found for that network and staging;
container operations are invalid. After container create it seals:

```text
X1C = SHA256("engram-v9-cleanup-container-v1\0" || raw_X1N || L(container_name) ||
              raw_container_id || raw_container_inspect)
```

X1C authorizes exact container then network cleanup. U1 cleanup/auth disposition authority survives
expiry only for these bound operations and V8's six exact P-bound auth receipts; expiry never
permits create/start/provider except the exact derived-credential revocation endpoint/action already
named in P/U1 as cleanup-only. Source authentication remains untouched.

## 7. Durable Docker calls and quiescent recovery

The watchdog seals a call intent before every Engine/BuildKit call:

```text
DI = SHA256("engram-v9-docker-call-intent-v1\0" || prior_call_head || raw_P || raw_M || raw_J ||
            u32be(call_ordinal) || u16be(operation) || raw_request_digest || L(exact_name) ||
            raw_known_ID_or_zero || raw_daemon_identity || raw_event_cursor ||
            u64be(start_continuous_ns) || u64be(call_deadline_ns))
```

Only one call is in flight. Completion is an O_EXCL receipt binding DI, response status/digest,
completion time, daemon/build-job ID, event high-watermark, resulting inspect/absence observations,
and the next call-head. BuildKit calls additionally bind solve/job ID, cancel request/result,
builder/cache/layer/image inventory and quiescence. Client EOF never completes a daemon call.

Before recovery, prove the original watchdog terminal: parent `waitpid` status when available;
otherwise exact PID/start identity must be absent or mismatched in two `proc_pidinfo` observations
one second apart after its lease deadline. `kill(pid,0)` alone is insufficient. If the last DI lacks
completion, recovery waits through its bound call deadline, keeps the same daemon event stream
filter active, and reconciles every matching event. A discovered object is ID-bound and cleaned.

No-object K requires two identical exact-name/label absence inspections at least five continuous
seconds apart, both after the call deadline, with unchanged daemon identity, successful daemon
barriers, and no matching create/start event through the second high-watermark. The observations,
event interval and barrier receipts are sealed. Any ambiguity is indeterminate, never absence.

Recovery claims are `recovery/generation-<four-digit>.bin`, maximum eight. Each binds prior claim,
actor PID/start/boot, generation, acquired time, 60-second lease, DI/call head, and cleanup-only mask.
Generation 0 requires watchdog-death proof. Takeover generation g+1 requires current time at/after
the prior lease plus two one-second-separated dead-owner observations. Operations must finish or
fault within the lease; renewal is forbidden. The actor may inspect, disconnect, stop/kill, remove,
verify absence and remove encrypted staging only. It cannot create/start/restart/exec/copy/attach/
adopt/build/pull or contact a provider.

## 8. V9 watchdog and abnormal states

V9 frames are exactly 420 bytes. Offsets 0–131 retain V8 meanings with magic
`45 4e 47 52 56 39 00 00`; then:

```text
132:32 raw X0
164:32 raw X1N; zero before network bind
196:32 raw X1C; zero before container bind
228:32 SHA-256(network name)
260:32 raw network-ID SHA-256; zero before network create
292:32 SHA-256(container name)
324:32 raw container-ID SHA-256; zero before container create
356:32 raw T-or-sanitized/control-fault SHA-256; zero before evidence bind
388:32 raw K SHA-256; zero before K bind
```

V8's kinds and normal transition ownership remain, but `NETWORK_CREATED` makes X1N/network ID
nonzero and `CONTAINER_CREATED` makes X1C/container ID nonzero. Fault/recovery operations are
conditional:

```text
no ID       -> DI reconciliation and quiescent two-observation absence only
network ID  -> X1N network/staging containment; no container operation
container ID-> X1C container then network/staging containment
watchdog dead -> bounded-generation recovery protocol in Section 7
```

Every normal/fault Docker call has DI/completion. Original-watchdog terminal proof precedes recovery.
Coordinator death leaves the watchdog to contain; watchdog death can never be treated as object
absence. Safe fault/K receipts contain only identities, counts, status and evidence digests.

## 9. Gates, durability, and freeze

G0 uses Section 2. G1 uses V9 A0, fail-stop host-only pre-I build and zero Docker objects. G2 builds
and qualifies one image post-I and tests encrypted metadata, broker/helper sub-boundary, X0/X1N/X1C,
DI/completion, fault, watchdog-death and lease-takeover paths. G4 verifies U1's zero-based contiguous
indices, exact ASCII name grammar, one `epoch_expires_at`, P/R4 bijection, cleanup carve-out, broker
receipts and six auth basenames. Later gates retain V8 ordering and wait for all normal/fault,
broker/helper, Docker-call, cleanup, auth and HA receipts before any terminal publication.

Durability claims remain same-boot local process-crash consistency only. They exclude power loss,
reboot, Docker Desktop daemon/VM/cache/layer persistence, filesystem/hardware rollback and
undetectable snapshots. Pre-I interruption consumes A0; M-without-J is same-boot/deadline bounded;
post-claim and watchdog-death paths are cleanup-only and never replay provider work.

At freeze, the V9 namespace/prefixes, A0/I/Q/P/R4/U1, E/auth, Docker/BuildKit objects/jobs, provider
outputs, M/J/X/DI/T/K/H/HA/broker/audit receipts and reports are absent. V7/V8 and predecessors are
untouched; repository `target/debug` and the forbidden combined-output path remain absent; user
changes remain unstaged.

This document authorizes only read-only advisory review and presentation of exact future A0 bytes.
It authorizes no publication, implementation, build, test, authentication access/copy, Docker
mutation, provider/pilot run, adapter/daemon change, cleanup, staging, commit, or deletion.
