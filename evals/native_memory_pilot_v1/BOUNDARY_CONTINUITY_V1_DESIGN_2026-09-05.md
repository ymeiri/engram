# Native boundary continuity v1 — frozen design candidate — 2026-09-05

## Status, sequence, and claim boundary

This is a design candidate for the smallest honest native boundary-continuity evaluation after the
schema-15 repeated outcome. It has not been prepared, frozen, authenticated, or run. It must use a
new experiment root, protocol, lifecycle, and report. Schema 15's lanes, teaching receipts,
activation/evaluation lifecycle, authentication copies, and Pareto signal must not be repurposed.

Run the native stale/TTL-expiry evaluation first. That safety result has higher product priority,
uses an already-understood retrieval boundary, and can distinguish refusal from unsafe stale-memory
application without first depending on Claude compact automation. Boundary continuity should begin
only after stale/expiry has a final audit, or after an explicit record explains why it is blocked.

This evaluation may prove only that an opaque record seeded directly into isolated Engram is
retrievable after a real, host-specific boundary. For Claude, the boundary is a provider-observed
manual compact in one resumed session. For Codex, it is a stopped-process/fresh-process boundary,
not compaction. A passing result must not be described as transcript-to-Engram capture, Codex
compaction or session continuation, automatic consolidation, correction propagation, or
independent-machine portability.

## Separate pipeline architecture

Build a dedicated `boundary-continuity-v1` pipeline. Reuse audited implementation components, but
never reuse mutable schema-15 state or interpret its receipts as evidence for this protocol.

| Component | Responsibility | Evidence boundary |
| --- | --- | --- |
| Provider-free preparer | Creates fresh lane homes, workspaces, isolated Engram stores, prompts, hook configuration, and an immutable plan. | No provider call, target, lane credential copy, or result exists yet. |
| Credential provisioner/checker | Uses the existing guarded no-overwrite and owner-only checks for fresh lane credentials. | Never reads, prints, hashes, scans, or copies credential bytes except for the separately authorized provisioning operation. |
| Target seeder | Generates a cryptographically random lane-unique `BND-...` value after credentials are ready, writes it only through the frozen Engram seed path, records its SHA-256 and returned memory ID, then removes runner-owned plaintext setup material. | Plaintext remains only in the isolated treatment Engram store and later trusted provider/Engram traces; controls receive no target. |
| Host-boundary runner | Runs the exact Claude or Codex process sequence below with Engram unavailable before the boundary and the lean Engram profile available only afterward. | Process/session facts come from the runner and provider trace, never agent self-report. |
| Boundary observer | Correlates process receipts, Claude compact events/hooks, transcript identity, and Codex non-resumption evidence. | Emits structured facts and hashes, not inferred prose. |
| Leakage scanner | Scans the frozen, credential-excluded pre-boundary byte universe for target-shaped values and candidate SHA-256 matches. | Does not traverse credentials or the intentionally seeded Engram database. |
| Auditor/reporter | Joins plan, seeding, scan, boundary, Engram trace, and structured agent output by lane ID. | Fails closed on missing, duplicate, unordered, reused, or uncorrelated evidence. |

The host runner and auditor must communicate through append-only receipts. The reporter may consume
only frozen plan fields and trusted receipts; it must not repair a lane, synthesize a missing event,
or infer a boundary from a successful answer.

## Matrix and seed isolation

Run 12 fresh lanes: two hosts (`codex`, `claude_code`) × two conditions (seeded Engram, matched
empty-Engram control) × three repetitions. Disable host-native memory in every lane. Seed the same
scoped decoy into both conditions and one unique opaque `BND-...` target only into each treatment.
Targets must differ across all treatment repetitions and hosts. Retain only the target SHA-256 and
memory ID in the trusted immutable plan; verify the eventual answer by hashing it rather than
retaining another plaintext copy. Claude's session transcript remains enabled because it is the
boundary under test, but Claude auto-memory and every other host-native knowledge source are off.

Before the boundary, neither host may have an Engram MCP server, tool declaration, adapter, skill,
prompt, or filesystem route capable of reaching the isolated store. The neutral transcript contains
no target, memory ID, target hash, retrieval hint, or target-bearing derived value. Treatment and
control prompts and neutral work are otherwise byte-equivalent after lane-specific values are
normalized. Cross-lane target isolation is mandatory.

## Credential-excluded pre-boundary byte scan

Freeze a scan manifest before target generation. It must enumerate every runner-controlled byte
surface the host can read before the boundary: system and user prompts, instruction and skill files,
workspace fixtures, launch scripts, non-secret configuration, sanitized argv/environment manifests,
stdout/stderr/JSONL, hook payloads, transcript files, and (for Claude) the compact summary. Resolve
paths without following symlinks and fail if an included surface escapes its declared root.

The exclusion manifest must name only:

- lane authentication cache files, Keychain entries, and other provider credentials;
- provider-internal state that the runner cannot inspect;
- the isolated Engram database and the seeder's transient input, because treatment intentionally
  contains the target there; and
- post-boundary output, which is assessed separately.

Credential files are excluded by exact path and file identity established before target generation.
The scanner must neither open nor hash them. Generating every target after credential readiness
precludes intentional target pre-seeding into credentials; the frozen entropy requirement makes an
accidental equality negligible without claiming the excluded credential content was inspected. The
report must publish the included/excluded path classes, scanner version and executable hash, file
count and byte count, target-shaped match count, candidate-hash match count, unreadable-path count,
and symlink-escape count. Any match, unreadable included path, unexpected exclusion, or escape
invalidates the lane. The resulting claim is limited to the enumerated runner-controlled byte
universe; it is not a claim about inaccessible provider internals or secret contents.

## Mandatory disposable Claude compact preflight

Claude execution is blocked until a disposable, target-free preflight proves the exact compact
mechanism with the same frozen Claude executable, version, flags, hook programs, and transcript
observer intended for real lanes. The preflight must:

1. start a new session in one process, perform enough frozen neutral work to produce compactable
   history, record the provider session ID and canonical transcript path, and exit cleanly;
2. start a distinct process that resumes that exact session ID and issues the exact non-interactive
   `/compact` invocation proposed for the real runner;
3. observe exactly one provider `system/compact_boundary` event with `trigger=manual`, with exactly
   ordered `PreCompact(manual)` and `PostCompact(manual)` hook receipts bound to the same session and
   transcript; and
4. start a third distinct process that resumes the same session successfully, proving the compacted
   session remains resumable.

Freeze the successful preflight's sanitized argv hashes, executable/hook hashes, event schema,
ordering rules, and receipt SHA-256 into the protocol. Destroy or quarantine the disposable session;
never promote it into a real lane. If the installed Claude surface cannot produce all evidence, stop:
no real Claude lane may be prepared or run, and success output from `/compact` alone is insufficient.
The remedy is a new observable-boundary design and a fresh preflight, not adapting a live lane.

## Exact Claude process and session proof

Each real Claude lane uses the preflight-proven three-process sequence. Process A creates the
session and performs only the frozen neutral work. Process B begins only after A has a recorded clean
exit, resumes the exact provider session ID, and performs only the manual compact. Process C begins
only after the observer has accepted the compact event, hook ordering, and target-free summary; C
resumes the same session ID and receives the post-boundary retrieval task with lean Engram enabled.

For A, B, and C record runner nonce, PID plus process-birth identity, parent PID, executable SHA-256,
sanitized argv SHA-256, start/end monotonic timestamps, exit status, provider session ID, and
canonical transcript path. Require three distinct process identities, strict non-overlap, one stable
session ID, and one stable transcript path. The compact observer records only the compact-summary
SHA-256, byte length, target-shaped match count, and target absence; it must not copy the summary
into the plan or prompt. Reject a new session, implicit continuation, resume of any other ID,
duplicate/missing hook, wrong-session hook, transcript-path change, temporal overlap, or any target
match.

## Exact Codex stopped/fresh-process proof

Codex currently has no equivalent observable compaction marker. Label its result exactly
`stopped_process_to_fresh_process`; do not call it compaction, resume, or session retention.
Process A runs frozen neutral work with `codex exec --ephemeral --json --output-schema`, no Engram
surface, and no resume or fork. It must reach a confirmed clean exit. Process B then runs a separate
`codex exec --ephemeral --json --output-schema` with the post-boundary task and lean Engram enabled,
again with no resume or fork.

For both processes record runner nonce, PID plus process-birth identity, parent PID, executable and
sanitized argv SHA-256, start/end monotonic timestamps, exit status, and every emitted task/thread/
session identifier. Require distinct process identities, B start strictly after A exit, and no
identifier reuse. Reject a resume/fork flag, hidden continuation handle, process overlap, shared
stdin stream, or reuse of a provider task/thread/session ID. This proves retrieval after a fresh
Codex process only; it says nothing about Codex compaction or continuity of a Codex conversation.

## Post-boundary retrieval contract

The first relevant external action in Claude process C or Codex process B must be lean Engram
orientation. Trusted MCP trace correlation—not agent self-report—must establish the result. A
treatment passes only when the scoped Engram result contains the exact seeded memory ID and target,
the structured response cites and uses that ID, and the answer hash matches. An empty control passes
only when it orients, retrieves the shared decoy but no target, and abstains.

Minimal agent output fields are `answer`, `abstained`, `first_action`, `returned_memory_ids`,
`used_memory_ids`, and `evidence_targets`. Trusted reporting owns boundary/session/process facts.
Invalidate on any pre-boundary target visibility, missing or duplicate Claude boundary events,
wrong-session hooks, target-bearing compact summaries, Codex resume/reuse/overlap, missing Engram
trace correlation, cross-lane leakage, or control leakage.

Report lane integrity, boundary kind/proof, credential-excluded scan coverage, target pre-boundary
absence, compact-summary absence, post-boundary reorientation, Engram retrieval, answer/control
outcome, cross-lane isolation, and cost/token/latency/packet telemetry. Do not emit
`portable_incremental_value`, a Pareto claim, a transcript-capture claim, or a single shared
"continuity" result that erases the different Claude and Codex boundaries.

## Lifecycle and no-replay gates

The forward-only lifecycle is:

1. complete and audit the native stale/TTL-expiry run;
2. pass the disposable Claude compact preflight;
3. provider-free prepare fresh lanes, freeze protocol/plan/runtime/scanner/observer hashes, and
   attest the effective Engram tool surface;
4. provision and provider-free check fresh credentials under the existing guarded policy;
5. generate and seed targets, seal the scan manifest, remove runner-owned plaintext setup material,
   and audit treatment/control isolation;
6. run pre-boundary host processes once;
7. run and attest the Claude compact or Codex stopped/fresh-process boundary once;
8. require the credential-excluded leakage scan and every boundary receipt to pass before exposing
   lean Engram to the post-boundary process;
9. run post-boundary retrieval once, then produce a completion audit and comparison report; and
10. preserve all traces and stop on any invalid lane. Never repair, replay, or substitute a
    completed lane; prepare a new immutable successor if the harness or protocol is defective.

At every executable phase, revalidate plan and binary hashes, provider-free authentication status,
positive disk reserve, output absence for the phase about to run, and repository cache constraints.
No live adapter, settings, hook, or memory installation is part of this evaluation.

## Reusable components and later portability proof

Reuse source components from the native pilot for isolated homes, guarded no-overwrite credential
provisioning, authentication checks, disk reserve, executable/MCP attestation, exact sanitized
argument hashing, append-only trace parsing, lifecycle state, completion audits, and bounded
telemetry. Extend them with target generation/seeding, credential-excluded byte manifests, process-
birth and non-overlap receipts, Claude compact correlation, and distinct host-boundary result types.
Reuse code and validation rules only; create fresh homes, credentials, databases, prompts, plans,
receipts, and output paths.

After this protocol passes, replicate its exact frozen bundle in the already-running Colima Ubuntu
24.04.4 VM using VM-owned checkout, state, runtime, credentials, and Engram database. Record
environment ID, OS/architecture, executable hashes, state-source digest, and the transfer manifest.
A shared macOS mount, copied live host database, shared credential state, or host-owned runtime does
not establish an independent state source. That replication is a separate portability evaluation;
the local boundary-continuity result must remain valid and separately reported whether or not the VM
run succeeds.

Official Claude surfaces used by the design:

- <https://code.claude.com/docs/en/agent-sdk/skills#compact-history-with-compact>
- <https://code.claude.com/docs/en/headless#continue-conversations>
- <https://code.claude.com/docs/en/hooks#precompact>
- <https://code.claude.com/docs/en/hooks#postcompact>
