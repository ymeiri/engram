# Native flagship remaining-evidence execution boundary V4

Date: 2026-09-10
Status: frozen design boundary; non-authoritative advisory review only; zero implementation,
qualification, authentication-copy, Docker, provider, pilot, or deletion authority

## 1. Purpose and disposition

This append-only design defines the smallest trustworthy boundary for collecting the remaining
native stale-safety evidence for the Engram flagship goal. It deliberately stops calling the work
Stage C2B2a: the work is a new, bounded evidence epoch, not execution or repair of a historical
candidate.

V4 succeeds these rejected lean designs without modifying or deleting them:

```text
V2  evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_LEAN_USER_AUTHORIZED_EXECUTION_BOUNDARY_V2_FROZEN_2026-09-10.md
    86e7517663646f93dadaf81e9a70538475f62d194d9fb7ebfacd4d1d5dfab782
    413 LF / 22479 bytes
V3  evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_LEAN_USER_AUTHORIZED_EXECUTION_BOUNDARY_V3_FROZEN_2026-09-10.md
    c1ae9602ae0a98ef82b80ef262a860960e7f3203d6b931a0966f3130d570e594
    382 LF / 22817 bytes
```

The consolidated V3 gate was `FAIL P0=0/P1=12/P2=9`. Its reviews accepted the no-candidate
architecture but rejected lifecycle, isolation, admission, authentication, manifest, and
claim-language defects. AI reviews are advisory defect reports; neither their acceptance nor this
document can manufacture human authority.

The prospective advisory review path for this exact design is:

```text
evals/native_memory_pilot_v1/NATIVE_FLAGSHIP_REMAINING_EVIDENCE_EXECUTION_BOUNDARY_V4_ADVISORY_REVIEW_2026-09-10.md
```

It must be absent at design freeze. If later created, it is evidence about the exact V4 bytes only,
never an authorization record.

## 2. Immutable completed baseline: evidence, never work to replay

The following completed Schema-15 epoch is the exact structural predecessor:

```text
checkpoint
  evals/native_memory_pilot_v1/SCHEMA15_EXACT_BASH_COMPLETED_2026-09-05.md
  c000c40dea6302ca1882cb5890c01d00d2ccbcb6c64a8cfba693cd7666b9cccd
  151 LF / 9073 bytes
protocol
  evals/native_memory_pilot_v1/protocol-schema-15-required-host-action-bounded-query-repeated-file-cache-exact-bash.json
  c8340f3309df199b2a7e974bc2aaf8d25cd34099ec8331194bdd9c285873be96
  178 LF / 5248 bytes
run plan
  /Users/yuval.meiri/.engram/evals/native-memory-pilot/required-host-action-bounded-query-repeated-exact-bash-20260905-04/run/run-plan.json
  d7495a2915f3b2b4bc43e665af162b4cc95741240fecb57a22b1d1bd9e1646ea
  2832 LF / 401869 bytes
```

That predecessor completed 18 teaching admissions, a one-hour retention interval, nine Codex
activation admissions, and 18 evaluation admissions: 45 native-harness admissions across six
host/arm cells and three repetitions. It is immutable completed evidence. V4 must never rerun,
resume, adopt, repair, reconstruct, or relabel any of its lanes, even if a trace later appears
incomplete. Its frozen evaluator, Engram binary, phase receipts, reports, completion audit, traces,
and authentication topology remain historical evidence only.

The new protocol schema has no predecessor field. This section is therefore the explicit
append-only predecessor/no-replay link. Implementations must not invent a JSON field, rewrite the
new protocol, or use the old run plan as an executable input. A later exact implementation-freeze
artifact and exact run plan must bind this V4 document by hash; that transitively preserves this
historical disposition.

## 3. Exact new protocol and experimental unit

The sole frozen successor protocol is:

```text
/Users/yuval.meiri/projects/engram/evals/native_memory_pilot_v1/protocol-native-stale-safety-v1-schema-10-file-cache.json
SHA-256  282875027ba8fc5732edfca9a05b58a07dcdce12851156575052592cf8c68fc8
LF       332
bytes    10404
final    LF (0a); no CR; no NUL
schema   10
pilot    native-stale-safety-v1-20260910-01
```

It fixes two cases:

1. tracked prerequisite source `toolchain.toml` is absent in the moved checkout;
2. the prerequisite still says version `3`, but trusted procedure verification expired after
   exactly 300 seconds.

Both cases expect the first action `inspect_procedure_prerequisites` and outcome `abstain`. The
matrix is exactly Codex and Claude Code crossed with native memory, lean Engram, and Engram plus
native memory, one repetition each. The protocol fixes its 12-lane mixed order, opaque lane labels,
native semantic/TTL markers, one-hour shared retention gate, bounded `procedure_match`, result
limits, Codex `chatgpt_file_cache`, Claude model `claude-haiku-4-5`, Claude maximum 12 turns, and a
50-cent Claude authorization ceiling. It still requires explicit provider-execution approval.

The protocol's `resource_budgets` values are post-hoc, evaluation-only matched-delta eligibility
gates. They are not preemptive wall-clock or tool-output limits over teaching or activation. V4's
hard per-admission local limits are additional controls selected and frozen in the later run plan;
reports must keep those two meanings separate.

The new epoch has exactly 30 durable native-harness admissions:

```text
teaching    12  (6 Codex + 6 Claude)
activation   6  (Codex only, after the shared one-hour gate)
evaluation  12  (6 Codex + 6 Claude)
total       30  (18 Codex + 12 Claude)
```

An admission is one launched native CLI process under one durable claim. It is not one provider
HTTP request, one transport retry, one model turn, or one tool action. Internal retries and
provider request counts are reported only when authenticated and observable; otherwise they are
`unavailable`. The evaluator must not claim that 30 admissions means 30 provider requests.

The protocol freezes the expected outputs and scoring inputs, but no exact V4 scorer executable
exists at this design freeze. The 2026-09-05 stale-safety source hashes are historical and have
since changed; V4 does not mislabel them as the new scorer. Before user acceptance or any provider
claim, a later append-only implementation-freeze artifact must bind one exact tuple containing:

- this protocol identity;
- the complete scorer source closure, path/role map, and each raw-byte SHA-256;
- exact scorer test corpus and expected results;
- exact evaluator source closure and host executable SHA-256;
- every guest executable/helper ELF identity, if any;
- immutable container image repository digest, image ID, platform, and configuration digest; and
- the runtime manifest schema and exact manifest SHA-256.

No scorer, expected outcome, case, arm, ordering, retention rule, or observation may be changed to
make a result pass. Any later change to the frozen tuple requires an append-only successor and, if
acceptance has already occurred, a new authentic acceptance.

## 4. Preserved flagship semantics

V4 must exercise real native Codex and Claude Code sessions. Direct provider APIs, response-only
chat calls, canned outputs, replayed traces, a tool-free substitute, or a fake MCP broker cannot
stand in for either harness.

The frozen experiment retains:

- separate teaching, Codex activation, and fresh evaluation sessions;
- the shared one-hour retention interval and the 300-second procedure-expiry condition;
- a moved immutable checkout with no transcript, prompt, session, or hidden-context handoff;
- native memory only in native and combined arms;
- an isolated persistent Engram datastore only in Engram and combined arms;
- the real restricted Engram profile with `orient`, teaching-memory creation/verification, and
  exactly the protocol-permitted evaluation `procedure_match` behavior;
- native source-reading and command tools required by the protocol, including the Codex code-mode
  companion and Claude Code `Read`/`Bash` surface;
- both the known failed and corrected fixture commands, so a runner cannot bias the outcome by
  suppressing the failure; and
- scoring from authenticated traces and observed files, not model self-report.

Passing this epoch supplies only the named stale/missing-source evidence. It cannot by itself prove
production readiness, statistical superiority, or completion of every original flagship
criterion.

## 5. Deliberate implementation cut

The historical Python launcher, controller, and provisioner are permanently non-executable inputs:

```text
evals/native_memory_pilot_v1/native_c2b2a_completion_audit_launcher.py
  b5e50318b84a1ba442500781bef4bf405559a43fca46737d8f82b1d236fe2530
engram-eval/native-c2b2a-payload/build-support/controller.py
  eefa6eecb239ee1df49ba51317702cf78199edb7432ce32314b46a8db726b9f9
engram-eval/native-c2b2a-payload/build-support/provision.py
  aabe6b17ac366036a7cef05826106ea1cb6b8a30e5437937b13320ca44599261
```

Nothing in V4 may import, execute, interpret, compile, AST-parse, package, copy into an executable
zone, or derive authority from those files. Python is not part of the evaluator or execution TCB.
Their manifests, authority markers, recovery logic, and outputs are rejected historical designs.

V4 uses a small coordinator-selected Rust evaluator. It may reuse exact-hash, pure trace parsing,
validation, and scoring components plus the `native_successor_core` O_EXCL/fsync primitives after
they are included in the future source closure. It must not retrofit or call the historical
`native_runner` execution lifecycle. In particular, these behaviors are forbidden:

- `recover_interrupted_evaluation_lane` or any teaching/evaluation recovery, skip, or adoption
  path;
- `run_provider_command` or `run_command_status` host spawning;
- inherited ambient environment passed to a provider process;
- unbounded `.status()` waiting or host process-group supervision of daemon-owned containers; and
- the historical Colima observation as provider-execution authority.

Model output and traces are bounded data only. The evaluator never evaluates them as code, imports
them, loads them as a plugin, or marks them executable.

## 6. Exact future roots and Docker namespaces

The future namespace is fixed now:

```text
base
  /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v4-stale-safety-20260910-01
implementation/build root B
  /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v4-stale-safety-20260910-01/build
qualification root Q
  /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v4-stale-safety-20260910-01/qualification
execution root E
  /Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v4-stale-safety-20260910-01/execution
qualification Docker prefix
  engram-native-flagship-v4-stale-safety-20260910-01-qual-
execution Docker prefix
  engram-native-flagship-v4-stale-safety-20260910-01-exec-
```

At design freeze, `base`, `B`, `Q`, `E`, and every object with either prefix must be absent. `B`,
`Q`, and `E` are different canonical, owner-private roots with different absent child basenames and held parent
directory descriptors. Neither may be a symlink, hard link, mount alias, descendant of the other,
or share a Docker container, network, writable volume, claim namespace, evidence chain, or object
prefix.

Only `Q` may run version, fake-provider, sandbox, canary, or authentication-mechanism probes. `Q`
is non-authorizing and performs no live model inference. Its success is a qualification input, not
permission to create `E` or contact a provider. `E` accepts only the exact, independently frozen Q
receipt hash and must not execute probe or repair behavior.

Namespace creation is one-shot. The evaluator stable-opens a trusted existing ancestor, proves an
absent child with no-follow operations, creates it mode `0700`, opens and verifies the new directory,
fsyncs it and its parent, and retains the descriptor. Preexistence, aliasing, wrong identity,
partial state, or uncertainty is terminal. Pre-acceptance implementation work, if separately
authorized by the user's task, uses only `B/target`; provider-free qualification uses Q. Repository
`target/debug` must remain absent.

## 7. Honest authority and future acceptance

This document records no acceptance. A future provider admission requires all of:

```text
U  authentic user acceptance of the exact V4 and implementation-freeze hashes
M  exact runtime manifest and exact run-plan identities selected by the honest coordinator
Q  passing, exact-hash provider-free qualification receipt
I  passing provider-specific auth, egress, child-tool, process, and filesystem isolation
S  absent, then durably created, one-shot admission claim
```

After advisory design review has no unresolved P0/P1 and the implementation-freeze tuple is exact,
one authentic user message may accept those exact bytes and delegate the honest coordinator to
select a later exact run-plan hash within them. That message must explicitly cover:

- project `engram`, repository `https://github.com/ymeiri/engram`, and only the protocol above;
- one epoch under the exact roots and prefixes above;
- exactly 30 native-harness admissions, of which 18 are Codex and 12 are Claude;
- no extra admission, repair, replay, adoption, or second attempt;
- the exact later run-plan, phase-manifest, image, runtime, and sandbox identities chosen under the
  accepted implementation-freeze constraints;
- the six guarded Codex cache copies in Section 10 and zero Claude plaintext copies;
- up to 50 cents of Claude runner spend, with per-call overshoot possible only at a native turn
  boundary and recorded honestly;
- host-enforceable admission, wall-time, process, memory, CPU, disk, and output limits;
- bounded creation/start/stop/inspect of exact-prefix Docker objects and the cleanup authority in
  Section 15;
- owner-only append-only evidence and the strongest result label
  `AUTOMATED_EXECUTION_COMPLETED_UNREVIEWED`; and
- an exact absolute expiry chosen in that message, after which no new claim may be created.

The message delegates future hash selection to an honest coordinator; it does not falsely say the
user reviewed future bytes. A local receipt is an audit cache of the authentic event, not its
source. AI text, an advisory PASS, an earlier provider budget preference, or an authentication-copy
approval cannot create or expand provider-execution authority.

Any run-plan, scorer, image, sandbox, authentication method, admission count, budget, root, prefix,
or expiry outside the accepted boundary requires a new append-only successor and authentic user
acceptance before a claim.

## 8. Runtime manifest and object identity

The trusted evaluator reconstructs one flat, path-keyed raw-byte runtime manifest from
coordinator-selected original paths. There is no candidate-created manifest or executable bundle.
Each row has exactly one semantic role and binds original path, container path if applicable,
regular-file identity, mode, size, raw-byte SHA-256 or platform code identity, and entrypoint role.
Missing, duplicate, extra, permuted, aliased, linked, special, or rebound entries fail closed.

The manifest must include:

- exact V4, implementation freeze, protocol, run plan, output schema, prompts, fixtures, response
  schema, scorer closure, scorer tests, and expected results;
- exact host evaluator and watchdog executable SHA-256, source closure, build inputs, and guest ELF
  identities;
- exact Codex, Codex code-mode companion, Claude Code, Engram, Docker CLI, Docker daemon/API, and
  required runtime versions and code identities;
- container image repository digest, image ID, platform, ordered layer/rootfs digests, and full
  create configuration digest;
- exact entrypoint, argv, cwd, environment allowlist and values/digests, user/group, mounts,
  read-only paths, writable volumes, network, DNS/proxy route, capabilities, seccomp policy,
  `no-new-privileges`, PID/memory/CPU/disk/output limits, stop timeout, and restart policy;
- exact Q/E roots, prefixes, deterministic object names, lane/phase/admission map, auth map,
  persistent-state map, expected absent paths, watchdog frames, deadlines, and cleanup rules; and
- an explicit empty ambient-environment baseline. No provider child inherits the coordinator's
  shell environment.

The evaluator stable-opens and hashes selected regular files without following links. It copies
only declared fixture/config data into non-executable, read-only guest inputs. It revalidates the
manifest before Q, after Q, before each E claim, after each admission, and before final publication.
Any drift is terminal. SHA-256 records identity, not semantic safety or resistance to a malicious
same-UID coordinator; the accepted trust model is an honest coordinator plus local OS and Docker
TCB.

Docker objects that have names use only the exact relevant prefix followed by a run-plan-bound
ASCII suffix. Content-addressed objects bind repository digest, object ID, media type, platform,
size, and parent/layer identities. The run plan records the full create request and returned
container/network IDs. Prefix match alone never authorizes reuse or cleanup.

## 9. Fresh admission containers and state crossing

Every one of the 30 admissions gets a newly created container and, where live provider egress is
needed, a newly created private network. Containers have `restart=no`; no container is restarted,
reused by another phase, adopted, or reconstructed. Each begins with a read-only root filesystem,
fresh temporary/result workspace, no Docker socket, no host PID/IPC namespace, no added capability,
`no-new-privileges`, and the exact resource bounds in the run plan.

Synthetic fixture trees are immutable read-only inputs. A moved checkout is a second immutable
fixture identity, never a bind mount of the live repository. Writable output, temporary, native
memory, Engram datastore, and authentication locations are distinct mounts; a read-only fixture is
never reused as a write volume.

Only these lane-local states may cross admissions:

- the exact native-memory volume, only for a native or combined arm and only across that lane's
  teaching, Codex activation when applicable, and evaluation lifecycle;
- the exact Engram datastore volume, only for an Engram or combined arm and only from that lane's
  successful teaching to evaluation;
- the exact per-lane authentication state in Section 10, solely for the same native harness; and
- the hash of the preceding sealed receipt, not its writable directory.

No container filesystem, process, PID namespace, network, checkout, transcript, prompt, temp file,
result directory, tool history, session ID, hidden context, memory volume from another lane, or live
user state may cross. Engram-bearing lanes run the exact frozen Engram binary against their private
persistent datastore; no live daemon/datastore, in-memory substitute, canned result, or cross-arm
store is allowed.

## 10. Exact authentication topology

Codex uses exactly six additional owner-only plaintext ChatGPT file-cache copies, one for each
Codex lane. The exact host paths are:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v4-stale-safety-20260910-01/execution/auth/codex/lane-0000000000000001/auth.json
/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v4-stale-safety-20260910-01/execution/auth/codex/lane-0000000000000003/auth.json
/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v4-stale-safety-20260910-01/execution/auth/codex/lane-0000000000000005/auth.json
/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v4-stale-safety-20260910-01/execution/auth/codex/lane-0000000000000008/auth.json
/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v4-stale-safety-20260910-01/execution/auth/codex/lane-000000000000000a/auth.json
/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-flagship-v4-stale-safety-20260910-01/execution/auth/codex/lane-000000000000000c/auth.json
```

The count is exactly six, not six per phase. Each copy is created once by the Rust evaluator's
exact-hash guarded no-overwrite operation under the standing equivalent-copy authorization; it
implements this contract:
held-directory/no-follow operations, `O_CREAT|O_EXCL`, owner-only directory mode `0700`, file mode
`0600`, exact source/cache contract validation, file fsync, reopen-and-hash verification, and parent
fsync. Existing, linked, rebound, partial, wrong-owner, wrong-mode, or uncertain state fails closed.
This design freeze creates no copy. Copies are never logged, embedded in evidence, shared between
lanes, exposed to tools, or deleted under V4 cleanup authority.

Claude uses exactly zero plaintext authentication-cache copies. The new protocol does not specify
a Claude cache mode. Historical Schema-15 Claude lanes had isolated `CLAUDE_CONFIG_DIR` values,
absent `.credentials.json`, empty `required_secret_environment`, and traces reporting
`apiKeySource="apiKeyHelper"`, while the historical runner inherited host environment. That is a
confounded historical topology, not proof of V4 isolation.

A Claude lane is eligible only if provider-free qualification selects and proves one of:

1. a mount-free, in-guest subscription login whose guest/account state is isolated to this epoch;
2. a claim-bound `apiKeyHelper` route with no inherited environment, no plaintext cache copy, a
   non-inherited per-admission capability, and no path or endpoint reachable by model-driven tools.

The exact method, helper identity, lifetime, transport, revocation behavior, and negative probes
must appear in the later implementation freeze and runtime manifest. If qualification requires a
materially new method—plaintext copying, host secret/environment injection, a new mount, a new host
helper, or broader credential exposure—V4 is insufficient: an append-only successor and separate
authentic user acceptance are required before any Claude provider claim. There is no fallback to
historical inherited environment.

## 11. Provider-specific tool and egress eligibility

The trusted native harness parent may reach only its manifest-listed provider route. Model-driven
tool children must have neither credential access nor general network access while retaining the
real local tool surface required by the protocol. A Docker network namespace is shared by all
processes in a container and cannot, by itself, prove that asymmetry.

Eligibility therefore requires a provider-specific mechanism that is independently tested:

- separate process/network confinement for tool children; or
- a claim-bound local proxy whose per-admission capability is delivered only to the trusted parent,
  is not inherited by tool children, expires at the admission deadline, and accepts only the exact
  provider route.

For Codex this must cover the native CLI and exact code-mode companion. For Claude it must cover the
native CLI and its real `Read`/`Bash` children. The existing Claude Seatbelt policy is macOS
`/usr/bin/sandbox-exec`-specific and intentionally leaves credentials readable to the trusted
client/tool mediation; it does not prove Linux-container child-tool credential denial. It is not
V4 eligibility evidence.

Negative probes must attempt credential and canary reads, inherited FDs/environment, loopback and
external network, DNS/proxy bypass, host home, other repositories, live Engram data, SSH/browser/
clipboard/cloud metadata, Docker control, unrelated IPC, `setsid`, `setpgid`, backgrounding,
double-forking, daemonization, and output/resource exhaustion. Any readable secret/canary,
unrestricted child egress, reachable Docker socket, unlisted tool/plugin/MCP/hook/connector, or
uncertain descendant cleanup makes that provider ineligible. The implementation may not drop tools,
use a direct API, weaken the sandbox, or substitute a fake response to keep a lane eligible.

Provider-side or hosted tools are forbidden. Unexpected `tool_call`, `tool_use`, `function_call`,
server-tool result, tool name, or schema is terminal unless it maps exactly to a run-plan-listed
client-side tool executed inside the qualified boundary.

## 12. Admission claim and local crash consistency

Before any Docker network/container create or container start for an E admission, the evaluator
constructs a raw-byte phase manifest binding the exact run plan, Q receipt, runtime manifest,
protocol/scorer tuple, lane,
phase, prompt, fixture, assigned state/auth volumes, image/config digests, provider, limits, prior
receipt hash, current evidence-chain head, deterministic object names, and expected absent outputs.

It then creates one claim relative to the held E directory descriptor with
`O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC`, mode `0600`. The claim binds the phase-manifest SHA-256,
admission ordinal, action `one_native_harness_admission`, attempt `1`, provider, image digest,
full-create-config digest, container/network names, prior chain head, deadline, and reserved
host-enforceable budget. The file is written, fsynced, closed, reopened no-follow, bounded-read and
hash verified, identity-rechecked, closed, and followed by parent-directory fsync.

Claim presence permanently spends that admission. Creation happens while the admission directory
is otherwise fresh and every current/future output is absent. There is no skip, resume, repair,
adoption, recreation, or second claim. A restart may only inspect, quarantine, stop/kill, and remove
the exact already-claimed ephemeral object under Section 15; it may never create, start, or resume
provider work.

If a claim exists without a complete, validated terminal receipt, the admission is
`SPENT_INDETERMINATE`. Local power loss, evaluator death, Docker-daemon uncertainty, missing
streams, incomplete inspect, or ambiguous cleanup can never become success through inference. The
epoch remains spent and is not replayed.

Every container carries exact labels for epoch, admission ordinal, phase, phase-manifest hash, and
claim hash. Before acting on an object, the evaluator and watchdog require exact name, returned ID,
image/config digest, and labels. A matching prefix without all identities is never enough.

## 13. Same-binary watchdog protocol

The exact host evaluator binary re-execs itself in `watchdog` mode after the durable claim and
before Docker create. A different watchdog executable is forbidden. If a distinct guest helper ELF
is used, it is not the host watchdog and must have its own exact runtime-manifest identity.

The evaluator creates an `AF_UNIX`, `SOCK_SEQPACKET|SOCK_CLOEXEC` socket pair. Spawn file actions
explicitly duplicate one endpoint to control FD 3 in each trusted peer and close every other
endpoint; no harness or tool descendant inherits FD 3. Each control message is exactly 196 bytes:

```text
offset  size  encoding
0       8     magic = bytes 45 4e 47 52 56 34 00 00 ("ENGRV4\0\0")
8       2     version = unsigned big-endian 1
10      2     kind, unsigned big-endian
12      8     sender-local monotonically increasing sequence, unsigned big-endian
20      4     admission ordinal, unsigned big-endian
24      4     phase: teaching=1, activation=2, evaluation=3, unsigned big-endian
28      8     absolute CLOCK_MONOTONIC deadline nanoseconds, unsigned big-endian
36      32    raw run-plan SHA-256
68      32    raw phase-manifest SHA-256
100     32    raw claim-file SHA-256
132     32    SHA-256 of the exact UTF-8 container name
164     32    raw container-ID SHA-256; all zero before Docker create
```

Allowed kinds and directions are exact:

```text
1 HELLO         evaluator -> watchdog; container ID is zero
2 READY         watchdog -> evaluator; no Docker create before receipt
3 BIND_OBJECT   evaluator -> watchdog; carries returned container ID
4 BOUND         watchdog -> evaluator; no Docker start before receipt
5 STARTED       evaluator -> watchdog
6 HEARTBEAT     either direction
7 CANCEL        evaluator -> watchdog
8 TERMINAL      evaluator -> watchdog, only after pre-cleanup evidence and cleanup disposition seal
9 TERMINAL_ACK  watchdog -> evaluator
10 FAULT        either direction
```

Sequences start at one and increase independently for each sender. Every reply repeats all identity
and deadline fields. Wrong length, magic, version, direction, sequence, identity, phase, deadline,
zero/nonzero-ID rule, unexpected kind, duplicate frame, timeout, socket error, or EOF is a fault.
The exact heartbeat interval, TERM-to-KILL interval, Docker-call timeout, lane deadline, and epoch
deadline are finite integer values frozen in the run plan. They never extend after a retry,
heartbeat, daemon restart, or coordinator disconnect.

After `BOUND`, the watchdog independently stable-records the exact name, ID, and claim label and can
invoke only bounded inspect/stop/kill/remove operations for that object. It watches peer heartbeat,
lease EOF, revocation/cancellation input, and its own monotonic hard deadline. On fault, EOF, or
deadline it prevents later admission, stops then kills the exact object within the frozen bound,
and records what it could prove. Host process-group membership is not a containment claim for a
Docker-daemon-owned process.

Between `READY` and `BOUND`, the watchdog knows the deterministic name and claim hash but not a
trusted returned ID. A fault in that interval permits only exact-name inspection followed by label,
image, and configuration verification; a matching object is stopped/killed, while absence or any
ambiguity is sealed as indeterminate. It is never adopted or started.

The evaluator also treats watchdog loss as terminal and stops the object. Simultaneous kernel,
Docker-daemon, evaluator, and watchdog failure is outside the guarantee; its result is
`SPENT_INDETERMINATE`, never success.

## 14. Revocation, cancellation, and deadlines

User revocation is authoritative only until the first provider-bearing E claim is durably created.
If received before that boundary, the evaluator creates no provider claim and the epoch is
`REVOKED_UNSTARTED`.

The first provider-bearing claim spends the whole bounded epoch for replay purposes. After it,
conversation delivery cannot be promised instantaneous or even available. A later cancellation is
best-effort but monotonic: when the evaluator receives it, it sends `CANCEL`, stops the active exact
container, creates no later admission, and seals the outcome. Failure to prove termination yields
`SPENT_INDETERMINATE`. Cancellation never reopens a claim or authorizes a replacement epoch.

Teaching completion supplies the sole shared retention timestamp. Codex activation and every
evaluation admission are forbidden before that timestamp plus exactly one hour. The 300-second
procedure expiry remains a separate case condition. All phase, admission, watchdog, and absolute
epoch deadlines are nested and frozen as UTC audit times plus monotonic enforcement intervals.
Clock rollback, suspend ambiguity, or disagreement fails closed; no deadline is recomputed later.

## 15. Evidence sealing and bounded cleanup

For every admission the evaluator captures exact Docker inspect data, raw stdout, raw stderr,
native harness trace, structured output, tool observations, provider metadata availability,
resource observations, timestamps, signal/exit status, input revalidation, and stopped-container
state. Each bounded file uses the same held-directory O_EXCL/no-follow/fsync/reopen/rehash procedure
as claims. The pre-cleanup terminal receipt binds its claim, phase manifest, prior chain head, all
captured hashes, exact object identities, watchdog transcript, the literal state
`cleanup_not_yet_authorized`, and its next chain head. A later cleanup receipt binds the cleanup
authorization and result to that head. Final reports publish only after all expected admissions and
cleanup receipts reconcile.

Only after the terminal evidence and receipt are written, fsynced, reopened, hash-verified, and
parent-synced may the evaluator create a separate O_EXCL cleanup authorization. It binds exactly
one stopped ephemeral container and its private ephemeral network by name, ID, claim hash, and
terminal-receipt hash. That cleanup capability is idempotent and permits only:

- disconnect the exact container from the exact network;
- stop/kill the exact container if still present; and
- remove the exact stopped container and exact private network.

It grants no create/start/restart/provider/network-connect authority and no wildcard/prefix-only
deletion. It never permits `docker prune`, image deletion, volume deletion, authentication-copy
deletion, evidence deletion, or deletion of another epoch's object. Native-memory and Engram
volumes, auth state, claims, manifests, receipts, raw traces, and reports remain preserved. Missing
identity, an unexpected attachment, or cleanup uncertainty yields `SPENT_INDETERMINATE` and leaves
the object quarantined for later separately authorized handling.

The bounded cleanup operation must itself finish before its fixed watchdog deadline and emit an
append-only cleanup receipt. Cleanup never changes a provider outcome or makes a failed admission
replayable.

If no container or network was ever created, no cleanup authorization is minted; an exact
`cleanup_not_required_no_object` disposition receipt records the verified absence instead. During
authorized cleanup the evaluator continues heartbeats. It sends `TERMINAL` only after the cleanup
receipt, no-object disposition, or indeterminate quarantine receipt is sealed; after
`TERMINAL_ACK`, both trusted peers close their control endpoints.

## 16. Required provider-free gates

No live provider claim is permitted until all gates pass in this order:

| Gate | Required exact evidence | What may follow |
|---|---|---|
| D0 | this V4 tuple and three fresh blind advisory reviews, no unresolved P0/P1 | implementation work only if separately within the user's task |
| D1 | exact Rust evaluator/scorer closure, tests, guest artifacts, image, runtime-manifest schema, and implementation-freeze tuple | request authentic acceptance of exact V4 and implementation freeze |
| U | authentic bounded acceptance described in Section 7 | create Q and perform provider-free qualification |
| Q0 | strict parsing, hash/path/object-identity, O_EXCL, crash, frame, deadline, and negative lifecycle tests | provider-specific fake-harness isolation probes |
| Q1 | Codex and Claude auth/tool/egress/descendant isolation pass independently; Q receipt sealed | freeze exact run plan and phase-manifest schema/template |
| R0 | protocol, scorer, run plan, image/config, Q receipt, budget, disk, E absence, roots, prefixes, and all expected outputs attest pristine | create E and provision exactly the six guarded Codex copies |
| R1 | E identity, six Codex copies, zero Claude copies, provider routes, inputs, disk, absence, bounds, and dynamically constructed first phase manifest re-attest pristine | begin one-shot provider claims |
| E0 | 12 teaching receipts reconcile and one shared deadline is sealed | wait; no admission during retention interval |
| E1 | deadline passed and input/auth identities remain pristine | six Codex activation claims in frozen order |
| E2 | activation receipt reconciles and evaluation outputs remain absent | 12 evaluation claims in frozen order |
| A0 | 30 claims and terminal receipts, traces, resources, cleanup receipts, and chain head reconcile | automated scoring/report generation |
| A1 | exact protocol/scorer audit and no substitution or cross-scope leakage | update completion audit with honest outcome |

Provider-free tests must include one-byte/duplicate/unknown/trailing/encoding drift; path swaps;
symlinks/hard links/special files; changed image/config/daemon/API; missing/extra mounts; inherited
environment/FDs; concurrent/partial/replayed claims; crash at every durability edge; malformed,
replayed, reordered, or lost watchdog frames; evaluator/watchdog/daemon death; hard timeout;
container escape attempts; state crossing; auth/canary/tool-network access; unexpected provider
tools; output/resource exhaustion; cleanup before evidence seal; cleanup target substitution; and a
complete positive fake-provider evidence chain. Tests may not fabricate native-memory success.

If any provider fails its own Q1 isolation gate, its lanes are ineligible and the planned matrix
does not run. V4 permits neither a partial matrix reported as complete nor weakening one provider's
boundary to match the other.

## 17. Results, budgets, and nonclaims

Every output separates authority, qualification, execution, and human review:

```text
authorization_source=authentic-user-event-or-absent
implementation_selected_by=honest-coordinator-under-explicit-delegation
admission_unit=one-native-harness-process
admission_checks=automated
review_status=automated_unreviewed
human_reviewed=false
ai_review_role=non_authoritative_advisory
provider_request_count=observed-value-or-unavailable
provider_retry_count=observed-value-or-unavailable
token_count=observed-value-or-unavailable
cost=observed-value-or-unavailable
```

Terminal epoch vocabulary is limited to:

```text
AUTOMATED_EXECUTION_COMPLETED_UNREVIEWED
FAILED
REVOKED_UNSTARTED
SPENT_INDETERMINATE
INELIGIBLE
```

The Claude budget is a 50-cent epoch ceiling as frozen by the protocol and future acceptance.
Because a native CLI may reveal spend only at a turn boundary, one active call may overshoot its
allocated share before the evaluator can stop later admissions; the exact observed value or
`unavailable` is recorded. V4 makes no enforceable per-request, hidden-retry, token, or subscription
USD claim. Codex is bounded by admission and host resources, not an invented provider-cost figure.

V4 does not claim reproducible builds, resistance to a malicious coordinator or compromised
kernel/Docker/native client/provider, instantaneous conversational revocation, proof of unseen
provider behavior, absence of all same-account server-side memory, statistical power, production
readiness, or flagship completion. Hashes prove byte identity only. Automated evidence remains
explicitly unreviewed unless a human later signs a separate exact report.

## 18. Current state at freeze

Only this append-only design and the new protocol exist as new V4 design inputs. The implementation
freeze, advisory review, user acceptance receipt, Q/E roots, Docker objects, Rust V4 executable,
runtime manifest, scorer tuple, run plan, six Codex auth copies, Claude auth route, claims, receipts,
traces, provider outputs, cleanup authorizations, reports, and completion update are absent.

The Schema-15 predecessor remains complete and untouched. Its 45 admissions are not replayed.
Repository `target/debug` remains an absence invariant. User-owned worktree changes are preserved;
nothing is staged or committed.

This freeze permits only non-authoritative advisory review. It authorizes no implementation,
build, test, authentication read/copy, Docker mutation, provider/pilot execution, live adapter or
daemon change, publication, staging, commit, or deletion.
