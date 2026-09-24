# Native strict successor Stage C2B2a — lean user-authorized execution boundary V2

Date: 2026-09-10
Status: frozen design proposal; no implementation, test, or execution authority

## Disposition

This append-only successor rejects the first lean proposal at exact SHA-256
`0531771c53863948031032d1434540a8ef4b1a7c6d966c4236e54a2a2f0dc2a6`, 334 LF,
17846 bytes:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_LEAN_USER_AUTHORIZED_EXECUTION_BOUNDARY_FROZEN_2026-09-10.md
```

Three independent non-authoritative advisory reviews inspected that exact tuple without peer
results. Their verdicts were `FAIL 0/5/2` (Codex semantic), `FAIL 0/6/1` (Codex guard), and
`FAIL 0/2/3` (isolated Claude); aggregate `FAIL 0/13/6`. No review granted authority, and the
prospective advisory-review file stayed absent.

The first proposal's direction was sound but its user policy tried to bind trusted implementation
hashes that did not exist, trusted an ambiguous packager, did not close dynamic agent turns or the
provider-harness tool surface, left evidence durability incomplete, offered more than one isolation
mechanism, and claimed an unenforceable hard dollar bound.

V2 retains the lean authority model and repairs those defects. It remains a fresh top-level
successor to the rejected external-review/DAG/capability line identified in the first proposal.
Every prior artifact and absent review path remains immutable historical evidence. Nothing transfers
authority into V2.

## Exact paths and namespace

This proposal is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_LEAN_USER_AUTHORIZED_EXECUTION_BOUNDARY_V2_FROZEN_2026-09-10.md
```

Its prospective advisory review is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_LEAN_USER_AUTHORIZED_EXECUTION_BOUNDARY_V2_ADVISORY_REVIEW_2026-09-10.md
```

The proposed execution namespace is exactly:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/lean-authority-c2b2a-20260910-01
```

All three are absent before their respective create-new operations except this now-frozen proposal.
No predecessor path, capability, review, manifest, claim, receipt, output, or run ID may be reused.

## Decision and authority expression

AI reviews are useful defect-finding evidence but never authorization. A run is admitted only when:

```text
U_user_delegation
AND C_trusted_coordinator_selection
AND M_independently_reconstructed_source
AND R_exact_run_plan
AND I_exact_isolation
AND S_durable_unspent_claim
```

- `U_user_delegation` is one authentic user message accepting this exact V2 hash, policy, limits,
  and the explicit delegation below.
- `C_trusted_coordinator_selection` is the honest coordinator's later selection of exact
  supervisor, verifier, sandbox, runtime, provider-adapter, source, manifest, and plan hashes under
  the mechanical constraints in V2.
- `M_independently_reconstructed_source` is produced by the trusted verifier from held source
  roots. It is never accepted merely because a packager and bundle agree.
- `R_exact_run_plan` binds every selected hash, phase, brokered action, bound, and output.
- `I_exact_isolation` is one selected and tested macOS isolation contract.
- `S_durable_unspent_claim` is committed before each child, brokered tool action, or provider
  invocation.

The user does not pre-approve hashes that do not yet exist. Instead, the user explicitly delegates
their exact later selection to the coordinator under this closed policy and accepts the coordinator
as TCB. `U` never claims to contain or authenticate those future hashes. `R` records them after
selection. This is a deliberate reduced-assurance choice compared with a second per-hash user
approval, not an implicit extension of human authority.

No AI verdict, candidate claim, packager output, test result, local approval file, prior envelope, or
existing result is an authority term.

## User acceptance and limits

One exact user message may activate the delegation only after V2's advisory gate has no P0/P1. The
message must name V2's final SHA-256 and explicitly accept:

- project `engram`, repository `github.com/ymeiri/engram`;
- one epoch under the exact namespace above;
- honest-coordinator selection of later implementation and artifact hashes under V2;
- at most 18 Codex harness invocations and 18 Claude Code harness invocations;
- exact per-invocation turn/token/time limits frozen in `R`;
- no promise of a hard USD ceiling when the harness cannot expose/enforce one;
- provider-reported tokens/cost recorded where available, and unavailable values reported as such;
- an absolute policy expiry of `2026-10-10T20:59:59Z`;
- owner-only evaluation outputs in the new namespace;
- result status no stronger than `AUTOMATED_EXECUTION_COMPLETED_UNREVIEWED`;
- reliance on the stated coordinator, OS, verifier, launchd, sandbox, and provider-adapter TCB;
- no live adapter/hook/settings/daemon/datastore change, staging, commit, push, publication, or
  deletion.

A coordinator-written receipt may quote and hash the message, task ID, timestamp, V2 hash, and
limits, but it is only an audit cache. The authentic user-message event is the authorization source.
The trusted coordinator consumes the live event in process; no local file can manufacture it.

Revocation before the first claim cancels the epoch. Revocation while a phase is active makes the
supervisor terminate its process group immediately with SIGTERM, then SIGKILL after the frozen
grace period, blocks later admission, seals a `REVOKED` receipt, and preserves all spent evidence.
The policy expires at the absolute timestamp even if no claim was created.

## Trust boundary and nonclaims

Trusted:

1. the authentic Codex task event carrying exact user acceptance;
2. the honest coordinator and its in-process selection state;
3. the small Rust verifier/supervisor after its exact source and binary identities are recorded in
   `R`;
4. the host kernel, APFS durability, SHA-256, code-signing/runtime identity APIs, launchd, and the
   exact sandbox profile;
5. the exact response-only provider adapter and authenticated provider usage receipts.

Untrusted:

- the packager and every candidate Python source, generated input, prompt, model response, tool
  request, tool result, test, and self-assertion;
- all AI review prose as an authority source;
- any file claiming user or human approval;
- candidate/provider-reported identity, cleanup, budget, or completion unless independently observed.

Explicit nonclaims:

- no protection from compromised root/kernel/filesystem, malicious same-UID interference outside
  the selected sandbox, compromised coordinator/supervisor/provider, or lying trusted OS service;
- hash identity is not semantic correctness;
- automated review is not human review;
- the accepted delegation is not reusable outside this one project/epoch/expiry;
- provider subscription invocations may lack a mechanically enforceable dollar value.

## Raw-byte document rules

Authoritative documents are UTF-8 JSON without BOM, end in exactly one LF, and are identified by
SHA-256 of exact raw bytes. The strict loader rejects duplicate or unknown keys, invalid UTF-8,
non-LF endings, trailing bytes, non-integer numbers, and schema/bound violations. No identity is
derived from parse-and-reserialize output.

Every schema uses `deny_unknown_fields`; the strict duplicate-key visitor already present in
`engram-eval/src/native_document.rs` remains mandatory. Numeric resource values are nonnegative
integers in fixed units named by their fields.

Paths are relative UTF-8 names with no empty, `.`, `..`, absolute, NUL/control, backslash,
alternate-normalization, or case-equivalent component. Symlinks, hardlinks, special files, sparse
files, duplicate inodes, and undeclared entries are rejected.

## Trusted implementation boundary

The 2,096,688-byte Python launcher and the Python controller/provisioner remain candidate inputs,
never TCB. Their freeze-time identities are:

```text
launcher     b5e50318b84a1ba442500781bef4bf405559a43fca46737d8f82b1d236fe2530
controller   eefa6eecb239ee1df49ba51317702cf78199edb7432ce32314b46a8db726b9f9
provisioner  aabe6b17ac366036a7cef05826106ea1cb6b8a30e5437937b13320ca44599261
```

After user acceptance, implementation is limited to:

- one new `engram-eval/src/lean_authorized.rs`;
- narrow private extensions to `native_document.rs`, `native_successor_core.rs`,
  `native_execution.rs`, `native_runner.rs`, `main.rs`, and `lib.rs`;
- tests in those modules;
- no new Cargo dependency unless a newly named design amendment is accepted.

The new module owns `U/M/R/I/S/E`. It may reuse only independently read and reviewed Rust
primitives: strict document loading, ACL checks, sealed-artifact revalidation, create-new
intent/receipt pairs, `PreparedExecution::commit_intent`, process-group spawn/cleanup, native
runner auth readiness, argv attestation, trace parsing, and cost/token observations.

It never imports a Python candidate as a library and never trusts candidate-generated expected
hashes, authority markers, manifests, receipts, or completion claims.

## Source selection, bundle, and flat manifest

The packager is untrusted. Before packaging, the verifier no-follow opens and holds the repository
root and each coordinator-selected source file. Selection is restricted to the repository above,
the exact candidate paths named in `R`, the new trusted implementation paths allowed above, and
explicit run inputs. It records source path/device/inode/mode/size/hash and revalidates after copy.

The packager creates a sealed directory bundle `B`; it cannot choose expected identities. The
verifier independently:

1. enumerates the held source selection in raw path order;
2. rejects missing, extra, aliased, linked, special, or unstable entries;
3. reconstructs `M` from source FDs;
4. inventories `B` independently;
5. requires a bijection and byte-for-byte hash equality between source and bundle;
6. records both identities in `R`;
7. keeps `B` read-only and revalidates it before and after every phase.

The candidate cannot read the live repository. A flat manifest closes candidate-controlled
semantics because the sandbox exposes only `B`, declared inputs, and trusted runtime objects.

For every ambient runtime path in `M`, isolation evidence `I` contains exactly one matching
identity: file hash and stat tuple, or an explicitly named code-signing CDHash/OS-build/shared-cache
identity where bytes are not independently addressable. The verifier requires a bijection between
the ambient list and `I`; no coarse unmatched runtime category is allowed.

A malicious packager/provisioner test must inject a mutually consistent false bundle/manifest and a
forged supervisor marker. The verifier must reject it by comparison with held source roots and by
ignoring all candidate authority fields.

## Exact run plan and dynamic turn protocol

`R` binds proposal, user receipt, coordinator identity, `M/B/I`, trusted binary/profile/adapter
hashes, unique run ID, task/repository/project/relevance mode, inputs, phase order, output namespace,
resource limits, claim names, and terminal schema.

Future model text cannot be known at freeze time. Therefore `R` binds a bounded interaction
protocol rather than future prompt bytes:

- exact initial prompt/scenario bytes;
- maximum model turns, provider invocations, tool actions, input/output bytes, and wall time;
- exact provider/model and response schema per lane;
- an allowlist of brokered tool method plus schema, read-only/mutating class, project scope, and byte
  limits;
- no arbitrary shell, subprocess, filesystem, network, MCP discovery, adapter setting, or code
  execution;
- a read-only file broker restricted to `B` and an Engram broker restricted to the explicit
  project/task/relevance mode;
- one non-executable, no-suid, no-device staging tree.

Every model response, tool request, tool result, and next-turn prompt is one ordered event. Before an
event can influence another turn, the supervisor durably seals and hashes it, validates its schema
and scope, appends its receipt to the evidence chain, creates the next event claim, and only then
admits it. Newly generated files remain data and cannot execute. A future phase may consume them only
after a new phase manifest includes their exact hashes and the plan pre-authorizes that transition.

A model-selected unlisted action is rejected and recorded; it cannot be translated into a host tool
call.

## Exact provider adapter boundary

The provider adapter is not a general Codex or Claude coding session. It is a separately sandboxed,
response-only harness invocation with:

- one exact model turn or one broker-mediated continuation at a time;
- only the `R`-allowlisted supervisor tool broker;
- no Bash, shell, subprocess, arbitrary filesystem, MCP discovery, unrelated MCP server, repository
  checkout, user home, Keychain command, browser profile, clipboard, or general network tool;
- network access only inside the adapter binary to its declared provider endpoints;
- isolated owner-only authentication home mounted only in the adapter zone;
- no mechanism returning authentication bytes to the model, tool broker, candidate, trace, or output;
- exact argv/config/environment/profile identities in `R`;
- no hidden recovery or replay.

The supervisor, not the model or candidate, constructs provider requests from the sealed event.
Each provider invocation has a durable claim first. Provider response bytes and available request,
token, latency, and cost metadata are sealed before further use.

If either native harness cannot be proven to expose only this surface, that harness is ineligible for
V2 and the provider gate fails. V2 does not silently fall back to a general agentic CLI or direct API
credential not authorized by the user.

The candidate-zone and adapter-zone profiles each have negative probes for other-project files,
auth-cache contents, shell/tool execution, unlisted MCP, arbitrary network, canary exfiltration, and
trace persistence.

## Selected macOS isolation and supervisor-loss control

V2 selects one mechanism: a transient per-run user launchd job running the trusted supervisor, with
the candidate and adapter children separately wrapped by exact `/usr/bin/sandbox-exec` profiles.

The local macOS `launchd.plist(5)` contract states that when a job dies, launchd kills remaining
processes with the same process-group ID unless `AbandonProcessGroup=true`. The frozen plist sets
`AbandonProcessGroup=false`, a finite `ExitTimeOut`, no `KeepAlive`, no globbing, fixed argv and
environment, and owner-only paths.

The supervisor creates one process group, forbids candidate/adapter `setsid` or process-group
escape, and records every child. The isolation gate must prove all of the following with a
provider-free fixture before production admission:

- killing the supervisor causes launchd to terminate every process-group member within the bound;
- a fixture attempting `setsid`, `setpgid`, double-fork, daemonization, or exec escape fails;
- unload, timeout, SIGTERM/SIGKILL, and coordinator loss leave the group empty;
- sandbox violations and cleanup uncertainty produce no success receipt;
- exact plist, sandbox profiles, sandbox-exec binary/code-signing identity, launchd/OS build, and
  observed group identifiers are bound in `I`.

If any probe fails, V2 is not implementable on this host. A VM is not an automatic substitute; it
requires a newly named design amendment.

## Durable claims, events, and terminal evidence

Claims and authoritative evidence use a supervisor-only owner-mode `0700` parent inaccessible to
candidate and adapter children. Each object is a bounded regular file created with literal
`openat(O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600)` relative to a held same-device parent
FD. Immediately after open it must be regular, same-device, current UID/GID, mode `0600`, nlink
`1`, and size `0`.

The supervisor writes all bytes, fsyncs, closes, reopens
`O_RDONLY|O_NOFOLLOW|O_CLOEXEC`, proves path/FD inode equality, bounded full-reads and verifies
schema/hash/count, rechecks file and parent identity, closes, and fsyncs the parent. Existing,
partial, invalid, rebound, linked, wrong-owner/mode/device/type, readback, fsync, or close uncertainty
is terminal and the object is preserved.

Before each phase, tool event, or provider call, a claim records run/event sequence, predecessor
chain digest, `U/M/R/I` hashes, action identity, attempt `1`, and worst-case count/token/time
reservation. Only after durability may admission occur. Any surviving claim spends that action.
Crash or uncertainty before durability admits no action but abandons the epoch; after durability it
is spent. No repair, deletion, adoption, recovery, or same-ID replay exists.

Every receipt and raw event uses the same durable protocol, monotonically increasing sequence, and
previous-event SHA-256. A receipt binds its claim, exact observed status, timestamps, exit/signal,
resource use, provider metadata, stdout/stderr/output hashes, policy violations, cleanup, and input
revalidation. Any gap, duplicate, fork, partial leaf, unexpected basename, or chain mismatch is
terminal.

Candidate staging is a different non-executable parent. Only the supervisor copies validated bounded
outputs into create-new authoritative leaves. The final report and chain digest publish last after
all claims/receipts reconcile, the process group is empty, adapter access is closed, inputs match,
and counts/tokens/cost metadata reconcile. Otherwise status is `FAILED`, `REVOKED`, or
`INDETERMINATE`, never success.

## Budget semantics

V2 enforces hard counts, token requests where the adapter supports them, per-invocation wall time,
aggregate wall time, process/memory/output limits, and the exact maximum of 18 invocations per
harness. It reserves the entire declared invocation schedule before the first provider call and
reserves each action again in its claim.

It does not claim a hard USD ceiling for subscription/browser-auth Codex or any adapter lacking
authenticated pricing and billable-call receipts. Hidden provider retries are forbidden by policy;
if the adapter cannot prove retry count, the invocation is reported with
`provider_retry_count=unavailable` and V2 makes no provider-call-count claim beyond observed harness
invocations. Available token/cost data is recorded without extrapolation.

## Lifecycle

| Gate | Evidence | Permitted next action |
|---|---|---|
| G0 | exact V2 plus three blind advisory reviews with no unresolved P0/P1 | request one user delegation |
| G1 | authentic user acceptance of exact V2 hash and policy | implement trusted Rust boundary and tests |
| G2 | exact implementation hashes plus deterministic schema/source tests | run provider-free negative probes |
| G3 | exact launchd/sandbox profiles and all isolation/escape probes pass | run one provider-free fixture |
| G4 | fixture receipts and chain audit pass; result remains automated/unreviewed | freeze exact provider run plan |
| G5 | auth readiness, disk reserve, claims/output absence, source/profile hashes, and limits pristine | run each fresh lane once |
| G6 | all events, tokens/cost availability, scope/leakage, outputs, and cleanup reconcile | generate comparison report |
| G7 | requirement-by-requirement flagship evidence is direct and sufficient | decide full goal completion |

Advisory AI reviews may find defects, but do not grant or revoke authority. G0's no-P0/P1 rule is a
coordinator quality policy; the authentic user acceptance at G1 is the sole external authorization.

No completed historical lane is replayed or repaired. Any failed production epoch gets a new
namespace and fresh plan after its cause is corrected.

## Required tests before provider use

Tests must cover:

- strict JSON duplicate/unknown/trailing/number/encoding rejection;
- one-byte `U/M/R/I/B`, trusted binary, runtime, profile, prompt, event, and output drift;
- false-but-self-consistent packager manifest, forged authority marker, source swap, extra/missing/
  aliased/linked/special/case-equivalent entry;
- exact ambient-runtime-to-I bijection;
- project/repository/task/relevance mismatch and cross-project Engram access;
- extra environment, FD, mount, Mach service, reachable path, MCP method, tool, or network endpoint;
- candidate and adapter access to user files, other repositories, Keychain/auth cache, SSH, browser,
  clipboard, cloud metadata, and canary values;
- dynamic event gap/fork/reorder, unsealed response reuse, executable output, unlisted tool request;
- partial/concurrent/replayed claim, crash before/after claim, duplicate provider invocation,
  revocation, timeout, supervisor death, process-group escape, descendant leakage;
- token/invocation/output/resource overflow, missing provider metadata, partial receipt, forged
  completion, unexpected output, failed cleanup, and final-chain mismatch;
- one positive, provider-free, no-secret/no-network fixture.

Test namespaces are disjoint from the production namespace. No test or build runs before G1.

## Result vocabulary

Every report separates authorization from checks:

```text
authorization_source=authentic-user-event
authorization_scope=engram/one-epoch
implementation_selected_by=trusted-coordinator-under-v2-delegation
admission_checks=automated
review_status=automated_unreviewed
human_reviewed=false
ai_review_role=non_authoritative_advisory
```

The strongest terminal label is `AUTOMATED_EXECUTION_COMPLETED_UNREVIEWED`. It means only that the
declared mechanical checks and exact run plan passed. It is not semantic correctness, reviewed code,
production readiness, or flagship completion. Only an independently verifiable later human event
may change `human_reviewed`.

## Current state

The V2 advisory-review path, execution namespace, user receipt, implementation, bundle, manifest,
run plan, isolation evidence, claims, events, provider outputs, and terminal report are absent.
Candidate sources remain unexecuted and unchanged. The forbidden combined preflight output and
repository `target/debug` remain absent. User-owned worktree changes remain preserved; nothing is
staged or committed.

V2 authorizes only non-authoritative advisory design review. It authorizes no source edit, build,
test, candidate/static/runtime/provider/pilot execution, live adapter change, publication, or
deletion.

