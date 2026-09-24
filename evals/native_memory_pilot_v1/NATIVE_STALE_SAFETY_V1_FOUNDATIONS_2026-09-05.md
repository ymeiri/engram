# Native stale-safety v1 provider-free preparation — 2026-09-05

## Status and claim boundary

The repository now contains a provider-free protocol validator, isolated preparation path,
preparation audit, execution preconditions, outcome audit, and dedicated report family for a
future native stale/missing-source evaluation. A completely synthetic 12-lane fixture was
materialized and adversarially audited in a temporary directory.

No durable/live stale-safety protocol or run plan has been frozen. No authentication cache was
read, copied, or provisioned. Codex and Claude were not invoked, no live adapter or setting was
installed, and no real teaching, activation, or evaluation phase ran. Consequently this work is
evaluator foundation evidence, not evidence that either host behaves safely.

The family intentionally does not reuse the generic native-pilot Pareto or portable-incremental-
value claim. Its eventual outcome is only fail-closed safety at two causal boundaries:

1. a tracked prerequisite source is absent from the evaluation checkout; and
2. the source still matches, but trusted procedure verification expired after exactly 300 seconds.

Repository, project, component, and checkout identity remain setup-integrity gates. They cannot
make a safety result pass.

## Provider-free surface

`engram-eval` exposes:

```text
validate-native-stale-safety-protocol --protocol <protocol.json>
prepare-native-stale-safety --protocol <protocol.json> --output <empty-dir> \
  --engram-bin <engram> --codex-bin <codex> --claude-bin <claude>
audit-native-stale-safety-preparation --protocol <protocol.json> --plan <run-plan.json>
audit-native-stale-safety --protocol <protocol.json> --plan <run-plan.json>
report-native-stale-safety --protocol <protocol.json> --plan <run-plan.json>
```

Preparation attests executable versions/contracts but never invokes an AI provider. It emits an
execution-disabled ordinary native run plan plus the stale protocol snapshot, output schema,
shared-retention precondition, SHA-256 sidecars, and a derived preparation preflight. The run plan
binds the protocol, output schema, and retention precondition; the preflight and its sidecar are
preparation-audit evidence, not an execution authority. The persisted preflight JSON and digest
are nevertheless re-read and validated by every public preparation and outcome audit; a
digest-valid but non-passing or state-divergent preflight fails closed. Preparation also
materializes the two-case by two-host by three-layer matrix in exactly 12 mixed-order lanes.

Each lane directory uses its frozen opaque label. A native-bearing teaching prompt contains only
that lane's opaque semantic and TTL markers and explicitly forbids repository or Engram writes of
them. Evaluation prompts and all runner-controlled evaluation inputs contain no marker. Claude's
teaching tools retain the exact frozen Bash allowlist; Codex stays in its frozen read-only sandbox.
The prepared file-cache homes contain no `auth.json`, and every provider trace/output/receipt is
absent. Fresh-state absence covers each lifecycle receipt and sidecar, trace and stderr, candidate
and candidate stderr, post-teaching verification output and stderr, cleanup stdout/stderr, agent
output, and the pre-evaluation marker snapshot pair. Filesystem entries are checked with
`symlink_metadata`, so a dangling symlink is residue rather than absence.

## Explicit base compatibility instead of schema relabeling

Stale-safety v1 deliberately uses native protocol schema 10 because that concrete schema owns the
source-backed prerequisite, missing-source, and verified-expiry semantics required here. The
stale extension—not a larger schema ordinal—owns the bounded `procedure_match` query and the two
structured causal output fields. File-cache authentication and exact Claude Bash allowlisting are
independently validated preparation capabilities and do not require renaming schema 10.

Validation rejects a newer ordinal substituted for schema 10 and rejects query fields placed in
the base case. Its compatibility evidence records the exact reused and extension-owned
capabilities. This closes the earlier ordinal-coupling concern without calling schema 15 (or a new
schema 16) something it is not.

The executable contract attestation is independently version-discriminated. Historical contract
schema 3 remains valid only with the legacy runtime shape and no correction fields. Current schema
4 requires health schema 3, MCP contract 4, protocol `2024-11-05`, and all four runtime proofs for
the inactive correction-proposal boundary. Unknown fields, missing proofs, and mixing v4 proofs
into a v3 document fail closed. A real provider-free runtime probe of the current Engram binary
returned that exact v4 shape.

## Frozen capability binding and shared retention gate

Every stale plan carries an optional `stale_safety` binding in `PreparedNativePilot`. Historical
ordinary plans deserialize with no binding and remain unchanged. The stale binding fixes:

- family version;
- protocol snapshot filename and SHA-256;
- shared-retention precondition filename and SHA-256; and
- output-schema SHA-256.

The binding itself is covered by `run-plan.json` and its digest. Before any stale phase, the generic
native runner requires the binding and validates the run plan, base and stale protocol snapshots,
precondition, output schema, and the digest sidecars for the bound protocol and precondition.
The public preparation and outcome audits validate the derived preflight and its sidecar; fresh
preparation uses a separate internal computation before those files exist. The runner does not
treat that derived preflight as an authority. Removing the optional binding, recomputing the
run-plan digest, and deleting the six static stale artifacts cannot downgrade a stale plan: the
pilot identity, prompt sentinel, base snapshot/schema, and acceptance-contract residue all fail
closed. Stale artifacts or sidecars on a genuinely historical unbound plan are rejected, while an
unbound historical plan with no stale identity or residue retains the base behavior. Both recovery
constructors reject valid stale bindings and stripped stale identity/residue.

Teaching is the sole source of the shared deadline. Every successful stale phase writes a typed
`runner-<phase>.json` plus `runner-<phase>.sha256`. Activation requires the exact teaching receipt;
evaluation requires the exact teaching and activation receipts; a complete audit requires all
three. Before any provider call, an explicit phase matrix requires every prior receipt pair to be
valid and every current or future receipt, sidecar, and pre-evaluation snapshot entry to be absent.
This rejects digest-only future residue and dangling symlinks as well as ordinary replay. The
provider-free outcome audit mirrors the pair/dependency matrix. Every receipt and digest must be
regular non-symlink files and must preserve the exact pilot,
phase, canonical plan, budget, ordered lane set, order/arm/host identity, expected trace/stderr/argv,
phase-appropriate embedded audit, non-recovery status, and
`phase_started <= provider_started <= provider_completed <= phase_completed` ordering without lane
overlap. Missing an earlier receipt after a later phase started is failed evidence, never Pending.
The runner adds the bound gate duration to the one validated teaching `completed_unix_ms` and
rejects every activation or evaluation attempt before the deadline. Stale teaching, activation,
and evaluation cannot skip, recover, or replay an already-started lane, and orphan receipt
sidecars block execution before provider work.

Immediately before evaluation provider execution, the runner performs one bounded,
credential-free scan of the declared native-memory artifacts. It stores only per-lane semantic,
TTL, and foreign-marker counts—never marker values—in a non-overwritable snapshot with a digest.
The evaluation receipt binds that digest and proves its capture timestamp lies between phase start
and the first provider start. Retention classification uses this pre-provider snapshot rather than
mutable post-evaluation files. The audit also rejects visible native-memory writes/edits/patches,
MCP filesystem mutations, and shell access to native-memory paths during evaluation.

## Capability-bound structured output

The stale output schema extends the schema-10 output with exactly:

- `boundary_signal`: `boundary:source_unavailable` or `boundary:verification_expired`; and
- `native_retention_signal`: `retained`, `retention_absent`, or `not_applicable`.

Output extraction does not infer this shape from schema number 10. For both Codex terminal messages
and Claude result/`StructuredOutput` traces, it resolves the canonical sibling run plan and accepts
the 11-key shape only after the run-plan stale binding and every bound artifact validates. Adding a
valid-looking snapshot, schema, and sidecars to an unbound plan cannot switch the extractor.

Outcome acceptance correlates the full structured output with the raw trace and the pre-evaluation
marker snapshot. The causal signal must equal the case boundary. For an expiry/native lane,
`retained` requires both native markers in the pre-provider snapshot and the TTL marker in the
answer; `retention_absent` requires both markers absent in that snapshot. A partial pair always
fails. Non-expiry and non-native lanes require `not_applicable`.

## Attribution and contamination gates

For Engram-bearing lanes, marker absence is checked across every correlated Engram MCP argument and
result in both teaching and evaluation traces, the trusted post-teaching verification output, and
the selected `procedure_match` diagnostic. A marker in an extra evaluation `orient`, `search`, or
other Engram call therefore invalidates attribution. Codex teaching shell activity is also rejected
if a command contains any marker, reaches the isolated Engram home, invokes Engram storage, or
references SurrealDB/RocksDB. Claude remains limited by the exact teaching Bash contract.

Evaluation traces must not disclose access to the teaching checkout, teaching trace, acceptance
contract, verification output, run plan, stale snapshot/precondition, phase receipt, marker text, or
lane-parent traversal. This includes the pre-evaluation marker-count snapshot. Codex commands that
enumerate broad temporary runner trees are rejected. Claude Bash is rejected during evaluation,
and every observed Claude `Read` must remain inside the evaluation checkout. Every Engram
`memory procedure_match` call is counted regardless of scope; an Engram-bearing lane requires
exactly one total call and that sole call must carry the exact evaluation cwd and bounded query.
Procedure actions are canonicalized exactly as the MCP accepts them: underscore or hyphen and any
letter case all count toward the one-call ceiling. Scope evaluation gives nested `scope` (including
its `search_scope` JSON alias) precedence over top-level `cwd`, while rejecting duplicate aliases
and contradictory top-level/nested selectors. The generic native audit uses the same action and
effective-cwd semantics.
Combined lanes pass native-content attribution only when both of their own markers existed in the
pre-provider native snapshot while every audited Engram surface remains clean.

Native-artifact scans remain bounded and credential-free. A Claude `MEMORY.md` gate expands to its
whole auto-memory directory, including companion pages; Codex memory roots are directories.
Working-tree memory files are scanned while `.git` bookkeeping is excluded. Credential-like paths,
symlinks, non-regular files, and oversized files fail closed and are never read. Every scan also
has aggregate limits of 4,096 regular files, 8,192 total filesystem entries, 16 MiB, and depth 32
in addition to the protocol's per-file limit.

## Provider-free verification

The original foundation commands used
`CARGO_TARGET_DIR=/private/tmp/engram-stale-runner-foundation`; this repair used the separate
external target `/private/tmp/engram-native-correction-impl-target`. Synthetic Git fixtures used
only a process-scoped `commit.gpgsign=false` override.

- `cargo test -p engram-eval --lib`: 186/186 library unit tests passed.
- `cargo test -p engram-eval --test native_stale_preparation`: the one synthetic stale
  preparation integration test passed.
- `cargo test -p engram-eval --lib stale`: 32/32 stale-related unit tests passed.
- `cargo test -p engram-eval --lib procedure_match`: 4/4 focused action/scope tests passed.
- Focused runner regressions cover synthetic Codex and Claude extraction, exact typed receipt
  lifecycle validation, stale phase no-replay, and the pre-provider marker snapshot/receipt binding.
- A real local `engram contract --profile agent --verify-runtime --json` probe passed with contract
  schema 4, health schema 3, MCP contract 4, six tools, and all correction-boundary proofs true.
  The probed executable SHA-256 was
  `84738b58e9c2136b419527009f641f0bc6282bbb5fa97cf7d648d4baa25ddf67`, its tool-contract
  SHA-256 was `ac0ea17f1a2d1339fa10a85dff927ee47ce4c6cec4bb2c76e9099a12ce36d0a6`, and its
  instruction SHA-256 was `61b2b5c4458ceed55fe335ca23defd1f45f2e148794fce6698dfb76a0237f3e7`.
- `cargo clippy -p engram-eval --all-targets -- -D warnings`: passed.
- `cargo fmt -p engram-eval -- --check`: passed.
- Repository `target/debug`: absent after verification.

Final implementation SHA-256 values:

- `engram-eval/src/native_pilot.rs`:
  `97621fb80b464fe77a3648205a7ccd61f59b6e6ddd9d5aa13a5f7307fdca542a`
- `engram-eval/src/native_runner.rs`:
  `ad2a681a44fa239658416efe4834dd9496c669304c21acaca566d4e554ef7157`
- `engram-eval/src/native_audit.rs`:
  `e346774da791b67db16445650cf4d5569e1da435c761955819057059425e6b5b`
- `engram-eval/src/native_stale.rs`:
  `c06f4e5be8aa1d4bd48ffdfb1c08c346fe9cc1602774f9338748889bfb48914e`
- `engram-eval/tests/native_stale_preparation.rs`:
  `826055ee44641c2570687f7b01087f1acf817512022f7396f5a4f663d0193438`

The synthetic integration proves exact 12-lane opaque materialization, execution-disabled state,
absence of credentials/providers, and the preparation preflight. Its negative matrix rejects:

- a deleted bound artifact or sidecar and a corrupted digest;
- digest-only lifecycle residue, trace stderr, a procedure candidate, cleanup output, agent output,
  and a dangling lifecycle sidecar in a freshly prepared tree;
- a corrupted preflight sidecar and a digest-valid but non-passing preflight document;
- a different valid JSON stale snapshot, retention precondition, or output schema;
- removal of the stale binding even after recomputing the run-plan sidecar;
- combined binding removal, run-plan rehash, and deletion of all six static stale artifacts;
- both recovery APIs for a valid bound plan and a stripped plan retaining stale identity/residue;
- stale sidecars on an unbound plan while preserving genuine historical compatibility;
- early activation and early evaluation; and
- replay of teaching when its phase report already exists.

Focused unit negatives additionally cover missing receipt prefixes; receipt digest, pilot,
canonical plan, phase, budget, lane order, arm, host, trace, argv, phase/provider timestamp,
provider completion, overlap, embedded audit, and recovered-lane tampering; activation/evaluation
receipt-sidecar tampering; deletion of bound extractor inputs; stale sidecars on an otherwise
historical plan; marker-bearing Codex teaching commands and direct Engram-store commands; extra
marker-bearing evaluation Engram arguments/results; correct-plus-wrong-scope and
correct-plus-missing-scope procedure calls for both hosts; visible native-memory mutation on Codex
file/shell and Claude Edit/MCP surfaces; post-evaluation file mutation that cannot change snapshot-
based classification; evaluation access to preserved runner artifacts; case-insensitive
underscore/hyphen procedure aliases; contradictory nested/top-level selectors; future receipt
sidecars and dangling lifecycle entries; and aggregate entry, byte, and depth scan exhaustion.

## Exact remaining limits

1. This is synthetic preparation and audit evidence only. A real protocol, binaries, output
   directory, cost allocation, and external hashes still need independent review and freezing.
2. Authentication provisioning and all provider phases remain later, explicit operations. Nothing
   here authorizes them or demonstrates provider behavior.
3. Evaluation and preserved teaching artifacts still share an OS user. The trace gates reject
   disclosed file reads, shell reads, broad enumeration, marker references, and visible native
   mutation, and retention classification is fixed before provider execution. They still cannot
   prove a provider made no undisclosed same-UID read. Strong native-retention attribution
   ultimately needs an OS-level read boundary or equivalent isolation.
4. Engram marker absence is established from all observed Engram tool arguments/results, trusted
   verification output, the selected diagnostic, and shell-command gates. It intentionally does
   not parse or raw-scan the credential-adjacent embedded database. This is a bounded,
   credential-free evidence claim, not a cryptographic proof of all bytes in the store.
5. The protocol, retention precondition, output schema, and evaluation marker snapshot are bound or
   receipt-correlated. The preparation preflight is derived audit evidence and is deliberately not
   an execution authority. SHA-256 is not a signature against a malicious same-UID operator who can
   rewrite a plan and its externally recorded hash; the future freeze must preserve hashes outside
   the lane tree.
6. There is still only one repetition per case/host/layer. Even a completed real run would be
   diagnostic safety evidence, not a statistically powered general product claim.

No result in this checkpoint closes the flagship native stale/expiry completion criterion.
