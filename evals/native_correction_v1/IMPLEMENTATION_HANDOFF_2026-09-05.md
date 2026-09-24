# Native correction v1 implementation handoff

## State

The provider-free correction foundation is implemented and verified, but it is intentionally not an
executable pilot family. No AI provider was called, no authentication was inspected or copied, no
live adapter or setting was changed, and no repository artifact was staged or committed.

Implemented evidence boundaries include the exact full-profile Engram stdio seed/state/apply/restart
backend; closed-world Codex and Claude JSONL extraction; exact proposal, digest, identity, scope,
full writer/evidence/confidence/lifecycle state-transition, token, cost, chronology, and canary
checks; exact ADR bytes and scope-wrapper warnings; rejection of earlier provider prose and malformed
reasoning/thinking; durable pre-dispatch intent/WAL and honest no-replay recovery;
private/closed-world materialization; lifecycle residue checks; and a guarded two-destination Codex
auth-copy primitive exercised only with synthetic credentials.

The final source candidate also rejects recursive duplicate JSON keys, lexical/canonical/hardlink
aliases, unlisted lane/config residue, unbounded or external Git metadata, ambient daemon
environment injection, and orphan or oversized digest artifacts. Evaluator subprocesses are
group-owned and bounded by a process-start-computed terminal deadline with a pre-reserved teardown
window. Successful cleanup re-signals late descendants, proves two stable empty group observations,
and reaps the reserved leader. Persistent EPERM/enumerator failure returns bounded and explicitly
non-authorizing, aggregates completion/reap/collector evidence, and may detach an unreaped process
or unfinished collector rather than retry an identity after its safe authority boundary is lost.
Stdio shutdown joins and completely drains collectors on the successful path, rejects all trailing
or malformed bytes, and returns its one causal stop record without a second post-hoc stop. Valid
trailing JSONL is appended to the in-memory transcript before rejection; malformed bytes produce a
bounded diagnostic only. Lossless durable malformed-output evidence remains a sealed-runner
requirement and is not claimed by this standalone failure path. Detached
daemon cleanup is a typed direct-SIGTERM operation after two frozen executable/argv/environment/
partition/health attestations; usable PID-only and PID+port startup residue is cleaned but degrades
the enclosing result when complete spawn metadata is absent or invalid.

The third repair pass also makes request delivery itself bounded: retained ChildStdin descriptors
are nonblocking, partial writes and poll share the process-start response deadline, and slow-progress
writers recheck that absolute deadline every loop. Only complete frames enter the transcript. A
partial or stalled delivery triggers an in-session teardown against the original terminal deadline,
aggregating group, collector, bounded drain, stderr, abandonment, and daemon-cleanup failures while
disarming later Drop retries. That aggregate is retained as an explicit terminal session state, so
the existing unconditional production `finish` calls return the same non-authorizing failure
without dereferencing disarmed handles or panicking. Stdio startup fallback receives that same
terminal Instant, and every post-allocation full-profile startup/handshake failure aggregates
unconditional temporary probe-root removal; neither path grants itself a fresh cleanup interval.
Runtime probe allocation now resolves the operating system temporary directory before creating an
exclusive child. On macOS this converts the system-provided `/var/...` alias to `/private/var/...`,
so the strict daemon-home no-symlink/canonical-identity check remains intact rather than being
bypassed or weakened.
The next installed gate proved the daemon argv itself was not the mismatch. A bounded isolated
provider-free diagnostic observed the exact seven-element frozen argv plus five environment names:
the four intended variables and macOS-inserted `__CF_USER_TEXT_ENCODING`. No environment value or
token was emitted. Validation now diagnoses argv and environment separately and admits only that
single platform variable with its exact effective-UID-bound system spelling. Extra variables and
wrong, ambiguous, reordered, or duplicated partition arguments continue to fail closed.
The third installed rerun reached health framing and exposed a client transport defect: calling
`Shutdown::Write` after the complete GET caused the frozen Hyper/Axum server to return zero bytes.
A bounded comparison proved the same request without the half-close returns one exact HTTP/1.1
Content-Length response under `Connection: close`. The half-close is removed. Parsing now admits
only the observed status and four-header framing, with a single canonical Content-Length and no
Transfer-Encoding, and rejects duplicate/conflicting length, smuggling ambiguity, overflow,
truncation, and trailing bytes before the unchanged exact semantic health checks.
The fourth installed rerun passed those gates and reached real seed orientation. It exposed one
evaluator-only v4 contract mismatch: explicit-project authorization intentionally emits no
`project_link_ids`; topology-link evidence comes from the earlier exact `repo(link_project)`
request/result. The corrected validator now freezes an empty link-ID array, the one Atlas
candidate, the source-defined explicit-project reason, empty component evidence, and no ambiguity.
It directly rejects wrong project/cwd, ambiguous, missing, extra, and falsely link-derived shapes.
The current repair completes that v4 alignment: the seed receipt binds the checkout ID and HEAD to
the preceding repository-detection result; no-intent orientation requires exactly one Brain Loop
top/used item for the seeded active Atlas decision while both hot-context arrays remain empty; and
fresh retrieval requires the applied replacement in that same Brain Loop contract. Seed and fresh
state use exact local/global/selector envelopes, and operator code validates each wrapper before
parsing state or selecting apply. Proposal writer/evidence/replacement/proposal timestamps, apply
evidence/item/applied timestamps, and applied-before-restart ordering are fail-closed. The ignored
installed test also validates the real orient result immediately before its direct proposal call.
Production provider execution remains blocked: without a future execution-owned phase-filtered
relay, provider trace validation is necessarily post-hoc and cannot serve as a pre-mutation orient
interposer.

The fifth provider-free installed run progressed through preparation and into fresh seed-state
re-attestation, then failed after 229.10 seconds with the formerly unlabelled
`cleanup crossed its deadline while proving absent controls` error. The relevant re-attestation
performs a standalone daemon-stop preflight before the later 60-second proposal sessions. That
preflight had only two seconds while it still had to stably read and hash the frozen
219,172,408-byte Engram executable and validate the exact isolated home, partition, and token even
when controls were already absent. The old diagnostic did not record lane/sub-phase, so the total
229.10 seconds cannot be honestly decomposed further; it also supplies no evidence that the later
60-second sessions are too short, and those remain unchanged. The repair gives standalone stop
attestation one named 30-second absolute budget, passed without refresh, tightens both final guards
to `now >= deadline`, and adds content-free lane/phase/elapsed/budget diagnostics for fresh-state
preflight versus session finish. Existing stdio sessions still compute response and terminal
deadlines once at process start and pass the original terminal Instant into cleanup. Deterministic
tests cover exact-deadline rejection, adequate budget, and original-deadline propagation with
primary-plus-cleanup aggregation.

The sixth provider-free installed run then reached the real operator apply/restart lifecycle and
failed after 504.49 seconds with the old aggregate stop/session-or-process diagnostic. The defect
was an evidence-capture contradiction, not an observed Engram failure: each session's completion
timestamp was captured after `finish()` had already performed EOF, teardown, and the evidenced
daemon stop, while validation required completion before stop start. Operator trace schema 2 now
uses the explicit `observation_completed_unix_ms` field, captured immediately after the last
expected MCP response and before finish. The receipt still completes only after both session
teardowns. Validation keeps the full causal chain, including normal start <= dispatch <= applied <=
observation completion <= stop, recovery's historical dispatch <= applied <= current receipt start,
and the recovery process-ID offset. Synthetic traces now use separated nonzero intervals instead of
equality masking; direct checks cover equality, reversed ordering, normal/recovery mutation
chronology, and wrong dispatch/session process IDs. Diagnostics expose only field names, deltas,
and process IDs.

The guarded Codex auth copier is byte-opaque: source material remains in a zeroizing buffer and is
never parsed, hashed, logged, or returned. It retains and revalidates owner-only directory
descriptors, creates both leaves with no-follow/exclusive relative opens, inode-binds handles to
entries, and re-reads both copies through the retained file descriptors into zeroizing buffers for
byte-exact comparison with the opaque source before success. It rolls back only exact
evaluator-created inodes through those descriptors. Git attestation rejects `commondir` and all
special privilege bits. Claude telemetry requires every terminal and per-model web-search/fetch
counter to be present and exactly zero.

The local managed Claude environment is explicitly rejected because observed startup uses a host
`apiKeyHelper`. A strong Claude result requires a mount-free VM, in-guest login, and an exact
execution-owned auth/terminal contract frozen from that environment.

## Frozen candidate evidence

- Focused correction suite: 54 passed, 0 failed, with 1 exact-binary gate skipped.
- Exact installed-binary provider-free gate: 1 passed, 0 failed, 54 filtered out, in 746.72
  seconds against the frozen source hash below. It exercised both real isolated lanes through seed,
  proposal, exactly-once apply, interrupted recovery without replay, restart persistence, and
  terminal cleanup; it invoked no AI provider and did not touch a live authentication cache or
  installed adapter setting.
- Focused strict Clippy, correction source format, scoped diff, and repository
  `target/debug`-absence checks passed.
- A historical pre-repair source passed the ignored exact-installed-binary test once (1 passed,
  0 failed in 398.92 seconds). Security repairs changed the candidate afterward, so that run is not
  evidence for the hash below. A provider-free run against the subsequent fourth-pass candidate
  failed after 41.23 seconds because the runtime probe used macOS's `/var/...` temporary-directory
  alias and the daemon-home gate correctly required canonical `/private/var/...` spelling. This
  was fixed at allocation. The next provider-free installed run reached daemon process attestation
  and failed after 39.64 seconds because its generic error combined argv and environment mismatch:
  argv was exact, while macOS had inserted its standard fifth text-encoding variable. This
  was narrowly bound. The third provider-free installed run reached health attestation and failed
  after 40.08 seconds because the evaluator prematurely half-closed the request socket; a bounded
  comparison established the exact server response described above. The fourth provider-free run
  passed runtime and health, reached seed orientation, and failed after 131.11 seconds on the
  incorrect non-empty explicit-project link-ID expectation described above. The next candidate
  fixed that evaluator defect. A fifth provider-free installed run then reached fresh seed-state
  re-attestation and failed after 229.10 seconds on the accidental two-second standalone stop
  budget described above. The deadline-repaired candidate then reached real operator work in a
  sixth run and failed after 504.49 seconds on the contradictory session-completion capture
  described above. The schema-2 repair was independently accepted with no P0/P1/P2 finding. A
  seventh exact installed-binary provider-free run then passed against this candidate: 1 passed,
  0 failed, 54 filtered out, in 746.72 seconds.
- Repository-wide tests and all-target Clippy remain a later shared-integration gate.
- `engram-eval/src/native_correction.rs` SHA-256:
  `98c3559b64ee6bf8daa8fdcaa9232ef6e70985209e145ed109001a9e35a95c7b`
- `engram-eval/tests/native_correction_foundation.rs` SHA-256:
  `8e2da2b1ce469709f753f6d9e88ca20319dbcaf4de7b903861d05c96a1eee050`
- `evals/native_correction_v1/protocol.template.json` SHA-256:
  `887edfabbb6fb25fc7ce8960ce2887f8960c152c24a8309a426d35855647aea8`
- `evals/native_correction_v1/FOUNDATIONS_2026-09-05.md` SHA-256:
  `7b93bfb74e44ae5769fe2173ee50f2cd9a86ccc86f0107de1c85245f9aeab33c`

Cargo output is under `/private/tmp/engram-native-correction-repair-20260905`; repository
`target/debug` remains absent.

## Hard blockers before any live pilot

1. Build and independently review an execution-owned provider subprocess runner that exclusively
   creates launch/trace/stderr/output/receipt evidence and binds actual argv, cleared environment,
   executable identity, PID/session, timing, status, timeout, and output limits. It must execute
   descriptor-bound bytes (or private immutable snapshots), persist bounded raw stdout/stderr plus
   a terminal failure receipt before returning on malformed output, own or prestart each lane daemon
   under the exact evaluator environment, and retain atomic process identity through cleanup. The
   current pathname-hash/exec and macOS PID-attest/SIGTERM intervals do not defend against a
   concurrent malicious same-UID replacement.
2. Wire the ordered prepare, attest, state-check, auth-check, proposal, operator, retrieval, audit,
   comparison, and completion-report lifecycle through shared library and CLI entry points.
3. Close the underlying CLI daemon-start failure seam: if spawn succeeds before the first usable
   PID control is written, this standalone module cannot distinguish the untracked child from a
   cleanly stopped daemon. The CLI must kill its owned child on all control-write failures, or the
   sealed runner must own the daemon directly.
4. Prepare and attest the mount-free Claude VM and its in-guest authentication contract.
5. Replace the template pilot identity/nonce/opaque labels exactly once, freeze the concrete run
   plan and protocol, then repeat all provider-free gates before seeking a real execution.

Do not reinterpret the current module or this handoff as provider-run readiness. The smallest honest
next step is the execution-owned runner plus shared orchestration/reporting, followed by an
independent audit before any credentials or provider calls are admitted.
