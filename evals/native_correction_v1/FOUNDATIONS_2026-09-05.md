# Native correction v1 provider-free foundations

Status: provider-free backend, materialization, parsers, and audit foundation implemented as a
standalone test module. This is **not an executable pilot family**: shared library/CLI orchestration,
an execution-owned provider runner, comparison/completion reporting, and a clean Claude execution
environment remain deliberately absent.

## Conditional claim boundary

Once the missing runner and environment gates exist, this family is designed to measure one narrow
behavior on each supported coding host:

1. A restricted agent reads the tracked Atlas ADR, creates one inactive, server-minted,
   digest-bound correction proposal, and proves the old item remains the only active same-tag Atlas
   item.
2. A fresh full-profile operator process attempts only a wrong-scope inspection of that Atlas
   proposal while selecting Orbit, rediscovers the proposal under Atlas, applies the exact proposal
   and digest once, and restarts to prove the replacement is active and the old item is superseded.
3. A fresh restricted process receives only the project, cwd, and tag and retrieves exactly the
   replacement without file, shell, network, or mutation tools.

The Orbit canary is seeded and proved privately. It is never retrieved into a controlled provider
or operator trace; the public operator evidence contains only the bounded wrong-scope error. Operator
selection is administrative selection only. It is not authenticated human identity, verified
intent, reviewer authority, or a general same-UID security boundary. Procedure correction remains
unsupported.

## Frozen runtime boundary

The foundation re-stats and hashes the exact Engram, Codex, sibling Codex code-mode host, and Claude
Code executables and runs bounded version/contract probes. The current Engram contract is agent
schema 4 / MCP contract 4. The frozen full profile is exactly 32 tools with SHA-256
`714e07ffd5fcc5e615ef3eeb6df2e479801495a834bc0f6fa7e0672a68410422`; seed and operator transcripts
are compared with the count and digest in the attested plan, not an inherited schema ordinal.
Unknown binaries, contracts, tool inventories, timeouts, output overflow, non-zero child exits, and
dangling lifecycle artifacts fail closed.

All evaluator-owned subprocesses start with a cleared, exact environment and bounded output/time.
Each stdio session computes one terminal deadline at process start and reserves at most one quarter
of that interval (capped at two seconds) for teardown; every request shares the earlier response
deadline. Child stdin is retained in nonblocking mode and every JSONL request is delivered by
partial writes plus bounded poll against that original response deadline. The loop checks the
deadline even while a child makes slow forward progress; no blocking `write_all` or `flush` remains.
Only a fully delivered frame enters the transcript. A partial delivery reports only its byte count,
then consumes the original terminal teardown window and aggregates process, collector, bounded
stdout-drain, stderr, abandonment, and daemon-cleanup failures. Stdio construction and fallback
daemon cleanup receive those same process-start-computed deadlines rather than allocating a fresh
interval. Successful shutdown joins both collectors, drains stdout to disconnection, rejects every
unaccounted line or collector failure, aggregates process/collector/daemon-cleanup failures, and
returns the one causal daemon-stop record used by the receipt rather than performing an unrecorded
stop followed by an already-absent observation. Successful process-group cleanup keeps the direct
child WNOWAIT-reserved, re-signals every newly observed same-group member, requires two fresh empty
observations, and only then reaps the leader. Permission-denied or persistently unavailable cleanup
authority is an explicit non-authorizing failure: the call returns within the precomputed terminal
window, reports any detached collector/process resources, and never retries a possibly reused group
identity. Kernel/filesystem metadata calls are bounded by fail-closed checks around the operation;
this is not a hard-real-time guarantee.

This standalone failure path appends every valid trailing JSONL line to its in-memory transcript
before rejecting it. Malformed trailing bytes also fail closed, but are represented only by a
bounded diagnostic and do not become a successful receipt or a durable lossless raw artifact. The
future sealed runner must create bounded raw stdout/stderr artifacts and a terminal failure receipt
before returning on malformed output; that stronger failure-forensics boundary is not claimed here.
The detached Engram daemon is accepted for cleanup only after two live checks bind the frozen
executable, exact serve argv, isolated home and partition, four-variable environment, process group,
and health identity; cleanup sends SIGTERM directly to that attested PID instead of executing an
unbound `daemon stop` argv. The run plan records this as a typed cleanup method, and the receipt
distinguishes an already-absent daemon from a direct signal. Usable PID-only and PID+port startup
residue can be identified and safely terminated; missing, corrupt, or inconsistent metadata makes
the enclosing operation fail even after safe cleanup.

This standalone module does not claim protection against a concurrent malicious same-UID process
replacing an executable between hash and pathname execution, or replacing a PID between the final
macOS process attestation and SIGTERM. Those races require descriptor-bound execution and a
runner-owned daemon child. They remain outside this foundation's threat boundary and are hard
blockers for a live pilot.

The local managed Claude environment is not an admissible execution environment. Provider-free
observation found Claude 2.1.260 reporting `apiKeySource: "apiKeyHelper"`, backed by a host-managed
credential helper. Correction traces require a directly observed `apiKeySource: "none"`; the public
local auth-readiness gate intentionally fails closed. The intended strong Claude run therefore needs
a mount-free VM with an in-guest login, no host MDM/drop-ins, and a new execution-owned parser frozen
from the exact observed in-guest auth-status schema. This foundation does not claim that such a VM is
prepared or authenticated.

## Provider-free preparation and state proof

Production preparation is fixed to the concrete full-profile Engram stdio backend; callers cannot
substitute an asserted seed backend. It exclusively creates two opaque lanes, one per host. Every
lane has its own owner-only Engram home, daemon partition, closed-world clean Git fixture with exact
origin, registered repository/project identity, Atlas obsolete row, same-tag Orbit row, private
canary contract, and distinct proposal/retrieval host configuration.

The visible fixture tree is exact. Its `.git` metadata is recursively owner/mode/link/symlink
checked, capped at 1,024 entries, 8 MiB per file, and 64 MiB total, rejects `commondir`, external
object-store alternates, special privilege bits, and non-directory `.git`, and is content-bound into
the fixture revision before any bounded Git inspection.

Seed JSON-RPC request/result pairs and empty stderr are preserved losslessly in private artifacts,
hashed into the plan, and reparsed by every plan consumer. A separate provider-free pre-execution
gate starts the attested Engram binary and freshly checks identity, both scoped rows, global row IDs,
and the absence of replacements/proposals against the seed receipt. It returns IDs/counts only and
does not publish canary content.

Codex has two absent, create-new `auth.json` destinations. Its guarded copier treats the source as an
opaque zeroizing byte buffer: it never parses, hashes, logs, or returns credential content. It walks
and retains owner-only directory descriptors, uses relative no-follow/exclusive creates, binds every
created handle to its directory entry, fsyncs files and parents, globally revalidates both chains,
re-reads each destination through its retained read/write file descriptor into a second zeroizing
buffer, and requires byte-for-byte equality with the opaque source before success. It rolls back
only the exact created inodes through held descriptors; an unknown replacement is never removed.
Claude has distinct configuration directories, strict MCP configuration, disabled auto-memory, and
no session persistence. The future VM must add two independent in-guest auth-status checks; this
local foundation refuses to infer them. No live credential was provisioned while building or
testing this foundation.

## Trace, causality, and recovery evidence

Codex and Claude JSONL are parsed as closed-world host-specific state machines. Every accepted tool
call has one correlated result ID; unknown events/tools, mirrors used as evidence, duplicate/orphan
IDs, extra terminal output, forbidden canaries, cost/token overflow, stderr drift, and tool/result
shape drift fail closed. Codex cannot emit an earlier prose agent message or a started reasoning item;
Claude cannot emit assistant prose before StructuredOutput, and both hosts have closed grammars for
their permitted reasoning/thinking records. Every Claude terminal and per-model web-search/fetch
counter is required to be present and exactly zero. Proposal/list/get/apply results are correlated to
server-minted distinct IDs, the recomputed canonical digest, complete writer/evidence/confidence and
lifecycle snapshots, inactive old/new state, and the exact allowed post-apply and post-restart
transition. Orient is bound to the private seed identity, including the exact repository, checkout,
HEAD, normalized remote, and root derived from the preceding repository-detection evidence. For the
synthetic no-intent request, Brain Loop must return the one expected active Atlas decision while hot
context remains exactly empty; retrieval binds that same shape to the applied replacement. The Codex
ADR read is byte-exact, and every seed, fresh-state, provider, operator, apply, and restart normal,
global, or correction-selector scope wrapper requires the exact non-degraded fields and warnings
emitted by the frozen Engram contract. Proposal writer/evidence/replacement/proposal timestamps and
apply-evidence/item/applied/restart timestamps must preserve the source-defined causal order.

The operator writes a create-new intent and a fsynced pre-dispatch journal containing the canonical
plan, proposal/digest tuple, process ID, JSON-RPC ID, request bytes, and digests before sending apply.
Its receipt is derived from a closed-world two-session JSON-RPC transcript plus three bounded,
typed daemon-stop records. A pending tuple with no dispatch may be applied once. Applied-state recovery is allowed
only with the exact prior dispatch journal and is labeled
`applied_state_observed_without_dispatch_attribution`; it cannot claim observed authority and cannot
replay apply. Proposal and retrieval traces without receipts freeze their lanes and are never
recovered. Partial receipts, orphan sidecars, future-lane residue, and reordered phases freeze or
invalidate the run. Completion and provider-call counts include only fully validated receipts.

## Verification and remaining blockers

The synthetic provider-free correction suite passes 54 tests, with the exact-installed-binary test
ignored by default. It covers protocol shape, runtime drift, private materialization, raw seed
causality/tamper, scoped identity, strict proposal/retrieval host traces, exact result keys, raw
canary scanning, cost/token/error terminals, operator transcript and session tampering, durable
dispatch/no-replay recovery, restart persistence, lifecycle residue, completion-count validity, and
guarded auth behavior. Adversarial cases additionally cover coordinated provenance/lifecycle drift,
identity/source/ADR/scope drift, hidden non-empty zero-count collections, earlier host prose,
malformed reasoning/thinking, and Codex diagnostic suffixes containing auth/error/panic/credential
signals. Added adversarial cases cover nested duplicate JSON keys, lexical/canonical/hardlink path
aliases, unlisted lane/config residue, bounded digest reads, Git config and alternates injection,
ambient environment injection, same-process-group descendant cleanup, ECHILD and EPERM handling,
cleanup-error aggregation, partial daemon-control ordering, one-time causal stop evidence, absolute
stdio deadlines, trailing/truncated/non-UTF-8 stdout rejection, exact-zero Claude network counters,
opaque invalid-UTF-8 credential copying, held-directory parent/source/entry replacement, and
inode-bound auth-copy rollback. The second repair pass adds direct production-path negatives for a
late descendant appearing between empty observations, one-signal-only error handling, persistent
EPERM and process-enumerator failure with bounded non-authorizing return, simultaneous cleanup plus
stdout/stderr cap failures, collector detachment diagnostics, and same-inode/same-size credential
content mutation before final retained-descriptor verification. The third repair pass adds a
non-draining multi-megabyte stdin delivery negative (including aggregated EPERM/collector cleanup
failure), a same-original-deadline stdio-start fallback fault, and a full-profile startup/probe-root
cleanup fault proving temporary state removal while preserving both the primary and cleanup errors.
The fourth repair pass makes bounded request-delivery cleanup an explicit terminal session state:
all unconditional production callers may still invoke `finish`, but it returns the preserved
aggregated failure without dereferencing disarmed child or collector handles. A deterministic
closed-stdin full-profile caller regression proves no panic and unconditional probe-root removal.
The fifth repair resolves the macOS temporary-directory spelling before allocating runtime probe
roots. This preserves the no-symlink/non-canonical daemon-home check while ensuring the evaluator
passes `/private/var/...`, rather than the OS-provided `/var/...` alias, to the isolated daemon. A
direct symlinked-base regression proves allocation occurs only under the resolved canonical base.
The sixth repair separates daemon argv and environment diagnostics and models the exact installed
macOS spawn surface. A bounded provider-free probe observed the intended seven-element argv
unchanged, while macOS added `__CF_USER_TEXT_ENCODING` to the four explicitly cleared-and-set
variables. The evaluator admits only that one platform variable, with the exact effective-UID-bound
system spelling; every other extra, missing, duplicated, or drifted variable still fails closed.
Wrong, reordered, or repeated `--project` arguments remain rejected. The probe reported only argv
and environment names/count, never the daemon token or any environment value, then stopped the
isolated daemon and removed its temporary home.
The seventh repair removes a premature client write-half-close from the health request. A bounded
comparison against the frozen server observed zero response bytes after that half-close, versus an
exact `HTTP/1.1 200 OK` Content-Length response when the complete request remained open under
`Connection: close`. The response parser now freezes the observed lower-case header order and
values for content type, canonical single Content-Length, connection close, and bounded Date
shape. It rejects duplicate or conflicting lengths, every Transfer-Encoding variant, unexpected
headers, oversized or malformed framing, and truncated or trailing body bytes before applying the
unchanged strict PID/schema/contract/token/storage body checks. Neither diagnostic emitted health
body values or a token.

The eighth repair aligns the frozen lean-orient identity contract with the installed v4
implementation without weakening scope checks. For `source=explicit_project`, Engram deliberately
emits an empty `identity.project.project_link_ids` array: the explicit caller selection is the
authorization source, while the preceding exact `repo(link_project)` request/result remains the
independent topology-link evidence. The evaluator had incorrectly injected and required a link ID
in that explicit-project response. It now requires exactly no link IDs, exactly the Atlas project
candidate, the source-defined explicit-project reason, no components, and no identity/resolution
ambiguity for this synthetic fixture. Direct regressions accept the current real-contract shape and
reject a wrong project, wrong cwd, ambiguity, missing or extra schema fields, and a falsely
link-derived explicit identity.

The ninth repair aligns the remaining orientation and scoped-result contract with the current v4
implementation. Brain Loop and hot context are validated independently: the seeded active Atlas
decision must be the sole top/used Brain Loop item with exact compiled context, while both
hot-context arrays are empty because the request supplies no intent. Fresh retrieval instead binds
that exact shape to the applied replacement. Checkout ID and HEAD are frozen in the seed receipt and
must equal the preceding repository-detection result on every transcript replay and fresh-state
inspection. Seed fixtures use the real local/global/selector response envelopes rather than bare
list objects, and the operator validates every wrapper before parsing it or deciding whether apply
is permitted. Proposal and apply chronology plus applied-before-restart chronology are independently
rejected when drifted. The ignored provider-free installed test now validates its real orient result
immediately before issuing `propose_correction`, closing the direct-test mutation-ordering defect.
There is still no production provider relay in this standalone module: host-trace validation is
post-hoc evidence validation, not a pre-mutation interposer. A future phase-filtered execution-owned
relay must validate the real orient result before forwarding any provider-requested correction
proposal; until then this foundation does not authorize provider execution.

The tenth repair addresses the next installed-gate failure without weakening daemon identity. A
fifth provider-free run reached the fresh seed-state re-attestation and failed after 229.10 seconds
with the previously unlabelled `cleanup crossed its deadline while proving absent controls` error.
That re-attestation performs an idempotent daemon-stop preflight before either later 60-second
proposal session. The preflight had an unrelated hard-coded two-second deadline while still
requiring a stable read and SHA-256 of the frozen 219,172,408-byte Engram executable plus exact
home, partition, and token validation even in the expected all-controls-absent state. The old error
did not record lane or sub-phase, and the same inner message can also arise during session finish,
so the 229.10-second whole-test duration cannot honestly be decomposed further. The source path and
unwrapped error make the standalone preflight the specific defect repaired here; they do not prove
that either later 60-second proposal session is too short, so those session budgets remain
unchanged. Standalone stop attestation now receives one checked, named 30-second absolute budget,
passed unchanged through validation and cleanup. Stdio response/terminal deadlines remain computed
once at process start and are never refreshed for cleanup. Both terminal evidence guards now use
the fail-closed `now >= deadline` boundary. Fresh-state failures report only lane, phase, elapsed
milliseconds, and declared budget—never memory, token, or path contents. Deterministic regressions
cover equality-at-deadline rejection, an adequate declared budget, and preservation and aggregation
of the original cleanup deadline.

The eleventh repair resolves a deterministic operator-evidence timestamp mismatch exposed only
after the sixth provider-free installed run reached the real apply/restart lifecycle. That run
failed after 504.49 seconds with the old aggregate `operator stop/session chronology or dispatch
process binding drifted` diagnostic. Static inspection identifies the exact contradiction: the
evaluator captured each session's `completed_unix_ms` only after `finish()` had completed EOF,
proxy/collector teardown, and the separately evidenced daemon stop, while validation required that
same timestamp to precede the stop's start. Synthetic evidence used equal millisecond timestamps,
which masked the mismatch. Operator trace schema 2 now names this boundary
`observation_completed_unix_ms`, captures it immediately after the final expected MCP response and
before `finish()`, and retains the causal order observation completion <= stop start <= stop
completion. The enclosing receipt remains the teardown-completion boundary. Normal apply requires
first-session start <= dispatch journal creation <= server apply time <= first observation
completion; recovery requires the historical dispatch <= server apply time <= current receipt
start and shifts the two current session process IDs past the historical dispatch process. Named
failures report only field names, timestamp deltas, or process IDs. Deterministic regressions cover
nonzero separated intervals, equality, reversed completion/stop order, normal and recovery mutation
ordering, and wrong dispatch/session PID binding without weakening any causal comparison.

A historical pre-repair candidate passed the ignored installed-binary test once (1 passed, 0 failed
in 398.92 seconds), but subsequent security changes invalidate that result as evidence for the
current source hash. The fourth-pass candidate was then exercised provider-free under the fully
qualified installed test name and failed after 41.23 seconds because macOS returned the symlinked
`/var/...` spelling for its temporary directory; the unchanged daemon-home gate correctly rejected
that non-canonical spelling. The current candidate fixes allocation rather than weakening the gate
and reached the next gate on an installed rerun: after 39.64 seconds, the generic combined
argv/environment check rejected the macOS-inserted fifth variable even though the exact partition
argv matched. The current candidate implements the narrowly observed platform allowlist and was
then run through the installed gate: it reached health attestation and failed after 40.08 seconds
because the evaluator half-closed its write side immediately after the request. The current
candidate removed that client transport defect. A fourth provider-free installed run then passed
runtime, process, and health attestation and reached real seed orientation before failing after
131.11 seconds: the evaluator required a non-empty topology-link ID from an explicit-project
identity even though the v4 implementation intentionally emits an empty array for that source. The
next candidate fixed that evaluator defect. A fifth provider-free installed run progressed through
preparation and into fresh seed-state re-attestation, then failed after 229.10 seconds on the
accidental two-second standalone stop budget described above. The deadline-repaired candidate was
subsequently exercised a sixth time and progressed through real seed, proposal, and operator work
before failing after 504.49 seconds on the deterministic session-completion capture mismatch
described above. After the schema-2 observation-completion repair was independently accepted, the
exact installed-binary provider-free gate was exercised a seventh time against this source and
passed: 1 test passed, 0 failed, in 746.72 seconds. That run covered real isolated seed state,
proposal creation, the exactly-once operator apply, interrupted-apply recovery without replay,
restart persistence, and terminal daemon cleanup for both lanes. It invoked no AI provider and did
not read, copy, or modify a live authentication cache or installed adapter setting.

The final provider-free verification used an external Cargo target and produced these results:

- focused correction suite: 54 passed, 0 failed, with 1 exact-binary gate skipped;
- exact installed-binary provider-free gate: 1 passed, 0 failed, 54 filtered out, in 746.72
  seconds, using the same external Cargo target and source hash recorded below;
- focused strict Clippy (`-D warnings`), source format, scoped diff, and repository `target/debug`
  absence checks: passed;
- repository-wide tests and all-target Clippy were not part of this frozen-source checkpoint; they
  remain a shared-integration gate after the standalone module is independently accepted; and
- SHA-256: correction source
  `98c3559b64ee6bf8daa8fdcaa9232ef6e70985209e145ed109001a9e35a95c7b`, correction test wrapper
  `8e2da2b1ce469709f753f6d9e88ca20319dbcaf4de7b903861d05c96a1eee050`, protocol template
  `887edfabbb6fb25fc7ce8960ce2887f8960c152c24a8309a426d35855647aea8`.

Before any real run, the following remain hard blockers:

- build an execution-owned provider subprocess runner that creates raw artifacts exclusively and
  binds observed argv, cleared environment, PID/session, start/end/status, timeout, output caps, and
  actual binary identities; it must persist bounded raw stdout/stderr and a terminal failure receipt
  before returning on malformed provider or MCP output, use descriptor-bound/private-snapshot
  execution, and own or prestart each lane daemon under the exact evaluator environment before
  providers connect; the internal receipt helpers are test scaffolding, not a runner;
- repair or bypass the underlying CLI startup seam in which daemon spawn can succeed before the
  first usable `daemon.pid` write. With no PID control, normal stopped state and an untracked child
  are indistinguishable to this standalone module; token/log files persist across normal stops, so
  they cannot safely identify the process. The CLI must kill its owned child on every
  `save_daemon_info` failure, or the sealed runner must own that child directly;
- wire prepare → runtime/state/auth attestation → ordered proposal/operator/retrieval → audit → report
  through reviewed shared library/CLI entry points;
- prepare and attest the mount-free Claude VM described above, including in-guest auth and matching
  terminal `apiKeySource`; and
- add the comparison/completion report and freeze one concrete replacement protocol/run plan.

The protocol template is intentionally non-executable: its pilot ID, nonce, and opaque labels must be
replaced exactly once before a future freeze. No provider was invoked, no live authentication cache
was copied, and no adapter or installed setting was changed while creating these foundations.
