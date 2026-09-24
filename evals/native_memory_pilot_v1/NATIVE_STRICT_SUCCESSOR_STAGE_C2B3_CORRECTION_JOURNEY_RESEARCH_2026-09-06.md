# Native strict successor Stage C2B3 — correction-journey research

Date: 2026-09-06 (Asia/Jerusalem)

## Status and authority boundary

**Research only. Not frozen. Not implementation authority.**

This note records provider-free source research completed while C2A acceptance remained blocked
only by the mandatory filesystem-reserve gate. It authorizes no source, manifest, lockfile,
datastore, VM, daemon, provider, adapter, authentication, correction, deletion or settings change.

The mandatory order remains:

1. pass C2A's deferred integration gate and record accepted C2A evidence;
2. implement and accept the already frozen C2B1 Memory acquisition slice;
3. freeze, review, implement and accept C2B2 persistent containment and close/reopen acquisition;
4. freeze, review, implement and accept C2B3 correction-action authority; and
5. separately freeze any stronger deletion-propagation successor.

C2B3 must stay narrow. It is one pending-to-applied correction journey. Confirmed forget is not the
same transition: it deletes a variable set of rows, performs resumable cleanup across more tables
and may refresh a filesystem projection. Combining both behind one "exactly one action" claim would
erase materially different authority, recovery and completeness boundaries.

## Exact research inputs

```text
C2A frozen semantic design
  NATIVE_STRICT_SUCCESSOR_STAGE_C2A_SEMANTIC_VALIDATOR_FROZEN_2026-09-06.md
  5a3599038198ba95e1c6ad7421b4ba2ee703b2b96021c04a34dca196952e27bc

C2A semantic implementation
  engram-eval/src/native_successor_semantic.rs
  4da0ce228b5127587a72f65d19efd58ddbc7d6e7d00dc98a41117ff18dbce4b5

C2B1 frozen Memory design
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B1_MEMORY_ACQUISITION_FROZEN_2026-09-06.md
  04f1be24c758f4a7f52baafab2d9ff27b9fd59241eecc48545deb3e5365600cb

C2B1 review record
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B1_MEMORY_ACQUISITION_REVIEW_2026-09-06.md
  56ee4d0b7dbd99d5742d6a5a80e1a0665a379d0dc44a54c9e092ecce497d73e4

C2B2 persistent-containment research
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2_PERSISTENT_CONTAINMENT_RESEARCH_2026-09-06.md
  00735349dd10c8c18414a4e611fc5030a82ae014c9ac95bfd9872b3f9427ece0

C2B2 review record
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2_PERSISTENT_CONTAINMENT_REVIEW_2026-09-06.md
  f68400713ceef924d2e0f24d642ffa77b61bfc3dd15c7a1ad010a60970ba07cd

Current correction domain
  engram-core/src/memory.rs
  26f9f2e7f8920b1328220c1408a8a6f638e7aba8dc3f61d75d4d83cb547a911c

Current correction service
  engram-index/src/memory.rs
  7ece3d1c85c325a2d6e0cd5ba9dcaebd77d32d5d5240bffec2a42375018886b4

Current correction persistence
  engram-store/src/repos/memory.rs
  c285cb5b8359e2768d2f031f12e9ff3e87717879b59d17b194cd15ae2f3ae8ca

Current telemetry cleanup
  engram-store/src/repos/telemetry.rs
  0daeaf11ddcf11ddaf10b8a24ea0553dac52da6539ec6c02caafa1d3a3f44eee

Current MCP surface
  engram-mcp/src/tools.rs
  a5ab8cd0804851bd7ff7e1ca3c47ee002b7da8a55b92e5561bc453591b51592b

Historical correction evaluator, evidence only
  engram-eval/src/native_correction.rs
  2cddf80f9e21ceb71c0949093ac92e329d32358b3e78ecc468acce1ad6d22f51
```

The research made no source, manifest, lockfile, live-store, daemon, provider, authentication,
network, adapter or settings change.

## Current product behavior established from source

`CorrectionProposal` is a server-minted pair binding, not reviewer authority. It names one active
obsolete item and one inactive `needs_review` replacement, carries a schema-1 canonical digest and
has only `pending` and `applied` states. The proposal type and its authority warning are in
`engram-core/src/memory.rs` under `CorrectionProposalStatus` and `CorrectionProposal`.

`MemoryService::propose_correction` currently:

- requires an active, non-procedure obsolete item;
- requires source evidence and rejects manual-review evidence;
- derives kind, scope, confidence, tags and review schedule from the obsolete item;
- forces writer actor `agent`, origin `agent_inferred` and replacement status `needs_review`;
- writes reciprocal proposal markers; and
- computes the canonical pending-pair digest.

`MemoryRepo::save_correction_proposal` guards the old item and creates the replacement and proposal
inside one transaction. The unique pending-obsolete projection prevents ordinary creation of two
pending proposals for the same obsolete item.

`MemoryService::apply_correction` re-reads the exact proposal and pair, checks the selected digest,
scope selector, pair identity, statuses, origin, evidence, reciprocal markers and canonical digest,
then constructs the sole supported transition. `MemoryRepo::apply_correction_proposal` performs the
three-record compare-and-swap in one transaction:

- proposal: `pending -> applied`, with `applied_at` and `applied_digest`;
- replacement: `needs_review -> active`, one `supersedes` edge, one application evidence record,
  and no proposal marker; and
- obsolete: `active -> superseded`, the same application evidence record and no pending lock.

The public result honestly labels the boundary `operator_selected` and reports
`human_identity_authenticated=false`, `intent_verified=false` and
`reviewer_authority_conferred=false`. A C2B3 success may therefore prove only an evaluator-sealed
operator selection. It must not be called authenticated user correction or manual review.

The mutation branch of the separate compatibility `correct_memory` path starts from an active
`user_corrected` replacement and has no pending proposal or C2A prior binding. Its idempotent fast
path can return before validating origin, kind/scope or evidence. Both branches are outside the
C2B3 transition and neither may supply correction-journey authority.

## Why C2A plus one apply response is insufficient

### Idempotent success is not causal attribution

If the exact pair is already applied, `MemoryService::apply_correction` returns success without a
new mutation. An apply response therefore cannot distinguish this execution's transition from a
transition performed before the call. A persisted dispatch journal prevents blind resend, but it
does not prove that the journaled process caused a later applied state.

C2B3 needs exclusive writer capability plus a live, move-only sequence proving:

1. the store was pending at the successful pre-observation;
2. no other in-scope workload writer existed under the frozen C2B2 trusted-principal model;
3. exactly one sealed action was dispatched once;
4. its executor reached a categorical terminal state and was reaped; and
5. a fresh post-observation saw the exact permitted applied state.

An ambiguous action response may justify a bounded forensic post-read. It may never mint causal
success even when that read observes an applied pair.

### C2A proves the selected transition, not every unchanged row

The current `C2PriorCorrectionBinding` retains complete raw rows only for the selected pending
proposal, obsolete item and replacement item. `validate_applied_transition` reconstructs those
pending rows from the post-state and compares them to the moved binding. This is the correct
three-row semantic proof.

Every other row is validated independently in each snapshot, but C2A does not compare those rows
across snapshots. Fresh keys also make pre/post audit MAC equality meaningless. A concurrent or
accidental second mutation can therefore coexist with a valid selected transition unless C2B3 adds
a separate private whole-state equality witness.

### Projection-only drift can be overwritten

The current three-record store transaction guards `snapshot_digest` values. C2A's persisted
snapshot digest intentionally covers the embedded typed payload but excludes the outer
denormalized projections. A raw writer could alter a top-level projection after pre-observation;
the apply transaction could then overwrite that drift while still matching the old embedded
snapshot digest. C2B3 must exclude all competing writers and compare complete canonical raw rows,
not treat the transaction's digest guards as a whole-row chronology proof.

### The normal selector is not strict identity authority

Current correction authorization accepts caller-selected project/task/cwd selectors and uses
case-insensitive or alternative-field matching in several scopes. C2A instead resolves one strict,
conjunctive project/repository/checkout/task identity. The action worker may exercise the production
selector, but the live seal must derive the exact action boundary from the validated C2A target and
pending binding. Normal proposal list/get results are corroborating output only: their pair check
does not recompute the full strict pending contract that C2A already proves.

## Inherited and deliberately superseded contracts

C2B3 inherits C2A's exact target resolution, semantic validation, move-only binding, one-shot MAC
keys and sanitized audit boundary. It inherits C2B2's dedicated guest, private disposable disk,
frozen workload identities, network denial, exclusive trusted-principal model, direct-Core
pre/post collectors and host/guest live-witness cleanup proof. It also inherits C2B1's bounded
native-value gates and canonical comparison rules, but not C2B1's standalone lifecycle.

The lifecycle differences are explicit:

| Earlier contract | C2B3 treatment |
| --- | --- |
| C2B1 collects once, has no reopen and accepts no prior binding | superseded by one pre-collection, exactly one moved pending binding and one phase-gated post-collection |
| C2B2 initializes through a writer, proves lock contention and release, then uses one final fresh semantic-reader process | the initialization/lock/release prefix is repeated; before C2B2 receipt minting, a consuming live-witness extension starts C2B3's semantic authority as the fresh reader and keeps it alive across the action, claiming only a fresh post handle/key |
| C2B2 permits one initial typed-generation frame and one late host challenge into the guest | direction and cardinality are inherited, but adding the precreated host attempt-claim binding to the initial frame is an explicit wire-schema delta that must be frozen and reaccepted; no extra host acknowledgement is added |
| C2B2 forbids the high-level SDK and uses direct Core | inherited by pre/post collectors; deliberately superseded only for the C2B3 action child after the production-path delta below is separately frozen and reaccepted |
| C2B2 proves guest/private-disk stable absence | inherited exactly; one separately attested host evidence root and its exact deny-only attempt marker are outside that cleanup domain |

Previously accepted C2B2 is independent suite and provenance evidence only. Every C2B3 execution
creates a fresh guest/private disk and repeats C2B2's initialization writer, lock contender and
release-probe prefix. After those processes are terminal and reaped—but before the C2B2 guest
witness is consumed into a structural receipt or any cleanup begins—a separately frozen consuming
`C2B2LivePrefixReady -> C2B3GuestJourneyPrepared` transition moves the still-live guest and host
witnesses and the one guest-local expected-generation witness directly into C2B3. That expected
witness was derived independently from the initial typed-generation frame before the writer ran.
No receipt is input authority. C2B3 then starts its semantic authority as the fresh reader; it never
reuses an earlier store handle or resurrects disposed state.

## Recommended C2B3 direction

Use the already researched C2B2 dedicated guest and private data disk. Keep one long-lived frozen
guest supervisor as the sole guest-level causal authority and one long-lived semantic-authority
child as the sole owner of C2A's private pre-state binding and canonical witness. The semantic
authority opens the store for the pre-read, drops every datastore/session/response handle, proves
release through a distinct probe while remaining alive, waits while a separate pinned action child
mutates the store, then independently reopens it for the post-read with a fresh key. The binding and
witness therefore move only within one address space; they are never serialized or reconstructed
over IPC. No datastore, session, raw row, MAC key or private action capability crosses to the host.

The future freeze should refine this consuming state machine:

```text
HostJourneyPrepared
  -> HostProposalAttemptClaimDurable
  -> C2B2LivePrefixReady
  -> C2B3GuestJourneyPrepared
  -> SemanticAuthorityStarted
  -> PreCollectorOpened
  -> PreRawSnapshotAcquired
  -> C2B2ExpectedStateWitnessConsumed
  -> C2B2CanonicalGenerationMatched
  -> PreC2ASnapshotValidatedWithFreshKey
  -> PreStateValidated
  -> ExactlyOnePendingBindingSelected
  -> WholeStateWitnessMinted
  -> PreCollectorHandlesDropped
  -> PreCollectorReleaseProbePassed
  -> PreCollectorReleaseProbeTerminalAndReaped
  -> OperatorSelectionBound
  -> ActionSealMinted
  -> ActionSealConsumedIntoFrame
  -> DispatchJournalDurable
  -> ActionExecutorStarted
  -> ActionFrameConsumedByExecutor
  -> ActionOperationReturnedPrivately
  -> ActionHandlesDropped
  -> ActionReleaseProbeOpenedAndClosed
  -> ActionReleaseProbeTerminalAndReaped
  -> ActionResultEmitted
  -> ActionExecutorTerminalAndReaped
  -> ActionTerminalTokenVerified
  -> ActionObservationBound
  -> ObservationBoundPostOpenPermitMinted
  -> PostCollectorReopenedWithFreshKey
  -> C2AAppliedTransitionValidated
  -> WholeStateDeltaMatched
  -> SemanticAuthorityTerminalAndReaped
  -> GuestPreReceiptLiveRecheckPassed
  -> LateHostLivenessChallengeMinted
  -> LateHostLivenessChallengeSent
  -> LateHostLivenessChallengeAnswered
  -> GuestWitnessConsumedIntoStructuralReceipt
  -> HostReceiptBound
  -> HostCleanupInProgress
  -> GuestAndDiskDisposed
  -> StableAbsenceVerified
  -> C2B3AcceptedOutcome

AnyTerminalStateAfterGuestCreation
  -> CleanupAttempted
  -> { StableAbsenceVerified -> TerminalFailureOutcome
     | CleanupFailureRecorded -> TerminalFailureOutcome }
```

The guest journey state, selected binding, canonical witness, action seal, live receipt and accepted
outcome implement none of `Clone`, `Copy`, `Debug`, `Display`, `Serialize` or `Deserialize`.
Uncertainty consumes the current state and exposes no retry transition. Persisted artifacts are
structural evidence only and can never recreate live authority.

The linear action labels depict the sole success-capable leg. Once the journal is durable, every
other observation—no frame consumption, timeout, EOF, malformed/early result, probe failure,
nonzero/signal exit or process death—still converges, after termination and reap when possible, on
one move-only `ActionObservationBound` carrying irreversible uncertainty. That observation may mint
one forensic post-open permit but can never rejoin the success-capable leg. If the executor cannot
be made terminal and reaped, the journey takes common failure cleanup and records post acquisition
as unavailable rather than opening concurrently.

C2A's deferred wording asks for one process receipt, while C2B2 deliberately requires distinct
writer, probe and reader roles. C2B3 resolves the research ambiguity as one live guest-supervisor
receipt binding the complete process roster and monotonic sequence, nested inside the distinct
host-local witness. It must not falsely claim that acquisition and action ran in one OS process.
The semantic binding remains in one long-lived semantic-authority process, while the mutation
executor and release probe exchange only separately frozen bounded structural frames.

## Pre-state contract

The semantic authority independently opens the evaluator-owned persistent store inside the
accepted C2B2 boundary and performs the exact bounded acquisition. `C2B2LivePrefixReady` carries
the independent expected-generation witness derived from the initial host frame; the consuming
live transition moves it exactly once into this semantic authority. A closed comparator consumes
that witness against the exact owned raw snapshot just acquired and returns
`C2B2CanonicalGenerationMatched` only when the complete canonical nine-table generation is equal.
Omission, duplication, substitution or mismatch consumes authority before any action seal exists.

The successful comparator returns one `C2B2MatchedPreInput` that still owns the same raw snapshot.
The candidate C2B3 whole-state witness is derived from an immutable borrow of that input, after
which the raw snapshot moves exactly once into C2A with a fresh one-shot key `K0`. Thus the writer
expectation, C2A audit and future pre/post witness share one acquisition rather than independently
reconstructing compatible state. C2A success then requires:

- exactly one target-affecting pending proposal binding in the complete pre-state, whose ID equals
  the plan-selected proposal ID;
- zero target-affecting applied proposal bindings;
- exact proposal, obsolete and replacement IDs from that binding;
- the obsolete item is the applicable active relay candidate;
- the pending replacement is absent from relay candidates;
- no forget receipt exists; and
- the target and all nine table inventories are unambiguous.

The exact cardinality check precedes selection. Selection is by the exact plan-bound proposal ID,
never by row/list order or "first match." Other out-of-target proposals may exist only when their
complete nine-table rows remain unchanged. The proposal ID, canonical digest, digest schema, pair
IDs and target selector move out of the successful pre-state; a later caller cannot resupply or
override them.

The future freeze must not reconstruct this witness from C2A's sanitized result. Only a successful
C2A validation whose moved binding matches the candidate's exact selected IDs and rows may promote
the candidate into the private key-independent witness. Failure consumes and destroys candidate,
matched input and expected-generation authority. The promoted witness retains bounded canonical
raw state:

- target selector and resolved target;
- fixed table order and complete sorted record-ID inventory;
- complete canonical raw rows for every unselected record;
- exact selected proposal/obsolete/replacement identities; and
- exact relay-candidate and correction-edge membership needed for the expected delta.

It may remove the selected three rows from the unchanged subset because C2A owns their inverse
transition proof. It stores no public stable content digest, serialization or path. Its currently
owned canonical buffers receive the same bounded-lifetime and best-effort zeroization treatment as
the C2B acquisition boundary.

Before the action seal can be minted, the semantic authority drops every pre-acquisition datastore,
transaction, response and session handle. A fresh release probe must open and close the same path,
then become terminal and be reaped while that authority process remains alive. Source gates
additionally prove no handle was cloned, returned, serialized or retained. Only the private
binding, canonical witness and structural control state survive inside the semantic authority.

Knowing a path is not post-open authority. The semantic process has no generic reopen entry: its
post-collector constructor consumes one unforgeable observation-bound `PostOpenPermit` sent on the
already frozen supervisor control channel. The guest supervisor can mint that permit only after
authenticating an action-terminal token against the pinned executor handle, proving the executor
terminal and reaped, and binding exactly one closed `ActionObservation`. Only an exact
`AppliedNow` frame emitted after a successful action-release probe and followed by a clean executor
exit produces a success-capable observation. Every other permit irreversibly carries its
non-authorizing categorical or uncertain observation through the post-read; even an exact applied
post-state cannot upgrade it. Early, duplicate, caller-supplied or structurally valid but
unauthenticated signals consume the journey into abstention. The future source gate and adversarial
suite must prove that every post-open call is dominated by that consuming permit and that no
alternate datastore constructor is reachable in the semantic process.

## Operator-selection and action seal

An operator selection is accepted only after the exact pending pre-state exists. The future freeze
must choose its source. Until independently authenticated, it remains a fixture/evaluator
administrative choice rather than proof of human identity or intent.

The private action seal should bind at least:

- C2B3 policy/version and exact plan identity;
- guest, disk, store-origin, target and process-roster identities;
- executable, dependency, source and effective-configuration identities;
- a fresh early execution nonce supplied before dispatch;
- resolved project/repository/checkout/task IDs;
- proposal, obsolete and replacement IDs;
- correction digest schema and exact pending canonical digest;
- the one internally constructed action kind and selector;
- action ordinal exactly one; and
- dispatch and terminal monotonic deadlines.

The action's proposal ID, expected digest and selector are derived from the sealed pre-state. No
caller path, endpoint, session, query, arbitrary JSON, retry flag, handle or general tool surface is
accepted.

Before any guest is launched, the host witness resolves one pre-attested owner-only evidence root
outside the disposable run root and atomically creates a no-replace attempt marker whose conflict
key is exactly `(store_generation_id, proposal_id)`. Journey ID, exact plan/policy identities and
action ordinal one are marker content, not part of the conflict key. Thus a new journey ID cannot
claim the same real proposal in the same store generation. Creation uses directory-relative,
no-symlink resolution, a new regular file, exact mode and owner checks, full write, file sync and
parent-directory sync before success is acknowledged. Existing, partial, truncated, duplicated,
substituted, unsynced, corrupt or unreadable marker state fails closed before guest creation. If
the affected proposal cannot be recovered from corrupt state, all C2B3 mutation for that store
generation abstains.

The marker's exact structural digest is bound into the one initial host-to-guest frame. No extra
host acknowledgement supersedes C2B2 directionality. After the exact action frame is derived, the
guest supervisor writes one create-new, owner-only journal containing only bounded structural
bindings and the action-frame digest, then syncs the file and its parent. Executor construction is
unreachable until both the precreated host marker and guest journal have been independently
acknowledged durable. Every partial barrier state is terminal and leaves the host marker in force.
Neither artifact contains raw rows, content, evidence text, a MAC key or a reconstructable private
witness.

Exactly-once here means one attempted dispatch for one immutable store-generation/proposal pair.
The host marker survives guest and private-disk disposal and can be removed or resolved only by a
separately frozen, authenticated operator-resolution protocol; no C2B3 journey or replayed receipt
can alter it. Any existing marker, including one left by pre-state abstention, failure or ambiguous
dispatch, permanently blocks automatic action on that pair. This is intentionally conservative
and is not globally exactly-once business behavior across arbitrary stores or independently created
fixtures.

## Action executor and production-path decision

The executor is a distinct pinned process with exactly one mutation-capable entry. The live guest
supervisor consumes the non-serializable seal while creating exactly one bounded canonical action
frame; the executor consumes that frame, never the C2A binding or live seal. It sends or invokes at
most one apply operation and retains the closed categorical result privately. It then drops the
service, every cloned repository/SDK/store/session/response handle and the sealed local origin while
remaining alive on one supervisor control pipe. A distinct action-release probe must open and close
the same store, become terminal and be reaped. Only after the supervisor authenticates that probe
against its retained process handle may the executor emit exactly one structural result frame and
exit. This distinguishes graceful handle release from kernel cleanup at process death.

The closed result schema binds the policy, plan, journey, store generation, proposal, action
ordinal, early nonce, action-frame digest, executor identity and exactly one categorical result.
Missing, duplicate, reordered, mismatched, trailing or additional fields—including any raw row or
full error text—are rejected. Probe non-exit/failure, a result emitted before probe acknowledgement,
or a spoofed/duplicate acknowledgement consumes the journey into terminal abstention.

A direct-Core reimplementation would prove only an evaluator-specific transition. It would not
prove current Engram behavior. The selected C2B3 direction is therefore the pinned production
`MemoryService::apply_correction` path, not duplicate evaluator SQL or a new abstraction justified
only by the evaluation. The future C2B3 freeze must specify an internal closed outcome that
distinguishes `AppliedNow`, `AlreadyApplied`, categorical rejection and post-dispatch engine/store
uncertainty. Only after that freeze and review may implementation change the production method;
acceptance must then prove the exact result contract. Only `AppliedNow` may contribute to causal
success. `AlreadyApplied` is permanently non-authorizing even if the post-state is otherwise exact.

This choice explicitly reopens the affected C2B2 containment boundary. The action child alone may
use a fully pinned, internally constructed local Rocks `Surreal<Any>` origin required by the current
production service; caller endpoint, configuration, path, session and remote engine remain
forbidden. A C2B3 freeze must include and independently reaccept a delta that proves the exact local
engine constructor and feature set, captured effective configuration, no network route or socket,
the bounded `engram-store`/`engram-index` call graph, non-escape of cloned repository handles, exact
drop before result emission, and conservative mapping of every SDK/engine/store error. The direct-
Core pre/post collectors remain unchanged. Starting a daemon or exposing MCP is not implied.

Duplicating the SQL and service transformation in an evaluator-only module is not an acceptable
production claim. If the production method is reached through the internally constructed type-
erased handle, the sealed local origin remains co-owned until every repository clone is dropped;
the handle never crosses IPC or escapes the action process.

During the journey, `correct`, `supersede`, `archive`, `forget`, generic writes, raw datastore
access and every second apply are structurally unreachable. The process has no MAC key or private
whole-state witness. Production release relies on the closed structural output schema and length
gate, not on canary matching. Adversarial tests additionally scan its standard streams, bounded IPC
and private logs for the existing content and secret canaries.

## Post-state and exact delta

After the action executor is terminal and reaped, the still-live semantic authority independently
reopens the same sealed store through a newly constructed datastore/session, performs the exact C2B
acquisition and mints a new one-shot OS-CSPRNG key `K1`. This is a fresh acquisition and handle, not
a claim that the semantic authority itself is a fresh process. C2B2 separately owns the
fresh-process close/reopen persistence proof.
Authority comes from two independent fills and move-only ownership, not from observed key-byte
inequality; a random collision is not a semantic failure.

Exactly the selected prior binding moves into the post C2A validation. Success then requires:

- identical complete table and record-ID inventory;
- every unselected canonical raw row byte-for-byte equal to the private pre-state witness;
- selected proposal/obsolete/replacement rows accepted by C2A's exact inverse-transition proof;
- unchanged correction-edge IDs and pair tuple, with status only `pending -> applied`;
- obsolete removed from the applicable active candidate set;
- replacement added to that set;
- every other relay candidate canonically unchanged;
- every topology, scope and target-identity row unchanged;
- no forget receipt, added proposal, or additional/unexpected supersession edge beyond the exact
  selected replacement-to-obsolete edge, and no second mutation within the frozen nine-table
  boundary; and
- all action and acquisition children terminal, handle-free and reaped.

The two sanitized C2A audits may be persisted as diagnostic evidence, but their MACs must never be
compared. Persist only structural facts such as row counts, selected IDs and
`unrelated_state_matched=true`; do not emit canonical rows or a stable unkeyed content digest that
would create a guessing oracle.

## Action/result decision matrix

Only this conjunction may mint a live `AcceptedCorrectionJourney`:

```text
valid unique pending pre-state
AND exact operator selection bound
AND one action seal consumed into one frame
AND one exact dispatch observed
AND exact `AppliedNow` response
AND action executor terminal and reaped
AND fresh post-acquisition valid
AND selected C2A transition valid
AND every unrelated canonical row unchanged
AND containment and chronology proven
AND guest/private-disk cleanup and stable absence proven
```

Pre-guest or pre-dispatch abstentions include:

- `ProposalAttemptAlreadyClaimed` for an exact valid pre-existing conflict marker;
- `AttemptClaimUnavailable` for creation, integrity, synchronization or ambiguous marker failure;
- `PreStateInvalid`;
- `PendingBindingAbsent`;
- `PendingBindingCardinalityMismatch`;
- `ScopeOrTargetAmbiguous`;
- `OperatorSelectionAbsent`; and
- `DispatchJournalUnavailable`.

Every terminal branch reached after guest creation uses the same consuming failure-cleanup path,
including inherited-prefix, acquisition, expected-generation comparison, C2A validation, binding
selection, operator-selection, seal and journal failures before dispatch. It prevents new child
creation, consumes cleanup authority once and attempts the fixed sequence: terminate/reap every
started child, close channels/handles, consume or destroy guest-private witnesses/bindings/seals,
shut down the guest, dispose the private disk and run root, and perform repeated stable-absence
checks. It records an authenticated result for every attempted step and may continue to later safe
best-effort steps after an earlier failure; no step's success is required merely to reach the
terminal failure record. The successfully created host deny marker remains in its separately
attested evidence root. The original failure reasons are retained; positive residue appends
`InvalidResidue`, while any failed, skipped, incomplete or unauthenticated cleanup step appends
`CleanupUnproven`. No failure receipt can recreate live authority.

Post-intent or post-dispatch outcomes are terminal and non-retryable:

- `ActionDispatchUncertain`;
- `ActionAlreadyApplied`;
- `ActionFailed`;
- `ActionReportedSuccessButStatePending`;
- `ActionReportedFailureButStateApplied`;
- `PostAcquisitionFailed`;
- `AppliedProvenanceUnproven`;
- `UnexpectedStateChange`;
- `ContainmentUnproven`;
- `CleanupUnproven`; and
- `InvalidResidue`.

Every post-dispatch branch consumes the live seal, retains the deny-only marker and attempts to make
the executor terminal and reaped. Once reap is proven, exactly one observation-bound permit performs
the bounded semantic or forensic post-read. If reap cannot be proven, no concurrent post-open is
allowed and post-state class 1 is recorded. The branch then consumes or destroys guest- and private-
disk-resident private state, attempts common cleanup and records whether stable absence was proven.
The exact host attempt marker is intentionally retained in its separately attested evidence root
and is the only authority-barrier residue exempted from that cleanup domain.

The fresh semantic state has exactly five exhaustive classes:

1. no complete bounded raw acquisition exists because open, read, bound, parse or collection failed;
2. a complete acquisition that passes the exact pending audit and equals the pre-state, with every
   unrelated row unchanged;
3. a complete acquisition that passes the exact C2A moved-binding applied transition, with every
   unrelated row unchanged;
4. a complete acquisition with every strict applied-shape and unrelated-row invariant satisfied,
   for which C2A fails solely with `AppliedProvenanceUnproven`; or
5. every other complete acquired state, including `InvalidRecord`, `InconsistentProjection`,
   selected-row deletion/malformation or any unrelated change.

The base action/state matrix is total:

| Action observation | Post-state class | Required base outcome |
| --- | --- | --- |
| `AppliedNow` | 1 | `PostAcquisitionFailed` |
| `AppliedNow` | 2 | `ActionReportedSuccessButStatePending` |
| `AppliedNow` | 3 | candidate success, subject to containment and cleanup |
| `AppliedNow` | 4 | `AppliedProvenanceUnproven` |
| `AppliedNow` | 5 | `UnexpectedStateChange` |
| `AlreadyApplied` | any | `ActionAlreadyApplied` and `ContainmentUnproven`; append the class diagnostic below, never causal success |
| exact categorical rejection | 1 | `ActionFailed` and `PostAcquisitionFailed` |
| exact categorical rejection | 2 | `ActionFailed` |
| exact categorical rejection | 3 | `ActionReportedFailureButStateApplied` |
| exact categorical rejection | 4 | `ActionFailed` and `AppliedProvenanceUnproven` |
| exact categorical rejection | 5 | `ActionFailed` and `UnexpectedStateChange` |
| timeout, EOF, process death, malformed result or engine/store uncertainty after journal | any | `ActionDispatchUncertain`; append the class diagnostic below, never causal success |

For the two `any` rows, class 1 appends `PostAcquisitionFailed`, class 4 appends
`AppliedProvenanceUnproven`, class 5 appends `UnexpectedStateChange`, and classes 2 and 3 append no
state diagnostic. `AlreadyApplied` additionally proves that the assumed pending-to-one-dispatch
chronology or exclusive-writer boundary failed, hence its mandatory `ContainmentUnproven` reason.

Outcomes carry an ordered nonempty reason set rather than discarding simultaneous failures. The
primary code is the first applicable item in this fixed order:

1. `ActionDispatchUncertain`;
2. `ActionAlreadyApplied`;
3. `ActionReportedFailureButStateApplied`;
4. `ActionReportedSuccessButStatePending`;
5. `ActionFailed`;
6. `PostAcquisitionFailed`;
7. `AppliedProvenanceUnproven`;
8. `UnexpectedStateChange`;
9. `ContainmentUnproven`;
10. `InvalidResidue`; and
11. `CleanupUnproven`.

`ContainmentUnproven` is appended when chronology, phase permit, handle/process or capability proof
fails. `InvalidResidue` means the exact final scan positively observed a non-allowlisted artifact.
`CleanupUnproven` means cleanup or the repeated absence observations could not be completed or
authenticated. Those checks run and append diagnostics even after a base failure. Candidate success
is accepted only when the reason set stays empty through cleanup.

The late host liveness challenge is distinct from the early execution nonce. Only the live host
witness may mint its one-shot OS-CSPRNG value, and only after the semantic authority and action
executor are terminal and reaped and the guest witness has passed a fresh live recheck. The host
then sends it over the sole supervisor channel. The guest response and structural receipt bind the
exact challenge, journey, store generation, proposal, marker, transcript and process roster. Early,
caller-supplied, reused, duplicated or replayed challenges cannot advance typestate. The guest
witness is consumed into that receipt; the host witness binds it, performs cleanup and stable-
absence checks, and alone mints the accepted outcome.

The historical runner's `applied_state_observed_without_dispatch_attribution` recovery remains
honest evidence of an applied state, but it cannot be upgraded to C2B3 causal authority. Historical
serializable intents, dispatch journals, receipts and raw transcripts are research inputs only,
not move-only live witnesses.

## Required adversarial evidence for a future freeze

At minimum, provider-free acceptance must cover:

- zero, exactly one and more than one target-affecting pending bindings; only exactly one can pass,
  and order substitution cannot select a different one;
- omission, duplicate consumption, replay, substitution or mismatch of the initial-frame-derived
  C2B2 expected-generation witness, including a different but independently C2A-valid generation;
- target-affecting applied state before dispatch, including idempotent-success replay;
- two concurrent journeys with different journey IDs but the same store generation and proposal;
- `AlreadyApplied` returned after an apparent exact delta, which remains non-authorizing;
- attempt-marker collision, concurrent creation, truncation, owner/mode/path substitution, file or
  parent sync failure, missing/corrupt state and attempted automatic removal or resolution;
- a foreign writer applying between pre-read and dispatch;
- attempted use of a persisted C2B2 receipt as live input authority, disposal followed by
  continuation, and failure to move both still-live witnesses at the exact prefix transition;
- crash, replay or substitution at seal-to-frame, frame-to-journal, journal-to-executor and
  executor-frame-consumption edges, proving the journal always binds an already derived frame;
- action frame, proposal ID, digest, selector, ordinal, store, executable and process substitution;
- a second mutation call or automatic retry becoming structurally impossible;
- pre-collector release-probe non-exit or missing reap, premature semantic reopen and duplicate,
  spoofed or caller-supplied `PostOpenPermit`;
- action-release-probe open failure, non-exit or missing reap, retained repository/SDK handle,
  result emission before probe acknowledgement and spoofed or duplicate acknowledgement;
- action response forgery, pre-operation emission, truncation, duplicate frame, extra or full-row
  fields, trailing data, timeout, early EOF and death;
- a categorical and every uncertain action observation each consuming exactly one bound post-open
  permit, with an exact applied forensic state unable to upgrade an uncertain observation;
- every cell in the total action/state matrix, including failure plus unexpected state and
  ambiguous dispatch plus failed post-read, with exact reason ordering;
- every allowed selected-row change and every forbidden current/prior field mutation already in C2A;
- insertion, deletion or modification of every unselected row/table class inside the exact frozen
  nine-table boundary, while source firewalls prove the action cannot reach unobserved tables;
- outer-projection-only tampering before action;
- topology, scope, candidate and correction-edge drift;
- reuse of `K0`, caller-provided `K1`, comparison of pre/post MACs, two independent key fills that
  happen to contain equal bytes and a forged public audit;
- retained pre/action handles during post-open and a non-fresh post reader;
- persistence or reconstruction attempts for a binding, witness, seal or accepted outcome;
- process death at every typestate edge and receipt creation before all children are reaped;
- every inherited-prefix and post-guest/pre-dispatch abstention taking the common consuming cleanup
  path, with injected child-reap, disposal, residue-scan and stable-absence failures;
- store path, mount, disk, cgroup, namespace, binary, dependency and guest substitution;
- unexpected standard-output/error, IPC or private-log canaries;
- early, caller-supplied, reused, duplicated and replayed late host challenges;
- persisted receipt replay after loss of the live guest witness; and
- exact guest/private-disk cleanup and stable absence before the host accepted outcome, with only
  the exact durable host deny marker remaining in its separately attested evidence root.

Acceptance also requires the inherited C2A/C2B1/C2B2 focused and full suites, strict Clippy,
formatting, whitespace, external-target and repository-`target/debug` gates, exact dependency and
artifact hashes, plus three fresh exact-source audits with P0=0 and P1=0.

## Deletion is a separate successor

Current confirmed forget spans a materially larger state surface:

1. canonical target deletion;
2. correction-proposal deletion;
3. pending replacement deletion or surviving obsolete unlock;
4. cleanup of structured references in other memory items;
5. knowledge-commit change removal and message redaction;
6. trace and feedback deletion;
7. a transient durable cleanup receipt; and
8. optional regeneration of one selected generated vault.

C2A covers the `memory_item`, `correction_proposal` and `memory_forget_receipt` tables but not
`knowledge_commit`, `brain_harness_trace`, `agent_feedback`, derived graph/search/orientation
output or generated vault files. It intentionally rejects every nonempty forget receipt as
`IncompleteDeletion`. It therefore cannot prove the flagship phrase "every Engram-owned
projection" by itself.

A later deletion freeze must define projection/reference precisely rather than promising arbitrary
text erasure. Current cleanup follows structured IDs in selected fields and sometimes substring
matches evidence text; it does not erase every textual mention in titles, content, tags, commit
fields, trace queries/warnings, feedback notes, logs, backups or external copies. The later design
must enumerate every owned field, use distinct target/proposal/replacement/control canaries and
prove unrelated state unchanged.

That freeze must derive the deletion target and its project/repository/task boundary from a strict
pre-state witness. Current MCP `archive` and `forget` dispatch directly by exact ID and do not use
the correction scope-resolution path, so their request fields are not scope or identity authority.

It must also resolve these source-observed blockers before claiming complete propagation:

### Unlocked obsolete item can lose its integrity projection

When a pending replacement is forgotten, the canonical transaction clears the surviving obsolete
item's pending marker and sets its top-level `snapshot_digest` to `NONE`. The later cleanup loop
does not normally rewrite that unlocked obsolete item. The row can therefore be retrieval-usable
but invalid under C2A, which requires the exact recomputed snapshot digest. The future deletion
transition must atomically write the correct digest and test the complete raw row.

### Cleanup is sequential and needs writer exclusion

Canonical deletion and receipt creation are atomic, but memory/commit/telemetry cleanup occurs in
later scans and writes. Without C2B2-style exclusive writer capability, a new reference can appear
after its table was scanned and before the receipt is removed. A future proof needs fault injection
at every cleanup edge, receipt-bound recovery after restart, exhaustive final scans and stable
observations with no intervening mutation opportunity.

### Deleted proposal IDs are not all cleanup targets

The current cleanup loop purges the requested item and deleted pending replacement IDs, not every
deleted correction-proposal ID. A later freeze must decide which proposal-ID references are owned
and either purge them or state the narrower typed-reference boundary.

### Vault failure loses automatic retry state

The service removes its cleanup receipt before the MCP layer refreshes an optional vault. If vault
export then fails, a retry sees `deleted=false` and skips refresh. Complete selected-vault deletion
needs a durable projection obligation that survives until refresh and exhaustive filesystem
verification, or a separately idempotent refresh operation independent of `deleted=true`.

### Applied-pair references can be invalidated by unrelated forget

Forgetting a third memory can remove evidence from an applied correction member without updating
the applied proposal digest. A later deletion contract must decide whether the proposal is
atomically rewritten, retired or deleted and prove that no applied pair is left internally stale.

`confirm_forget` and its reason currently exist at the MCP boundary, while the service receives only
the ID and the retry receipt does not preserve the reason. This proves an explicit request field,
not authenticated human confirmation. External copies, unselected vaults and prior chat transcripts
remain outside Engram's bounded deletion claim.

## Other deferred lifecycle issues

- There is no typed reject/cancel/archive state for `CorrectionProposal`; pending pairs are resolved
  only by apply or forget, and proposals have no expiry check.
- Applied pair members can later be archived after their markers are cleared, leaving the proposal
  record as historical state whose exact pair no longer satisfies idempotent apply.
- Legacy `supersede`, direct `correct`, archive, forget and raw store writes remain broader mutation
  paths. C2B3 excludes them through a closed call graph; it does not claim they cease to exist.
- Compatibility `correct_memory` can return from its idempotent fast path before validating origin,
  kind/scope or evidence; C2B3 neither calls nor treats that response as authority.
- Normal repository decoding overwrites embedded IDs from record IDs and ignores top-level
  projections. Only raw complete C2A acquisition is suitable as strict evidence.
- Current timestamps are wall-clock domain evidence. Live chronology requires monotonic supervisor
  ordering and retained process handles.

These are inputs to later product and proof design, not authorization to repair them in the C2A
slice.

## Current gate

C2A's frozen design and implementation remain unchanged. Repository `target/debug` remains absent.
Available space is still below the mandatory 19,427,004 KiB reserve. The verified cache at
`/private/tmp/engram-claude-host-action-target-20260904-01` remains untouched pending exact
path-specific deletion confirmation.

This note does not authorize C2B3 freeze or implementation. The immediate executable next action
remains the already frozen C2A integration test after the disk gate is restored. C2B3 freeze work
may begin only after accepted C2A, C2B1 and C2B2 evidence exists.

## Explicit nonclaims

This research does not prove or authorize C2A acceptance, C2B1/C2B2/C2B3 implementation, RocksDB
persistence, correction execution, causal attribution, authenticated human identity or intent,
reviewer authority, procedure correction, proposal expiry/cancellation, deletion propagation,
selected-vault cleanup, external-copy deletion, live-user-store or arbitrary-writer safety,
daemon/MCP/adapter/provider behavior, crash or power-loss durability, filesystem byte immutability,
complete memory erasure, protection from a malicious trusted host/guest principal, kernel, debugger
or hardware, independent-machine replication or flagship completion.
