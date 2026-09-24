# Native strict successor Stage C2B2a — lean user-authorized execution boundary V3

Date: 2026-09-10
Status: frozen design proposal; advisory review only; no implementation, test, or execution authority

## Disposition

This append-only successor rejects V2 at exact SHA-256
`86e7517663646f93dadaf81e9a70538475f62d194d9fb7ebfacd4d1d5dfab782`, 413 LF,
22479 bytes:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_LEAN_USER_AUTHORIZED_EXECUTION_BOUNDARY_V2_FROZEN_2026-09-10.md
```

Three fresh blind non-authoritative reviews inspected that exact tuple without peer results:

- Codex semantic: `FAIL P0=0/P1=3/P2=2`;
- Codex operational: `FAIL P0=0/P1=6/P2=2`;
- isolated Claude: `FAIL P0=0/P1=1/P2=3`;
- aggregate: `FAIL P0=0/P1=10/P2=7`.

No review granted authority and the V2 advisory-review path stayed absent. The blocking findings
were: a provider-side tool-definition gap; non-path-keyed source/bundle comparison; interpreter
execution of nominally non-executable generated data; hidden retry and token-limit overclaims;
launchd scope conflict and process-group escape; and no exact coordinator-liveness/revocation
channel. Precision findings covered phase-manifest claims, new-epoch authority, namespace creation,
evaluation capability, coordinator identity, grace-period provenance, and an unnamed forbidden
output path.

V3 does not add another wrapper around those defects. It removes their common cause: the candidate
Python execution path. Nothing from V2, its reviews, or any earlier authority artifact transfers
authority into V3.

## Exact paths and fresh namespace

This proposal is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_LEAN_USER_AUTHORIZED_EXECUTION_BOUNDARY_V3_FROZEN_2026-09-10.md
```

Its prospective advisory review is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_LEAN_USER_AUTHORIZED_EXECUTION_BOUNDARY_V3_ADVISORY_REVIEW_2026-09-10.md
```

Its proposed execution namespace and Docker object prefix are exactly:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/lean-authority-c2b2a-v3-20260910-01
engram-c2b2a-v3-20260910-01-
```

The proposal path alone exists after this freeze. The review path, namespace, Docker objects,
implementation, manifests, claims, receipts, traces, provider outputs, and reports must remain
absent until their named gates.

## What V3 is actually testing

C2B2a is a controlled evaluation of engineering-memory behavior. It is not a general hostile-code
execution service and it does not need to execute a historical orchestration candidate.

V3 must preserve the real experiment:

- exact native Codex and Claude Code harnesses, not direct provider APIs or response-only model
  substitutes;
- teaching and fresh evaluation sessions with the real retention interval;
- a moved checkout and no prompt, transcript, or hidden context handoff between sessions;
- host-native memory only in native-memory and combined arms;
- an isolated persistent Engram runtime only in Engram-bearing arms;
- native command and source-reading tools required by each harness, including the Codex code-mode
  companion and Claude Code `Bash` and `Read`; `Write` or `Edit` is allowed only where the frozen
  native-memory protocol requires it inside the synthetic fixture;
- the exact restricted Engram agent profile required by the protocol, including real `orient`,
  teaching-memory creation and verification, and evaluation-time `procedure_match`;
- both the known failed command and corrected command inside the fixture so the runner cannot bias
  the comparison by blocking the failure;
- decisions derived from authenticated host traces and observed files, never model self-report;
- the preregistered identity, first-action, repeated-failure, scope-leakage, stale-influence,
  correction-burden, evidence, latency, token-availability, and packet-size observations;
- the existing comparison arms and scenarios named by the frozen evaluation protocol. V3 neither
  deletes an arm nor changes a scorer.

The broader flagship still requires the instructions-only, native-memory, native-plus-skills,
current-Engram, combined, and lean-Engram comparisons described by the active goal. Passing C2B2a
is evidence for that goal, not automatic flagship completion.

## The cut: no candidate execution

The following files remain immutable historical design evidence only:

```text
launcher     evals/native_memory_pilot_v1/native_c2b2a_completion_audit_launcher.py
             b5e50318b84a1ba442500781bef4bf405559a43fca46737d8f82b1d236fe2530
controller   engram-eval/native-c2b2a-payload/build-support/controller.py
             eefa6eecb239ee1df49ba51317702cf78199edb7432ce32314b46a8db726b9f9
provisioner  engram-eval/native-c2b2a-payload/build-support/provision.py
             aabe6b17ac366036a7cef05826106ea1cb6b8a30e5437937b13320ca44599261
```

V3 never imports, interprets, compiles, AST-parses, packages, copies into an executable zone, or
executes those files. No Python interpreter is part of the evaluator. Their candidate manifests,
self-hashes, review envelopes, capability DAGs, authority markers, and recovery logic are rejected
inputs, not trusted evidence.

One small Rust evaluator replaces them. It owns strict plan loading, lane scheduling, Docker
admission, durable intent/receipt publication, trace collection, schema validation, scoring, and
report publication. It may reuse already-reviewed Rust primitives in `engram-eval`, but it may not
interpret candidate output as authority or executable code.

Model responses and tool traces are bounded data. The evaluator never evaluates them as source,
loads them as plugins, or makes them executable.

## Authority and one explicit delegation

AI review is defect-finding evidence only. Runtime admission requires:

```text
U_authentic_v3_delegation
AND C_honest_coordinator_selection
AND R_exact_run_plan
AND I_qualified_container_and_native_tool_sandbox
AND S_unspent_harness_admission
```

After V3 has three fresh blind advisory reviews with no unresolved P0/P1, one authentic user
message may accept V3's exact final SHA-256. That message must explicitly accept:

- project `engram` and repository `github.com/ymeiri/engram`;
- one later exact run epoch under the namespace and Docker prefix above;
- the honest coordinator as TCB for selecting the later exact evaluator source/binary, Docker image
  digest, harness/runtime identities, sandbox policies, protocol, and run-plan hashes under V3;
- implementation and provider-free qualification of that evaluator;
- at most 18 native Codex harness admissions and 18 native Claude Code harness admissions;
- hard harness-admission, wall-time, process, memory, disk, and output limits frozen in `R`;
- no hard provider-request, internal-retry, token, or USD ceiling when a native subscription-backed
  harness cannot expose and enforce one;
- provider-reported tokens, retry counts, and cost recorded only when authenticated and observable,
  otherwise the literal value `unavailable`;
- bounded Docker image/container/network/volume creation, start, stop, and inspection using the
  exact object prefix, with no deletion during the epoch;
- owner-only evidence in the exact namespace and no result label stronger than
  `AUTOMATED_EXECUTION_COMPLETED_UNREVIEWED`;
- an absolute expiry of `2026-10-10T20:59:59Z`;
- no live Codex, Claude, Cursor, or Engram adapter/hook/settings installation; no live Engram daemon
  or datastore mutation; and no stage, commit, push, publication, or deletion.

The user is delegating selection of later hashes to the coordinator; the user is not asserting that
they reviewed future bytes. The authentic task event is the authorization source. A local receipt
is only an audit cache and cannot manufacture that event.

The delegation covers implementation, provider-free tests, and one provider-bearing epoch only if
every gate below passes. Before the first provider-bearing claim, the user may revoke admission.
After a provider-bearing claim exists, the epoch is spent whether it succeeds, fails, crashes, or
becomes indeterminate. Any later provider-bearing epoch or namespace requires a new authentic user
acceptance. No AI statement or standing budget preference creates that new epoch.

## Honest cancellation and liveness boundary

V3 does not claim that a conversational message can cryptographically or instantaneously revoke an
already-running native CLI. Revocation before the next durable admission is authoritative. During
an active admission the coordinator may request cancellation; the evaluator and watchdog then stop
the container, apply the frozen TERM-to-KILL bound, and record the outcome. If termination and
container emptiness are not proven, the spent epoch is `INDETERMINATE`.

The evaluator starts a second exact-hash watchdog before any harness admission. The two trusted
processes use close-on-exec private pipes and fixed frames. Loss of either peer, malformed or replayed
frames, heartbeat expiry, or the absolute lane deadline disables later admission and invokes
container stop/kill. Children inherit no control endpoint. The watchdog retains the absolute
deadline even if the conversational coordinator disconnects.

V3 claims bounded automatic cleanup through the selected container runtime, not zero-latency user
revocation and not survival of simultaneous kernel, Docker-daemon, evaluator, and watchdog failure.

## One exact path-keyed manifest, no executable bundle

There is no candidate packager and no candidate-created source manifest. The trusted evaluator
reconstructs one flat raw-byte manifest from coordinator-selected original paths before admission.
Every row is keyed by exact semantic role, original absolute or repository-relative path, container
path, device/inode where meaningful, mode, size, and SHA-256 or platform code identity. Exact
entrypoint mappings are mandatory; hash-only permutation cannot satisfy the manifest.

The manifest includes only inputs that can affect the test:

- evaluator source identity and exact executable hash;
- exact Codex, code-mode companion, Claude Code, Engram, Docker client/engine, container image
  digest, and required runtime identities;
- protocol, scenarios, prompts, response schemas, scorer configuration, lane matrix, retention
  interval, sandbox policies, fixture-tree identity, auth mode, environment, cwd, argv, limits,
  phase order, and output names;
- exact native-memory and Engram persistent-volume assignments per lane;
- the exact absent-path invariants.

The evaluator stable-opens selected regular files without following links, hashes held descriptors,
copies only declared fixture/config data into non-executable container inputs, then requires exact
path-preserving equality. It never copies an executable from a candidate output.

V3 records source and binary identity for auditability but makes no reproducible-build or
malicious-coordinator-resistance claim. Execution authority rests on the exact binary and run-plan
identities selected under the user's explicit coordinator delegation.

## Container and native-tool isolation

V3 selects one outer lifecycle boundary: a Linux container managed by the already-installed local
Docker engine. There is no launchd registration and no automatic VM or host-only fallback. If the
exact Docker engine is unavailable or fails qualification, V3 is ineligible on this host and no
provider lane runs.

Each lane uses its own create-new container and private volumes under the exact prefix. The run plan
binds an immutable image digest, entrypoint, user, environment, mounts, network, capabilities,
seccomp profile, resource limits, and stop timeout. Required settings include a read-only root
filesystem, no Docker socket, no host PID/IPC namespace, no added capability, `no-new-privileges`,
bounded PIDs/memory/CPU/disk/output, read-only fixture inputs, and only narrowly writable native
memory, Engram state, temporary, and result volumes.

Teaching and evaluation admissions for one lane may reuse only that lane's explicitly assigned
persistent memory volumes. Different arms, hosts, scenarios, and epochs never share them. A moved
checkout is a second immutable fixture path, not a bind to the live repository. Engram-bearing lanes
run the exact Engram binary against their private persistent datastore; no in-memory substitute,
canned MCP response, trace replay, or live user datastore is permitted.

The native harness process may read its isolated authentication material. Model-driven tool
subprocesses must be confined by the harness's exact native sandbox and container policy so they
cannot read authentication files, ordinary home data, other projects, host mounts, SSH material,
browser state, clipboard, cloud metadata, Docker control, or unrelated IPC, and cannot make general
network connections. The negative gate asks each real harness to attempt those accesses. A readable
canary or credential path, reachable Docker socket, unrestricted tool network, or unlisted host
mount makes that harness ineligible.

The container runtime supplies PID-namespace and container teardown rather than relying on process
group membership. Provider-free probes must attempt `setsid`, `setpgid`, background processes,
double-forking, daemonization, and timeout. Container stop/kill must leave no process in the lane's
PID namespace and no active lane network endpoint. Uncertainty is terminal.

Stopped containers, volumes, traces, and claims are preserved for audit. V3 authorizes no cleanup
deletion.

## Real native tools and provider-side tool rules

V3 does not replace coding agents with tool-free chat clients. `R` enumerates the exact native
client-side tool surface needed by the frozen protocol. Those tools operate only inside the
synthetic lane boundary. Any unlisted local tool, MCP server, plugin, hook, connector, browser,
computer-use tool, or filesystem/network capability is forbidden.

Provider request configuration may describe only the exact allowed client-side native tools and
the restricted Engram broker methods. It must request no provider-hosted code interpreter, web
search, computer use, remote connector, or other server-executed tool. An unexpected
`tool_call`, `tool_use`, `function_call`, server-tool result, tool name, or schema is terminal unless
it matches an exact `R` entry and is executed inside the qualified client/container boundary.

If the exact native harness version cannot prove its configured tool/MCP/plugin/hook surface and
emit an authenticated trace sufficient to distinguish allowed client-side calls from provider-side
execution, that harness is ineligible. V3 never falls back to a direct API, canned broker, or
tool-free response mode merely to make the gate pass.

## Admission, retries, and evidence

The durable authorization unit is one native harness admission, not an invisible provider request
or every internal tool action. Before each teaching or evaluation admission, the evaluator creates
a raw-byte phase manifest binding `R`, lane, phase, prompt, checkout, assigned volumes, exact prior
receipt hashes, limits, and current chain head. It then durably creates an unspent claim that binds
the exact phase-manifest hash, prior chain digest, harness/container identity, action
`one_native_harness_admission`, attempt `1`, and reserved host-enforceable budget.

Only after that claim is durable may the evaluator start the exact container command. Any claim
spends the admission. There is no second evaluator admission, repair, adoption, or replay with the
same lane/phase/epoch. Native transport retries inside that one process are not represented as
additional evaluator admissions. Their count is recorded only when authenticated and observable;
V3 otherwise makes no provider-request or no-hidden-retry claim.

Each claim, receipt, phase manifest, raw trace, scored observation, and terminal report is a bounded
regular file in an evaluator-only owner-mode `0700` directory. It is created relative to a held
directory descriptor with `O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC`, mode `0600`, then validated,
written, fsynced, closed, reopened no-follow, bounded-read and hash-verified, path/FD identity
rechecked, closed, and followed by parent fsync. Existing, partial, rebound, linked, wrong-owner,
wrong-mode, wrong-device, malformed, or uncertain evidence is terminal and preserved.

The evidence chain is monotonic and path-keyed. A receipt binds its claim, phase manifest, prior
chain head, exact exit/signal/timestamps, container cleanup, observed resource use, stdout/stderr/
trace/output hashes, allowed and forbidden tool observations, provider metadata availability,
scope/canary findings, and post-run input identities. The final report publishes last and only after
all admissions reconcile. Otherwise it reports `FAILED`, `REVOKED`, or `INDETERMINATE`.

## Namespace creation and exact forbidden paths

`R` binds an existing trusted ancestor and one absent child basename. The evaluator opens and
validates the ancestor without following links, calls `mkdirat(..., 0700)` exactly once, treats
preexistence as terminal, opens the child no-follow, validates owner/mode/device/inode/type, fsyncs
child and parent, and holds the directory descriptor for every evidence operation.

The following paths must remain absent through G4 and are rechecked immediately before G5:

```text
/Users/yuval.meiri/projects/engram/target/debug
/Users/yuval.meiri/projects/engram/evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_IMPLEMENTATION_PREFLIGHT_REVIEW_2026-09-07.md
```

Builds after authorization use a non-repository `CARGO_TARGET_DIR` inside the new evaluation
namespace so repository `target/debug` remains absent.

## Required provider-free qualification

Before provider use, deterministic tests must prove:

- the three Python candidate hashes/paths and every Python interpreter are absent from packaged
  executables, container image entrypoints, process observations, and executed argv;
- strict JSON duplicate/unknown/trailing/encoding/number rejection and one-byte drift rejection for
  proposal, plan, manifest, binary, image, profile, protocol, prompt, fixture, claim, and receipt;
- exact path-keyed source/container mappings, entrypoints, missing/extra/aliased/linked/special files,
  and forbidden executable staging;
- the full native-host/arm/session/retention/moved-checkout/tool/scorer protocol remains intact;
- per-lane native and Engram persistence works only across the intended teaching/evaluation pair and
  never across arms, scenarios, projects, or epochs;
- auth files, canary values, host home, other repositories, live Engram data, SSH, browser,
  clipboard, cloud metadata, Docker socket, unrelated IPC, and general tool network are unreachable
  from model-driven tools;
- provider request configuration contains no unlisted or provider-hosted tool and unexpected tool
  response variants fail closed;
- timeout, output/resource overflow, evaluator death, watchdog death, lost heartbeat, malformed
  control frame, container stop/kill, `setsid`, `setpgid`, background, double-fork, daemonization,
  and descendant cleanup;
- partial/concurrent/replayed claims, phase-manifest drift, chain gaps/forks/reordering, duplicate
  admission, forged completion, trace mismatch, missing provider metadata, and unexpected output;
- one positive no-secret/no-network fake-provider fixture producing a complete evidence chain.

If the provider-free fake cannot exercise a native harness without broadening the real tool,
configuration, or process surface, the qualification must be split into an exact evaluator fixture
and a read-only native-harness capability attestation. It may not fabricate native-memory success.

## Gates

| Gate | Required evidence | Permitted next action |
|---|---|---|
| G0 | exact V3 plus three blind advisory reviews with no unresolved P0/P1 | request one exact user delegation |
| G1 | authentic acceptance of exact V3 and its reduced-assurance delegation | implement the small Rust evaluator only |
| G2 | exact implementation/image/runtime manifest and deterministic tests | run provider-free isolation and failure probes |
| G3 | every container, native-sandbox, credential, process, and canary probe passes | run one provider-free full fixture |
| G4 | fixture chain and preregistered scorer audit pass | freeze exact provider run plan and attest auth readiness |
| G5 | plan, identities, auth, disk, absence, bounds, and fresh namespace are pristine | create the namespace and run each provider lane once |
| G6 | every claim, receipt, trace, result, resource observation, and cleanup reconciles | generate the comparison report |
| G7 | comparison evidence is direct and no protocol substitution occurred | update the C2B2a completion audit |
| G8 | every original flagship criterion has independently sufficient current evidence | decide whether the full goal is complete |

No completed historical lane is replayed or repaired. A provider-bearing failure spends the epoch
and requires a fresh namespace, plan, and authentic user acceptance.

## Result vocabulary and nonclaims

Every report separates authorization, qualification, and outcome:

```text
authorization_source=authentic-user-event
authorization_scope=engram/one-v3-epoch
implementation_selected_by=trusted-coordinator-under-v3-delegation
admission_checks=automated
review_status=automated_unreviewed
human_reviewed=false
ai_review_role=non_authoritative_advisory
provider_request_count=observed-value-or-unavailable
provider_retry_count=observed-value-or-unavailable
token_count=observed-value-or-unavailable
cost=observed-value-or-unavailable
```

The strongest label is `AUTOMATED_EXECUTION_COMPLETED_UNREVIEWED`. V3 does not claim reproducible
builds, malicious-coordinator resistance, an exact opaque provider-call count, a hard token/USD
ceiling, zero-latency conversational revocation, protection from compromised kernel/Docker/native
harness/provider, semantic correctness from hashes, production readiness, or flagship completion.

## Current state

The V3 advisory-review path, execution namespace, Docker objects, user receipt, Rust evaluator,
manifest, run plan, claims, receipts, traces, provider outputs, and terminal report are absent. The
three candidate Python files remain unchanged and unexecuted. Repository `target/debug` and the
exact forbidden combined preflight-review path above remain absent. User-owned worktree changes are
preserved; nothing is staged or committed.

V3 authorizes only fresh non-authoritative advisory design review. It authorizes no implementation,
build, test, Docker mutation, candidate/static/runtime/provider/pilot execution, live adapter change,
publication, or deletion.
