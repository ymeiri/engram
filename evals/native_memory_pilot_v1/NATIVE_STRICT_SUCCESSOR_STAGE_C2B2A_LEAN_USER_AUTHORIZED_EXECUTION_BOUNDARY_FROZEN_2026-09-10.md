# Native strict successor Stage C2B2a — lean user-authorized execution boundary

Date: 2026-09-10
Status: frozen design proposal; no implementation or execution authority

## Disposition

This is a newly named top-level successor to the rejected external-review authority line. It does
not amend, adopt, or inherit execution authority from that line. The exact rejected design inputs
remain immutable evidence:

| Artifact | SHA-256 | LF | Bytes |
|---|---|---:|---:|
| external-source boundary | `7d0ae76f3fd721d61fc3797d3627c4cc4a99d413e7f2c39e4ae47417dc42ec80` | 556 | 36410 |
| boundary successor | `f1d7ddc8371c2a035f527f6d1784fe3f225cc89b3e0bac23e1616bc1f7a8ebaf` | 690 | 47347 |
| two-manifest boundary | `74c883fec8e075ddc20a5977fc5a36751f156e1a170554b3a414f3aca27fd0e2` | 733 | 48654 |
| two-manifest enforcement | `bed960697d497a8b721e53a7d9bad7a084c9a1e169339c290a05cf2ee9446bd6` | 677 | 41736 |
| enforcement successor | `5fac7622f192d42baf29d1ccaacb5abd3179d87ac8471286eaf6d206e9ce43b3` | 591 | 35226 |
| enforcement closure | `bbdb5af7f55f5ee6c504f3ccbf3cf02c0711e5404fc33bc51366daa72700aaec` | 515 | 32801 |
| closure V2 | `a5e9ed0d7ab33cf9ef2714ef62d8e638a851f2b7ecbf96b02a7cef2af0a8ef9d` | 372 | 23523 |
| closure V3 | `46b9276d626324508841d9163ceb8c3c8d3a659c72fc305981a7be56169f9aae` | 378 | 24449 |
| closure V4 | `b41528a41f1dd61e82a971c57f51530378e97adbaea1f5169751b3d4a08611fa` | 395 | 25261 |

V4 was independently reviewed as the same exact regular-file tuple by two Codex reviewers and one
isolated Claude reviewer. Their rejection-only findings were `FAIL 0/5/0`, `FAIL 0/4/1`, and
`FAIL 0/5/2`; aggregate `FAIL 0/14/3`. None emitted a canonical authority observation, and the
V4 consolidated-review path remained absent. These results are advisory evidence only.

The rejected line attempted to make AI reviews part of execution authority. Its custom observation,
DAG, capability, envelope, and scanner protocols became the dominant source of ambiguity. That
machinery was solving a hostile-host theorem while explicitly excluding compromised coordinator,
host/root, same-UID attacker, reviewer, kernel, and storage. It is removed here, not weakened.

This proposal preserves the properties relevant to the Engram flagship evaluation: exact source
identity, complete candidate-controlled input closure, bounded isolation, no candidate-held
credentials, durable at-most-once execution, strict budgets and deadlines, append-only evidence,
scope isolation, and honest result labels.

## Exact paths

This proposal is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_LEAN_USER_AUTHORIZED_EXECUTION_BOUNDARY_FROZEN_2026-09-10.md
```

Its prospective advisory review is exactly:

```text
evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_LEAN_USER_AUTHORIZED_EXECUTION_BOUNDARY_ADVISORY_REVIEW_2026-09-10.md
```

The advisory review has no execution-authority semantics. Existing or future AI review output is
never interpreted as human approval.

## Authority expression

A run is admitted only when all terms below are true:

```text
U_user
AND M_exact_source
AND R_exact_run_plan
AND I_isolation_preflight
AND S_durable_unspent_claim
```

AI review verdicts, reviewer identities, model outputs, candidate assertions, test success, and
existing result files are not terms in this expression.

- `U_user` is one explicit user acceptance of this exact proposal hash and its bounded policy.
- `M_exact_source` is an independently reconstructed flat manifest of every candidate-controlled
  byte that the sandbox can read or execute.
- `R_exact_run_plan` binds the manifest, commands, inputs, provider calls, outputs, budgets,
  deadlines, and one fresh run namespace.
- `I_isolation_preflight` proves that only declared inputs and trusted runtime surfaces are
  reachable and that the candidate receives no reusable credential.
- `S_durable_unspent_claim` is a create-new, durable, one-use claim written before child or
  provider admission.

The user authorizes bounded execution; this does not mean the user reviewed source code or accepts a
semantic-success claim. Automated evidence can prove only the assertions its checks actually cover.

## Trust boundary and explicit nonclaims

Trusted for this bounded evaluation:

1. the authentic Codex task user-message event that accepts this exact proposal hash;
2. the honest Codex coordinator transcribing that event without changing its words;
3. the host kernel, filesystem durability, SHA-256, and code-signing/runtime identity APIs;
4. a separately frozen small supervisor and manifest verifier;
5. the attested macOS sandbox profile and provider harness adapters;
6. the provider service's own signed or authenticated usage metadata.

Not trusted:

- the candidate launcher, controller, provisioner, generated source, prompts, agent output, or tests;
- any AI reviewer as an authorization source;
- any file merely claiming to contain user or human approval;
- candidate-reported cost, token, time, identity, cleanup, or completion data.

Nonclaims:

- no protection from compromised root/kernel/filesystem, malicious same-UID mutation outside the
  selected sandbox boundary, compromised provider service, or compromised trusted supervisor;
- no claim that hash identity proves semantic correctness;
- no claim that an automated review is a human review;
- no unattended reusable authorization until an external signed approval mechanism exists.

These are the same practical host assumptions under which the prior line operated, stated directly.

## User authorization policy

After this proposal is frozen and advisory-reviewed, one user message must accept its exact SHA-256
and these ceilings:

- project: `engram`;
- repository: `github.com/ymeiri/engram`;
- one newly named successor epoch;
- at most 36 provider calls;
- at most USD 5.00 total provider cost;
- models and per-call token ceilings must be enumerated in the run plan;
- provider execution may use Codex and Claude Code only;
- wall-clock ceiling: four hours after first durable claim;
- outputs: owner-only evaluation artifacts under the new run namespace only;
- no installation or modification of live adapters, hooks, settings, daemons, or datastores;
- no staging, commit, push, publication, or deletion.

The acceptance phrase will contain the final proposal hash. A coordinator-authored receipt may quote
that message and bind its task ID, timestamp, exact text, proposal hash, and policy values, but the
receipt is only a cache. The task's authentic user-message event remains the authority source.

Any changed ceiling, scope, proposal, supervisor, sandbox policy, provider adapter, or run plan
requires a newly named policy/epoch. Revocation before the first claim stops the run. After a durable
claim, revocation stops further admission and preserves the spent evidence.

## Raw-byte artifacts; no custom authority grammar

Every authoritative JSON artifact is UTF-8 without BOM, ends in one LF, contains only integer,
boolean, string, array, and object values, and is hashed as its exact raw bytes. The parser rejects
duplicate keys, unknown keys, non-integer numbers, invalid UTF-8, non-LF line endings, trailing
bytes, and schema or bound violations. No parser normalizes or reserializes an artifact for identity.

The schema is versioned and separately tested. Human-readable formatting is allowed because the raw
byte hash, not a purported canonical representation, is authoritative.

Paths in manifests are relative raw UTF-8 names with no empty, `.`, `..`, absolute, NUL, control,
backslash, alternate normalization, or case-equivalent component. The packager rejects symlinks,
hardlinks, sockets, FIFOs, devices, sparse files, duplicate inodes, and undeclared entries.

## Authoritative artifacts

### U — external user-policy acceptance

`U` binds the exact proposal hash, user-message evidence, scope, ceilings, allowed providers,
expiry/revocation state, and exact supervisor/sandbox/provider-adapter hashes once those exist.
Until the user accepts the proposal hash, `U` does not exist and nothing may execute.

### B and M — sealed bundle and flat source manifest

A trusted packager creates one sealed bundle `B` containing every candidate-controlled executable,
interpreted source, module, configuration, template, prompt, fixture, policy, and data input. It
includes the current candidate only if its exact source survives the new implementation preflight.

`M` lists every bundle entry in ascending raw path-byte order with type, mode permission bits,
size, and SHA-256. It also binds bundle size/hash, packager identity, and a complete explicit list of
trusted ambient runtime objects. Anything not in `B` or the trusted runtime list is inaccessible.

The bundle and manifest are create-new, owner-only, immutable for the run, and reopened through
held no-follow descriptors. The supervisor verifies every entry before launch and after termination.
A flat inventory is sufficient because the sandbox cannot reach undeclared candidate-controlled
paths; no authority DAG is required.

The existing provisioner already implements bounded no-follow reads, explicit archive manifests,
duplicate/path/type rejection, create-new copying, sealing, reopen verification, and source-tree
inventory. Those capabilities may be reused only after their exact source is included in `M` and
the small supervisor treats the provisioner as untrusted candidate code rather than its own trust
root.

### R — exact run plan

`R` binds:

- proposal and `U` identities;
- unique epoch and run ID;
- `B` and `M` hashes;
- trusted supervisor, verifier, sandbox profile, runtime, and provider-adapter identities;
- exact argv, cwd, environment allowlist, inherited file descriptors, mounts, inputs, and outputs;
- project/repository/task selectors and explicit relevance mode;
- exact ordered phases and provider calls;
- model names, call count, token/cost reservation, timeouts, process/memory/CPU/output limits;
- all claim, receipt, trace, result, and terminal-report basenames;
- expected success, abstention, scope-leakage, secret, and accounting checks.

Unknown or optional execution inputs are forbidden. A changed plan is a new epoch.

### I — isolation manifest

The candidate verification phases run under an attested macOS sandbox or VM with:

- a synthetic empty home and minimal fixed environment;
- read-only `B`, trusted runtime objects, and exact declared inputs;
- one empty private writable output tree;
- no repository checkout, other project, Keychain, auth cache, SSH material, cloud metadata, user
  home, clipboard, browser profile, arbitrary IPC, or general network;
- fixed file descriptors and no inherited terminal;
- independent process-tree containment, resource limits, and watchdog;
- kernel-enforced termination linkage on supervisor loss.

The selected mechanism, profile bytes, runtime identities, reachable paths, and negative probes are
frozen before execution. Availability of `/usr/bin/sandbox-exec`, Docker, or Lima alone is not
evidence that the isolation contract is met.

Provider calls execute outside the candidate sandbox through the trusted Codex/Claude harness
adapters. The candidate supplies no arbitrary network request. The run plan binds the exact prompt
bytes/hash, model, limits, and output destination for each call. The adapter owns authentication and
returns no credential. Isolated provider homes containing approved owner-only authentication caches
belong to the trusted adapter zone and are never mounted into `B` or candidate output.

### S — durable one-use claims

Before any phase child or provider request:

1. the trusted supervisor no-follow opens and holds the claim parent;
2. it verifies `U/M/R/I` and current budget;
3. it creates the exact claim basename using create-new/no-follow semantics;
4. it writes the run/phase/call ID, predecessor hashes, attempt `1`, and worst-case reservation;
5. it fsyncs, closes, reopens no-follow, verifies exact bytes and inode, then fsyncs the parent;
6. only after durability may it admit the child or provider request.

Any surviving complete, partial, malformed, rebound, or conflicting claim spends that phase. It is
never deleted or repaired. Crash, timeout, cancellation, supervisor loss, or uncertain spawn after
the claim is terminal and forbids replay. Failure before a durable claim admits no child but still
abandons the epoch; retry uses a newly named epoch.

### E — append-only execution evidence

Only the trusted supervisor writes authoritative receipts. Each receipt binds `U/M/R/I/S`, exact
start/end times, exit/signal/timeout, resource observations, provider request ID, adapter-reported
tokens/cost, stdout/stderr/output hashes, sandbox violations, descendant cleanup, and input
revalidation. Candidate output is evidence, never authority.

The supervisor publishes the terminal report last, create-new, only after all declared claims and
receipts reconcile, every descendant is terminated/reaped, provider access is closed, inputs still
match, output names and bounds are exact, and budget arithmetic matches adapter receipts.

Unexpected files, missing or duplicate receipts, source drift, undeclared access, timeout, budget
overflow, cleanup uncertainty, or partial terminal output yields `FAILED` or `INDETERMINATE`.
Nothing may translate those states to success.

## Minimal lifecycle

| Gate | Evidence | Permitted next action |
|---|---|---|
| G0 proposal | exact frozen proposal plus advisory findings | request exact user acceptance |
| G1 user policy | authentic user acceptance of proposal hash and ceilings | implement supervisor/schema/tests |
| G2 implementation | exact source freeze and independent code/guard review | provider-free negative tests |
| G3 isolation | all source-drift, extra-input, secret, network, replay, timeout, and output probes pass | one provider-free dry run |
| G4 dry run | exact receipts reconcile and result is automated/unreviewed | freeze one provider run plan |
| G5 provider plan | exact plan, auth readiness, disk reserve, and claims absent | one bounded provider run |
| G6 completion | deterministic audit of all artifacts, costs, tokens, scope, leakage, and outcomes | comparison report |
| G7 flagship audit | every original completion criterion has direct evidence | decide goal completion |

A gate failure preserves evidence and authorizes only a newly named successor. No completed lane is
replayed or repaired.

## Required pre-execution tests

Before any candidate or provider action, independent tests must prove rejection of:

- one-byte source, bundle, manifest, runtime, supervisor, sandbox, prompt, or run-plan drift;
- extra, missing, reordered, duplicate, aliased, hardlinked, symlinked, special, or case-equivalent
  bundle entries;
- extra environment variables, file descriptors, mounts, reachable host paths, or network endpoints;
- repository/project/task/relevance-mode mismatch and cross-project Engram retrieval;
- auth-cache, Keychain, home-directory, SSH, browser, clipboard, cloud-metadata, and canary-secret
  access from the candidate zone;
- claim replay, partial claim, crash after claim, duplicate provider call, timeout, supervisor loss,
  over-budget request, and incomplete accounting;
- unexpected output, partial output, forged candidate completion, undeclared descendant, and failed
  cleanup.

One positive provider-free fixture must prove exact success without using real credentials or
network. Test fixtures never share the production run namespace.

## Result vocabulary

Every machine-produced report contains:

```text
authority_basis=standing-user-policy-plus-automated-checks
review_status=automated_unreviewed
human_reviewed=false
ai_review_role=non_authoritative_advisory
```

The strongest terminal label before a later human review is
`AUTOMATED_EXECUTION_COMPLETED_UNREVIEWED`. It means only that the declared mechanical checks and
run plan passed. It does not mean reviewed code, semantic correctness, production readiness, or
flagship-goal completion.

Only an independently verifiable human event may change `human_reviewed`; an AI, candidate,
supervisor, test, or local file cannot.

## Supersession and current state

If the user accepts this exact proposal hash, it supersedes the rejected line's requirement that AI
reviews, canonical review observations, consolidated review records, authority DAGs, Gate0
attestations, E1/I_static/E2 review envelopes, or an ephemeral coordinator capability grant
execution authority.

It retains source/input closure, strict run plans, isolation, provider separation, durable spends,
watchdogs, append-only traces, no replay, budget accounting, scope/leakage checks, and completion
audits. Existing accepted earlier C2A/C2B1 evidence remains historical input, never execution
authority for this successor.

At freeze time the candidate source identities remain:

```text
launcher     b5e50318b84a1ba442500781bef4bf405559a43fca46737d8f82b1d236fe2530
controller   eefa6eecb239ee1df49ba51317702cf78199edb7432ce32314b46a8db726b9f9
provisioner  aabe6b17ac366036a7cef05826106ea1cb6b8a30e5437937b13320ca44599261
```

They have not been imported, compiled, AST-parsed, executed, built, statically qualified, run against
a provider, or accepted by this proposal. The prospective advisory review and all new policy,
bundle, manifest, plan, isolation, claim, receipt, and result paths are absent. The forbidden
combined preflight output and repository `target/debug` remain absent. User-owned worktree changes
remain preserved; nothing is staged or committed.

This proposal authorizes only independent advisory design review. It authorizes no source edit,
candidate action, static/build/runtime/provider/pilot execution, live adapter change, publication,
or deletion.

